use std::cell::Cell;

use leaklens_core::Policy;
use leaklens_demo::PendingZcashReview;
use leaklens_zcash::fixture::{sample_shielded_proposal, sample_transparent_proposal};
use leaklens_zcash::encode_canonical_standard_proposal;

#[test]
fn identical_sdk_proposals_produce_identical_canonical_bytes() {
    let first = sample_shielded_proposal();
    let second = sample_shielded_proposal();
    let encoded_first = encode_canonical_standard_proposal(&first);
    let encoded_second = encode_canonical_standard_proposal(&second);
    assert!(!encoded_first.is_empty());
    assert_eq!(encoded_first, encoded_second);
}

#[test]
fn warning_blocks_wallet_callback_without_acknowledgment() {
    let proposal = sample_transparent_proposal();
    let pending = PendingZcashReview::inspect(&proposal, Policy::default()).unwrap();
    let called = Cell::new(false);
    let result = pending.execute(&Policy::default(), |_| {
        called.set(true);
        Ok::<_, std::io::Error>(())
    });
    assert!(result.is_err());
    assert!(!called.get());
}

#[test]
fn acknowledgment_reaches_callback_with_exact_borrowed_sdk_proposal() {
    let proposal = sample_transparent_proposal();
    let original_address = &proposal as *const _;
    let mut pending = PendingZcashReview::inspect(&proposal, Policy::default()).unwrap();
    pending.review_mut().acknowledge().unwrap();
    let called = Cell::new(false);
    pending.execute(&Policy::default(), |actual| {
        assert_eq!(actual as *const _, original_address);
        called.set(true);
        Ok::<_, std::io::Error>(())
    }).unwrap();
    assert!(called.get());
}

#[test]
fn clean_proposal_reaches_callback_without_acknowledgment() {
    let proposal = sample_shielded_proposal();
    let pending = PendingZcashReview::inspect(&proposal, Policy::default()).unwrap();
    let called = Cell::new(false);
    pending.execute(&Policy::default(), |_| {
        called.set(true);
        Ok::<_, std::io::Error>(())
    }).unwrap();
    assert!(called.get());
}

#[test]
fn changed_policy_blocks_callback() {
    let proposal = sample_shielded_proposal();
    let pending = PendingZcashReview::inspect(&proposal, Policy::default()).unwrap();
    let called = Cell::new(false);
    let result = pending.execute(&Policy { version: 2 }, |_| {
        called.set(true);
        Ok::<_, std::io::Error>(())
    });
    assert!(result.is_err());
    assert!(!called.get());
}
