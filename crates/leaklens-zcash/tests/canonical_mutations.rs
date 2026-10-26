//! Security regression: SDK-native proposal serialization, not hand-built bytes.
//! To place in crates/leaklens-zcash/tests/; add `leaklens-gate = { path = "../leaklens-gate" }`
//! under leaklens-zcash dev-dependencies.

use std::{cell::Cell, collections::BTreeMap};

use leaklens_core::Policy;
use leaklens_gate::{GateError, GateReview, GateRunError, fingerprint_canonical_proposal};
use leaklens_zcash::{encode_canonical_standard_proposal, extract_proposal_facts};
use zcash_client_backend::{
    data_api::wallet::{ConfirmationsPolicy, TargetHeight},
    fees::{ChangeValue, StandardFeeRule, TransactionBalance},
    proposal::Proposal,
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

fn sdk_payment(recipient_tag: u8, payment_value: u64, fee_value: u64) -> Proposal<StandardFeeRule, u32> {
    let change_value = INPUT_VALUE
        .checked_sub(payment_value + fee_value)
        .expect("balanced synthetic proposal");
    let funding_address = TransparentAddress::PublicKeyHash([0x31; 20]);
    let funding = WalletTransparentOutput::<()>::from_parts(
        OutPoint::new([0x32; 32], 0),
        TxOut::new(
            Zatoshis::const_from_u64(INPUT_VALUE),
            funding_address.script().into(),
        ),
        Some(BlockHeight::from_u32(90)),
        None, None, None,
    ).expect("synthetic UTXO");

    let recipient = Address::Transparent(TransparentAddress::PublicKeyHash([recipient_tag; 20]));
    let request = TransactionRequest::new(vec![
        Payment::new(
            recipient.to_zcash_address(&Network::TestNetwork),
            Some(Zatoshis::const_from_u64(payment_value)),
            None, None, None, vec![],
        ).expect("payment"),
    ]).expect("request");
    let change = if change_value == 0 {
        vec![]
    } else {
        vec![ChangeValue::transparent(Zatoshis::const_from_u64(change_value))]
    };
    let balance = TransactionBalance::new(change, Zatoshis::const_from_u64(fee_value))
        .expect("balanced change and fee");

    Proposal::single_step(
        request,
        BTreeMap::from([(0usize, PoolType::TRANSPARENT)]),
        vec![funding],
        None,
        BlockHeight::from_u32(100),
        balance,
        StandardFeeRule::Zip317,
        TargetHeight::from(BlockHeight::from_u32(101)),
        ConfirmationsPolicy::default(),
        false,
        false,
    ).expect("typed SDK proposal")
}

#[test]
fn canonical_sdk_encoding_is_deterministic_for_same_proposal() {
    let proposal = sdk_payment(0x41, 70_000, 10_000);
    let first = encode_canonical_standard_proposal(&proposal);
    let second = encode_canonical_standard_proposal(&proposal);
    assert_eq!(first, second);
    assert_eq!(fingerprint_canonical_proposal(&first), fingerprint_canonical_proposal(&second));
}

#[test]
fn canonical_fingerprint_changes_for_amount_destination_and_fee() {
    let baseline = sdk_payment(0x41, 70_000, 10_000);
    let changed_amount = sdk_payment(0x41, 80_000, 10_000);
    let changed_destination = sdk_payment(0x42, 70_000, 10_000);
    let changed_fee = sdk_payment(0x41, 70_000, 20_000);

    let base_bytes = encode_canonical_standard_proposal(&baseline);
    let base_hash = fingerprint_canonical_proposal(&base_bytes);
    for changed in [&changed_amount, &changed_destination, &changed_fee] {
        let bytes = encode_canonical_standard_proposal(changed);
        assert_ne!(base_bytes, bytes, "SDK canonical bytes must capture economic change");
        assert_ne!(base_hash, fingerprint_canonical_proposal(&bytes));
    }
}

#[test]
fn actual_sdk_proposal_mutation_denies_authorization_before_callback() {
    let original = sdk_payment(0x41, 70_000, 10_000);
    let different_destination = sdk_payment(0x42, 70_000, 10_000);
    let facts = extract_proposal_facts(&original).expect("real SDK facts");
    let mut review = GateReview::inspect(
        &facts,
        &Policy::default(),
        &encode_canonical_standard_proposal(&original),
    ).expect("review");
    review.acknowledge().expect("transparent payment requires acknowledgment");

    let signer_called = Cell::new(false);
    let outcome = review.authorize_and_execute(
        &Policy::default(),
        || Ok::<Vec<u8>, ()>(encode_canonical_standard_proposal(&different_destination)),
        || { signer_called.set(true); Ok::<(), ()>(()) },
    );
    assert_eq!(outcome, Err(GateRunError::Gate(GateError::ProposalChanged)));
    assert!(!signer_called.get(), "signing callback must not run for changed SDK proposal");
}
