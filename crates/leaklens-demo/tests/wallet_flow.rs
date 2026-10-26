//! Contract tests for the opt-in, real-wallet review boundary.
//!
//! These tests intentionally use non-spendable, typed librustzcash proposal
//! fixtures. They NEVER open a funded wallet, sign, persist, or broadcast.
//! They establish the authorization preconditions the eventual wallet runner
//! must satisfy before it invokes the official SDK signer.

use std::{cell::Cell, io::Cursor};

use leaklens_core::Policy;
use leaklens_demo::wallet_flow::{WalletReviewOutcome, review_wallet_proposal};
use leaklens_zcash::fixture::{sample_shielded_proposal, sample_transparent_proposal};

#[test]
fn transparent_proposal_requires_explicit_acknowledgment_before_wallet_callback() {
    let proposal = sample_transparent_proposal();
    let policy = Policy::default();
    let mut transcript = Vec::new();

    let result = review_wallet_proposal(
        &proposal,
        &policy,
        Cursor::new(b"CONTINUE\n"),
        &mut transcript,
    )
    .expect("review should display findings and accept normal cancellation");

    assert!(matches!(result, WalletReviewOutcome::Abort));
    let text = String::from_utf8(transcript).expect("display is valid UTF-8");
    assert!(text.contains("LL-001"));
    assert!(text.contains("LL-002"));
}

#[test]
fn acknowledged_transparent_payment_delivers_exact_proposal_to_authorized_callback() {
    let proposal = sample_transparent_proposal();
    let policy = Policy::default();
    let mut transcript = Vec::new();

    let result = review_wallet_proposal(
        &proposal,
        &policy,
        Cursor::new(b"ACKNOWLEDGE\n"),
        &mut transcript,
    )
    .expect("review should permit acknowledgment of a complete warning");

    let pending = match result {
        WalletReviewOutcome::Approved(pending) => pending,
        _ => panic!("correct explicit acknowledgment must approve the reviewed proposal"),
    };

    let callback_count = Cell::new(0);
    let callback_result = pending.execute(&policy, |same_sdk_proposal| {
        callback_count.set(callback_count.get() + 1);
        assert!(std::ptr::eq(same_sdk_proposal, &proposal));
        Ok::<_, ()>("guarded callback")
    });

    assert_eq!(callback_result.expect("consumed gate approves"), "guarded callback");
    assert_eq!(callback_count.get(), 1);
}

#[test]
fn clean_shielded_proposal_requires_continue_but_no_warning_acknowledgment() {
    let proposal = sample_shielded_proposal();
    let policy = Policy::default();
    let mut transcript = Vec::new();

    let invalid = review_wallet_proposal(
        &proposal,
        &policy,
        Cursor::new(b"ACKNOWLEDGE\n"),
        &mut transcript,
    )
    .expect("invalid choice is not a policy error");
    assert!(matches!(invalid, WalletReviewOutcome::Abort));

    transcript.clear();
    let valid = review_wallet_proposal(
        &proposal,
        &policy,
        Cursor::new(b"CONTINUE\n"),
        &mut transcript,
    )
    .expect("clean proposal may proceed via CONTINUE");
    assert!(matches!(valid, WalletReviewOutcome::Approved(_)));
}

#[test]
fn revise_or_empty_input_never_returns_approved_wallet_review() {
    let proposal = sample_transparent_proposal();
    let policy = Policy::default();

    for (input, expected_revise) in [("REVISE\n", true), ("", false), ("YES\n", false)] {
        let mut transcript = Vec::new();
        let outcome = review_wallet_proposal(
            &proposal,
            &policy,
            Cursor::new(input.as_bytes()),
            &mut transcript,
        )
        .expect("non-approval is a normal outcome");
        if expected_revise {
            assert!(matches!(outcome, WalletReviewOutcome::Revise));
        } else {
            assert!(matches!(outcome, WalletReviewOutcome::Abort));
        }
    }
}
