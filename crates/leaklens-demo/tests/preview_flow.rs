use std::io::Cursor;

use leaklens_core::{PaymentFacts, Policy, PoolKind, ProposalFacts, RecipientKind, StepFacts};
use leaklens_demo::{preview_review, PreviewOutcome};
use leaklens_gate::GateReview;

fn review(recipient: RecipientKind, pool: PoolKind) -> GateReview {
    let facts = ProposalFacts {
        steps: vec![StepFacts {
            payments: vec![PaymentFacts {
                recipient_kind: recipient,
                selected_pool: pool,
                amount_zat: 125_000_000,
            }],
            has_transparent_inputs: false,
            has_shielded_inputs: true,
            has_prior_shielded_funding: false,
            change_pools: vec![],
        }],
    };
    GateReview::inspect(&facts, &Policy::default(), b"canonical-fixture-test-only").unwrap()
}

#[test]
fn warning_displays_disclosure_and_requires_explicit_acknowledgment() {
    let mut review = review(RecipientKind::Transparent, PoolKind::Transparent);
    let mut output = Vec::new();
    let outcome = preview_review(&mut review, Cursor::new(b"ACKNOWLEDGE\n"), &mut output).unwrap();
    let rendered = String::from_utf8(output).unwrap();
    assert_eq!(outcome, PreviewOutcome::ProceedToWallet);
    assert!(rendered.contains("LL-001"));
    assert!(rendered.contains("PUBLIC DISCLOSURE"));
    assert!(rendered.contains("LL-003"));
    assert!(rendered.contains("POSSIBLE CORRELATION"));
}

#[test]
fn empty_input_does_not_approve_a_warned_payment() {
    let mut review = review(RecipientKind::Transparent, PoolKind::Transparent);
    let outcome = preview_review(&mut review, Cursor::new(Vec::<u8>::new()), &mut Vec::new()).unwrap();
    assert_eq!(outcome, PreviewOutcome::Abort);
}

#[test]
fn nonmatching_input_does_not_approve_a_warned_payment() {
    let mut review = review(RecipientKind::Transparent, PoolKind::Transparent);
    let outcome = preview_review(&mut review, Cursor::new(b"yes\n"), &mut Vec::new()).unwrap();
    assert_eq!(outcome, PreviewOutcome::Abort);
}

#[test]
fn revise_input_returns_for_new_transaction_proposal() {
    let mut review = review(RecipientKind::Transparent, PoolKind::Transparent);
    let outcome = preview_review(&mut review, Cursor::new(b"REVISE\n"), &mut Vec::new()).unwrap();
    assert_eq!(outcome, PreviewOutcome::ReviseProposal);
}

#[test]
fn incomplete_assessment_cannot_be_acknowledged() {
    let mut review = review(RecipientKind::Unknown, PoolKind::Unknown);
    let mut output = Vec::new();
    let outcome = preview_review(&mut review, Cursor::new(b"ACKNOWLEDGE\n"), &mut output).unwrap();
    assert_eq!(outcome, PreviewOutcome::Abort);
    assert!(String::from_utf8(output).unwrap().contains("INSPECTION INCOMPLETE"));
}

#[test]
fn clear_assessment_requires_a_continue_decision() {
    let mut review = review(RecipientKind::Shielded, PoolKind::Orchard);
    let outcome = preview_review(&mut review, Cursor::new(b"CONTINUE\n"), &mut Vec::new()).unwrap();
    assert_eq!(outcome, PreviewOutcome::ProceedToWallet);
}

#[test]
fn explicit_acknowledgment_does_not_turn_preview_into_signing() {
    let mut review = review(RecipientKind::Transparent, PoolKind::Transparent);
    let outcome = preview_review(&mut review, Cursor::new(b"ACKNOWLEDGE\n"), &mut Vec::new()).unwrap();
    assert_eq!(outcome, PreviewOutcome::ProceedToWallet);
    // The preview never constructs, signs, or broadcasts a transaction.
    // The consuming wallet must separately call GateReview::authorize_and_execute.
}
