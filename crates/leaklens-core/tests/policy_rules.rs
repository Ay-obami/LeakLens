use leaklens_core::{assess, PaymentFacts, Policy, PoolKind, ProposalFacts, RecipientKind, StepFacts};

fn proposal(payment: PaymentFacts, transparent_inputs: bool, shielded_inputs: bool) -> ProposalFacts {
    ProposalFacts {
        steps: vec![StepFacts {
            payments: vec![payment],
            has_transparent_inputs: transparent_inputs,
            has_shielded_inputs: shielded_inputs,
            change_pools: vec![PoolKind::Orchard],
        }],
    }
}

fn payment(recipient_kind: RecipientKind, selected_pool: PoolKind) -> PaymentFacts {
    PaymentFacts { recipient_kind, selected_pool, amount_zat: 125_000_000 }
}

fn codes(facts: &ProposalFacts) -> Vec<&'static str> {
    assess(facts, &Policy::default()).findings.iter().map(|finding| finding.code).collect()
}

#[test]
fn transparent_destination_warns() {
    let facts = proposal(payment(RecipientKind::Transparent, PoolKind::Transparent), false, false);
    assert!(codes(&facts).contains(&"LL-001"));
}

#[test]
fn transparent_input_warns() {
    let facts = proposal(payment(RecipientKind::Shielded, PoolKind::Orchard), true, false);
    assert!(codes(&facts).contains(&"LL-002"));
}

#[test]
fn shielded_to_transparent_warns() {
    let facts = proposal(payment(RecipientKind::Transparent, PoolKind::Transparent), false, true);
    assert!(codes(&facts).contains(&"LL-003"));
}

#[test]
fn unified_transparent_selection_warns() {
    let facts = proposal(payment(RecipientKind::Unified, PoolKind::Transparent), false, true);
    let findings = assess(&facts, &Policy::default()).findings;
    assert!(findings.iter().any(|f| f.code == "LL-004"));
    assert!(findings.iter().any(|f| f.code == "LL-001"));
}

#[test]
fn shielded_only_has_no_known_disclosure() {
    let facts = proposal(payment(RecipientKind::Shielded, PoolKind::Orchard), false, true);
    let result = assess(&facts, &Policy::default());
    assert!(result.complete);
    assert!(result.findings.is_empty());
}

#[test]
fn later_step_risk_detected() {
    let mut facts = proposal(payment(RecipientKind::Shielded, PoolKind::Orchard), false, true);
    facts.steps.push(StepFacts {
        payments: vec![payment(RecipientKind::Transparent, PoolKind::Transparent)],
        has_transparent_inputs: false,
        has_shielded_inputs: true,
        change_pools: vec![],
    });
    let result = assess(&facts, &Policy::default());
    assert!(result.findings.iter().any(|f| f.code == "LL-001" && f.step_index == 1));
}

#[test]
fn unknown_pool_marks_incomplete() {
    let facts = proposal(payment(RecipientKind::Unknown, PoolKind::Unknown), false, true);
    let result = assess(&facts, &Policy::default());
    assert!(!result.complete);
    assert!(result.findings.iter().any(|f| f.code == "LL-005"));
}

#[test]
fn unrecognized_change_pool_marks_incomplete() {
    let mut facts = proposal(payment(RecipientKind::Shielded, PoolKind::Orchard), false, true);
    facts.steps[0].change_pools.push(PoolKind::Unknown);
    assert!(!assess(&facts, &Policy::default()).complete);
}

#[test]
fn empty_proposal_is_incomplete() {
    let result = assess(&ProposalFacts { steps: vec![] }, &Policy::default());
    assert!(!result.complete);
}
