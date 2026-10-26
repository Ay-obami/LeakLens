//! Testnet-only broadcast of an explicitly selected, locally signed transaction.
//! No keys, signing, proposals, or mainnet fallback are available here.
use anyhow::{Context, Result, bail, ensure};
use leaklens_wallet::storage::open_testnet_wallet;
use std::{env, path::Path};
use zcash_client_backend::{
    data_api::WalletRead,
    proto::service::{
        ChainSpec, Empty, RawTransaction, compact_tx_streamer_client::CompactTxStreamerClient,
    },
};
use zcash_protocol::{
    TxId,
    consensus::{BlockHeight, BranchId, Network},
};

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("failed to initialize TLS cryptography provider"))?;
    let args: Vec<String> = env::args().collect();
    ensure!(
        args.len() == 4,
        "usage: leaklens-wallet-broadcast HTTPS_ENDPOINT WALLET_DB APPROVED_TXID"
    );
    ensure!(
        args[1].starts_with("https://"),
        "TLS lightwalletd endpoint required"
    );
    let approved_txid = TxId::from_hex(&args[3]).context("invalid approved transaction ID")?;
    let db = open_testnet_wallet(Path::new(&args[2]))?;
    let transaction = db
        .get_transaction(approved_txid)?
        .context("approved signed transaction not stored in this wallet")?;
    ensure!(
        transaction.txid() == approved_txid,
        "wallet transaction ID mismatch"
    );
    let expiry = u32::from(transaction.expiry_height());
    let mut raw = Vec::new();
    transaction
        .write(&mut raw)
        .context("serializing selected signed transaction")?;
    ensure!(!raw.is_empty(), "refusing empty transaction");

    let mut client = CompactTxStreamerClient::connect(args[1].clone())
        .await
        .context("connecting to TLS testnet lightwalletd")?;
    let network = client
        .get_lightd_info(Empty::default())
        .await?
        .into_inner()
        .chain_name;
    ensure!(
        network == "test",
        "refusing to broadcast on non-testnet network"
    );
    let tip: u32 = client
        .get_latest_block(ChainSpec::default())
        .await?
        .into_inner()
        .height
        .try_into()
        .context("invalid chain tip")?;
    // NU7 activated on public Zcash testnet at height 4,465,026. The pinned
    // consensus library has no NU7 testnet activation and signs Nu6_3 txs.
    // Never send those transactions to an NU7 consensus node.
    ensure!(
        tip < 4_465_026,
        "NU7 is active on public testnet; pinned wallet SDK lacks NU7 signing support. Upgrade the consensus SDK and re-sign before broadcast"
    );
    println!("Approved signed tx: {approved_txid}");
    println!("Current testnet height: {tip}; transaction expiry: {expiry}");
    let expected_branch = BranchId::for_height(
        &Network::TestNetwork,
        BlockHeight::from_u32(tip.saturating_add(1)),
    );
    println!(
        "Transaction branch: {:?}; expected testnet branch: {:?}",
        transaction.consensus_branch_id(),
        expected_branch
    );
    ensure!(
        transaction.consensus_branch_id() == expected_branch,
        "signed transaction has incorrect consensus branch ID; do not send"
    );
    ensure!(
        expiry > tip.saturating_add(3),
        "signed transaction expired or too close to expiry; do not send"
    );
    let response = client
        .send_transaction(RawTransaction {
            data: raw,
            height: 0,
        })
        .await
        .context("submitting approved transaction to testnet")?
        .into_inner();
    if response.error_code != 0 {
        bail!(
            "testnet rejected transaction, code {}: {}",
            response.error_code,
            response.error_message
        );
    }
    println!("Accepted for testnet relay: {approved_txid}");
    println!("Not yet mined; monitor confirmations separately.");
    Ok(())
}
