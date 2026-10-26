//! Deterministic, non-spendable test fixture. Never use this to authorize or broadcast.

use std::collections::BTreeMap;

use orchard::{keys::FullViewingKey, keys::SpendingKey};
use zcash_keys::address::UnifiedAddress;
use zcash_client_backend::{
    address::Address,
    data_api::wallet::ConfirmationsPolicy,
    fees::{TransactionBalance, StandardFeeRule},
    proposal::Proposal,
};
use zcash_protocol::{consensus::{BlockHeight, Network}, value::Zatoshis, PoolType};
use zip321::{Payment, TransactionRequest};

/// A validated SDK proposal describing one testnet shielded payment.
/// It intentionally contains no spendable inputs: it is a structural fixture,
/// not a wallet-funded transaction and cannot be sent to the network.
pub fn sample_shielded_proposal() -> Proposal<StandardFeeRule, u32> {
    let spending_key = SpendingKey::from_bytes([7; 32]).expect("test key");
    let receiver = FullViewingKey::from(&spending_key).address_at(0u32, zip32::Scope::External);
    let unified = UnifiedAddress::from_receivers(Some(receiver), None, None)
        .expect("test unified address");
    let address = Address::Unified(unified).to_zcash_address(&Network::TestNetwork);
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
