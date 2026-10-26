//! Pre-authorization privacy gate for an already constructed Zcash wallet proposal.
//!
//! The caller must pass a canonical, complete encoding of the *actual* SDK
//! proposal to [`fingerprint_canonical_proposal`]. The gate cannot establish
//! that a caller-supplied byte sequence covers every relevant proposal field.

use sha2::{Digest, Sha256};

const FINGERPRINT_DOMAIN: &[u8] = b"LeakLens/ZcashProposal/v1\0";

/// Fingerprint of an entire canonical SDK proposal, not its privacy summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProposalFingerprint {
    bytes: [u8; 32],
    empty: bool,
}

impl ProposalFingerprint {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

/// Domain-separated SHA-256 over length-prefixed canonical proposal bytes.
///
/// The integration layer is responsible for providing the SDK's complete,
/// deterministic encoding, including inputs, outputs, amounts, change, fees,
/// and any other fields that affect signing.
pub fn fingerprint_canonical_proposal(canonical: &[u8]) -> ProposalFingerprint {
    let mut hasher = Sha256::new();
    hasher.update(FINGERPRINT_DOMAIN);
    hasher.update((canonical.len() as u64).to_be_bytes());
    hasher.update(canonical);
    ProposalFingerprint {
        bytes: hasher.finalize().into(),
        empty: canonical.is_empty(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PolicyVersion(pub u32);

/// An assessment already produced by the privacy rules engine.
///
/// Only the gate-relevant summary lives here. Conversion from the policy
/// crate's complete `PrivacyAssessment` belongs to the integration layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateAssessment {
    pub fingerprint: ProposalFingerprint,
    pub policy_version: PolicyVersion,
    pub complete: bool,
    pub warning_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateDecision {
    Allow,
    RequireAcknowledgment,
    DenyIncomplete,
}

pub fn prepare_decision(assessment: &GateAssessment) -> GateDecision {
    if !assessment.complete || assessment.fingerprint.empty {
        GateDecision::DenyIncomplete
    } else if assessment.warning_count > 0 {
        GateDecision::RequireAcknowledgment
    } else {
        GateDecision::Allow
    }
}

/// An explicit user acknowledgment bound to a particular assessed proposal.
/// Construct only after the interface has shown the findings to the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Approval {
    fingerprint: ProposalFingerprint,
    policy_version: PolicyVersion,
}

impl Approval {
    pub fn acknowledge(assessment: &GateAssessment) -> Self {
        Self {
            fingerprint: assessment.fingerprint,
            policy_version: assessment.policy_version,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateError {
    EmptyProposal,
    ProposalChanged,
    PolicyChanged,
    IncompleteAssessment,
    AcknowledgmentRequired,
    ApprovalMismatch,
}

/// Verify the exact proposal and policy again immediately before entering
/// the wallet's signing/authorization path.
pub fn authorize(
    assessment: &GateAssessment,
    current_fingerprint: ProposalFingerprint,
    current_policy_version: PolicyVersion,
    approval: Option<&Approval>,
) -> Result<(), GateError> {
    if assessment.fingerprint.empty || current_fingerprint.empty {
        return Err(GateError::EmptyProposal);
    }
    if assessment.policy_version != current_policy_version {
        return Err(GateError::PolicyChanged);
    }
    if assessment.fingerprint != current_fingerprint {
        return Err(GateError::ProposalChanged);
    }
    if !assessment.complete {
        return Err(GateError::IncompleteAssessment);
    }
    if assessment.warning_count > 0 {
        let approval = approval.ok_or(GateError::AcknowledgmentRequired)?;
        if approval.fingerprint != assessment.fingerprint
            || approval.policy_version != assessment.policy_version
        {
            return Err(GateError::ApprovalMismatch);
        }
    }
    Ok(())
}
