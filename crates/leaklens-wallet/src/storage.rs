//! Testnet-only persistent wallet storage built on the official librustzcash
//! SQLite backend. Using a production wallet database is out of scope.

use std::path::Path;

use anyhow::{Context, Result};
use rand_core::OsRng;
use rusqlite::Connection;
use secrecy::SecretVec;
use zcash_client_backend::data_api::{AccountBirthday, WalletWrite};
use zcash_client_sqlite::{WalletDb, util::SystemClock, wallet::init::init_wallet_db};
use zcash_keys::{
    keys::{UnifiedAddressRequest, UnifiedSpendingKey},
    address::UnifiedAddress,
};
use zcash_protocol::consensus::Network;

/// A persistent testnet database. Never point this at an existing mainnet DB.
pub type TestnetWalletDb = WalletDb<Connection, Network, SystemClock, OsRng>;

/// Open and migrate an existing testnet wallet DB, or initialize a new one.
///
/// It is safe to call this repeatedly. If an existing seed-derived DB needs
/// a seed for migration, this returns a migration error without attempting to
/// recover or read the seed from the environment or disk.
pub fn open_testnet_wallet(path: &Path) -> Result<TestnetWalletDb> {
    let mut wallet_db = WalletDb::for_path(path, Network::TestNetwork, SystemClock, OsRng)
        .with_context(|| format!("opening testnet wallet DB at {}", path.display()))?;
    init_wallet_db(&mut wallet_db, None)
        .context("initializing or migrating Zcash testnet wallet DB")?;
    Ok(wallet_db)
}

/// Import a caller-owned seed as a new account without storing the seed in
/// LeakLens. The caller is responsible for secure backup and lifecycle.
///
/// The birthday must come from authenticated chain state at or before the
/// first block relevant to this account; incorrect birthdays can miss funds.
pub fn create_testnet_account(
    wallet_db: &mut TestnetWalletDb,
    account_name: &str,
    seed: &SecretVec<u8>,
    birthday: &AccountBirthday,
) -> Result<(zcash_client_sqlite::AccountUuid, UnifiedSpendingKey)> {
    let (account_id, spending_key) = wallet_db
        .create_account(account_name, seed, birthday, None)
        .context("creating a testnet wallet account")?;
    Ok((account_id, spending_key))
}

/// Allocate a new shielded-only Unified Address. Transparent receivers are
/// intentionally omitted to avoid silent receiver downgrade.
pub fn next_shielded_testnet_address(
    wallet_db: &mut TestnetWalletDb,
    account_id: zcash_client_sqlite::AccountUuid,
) -> Result<UnifiedAddress> {
    let (address, _diversifier_index) = wallet_db
        .get_next_available_address(account_id, UnifiedAddressRequest::SHIELDED)
        .context("deriving a shielded-only testnet address")?
        .context("unknown wallet account")?;
    Ok(address)
}
