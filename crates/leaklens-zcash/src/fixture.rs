//! Deterministic, non-spendable test fixture. Never use this to authorize or broadcast.

use std::collections::BTreeMap;

use orchard::{keys::FullViewingKey, keys::SpendingKey};
use zcash_client_backend::{
    address::Address,
    data_api::wallet::ConfirmationsPolicy,
    fees::{StandardFeeRule, TransactionBalance},
    proposal::Proposal,
    wallet::WalletTransparentOutput,
};
use zcash_keys::address::UnifiedAddress;
use zcash_protocol::{
    PoolType,
    consensus::{BlockHeight, Network},
    value::Zatoshis,
};
use zcash_transparent::{
    address::TransparentAddress,
    bundle::{OutPoint, TxOut},
};
use zip321::{Payment, TransactionRequest};

/// A validated SDK proposal describing one testnet shielded payment.
/// It intentionally contains no spendable inputs: it is a structural fixture,
/// not a wallet-funded transaction and cannot be sent to the network.
pub fn sample_shielded_proposal() -> Proposal<StandardFeeRule, u32> {
    let spending_key = SpendingKey::from_bytes([7; 32]).expect("test key");
    let receiver = FullViewingKey::from(&spending_key).address_at(0u32, zip32::Scope::External);
    let unified = UnifiedAddress::from_receivers(Some(receiver), None, None, None, None)
        .expect("test unified address");
    let address = Address::Unified(Box::new(unified)).to_zcash_address(&Network::TestNetwork);
    let payment = Payment::new(address, Some(Zatoshis::ZERO), None, None, None, vec![])
        .expect("zero-value test payment");
    let request = TransactionRequest::new(vec![payment]).expect("valid request");
    let balance = TransactionBalance::new(vec![], Zatoshis::ZERO).expect("zero balance");
    Proposal::single_step(
        request,
        BTreeMap::from([(0, PoolType::ORCHARD)]),
        vec![],
        None,
        BlockHeight::from_u32(100),
        balance,
        StandardFeeRule::Zip317,
        BlockHeight::from_u32(101).into(),
        ConfirmationsPolicy::default(),
        false,
        false,
    )
    .expect("valid proposal")
}

/// SDK-validated transparent-payment fixture with a synthetic UTXO. There is
/// no corresponding spending key; this cannot sign or broadcast a transaction.
pub fn sample_transparent_proposal() -> Proposal<StandardFeeRule, u32> {
    let input_address = TransparentAddress::PublicKeyHash([0x42; 20]);
    let input = WalletTransparentOutput::<()>::from_parts(
        OutPoint::new([0x43; 32], 0),
        TxOut::new(
            Zatoshis::const_from_u64(100_000),
            input_address.script().into(),
        ),
        Some(BlockHeight::from_u32(90)),
        None,
        None,
        None,
    )
    .expect("synthetic test output");
    let recipient = Address::Transparent(TransparentAddress::PublicKeyHash([0x44; 20]));
    let request = TransactionRequest::new(vec![
        Payment::new(
            recipient.to_zcash_address(&Network::TestNetwork),
            Some(Zatoshis::const_from_u64(90_000)),
            None,
            None,
            None,
            vec![],
        )
        .expect("valid synthetic payment"),
    ])
    .expect("valid request");
    let balance =
        TransactionBalance::new(vec![], Zatoshis::const_from_u64(10_000)).expect("fee balance");
    Proposal::single_step(
        request,
        BTreeMap::from([(0, PoolType::TRANSPARENT)]),
        vec![input],
        None,
        BlockHeight::from_u32(100),
        balance,
        StandardFeeRule::Zip317,
        BlockHeight::from_u32(101).into(),
        ConfirmationsPolicy::default(),
        false,
        false,
    )
    .expect("valid synthetic transparent proposal")
}
