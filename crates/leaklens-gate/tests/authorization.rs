use leaklens_gate::{
    authorize, fingerprint_canonical_proposal, prepare_decision, Approval, GateAssessment,
    GateDecision, GateError, PolicyVersion,
};

fn assessment(contents: &[u8], complete: bool, warnings: usize) -> GateAssessment {
    GateAssessment {
        fingerprint: fingerprint_canonical_proposal(contents),
        policy_version: PolicyVersion(1),
        complete,
        warning_count: warnings,
    }
}

#[test]
fn clean_complete_proposal_can_pass_without_warning_acknowledgment() {
    let a = assessment(b"canonical-proposal-a", true, 0);
    assert_eq!(prepare_decision(&a), GateDecision::Allow);
    assert_eq!(authorize(&a, a.fingerprint, PolicyVersion(1), None), Ok(()));
}

#[test]
fn warned_proposal_requires_matching_acknowledgment() {
    let a = assessment(b"canonical-proposal-a", true, 2);
    assert_eq!(prepare_decision(&a), GateDecision::RequireAcknowledgment);
    assert_eq!(
        authorize(&a, a.fingerprint, PolicyVersion(1), None),
        Err(GateError::AcknowledgmentRequired)
    );
    let approval = Approval::acknowledge(&a);
    assert_eq!(authorize(&a, a.fingerprint, PolicyVersion(1), Some(&approval)), Ok(()));
}

#[test]
fn incomplete_analysis_fails_closed_even_when_acknowledged() {
    let a = assessment(b"canonical-proposal-a", false, 1);
    assert_eq!(prepare_decision(&a), GateDecision::DenyIncomplete);
    assert_eq!(
        authorize(&a, a.fingerprint, PolicyVersion(1), Some(&Approval::acknowledge(&a))),
        Err(GateError::IncompleteAssessment)
    );
}

#[test]
fn changed_proposal_bytes_rejects_previous_approval() {
    let a = assessment(b"recipient=A;amount=10;input=1;change=2;fee=3", true, 1);
    let approval = Approval::acknowledge(&a);
    for modified in [
        b"recipient=B;amount=10;input=1;change=2;fee=3".as_slice(),
        b"recipient=A;amount=10;input=9;change=2;fee=3",
        b"recipient=A;amount=10;input=1;change=9;fee=3",
        b"recipient=A;amount=10;input=1;change=2;fee=9",
    ] {
        let changed = fingerprint_canonical_proposal(modified);
        assert_ne!(a.fingerprint, changed);
        assert_eq!(
            authorize(&a, changed, PolicyVersion(1), Some(&approval)),
            Err(GateError::ProposalChanged)
        );
    }
}

#[test]
fn changed_policy_version_rejects_previous_approval() {
    let a = assessment(b"canonical-proposal-a", true, 1);
    let approval = Approval::acknowledge(&a);
    assert_eq!(
        authorize(&a, a.fingerprint, PolicyVersion(2), Some(&approval)),
        Err(GateError::PolicyChanged)
    );
}

#[test]
fn approval_cannot_be_reused_for_another_proposal() {
    let a = assessment(b"canonical-proposal-a", true, 1);
    let b = assessment(b"canonical-proposal-b", true, 1);
    let approval = Approval::acknowledge(&a);
    assert_eq!(
        authorize(&b, b.fingerprint, PolicyVersion(1), Some(&approval)),
        Err(GateError::ApprovalMismatch)
    );
}

#[test]
fn empty_canonical_proposal_is_rejected() {
    let a = assessment(b"", true, 0);
    assert_eq!(
        authorize(&a, a.fingerprint, PolicyVersion(1), None),
        Err(GateError::EmptyProposal)
    );
}

#[test]
fn fingerprint_is_domain_separated() {
    use sha2::{Digest, Sha256};
    let raw = Sha256::digest(b"canonical-proposal-a");
    let tagged = fingerprint_canonical_proposal(b"canonical-proposal-a");
    assert_ne!(tagged.as_bytes().as_slice(), raw.as_slice());
}
