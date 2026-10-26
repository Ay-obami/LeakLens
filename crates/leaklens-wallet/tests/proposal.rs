//! A database-backed, deliberately unfunded negative-path test.
//! The test proves that proposal generation does not fake spendable inputs.

use leaklens_wallet::{proposal::propose_testnet_transfer, storage::open_testnet_wallet};
use zcash_client_sqlite::AccountUuid;
use zcash_keys::address::Address;
use zcash_protocol::value::Zatoshis;
use zcash_transparent::address::TransparentAddress;

#[test]
fn real_wallet_without_synced_funds_cannot_create_transfer_proposal() {
    let directory = tempfile::tempdir().expect("temporary wallet directory");
    let mut wallet = open_testnet_wallet(&directory.path().join("wallet.sqlite"))
        .expect("real SQLite wallet bootstrap");
    // No account or funding information has been inserted. This unknown UUID
    // is deliberately chosen to exercise the genuine SDK selection failure.
    let unknown_account = AccountUuid::default();
    let destination = Address::Transparent(
        TransparentAddress::PublicKeyHash([0x22; 20])
    );

    let result = propose_testnet_transfer(
        &mut wallet,
        unknown_account,
        &destination,
        Zatoshis::from_u64(50_000).unwrap(),
    );
    assert!(result.is_err(), "unfunded unsynced wallet must not invent a proposal");
}

#[test]
fn zero_value_is_rejected_before_attempting_note_selection() {
    let directory = tempfile::tempdir().expect("temporary wallet directory");
    let mut wallet = open_testnet_wallet(&directory.path().join("wallet.sqlite"))
        .expect("wallet DB bootstrap");
    let destination = Address::Transparent(
        TransparentAddress::PublicKeyHash([0x22; 20])
    );

    let failure = propose_testnet_transfer(
        &mut wallet,
        AccountUuid::default(),
        &destination,
        Zatoshis::ZERO,
    ).expect_err("zero-value payment must fail before SDK selection");
    assert!(failure.to_string().contains("greater than zero"));
}
