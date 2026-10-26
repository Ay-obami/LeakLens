//! Review boundary for actual wallet-generated SDK proposals.
//!
//! The wallet owns its database, keys and immutable proposal. This module
//! never signs or broadcasts. Approval returns a one-shot proposal-bound gate.

use std::io::{self, BufRead, Write};

use leaklens_core::Policy;
use zcash_client_backend::{fees::StandardFeeRule, proposal::Proposal};

use crate::{InspectError, PendingZcashReview, PreviewOutcome, preview_review};

#[derive(Debug)]
pub enum WalletReviewError {
    Inspect(InspectError),
    Io(io::Error),
}

pub enum WalletReviewOutcome<'a, NoteRef> {
    Abort,
    Revise,
    Approved(PendingZcashReview<'a, NoteRef>),
}

/// Inspect the same borrowed proposal that the wallet must later authorize.
pub fn review_wallet_proposal<'a, NoteRef>(
    proposal: &'a Proposal<StandardFeeRule, NoteRef>,
    policy: &Policy,
    input: impl BufRead,
    output: &mut impl Write,
) -> Result<WalletReviewOutcome<'a, NoteRef>, WalletReviewError> {
    let mut pending =
        PendingZcashReview::inspect(proposal, *policy).map_err(WalletReviewError::Inspect)?;
    match preview_review(pending.review_mut(), input, output).map_err(WalletReviewError::Io)? {
        PreviewOutcome::Abort => Ok(WalletReviewOutcome::Abort),
        PreviewOutcome::ReviseProposal => Ok(WalletReviewOutcome::Revise),
        PreviewOutcome::ProceedToWallet => Ok(WalletReviewOutcome::Approved(pending)),
    }
}
