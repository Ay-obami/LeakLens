//! One-time testnet wallet setup; does not send a transaction.
use anyhow::{Context, Result, ensure};
use leaklens_wallet::setup::initialize_disposable_testnet_wallet;
use std::{env, path::PathBuf};
use zcash_client_backend::proto::service::{
    BlockId, Empty, compact_tx_streamer_client::CompactTxStreamerClient,
};
use zcash_protocol::consensus::Network;

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("TLS crypto provider was already initialized incompatibly"))?;
    // Caller supplies exactly four positional arguments. Never accept seeds
    // or spending keys on the command line.
    let args: Vec<_> = env::args_os().skip(1).collect();
    ensure!(
        args.len() == 4,
        "usage: leaklens-wallet-init HTTPS_TESTNET_LIGHTWALLETD WALLET_DB SEED_AGE AGE_IDENTITY"
    );
    let endpoint = args[0].to_string_lossy().to_string();
    ensure!(
        endpoint.starts_with("https://"),
        "lightwalletd must use TLS HTTPS"
    );
    let db_path = PathBuf::from(&args[1]);
    let seed_path = PathBuf::from(&args[2]);
    let identity_path = PathBuf::from(&args[3]);

    let mut client = CompactTxStreamerClient::connect(endpoint)
        .await
        .context("connecting to testnet lightwalletd")?;
    let info = client
        .get_lightd_info(Empty {})
        .await
        .context("reading lightwalletd chain identity")?
        .into_inner();
    ensure!(
        matches!(info.chain_name.as_str(), "test" | "testnet"),
        "lightwalletd is not advertising Zcash testnet"
    );
    let latest = client
        .get_latest_block(zcash_client_backend::proto::service::ChainSpec::default())
        .await
        .context("getting Zcash testnet tip")?
        .into_inner();
    ensure!(
        latest.height > 100,
        "testnet tip too low for safe birthday margin"
    );
    let prior_height = latest.height - 100;
    let checkpoint = client
        .get_tree_state(BlockId {
            height: prior_height,
            hash: vec![],
        })
        .await
        .context("fetching Zcash testnet prior tree state")?
        .into_inner();
    ensure!(
        checkpoint.network == info.chain_name,
        "tree-state network disagrees with lightwalletd reported chain"
    );
    ensure!(
        checkpoint.height == prior_height,
        "tree-state height mismatch"
    );

    let (id, shielded) = initialize_disposable_testnet_wallet(
        &db_path,
        &seed_path,
        &identity_path,
        "LeakLens disposable testnet",
        checkpoint,
        prior_height,
    )?;
    println!("Testnet account: {id:?}");
    println!(
        "Shielded address: {}",
        shielded.encode(&Network::TestNetwork)
    );
    Ok(())
}
