use std::collections::BTreeMap;

use leaklens_core::{PoolKind, ProposalFacts, RecipientKind};
use leaklens_zcash::extract_proposal_facts;
use zcash_client_backend::{
    data_api::wallet::{ConfirmationsPolicy, TargetHeight},
    fees::TransactionBalance,
    proposal::Proposal,
};
use zcash_keys::address::Address;
use zcash_protocol::{consensus::{BlockHeight, Network}, value::Zatoshis, PoolType};
use zcash_transparent::address::TransparentAddress;
use zip321::{Payment, TransactionRequest};

fn empty_sdk_proposal() -> Proposal<(), u32> {
    Proposal::single_step(
        TransactionRequest::new(vec![]).expect("valid empty payment request"),
        Default::default(),
        vec![],
        None,
        BlockHeight::from_u32(100),
        TransactionBalance::new(vec![], Zatoshis::const_from_u64(0)).unwrap(),
        (),
        TargetHeight::from(BlockHeight::from_u32(101)),
        ConfirmationsPolicy::default(),
        false,
        false,
    )
    .expect("real SDK proposal")
}

fn transparent_sdk_proposal() -> Proposal<(), u32> {
    let recipient = Address::Transparent(TransparentAddress::PublicKeyHash([0x11; 20]));
    let request = TransactionRequest::new(vec![
        Payment::new(
            recipient.to_zcash_address(&Network::TestNetwork),
            Some(Zatoshis::const_from_u64(0)),
            None,
            None,
            None,
            vec![],
        )
        .unwrap(),
    ])
    .unwrap();

    Proposal::single_step(
        request,
        BTreeMap::from([(0usize, PoolType::TRANSPARENT)]),
        vec![],
        None,
        BlockHeight::from_u32(100),
        TransactionBalance::new(vec![], Zatoshis::const_from_u64(0)).unwrap(),
        (),
        TargetHeight::from(BlockHeight::from_u32(101)),
        ConfirmationsPolicy::default(),
        false,
        false,
    )
    .expect("real SDK proposal with a transparent recipient")
}

#[test]
fn extracts_every_step_from_an_sdk_proposal() {
    let proposal = empty_sdk_proposal();
    let facts: ProposalFacts = extract_proposal_facts(&proposal).expect("inspect proposal");
    assert_eq!(facts.steps.len(), 1);
    assert!(facts.steps[0].payments.is_empty());
    assert!(!facts.steps[0].has_transparent_inputs);
    assert!(!facts.steps[0].has_shielded_inputs);
    assert!(facts.steps[0].change_pools.is_empty());
}

#[test]
fn extracts_transparent_recipient_from_selected_sdk_pool() {
    let proposal = transparent_sdk_proposal();
    let facts = extract_proposal_facts(&proposal).unwrap();
    assert_eq!(facts.steps[0].payments.len(), 1);
    let payment = &facts.steps[0].payments[0];
    assert_eq!(payment.recipient_kind, RecipientKind::Transparent);
    assert_eq!(payment.selected_pool, PoolKind::Transparent);
    assert_eq!(payment.amount_zat, 0);
}
