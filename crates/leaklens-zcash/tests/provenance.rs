//! SDK-backed regression cases for proposal provenance and change-output inspection.
//! Requires dev-dependency `nonempty = "0.11"` when included in leaklens-zcash.

use std::collections::BTreeMap;

use leaklens_core::PoolKind;
use leaklens_zcash::extract_proposal_facts;
use nonempty::NonEmpty;
use zcash_client_backend::{
    data_api::wallet::{ConfirmationsPolicy, TargetHeight},
    fees::{ChangeValue, TransactionBalance},
    proposal::{Proposal, Step, StepOutput, StepOutputIndex},
    wallet::WalletTransparentOutput,
};
use zcash_keys::address::Address;
use zcash_protocol::{
    consensus::{BlockHeight, Network},
    value::Zatoshis,
    PoolType,
};
use zcash_transparent::{
    address::TransparentAddress,
    bundle::{OutPoint, TxOut},
};
use zip321::{Payment, TransactionRequest};

const INPUT_VALUE: u64 = 100_000;
const PAID_VALUE: u64 = 60_000;
const CHANGE_VALUE: u64 = 30_000;
const FEE_VALUE: u64 = 10_000;

fn request(amount: u64) -> TransactionRequest {
    let recipient = Address::Transparent(TransparentAddress::PublicKeyHash([0x51; 20]));
    TransactionRequest::new(vec![
        Payment::new(
            recipient.to_zcash_address(&Network::TestNetwork),
            Some(Zatoshis::const_from_u64(amount)),
            None,
            None,
            None,
            vec![],
        )
        .unwrap(),
    ])
    .unwrap()
}

fn funding_input() -> WalletTransparentOutput<()> {
    let address = TransparentAddress::PublicKeyHash([0x52; 20]);
    WalletTransparentOutput::<()>::from_parts(
        OutPoint::new([0x53; 32], 0),
        TxOut::new(
            Zatoshis::const_from_u64(INPUT_VALUE),
            address.script().into(),
        ),
        Some(BlockHeight::from_u32(90)),
        None,
        None,
        None,
    )
    .expect("synthetic transparent input")
}

fn output_pools() -> BTreeMap<usize, PoolType> {
    BTreeMap::from([(0usize, PoolType::TRANSPARENT)])
}

fn balance_with_transparent_change() -> TransactionBalance {
    TransactionBalance::new(
        vec![ChangeValue::transparent(Zatoshis::const_from_u64(CHANGE_VALUE))],
        Zatoshis::const_from_u64(FEE_VALUE),
    )
    .unwrap()
}

#[test]
fn sdk_transparent_change_is_reported_as_transparent() {
    let proposal: Proposal<(), u32> = Proposal::single_step(
        request(PAID_VALUE),
        output_pools(),
        vec![funding_input()],
        None,
        BlockHeight::from_u32(100),
        balance_with_transparent_change(),
        (),
        TargetHeight::from(BlockHeight::from_u32(101)),
        ConfirmationsPolicy::default(),
        false,
        false,
    )
    .expect("valid real SDK transparent proposal");

    let facts = extract_proposal_facts(&proposal).expect("adapter must preserve change pool");
    assert_eq!(facts.steps[0].change_pools, vec![PoolKind::Transparent]);
    assert!(facts.steps[0].has_transparent_inputs);
}

#[test]
fn sdk_rejects_payment_without_a_selected_pool() {
    let invalid = Step::<u32>::from_parts(
        &[],
        request(PAID_VALUE),
        BTreeMap::new(),
        vec![funding_input()],
        None,
        None,
        vec![],
        balance_with_transparent_change(),
        false,
        false,
    );
    assert!(invalid.is_err(), "SDK must reject missing pool before adapter receives a typed Step");
}

#[test]
fn sdk_rejects_forward_or_nonexistent_prior_step_reference() {
    let proposed_step = Step::<u32>::from_parts(
        &[],
        request(PAID_VALUE),
        output_pools(),
        vec![],
        None,
        None,
        vec![StepOutput::new(1, StepOutputIndex::Payment(0))],
        TransactionBalance::new(vec![], Zatoshis::const_from_u64(FEE_VALUE)).unwrap(),
        false,
        false,
    );
    // Depending on where SDK validation runs, this can fail either while
    // constructing the step or while assembling the full proposal.
    if let Ok(step) = proposed_step {
        let proposal = Proposal::multi_step(
            (),
            TargetHeight::from(BlockHeight::from_u32(101)),
            ConfirmationsPolicy::default(),
            NonEmpty::singleton(step),
        );
        assert!(proposal.is_err(), "invalid reference must not become a valid proposal");
    }
}

#[test]
fn prior_transparent_payment_is_recognized_as_transparent_funding() {
    // Two real SDK-validated steps: 100,000 transparent input -> 60,000
    // transparent payment + 30,000 transparent change + 10,000 fee;
    // the next step spends the first payment's 60,000 output.
    let first = Step::<u32>::from_parts(
        &[],
        request(PAID_VALUE),
        output_pools(),
        vec![funding_input()],
        None,
        None,
        vec![],
        balance_with_transparent_change(),
        false,
        false,
    )
    .expect("first step");

    let second = Step::<u32>::from_parts(
        &[first.clone()],
        request(50_000),
        output_pools(),
        vec![],
        None,
        None,
        vec![StepOutput::new(0, StepOutputIndex::Payment(0))],
        TransactionBalance::new(vec![], Zatoshis::const_from_u64(FEE_VALUE)).unwrap(),
        false,
        false,
    )
    .expect("valid payment-reference step");

    let steps = NonEmpty::from_vec(vec![first, second]).unwrap();
    let proposal = Proposal::multi_step(
        (),
        TargetHeight::from(BlockHeight::from_u32(101)),
        ConfirmationsPolicy::default(),
        steps,
    )
    .expect("SDK validated chain");

    let facts = extract_proposal_facts(&proposal).expect("inspect chained proposal");
    assert_eq!(facts.steps.len(), 2);
    assert!(facts.steps[1].has_transparent_inputs);
    assert!(!facts.steps[1].has_prior_shielded_funding);
}

#[test]
fn prior_ephemeral_change_is_recognized_as_transparent_funding() {
    let ephemeral_balance = TransactionBalance::new(
        vec![ChangeValue::ephemeral_transparent(Zatoshis::const_from_u64(CHANGE_VALUE))],
        Zatoshis::const_from_u64(FEE_VALUE),
    )
    .unwrap();
    let first = Step::<u32>::from_parts(
        &[],
        request(PAID_VALUE),
        output_pools(),
        vec![funding_input()],
        None,
        None,
        vec![],
        ephemeral_balance,
        false,
        false,
    )
    .expect("first step with ephemeral transparent change");

    let second = Step::<u32>::from_parts(
        &[first.clone()],
        request(20_000),
        output_pools(),
        vec![],
        None,
        None,
        vec![StepOutput::new(0, StepOutputIndex::Change(0))],
        TransactionBalance::new(vec![], Zatoshis::const_from_u64(FEE_VALUE)).unwrap(),
        false,
        false,
    )
    .expect("second step spending ephemeral transparent change");

    let proposal = Proposal::multi_step(
        (),
        TargetHeight::from(BlockHeight::from_u32(101)),
        ConfirmationsPolicy::default(),
        NonEmpty::from_vec(vec![first, second]).unwrap(),
    )
    .expect("SDK validated ephemeral chain");

    let facts = extract_proposal_facts(&proposal).expect("inspect chained change");
    assert!(facts.steps[1].has_transparent_inputs);
    assert!(!facts.steps[1].has_prior_shielded_funding);
}
