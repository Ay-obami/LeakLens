//! Opt-in, bounded Zcash testnet sync; never signs or broadcasts.
use std::{env, path::PathBuf};

use anyhow::{Result, ensure};
use leaklens_wallet::{storage::open_testnet_wallet, sync::{SyncConfig, sync_testnet_wallet}};

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("TLS crypto provider was already initialized incompatibly"))?;
    let args: Vec<_> = env::args_os().skip(1).collect();
    ensure!(args.len() == 4,
        "usage: leaklens-wallet-sync HTTPS_TESTNET_LIGHTWALLETD WALLET_DB BLOCK_CACHE MAX_BLOCKS");
    let endpoint = args[0].to_string_lossy().to_string();
    let wallet_path = PathBuf::from(&args[1]);
    let cache_path = PathBuf::from(&args[2]);
    let budget: usize = args[3].to_string_lossy().parse()?;
    ensure!(wallet_path.is_file(), "testnet wallet database must already exist");
    ensure!(wallet_path != cache_path, "wallet and compact-block cache must be different files");
    let config = SyncConfig::new(endpoint, budget)?;
    let mut db = open_testnet_wallet(&wallet_path)?;
    let report = sync_testnet_wallet(&mut db, &cache_path, &config).await?;
    println!("Testnet tip: {}", u32::from(report.remote_tip));
    println!("Blocks scanned this run: {}", report.scanned_blocks);
    Ok(())
}
