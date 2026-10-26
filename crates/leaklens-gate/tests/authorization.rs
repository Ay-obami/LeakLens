use leaklens_core::{PaymentFacts, Policy, PoolKind, ProposalFacts, RecipientKind, StepFacts};
use leaklens_gate::{
    fingerprint_canonical_proposal, GateDecision, GateError, GateReview, GateRunError,
};
use std::cell::Cell;

fn facts(pool: PoolKind) -> ProposalFacts {
    ProposalFacts {
        steps: vec![StepFacts {
            payments: vec![PaymentFacts {
                recipient_kind: match pool {
                    PoolKind::Transparent => RecipientKind::Transparent,
                    PoolKind::Unknown => RecipientKind::Unknown,
                    _ => RecipientKind::Shielded,
                },
                selected_pool: pool,
                amount_zat: 1000,
            }],
            has_transparent_inputs: false,
            has_shielded_inputs: true,
            has_prior_shielded_funding: false,
            change_pools: vec![PoolKind::Sapling],
        }],
    }
}

fn review(pool: PoolKind, bytes: &[u8]) -> GateReview {
    GateReview::inspect(&facts(pool), &Policy::default(), bytes).unwrap()
}

#[test]
fn clean_review_executes_once_after_fresh_encoding_matches() {
    let review = review(PoolKind::Sapling, b"canonical-a");
    assert_eq!(review.decision(), GateDecision::Allow);
    let invoked = Cell::new(0);
    let result = review.authorize_and_execute(
        &Policy::default(),
        || Ok::<_, ()>(b"canonical-a".to_vec()),
        || {
            invoked.set(invoked.get() + 1);
            Ok::<_, ()>(42)
        },
    );
    assert_eq!(result, Ok(42));
    assert_eq!(invoked.get(), 1);
}

#[test]
fn warned_payment_requires_explicit_acknowledgment() {
    let review = review(PoolKind::Transparent, b"canonical-a");
    assert_eq!(review.decision(), GateDecision::RequireAcknowledgment);
    let invoked = Cell::new(false);
    let result = review.authorize_and_execute(
        &Policy::default(),
        || Ok::<_, ()>(b"canonical-a".to_vec()),
        || {
            invoked.set(true);
            Ok::<_, ()>(())
        },
    );
    assert_eq!(
        result,
        Err(GateRunError::Gate(GateError::AcknowledgmentRequired))
    );
    assert!(!invoked.get());
}

#[test]
fn acknowledged_payment_executes_and_review_is_consumed() {
    let mut review = review(PoolKind::Transparent, b"canonical-a");
    review.acknowledge().unwrap();
    let invoked = Cell::new(0);
    let result = review.authorize_and_execute(
        &Policy::default(),
        || Ok::<_, ()>(b"canonical-a".to_vec()),
        || {
            invoked.set(invoked.get() + 1);
            Ok::<_, ()>(())
        },
    );
    assert_eq!(result, Ok(()));
    assert_eq!(invoked.get(), 1);
    // authorize_and_execute takes ownership of the review, preventing reuse.
}

#[test]
fn incomplete_policy_analysis_fails_closed() {
    let mut review = review(PoolKind::Unknown, b"canonical-a");
    assert_eq!(review.decision(), GateDecision::DenyIncomplete);
    assert_eq!(review.acknowledge(), Err(GateError::IncompleteAssessment));
    let invoked = Cell::new(false);
    let result = review.authorize_and_execute(
        &Policy::default(),
        || Ok::<_, ()>(b"canonical-a".to_vec()),
        || {
            invoked.set(true);
            Ok::<_, ()>(())
        },
    );
    assert_eq!(
        result,
        Err(GateRunError::Gate(GateError::IncompleteAssessment))
    );
    assert!(!invoked.get());
}

#[test]
fn modified_canonical_bytes_and_policy_reject_before_signing() {
    for modified in [
        b"recipient=B;amount=10;input=1;change=2;fee=3".as_slice(),
        b"recipient=A;amount=10;input=9;change=2;fee=3",
        b"recipient=A;amount=10;input=1;change=9;fee=3",
        b"recipient=A;amount=10;input=1;change=2;fee=9",
    ] {
        let mut review = review(
            PoolKind::Transparent,
            b"recipient=A;amount=10;input=1;change=2;fee=3",
        );
        review.acknowledge().unwrap();
        let invoked = Cell::new(false);
        let result = review.authorize_and_execute(
            &Policy::default(),
            || Ok::<_, ()>(modified.to_vec()),
            || {
                invoked.set(true);
                Ok::<_, ()>(())
            },
        );
        assert_eq!(result, Err(GateRunError::Gate(GateError::ProposalChanged)));
        assert!(!invoked.get());
    }
    let review = review(PoolKind::Sapling, b"canonical-a");
    let result = review.authorize_and_execute(
        &Policy { version: 2 },
        || Ok::<_, ()>(b"canonical-a".to_vec()),
        || Ok::<_, ()>(()),
    );
    assert_eq!(result, Err(GateRunError::Gate(GateError::PolicyChanged)));
}

#[test]
fn equivalent_policy_value_not_pointer_identity_authorizes() {
    let reviewed_policy = Policy { version: 7 };
    let gate = GateReview::inspect(&facts(PoolKind::Sapling), &reviewed_policy, b"canonical-a")
        .expect("valid review");
    let independently_loaded_policy = Policy { version: 7 };
    let result = gate.authorize_and_execute(
        &independently_loaded_policy,
        || Ok::<_, ()>(b"canonical-a".to_vec()),
        || Ok::<_, ()>("approved"),
    );
    assert_eq!(result, Ok("approved"));
}

#[test]
fn encoder_failure_and_empty_proposal_never_authorize() {
    let review = review(PoolKind::Sapling, b"canonical-a");
    let invoked = Cell::new(false);
    let result = review.authorize_and_execute(
        &Policy::default(),
        || Err::<Vec<u8>, _>("cannot encode"),
        || {
            invoked.set(true);
            Ok::<_, &str>(())
        },
    );
    assert_eq!(result, Err(GateRunError::Encoding("cannot encode")));
    assert!(!invoked.get());
    assert!(matches!(
        GateReview::inspect(&facts(PoolKind::Sapling), &Policy::default(), b""),
        Err(GateError::EmptyProposal)
    ));
}

#[test]
fn proposal_fingerprint_is_domain_separated() {
    use sha2::{Digest, Sha256};
    let raw = Sha256::digest(b"canonical-a");
    let tagged = fingerprint_canonical_proposal(b"canonical-a");
    assert_ne!(tagged.as_bytes().as_slice(), raw.as_slice());
}
