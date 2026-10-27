//! A reviewed SDK proposal must not sign after the active policy changes.
use leaklens_core::Policy;
use leaklens_demo::wallet_flow::{WalletReviewOutcome, review_wallet_proposal};
use leaklens_zcash::fixture::sample_shielded_proposal;
use std::{cell::Cell, io::Cursor};

#[test]
fn stale_policy_denies_before_wallet_callback() {
    let proposal = sample_shielded_proposal();
    let policy = Policy::default();
    let pending = match review_wallet_proposal(
        &proposal,
        &policy,
        Cursor::new(b"CONTINUE\n"),
        &mut Vec::new(),
    )
    .expect("review displays the policy findings")
    {
        WalletReviewOutcome::Approved(pending) => pending,
        _ => panic!("expected clean proposal review approval"),
    };
    let invoked = Cell::new(false);
    let result = pending.execute(
        &Policy {
            version: policy.version + 1,
        },
        |_| {
            invoked.set(true);
            Ok::<_, ()>(())
        },
    );
    assert!(result.is_err(), "policy changes must deny approval");
    assert!(!invoked.get(), "signing callback must never run");
}
