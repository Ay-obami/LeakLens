use leaklens_core::{PaymentFacts, PoolKind, ProposalFacts, RecipientKind, StepFacts};
use zcash_client_backend::proposal::Proposal;
use zcash_keys::address::Address;
use zcash_protocol::{PoolType, ShieldedPool};

/// A proposal detail could not be inspected without losing privacy-relevant information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    /// An SDK payment has no amount, so the preview cannot accurately describe it.
    MissingAmount { step_index: usize, payment_index: usize },
    /// The proposed payment contains an unsupported receiver encoding.
    UnsupportedRecipient { step_index: usize, payment_index: usize },
    /// The SDK provided no selected output pool for a requested payment.
    MissingSelectedPool { step_index: usize, payment_index: usize },
}

fn pool_kind(pool: PoolType) -> PoolKind {
    match pool {
        PoolType::Transparent => PoolKind::Transparent,
        PoolType::Shielded(ShieldedPool::Sapling) => PoolKind::Sapling,
        PoolType::Shielded(ShieldedPool::Orchard) => PoolKind::Orchard,
        PoolType::Shielded(ShieldedPool::Ironwood) => PoolKind::Ironwood,
    }
}

fn recipient_kind(address: &Address) -> RecipientKind {
    match address {
        Address::Transparent(_) | Address::Tex(_) => RecipientKind::Transparent,
        Address::Unified(_) => RecipientKind::Unified,
        Address::Sapling(_) => RecipientKind::Shielded,
    }
}

/// Extracts all wallet-proposed payment paths without requiring spending or viewing keys.
///
/// A caller must treat every `Err` as an incomplete privacy assessment and prevent
/// authorization until a compatible adapter can inspect the proposal.
pub fn extract_proposal_facts<FeeRule, NoteRef>(
    proposal: &Proposal<FeeRule, NoteRef>,
) -> Result<ProposalFacts, AdapterError> {
    let mut steps = Vec::with_capacity(proposal.steps().len());

    for (step_index, step) in proposal.steps().iter().enumerate() {
        let mut payments = Vec::with_capacity(step.transaction_request().payments().len());

        for (&payment_index, payment) in step.transaction_request().payments() {
            let selected_pool = step.payment_pools().get(&payment_index).copied().ok_or(
                AdapterError::MissingSelectedPool {
                    step_index,
                    payment_index,
                },
            )?;
            let amount = payment.amount().ok_or(AdapterError::MissingAmount {
                step_index,
                payment_index,
            })?;
            let recipient = payment
                .recipient_address()
                .clone()
                .convert::<Address>()
                .map_err(|_| AdapterError::UnsupportedRecipient {
                    step_index,
                    payment_index,
                })?;

            payments.push(PaymentFacts {
                recipient_kind: recipient_kind(&recipient),
                selected_pool: pool_kind(selected_pool),
                amount_zat: amount.into_u64(),
            });
        }

        let has_transparent_inputs = !step.transparent_inputs().is_empty()
            || !step.prior_step_inputs().is_empty();
        let has_shielded_inputs = step.shielded_inputs().is_some();
        let change_pools = step
            .balance()
            .proposed_change()
            .iter()
            .map(|change| pool_kind(change.output_pool()))
            .collect();

        steps.push(StepFacts {
            payments,
            has_transparent_inputs,
            has_shielded_inputs,
            change_pools,
        });
    }

    Ok(ProposalFacts { steps })
}
