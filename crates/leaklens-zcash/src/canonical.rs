//! Deterministic SDK-native serialization of a standard-fee Zcash proposal.
//!
//! This is the complete SDK proposal protobuf (not a lossy privacy-facts
//! summary). The wallet must re-encode the *same immutable proposal* during
//! authorization, and must never alter it before passing it to its signer.

use prost::Message;
use zcash_client_backend::{fees::StandardFeeRule, proposal::Proposal, proto::proposal};

/// Serializes all proposal fields represented by the SDK's standard proposal
/// protobuf: inputs, selected pools, change, fee, target and policy metadata.
pub fn encode_canonical_standard_proposal<NoteRef>(
    proposal: &Proposal<StandardFeeRule, NoteRef>,
) -> Vec<u8> {
    proposal::Proposal::from_standard_proposal(proposal).encode_to_vec()
}
