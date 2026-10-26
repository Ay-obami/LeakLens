//! A single-use pre-authorization privacy review for a Zcash transaction.
//!
//! The integration must derive both `ProposalFacts` and canonical bytes from
//! the *same real SDK proposal*. This crate recomputes policy findings itself,
//! keeps the assessment private, and invokes the signing callback only after
//! a fresh canonical encoding matches. It cannot stop a separate wallet path
//! that bypasses this API, or stop mutations after the encoding callback if
//! the caller does not keep the proposal immutable/locked through signing.

use leaklens_core::{assess, Policy, PrivacyAssessment, ProposalFacts};
use sha2::{Digest, Sha256};

const FINGERPRINT_DOMAIN: &[u8] = b"LeakLens/ZcashProposal/v1\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProposalFingerprint([u8; 32]);

impl ProposalFingerprint {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Hash a complete, deterministic SDK proposal encoding, including all
/// signing-relevant fields. The caller must not pass a lossy facts summary.
pub fn fingerprint_canonical_proposal(canonical: &[u8]) -> ProposalFingerprint {
    let mut hasher = Sha256::new();
    hasher.update(FINGERPRINT_DOMAIN);
    hasher.update((canonical.len() as u64).to_be_bytes());
    hasher.update(canonical);
    ProposalFingerprint(hasher.finalize().into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateDecision {
    Allow,
    RequireAcknowledgment,
    DenyIncomplete,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateError {
    EmptyProposal,
    IncompleteAssessment,
    AcknowledgmentRequired,
    AcknowledgmentNotRequired,
    PolicyChanged,
    ProposalChanged,
}

#[derive(Debug, PartialEq, Eq)]
pub enum GateRunError<E> {
    Gate(GateError),
    Encoding(E),
    Signing(E),
}

/// Non-cloneable, one-use review. No public assessment or approval
/// constructors are exposed; the core engine is run inside `inspect`.
pub struct GateReview {
    fingerprint: ProposalFingerprint,
    assessment: PrivacyAssessment,
    acknowledged: bool,
}

impl GateReview {
    /// Inspect normalized facts produced by the trusted SDK adapter.
    /// Integration must guarantee facts and canonical bytes describe the
    /// exact same immutable transaction proposal.
    pub fn inspect(
        facts: &ProposalFacts,
        policy: &Policy,
        canonical_proposal: &[u8],
    ) -> Result<Self, GateError> {
        if canonical_proposal.is_empty() {
            return Err(GateError::EmptyProposal);
        }
        Ok(Self {
            fingerprint: fingerprint_canonical_proposal(canonical_proposal),
            assessment: assess(facts, policy),
            acknowledged: false,
        })
    }

    /// Show this assessment to the user before accepting an acknowledgment.
    pub fn assessment(&self) -> &PrivacyAssessment {
        &self.assessment
    }

    pub fn decision(&self) -> GateDecision {
        if !self.assessment.complete {
            GateDecision::DenyIncomplete
        } else if !self.assessment.findings.is_empty() {
            GateDecision::RequireAcknowledgment
        } else {
            GateDecision::Allow
        }
    }

    /// Called only by the wallet UI's explicit confirmation handler after
    /// displaying this review. The crate cannot authenticate a UI event.
    pub fn acknowledge(&mut self) -> Result<(), GateError> {
        match self.decision() {
            GateDecision::Allow => Err(GateError::AcknowledgmentNotRequired),
            GateDecision::DenyIncomplete => Err(GateError::IncompleteAssessment),
            GateDecision::RequireAcknowledgment => {
                self.acknowledged = true;
                Ok(())
            }
        }
    }

    /// Consume the review exactly once. The wallet must hold the actual
    /// proposal immutable/locked while `encode_current` and `sign` run.
    /// The signing callback is never called on an error.
    pub fn authorize_and_execute<R, E>(
        self,
        current_policy: &Policy,
        encode_current: impl FnOnce() -> Result<Vec<u8>, E>,
        sign: impl FnOnce() -> Result<R, E>,
    ) -> Result<R, GateRunError<E>> {
        if current_policy.version != self.assessment.policy_version {
            return Err(GateRunError::Gate(GateError::PolicyChanged));
        }
        if !self.assessment.complete {
            return Err(GateRunError::Gate(GateError::IncompleteAssessment));
        }
        if !self.assessment.findings.is_empty() && !self.acknowledged {
            return Err(GateRunError::Gate(GateError::AcknowledgmentRequired));
        }

        let bytes = encode_current().map_err(GateRunError::Encoding)?;
        if bytes.is_empty() || fingerprint_canonical_proposal(&bytes) != self.fingerprint {
            return Err(GateRunError::Gate(GateError::ProposalChanged));
        }
        sign().map_err(GateRunError::Signing)
    }
}
