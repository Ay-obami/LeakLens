//! One-time testnet account bootstrap from an existing age-encrypted seed.
//! The caller must obtain TreeState from a verified testnet lightwalletd.

use std::{path::Path, process::{Command, Stdio}};
use anyhow::{Context, Result, ensure};
use secrecy::SecretVec;
use zcash_client_backend::{data_api::AccountBirthday, proto::service::TreeState};
use zcash_client_sqlite::AccountUuid;
use zcash_keys::address::UnifiedAddress;
use crate::storage::{create_testnet_account, next_shielded_testnet_address, open_testnet_wallet};

/// Decrypt a pre-existing age-encrypted 32-byte seed entirely in memory.
/// The raw seed and private key never enter command arguments or disk writes.
pub fn decrypt_age_testnet_seed(encrypted_seed_path: &Path, identity_path: &Path) -> Result<SecretVec<u8>> {
    ensure!(encrypted_seed_path.is_file(), "encrypted seed file must exist");
    ensure!(identity_path.is_file(), "age identity file must exist");
    let output = Command::new("age")
        .arg("--decrypt")
        .arg("--identity").arg(identity_path)
        .arg(encrypted_seed_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .context("launching age to decrypt existing seed")?;
    ensure!(output.status.success(), "age failed to decrypt testnet seed");
    ensure!(output.stdout.len() == 32, "decrypted testnet seed must be exactly 32 bytes");
    Ok(SecretVec::new(output.stdout))
}

/// Initialize an empty testnet DB using existing encrypted seed and the prior
/// block TreeState. No secrets or spending keys are returned or printed.
/// Checkpoint network identity and authenticity must be verified by the caller.
/// For restored seeds with past funds, use a historical birthday and rescan.
pub fn initialize_disposable_testnet_wallet(
    db_path: &Path,
    encrypted_seed_path: &Path,
    identity_path: &Path,
    account_name: &str,
    prior_block_tree_state: TreeState,
    expected_prior_block_height: u64,
) -> Result<(AccountUuid, UnifiedAddress)> {
    ensure!(!account_name.trim().is_empty(), "account name cannot be empty");
    ensure!(!db_path.exists(), "wallet database already exists; refusing duplicate initialization");
    ensure!(db_path != encrypted_seed_path && db_path != identity_path,
        "wallet database and secret paths must differ");
    ensure!(prior_block_tree_state.height == expected_prior_block_height,
        "chain state height does not match explicitly confirmed testnet height");
    let birthday = AccountBirthday::from_treestate(prior_block_tree_state, None)
        .context("decoding lightwalletd account birthday chain state")?;
    let seed = decrypt_age_testnet_seed(encrypted_seed_path, identity_path)?;
    let mut wallet_db = open_testnet_wallet(db_path)?;
    let (account_id, _spending_key) =
        create_testnet_account(&mut wallet_db, account_name, &seed, &birthday)?;
    let shielded_address = next_shielded_testnet_address(&mut wallet_db, account_id)?;
    Ok((account_id, shielded_address))
}
