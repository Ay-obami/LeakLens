use crate::model::{
    Certainty, Finding, Policy, PoolKind, PrivacyAssessment, ProposalFacts, RecipientKind,
    Severity,
};

pub fn assess(facts: &ProposalFacts, policy: &Policy) -> PrivacyAssessment {
    let mut findings = Vec::new();
    let mut complete = !facts.steps.is_empty();

    if facts.steps.is_empty() {
        findings.push(incomplete(0, None, "Transaction proposal contains no steps"));
    }

    for (step_index, step) in facts.steps.iter().enumerate() {
        if step.has_transparent_inputs {
            findings.push(Finding {
                code: "LL-002",
                step_index,
                payment_index: None,
                severity: Severity::Warning,
                certainty: Certainty::PublicDisclosure,
                message: "Transparent funding inputs and their public history are observable",
            });
        }

        for (payment_index, payment) in step.payments.iter().enumerate() {
            if payment.selected_pool == PoolKind::Unknown || payment.recipient_kind == RecipientKind::Unknown {
                complete = false;
                findings.push(incomplete(step_index, Some(payment_index), "Recipient or selected payment pool cannot be assessed"));
                continue;
            }

            if (payment.recipient_kind == RecipientKind::Transparent
                && payment.selected_pool != PoolKind::Transparent)
                || (payment.recipient_kind == RecipientKind::Shielded
                    && payment.selected_pool == PoolKind::Transparent)
            {
                complete = false;
                findings.push(incomplete(step_index, Some(payment_index), "Recipient kind conflicts with the selected output pool"));
                continue;
            }

            if payment.selected_pool == PoolKind::Transparent {
                findings.push(Finding {
                    code: "LL-001",
                    step_index,
                    payment_index: Some(payment_index),
                    severity: Severity::Warning,
                    certainty: Certainty::PublicDisclosure,
                    message: "The transparent recipient and payment amount will be publicly visible",
                });

                if step.has_shielded_inputs || step.has_prior_shielded_funding {
                    findings.push(Finding {
                        code: "LL-003",
                        step_index,
                        payment_index: Some(payment_index),
                        severity: Severity::Caution,
                        certainty: Certainty::PossibleCorrelation,
                        message: "Deshielding may expose amount or timing patterns; it does not prove an identity link",
                    });
                }

                if payment.recipient_kind == RecipientKind::Unified {
                    findings.push(Finding {
                        code: "LL-004",
                        step_index,
                        payment_index: Some(payment_index),
                        severity: Severity::Warning,
                        certainty: Certainty::PublicDisclosure,
                        message: "The Unified Address is being paid through its transparent receiver",
                    });
                }
            }
        }

        if step.change_pools.contains(&PoolKind::Transparent) {
            findings.push(Finding {
                code: "LL-007",
                step_index,
                payment_index: None,
                severity: Severity::Warning,
                certainty: Certainty::PublicDisclosure,
                message: "Transparent change output and its value will be publicly visible",
            });
            if step.has_shielded_inputs || step.has_prior_shielded_funding {
                findings.push(Finding {
                    code: "LL-003",
                    step_index,
                    payment_index: None,
                    severity: Severity::Caution,
                    certainty: Certainty::PossibleCorrelation,
                    message: "Shielded funding alongside transparent change may create public correlation signals",
                });
            }
        }

        if step.change_pools.contains(&PoolKind::Unknown) {
            complete = false;
            findings.push(incomplete(step_index, None, "A change output pool cannot be assessed"));
        }
    }

    PrivacyAssessment { findings, complete, policy_version: policy.version }
}

fn incomplete(step_index: usize, payment_index: Option<usize>, message: &'static str) -> Finding {
    Finding {
        code: "LL-005",
        step_index,
        payment_index,
        severity: Severity::Incomplete,
        certainty: Certainty::Unknown,
        message,
    }
}
