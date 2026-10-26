//! A one-shot review session bound to a single borrowed SDK proposal.
//!
//! This does not sign or broadcast: wallet code supplies the authorization
//! callback, which is invoked only after the gate accepts the review.

use leaklens_core::Policy;
use leaklens_gate::{GateError, GateReview, GateRunError};
use leaklens_zcash::{AdapterError, encode_canonical_standard_proposal, extract_proposal_facts};
use zcash_client_backend::{fees::StandardFeeRule, proposal::Proposal};

#[derive(Debug)]
pub enum InspectError {
    Adapter(AdapterError),
    Gate(GateError),
}

/// Holds one immutable Zcash transaction proposal until checked execution.
/// The review cannot be detached and applied to a different proposal.
pub struct PendingZcashReview<'a, NoteRef> {
    proposal: &'a Proposal<StandardFeeRule, NoteRef>,
    review: GateReview,
}

impl<'a, NoteRef> PendingZcashReview<'a, NoteRef> {
    pub fn inspect(
        proposal: &'a Proposal<StandardFeeRule, NoteRef>,
        policy: Policy,
    ) -> Result<Self, InspectError> {
        let facts = extract_proposal_facts(proposal).map_err(InspectError::Adapter)?;
        let encoded = encode_canonical_standard_proposal(proposal);
        let review = GateReview::inspect(&facts, &policy, &encoded).map_err(InspectError::Gate)?;
        Ok(Self { proposal, review })
    }

    /// Allows the UI to display findings and record an explicit acknowledgment.
    pub fn review_mut(&mut self) -> &mut GateReview {
        &mut self.review
    }

    /// Consumes this review session and authorizes exactly the borrowed proposal.
    /// The SDK proposal bytes and active policy are revalidated immediately before
    /// the callback; failed inspection or missing consent prevent invocation.
    pub fn execute<R, E>(
        self,
        policy: &Policy,
        wallet_callback: impl FnOnce(&Proposal<StandardFeeRule, NoteRef>) -> Result<R, E>,
    ) -> Result<R, GateRunError<E>> {
        let proposal = self.proposal;
        self.review.authorize_and_execute(
            policy,
            || Ok::<Vec<u8>, E>(encode_canonical_standard_proposal(proposal)),
            || wallet_callback(proposal),
        )
    }
}
