//! Read-only self-transfer testnet proposal; no signing or broadcast.
use anyhow::{Context, Result, ensure};
use leaklens_wallet::{proposal::propose_testnet_transfer, storage::open_testnet_wallet};
use std::{env, path::Path};
use zcash_client_backend::data_api::WalletRead;
use zcash_keys::{address::Address, keys::UnifiedAddressRequest};
use zcash_protocol::value::Zatoshis;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    ensure!(
        args.len() == 3,
        "usage: leaklens-wallet-propose WALLET_DB AMOUNT_ZATOSHIS"
    );
    let amount: u64 = args[2].parse().context("amount must be integer zatoshis")?;
    let amount = Zatoshis::from_u64(amount).context("amount outside monetary range")?;
    let mut db = open_testnet_wallet(Path::new(&args[1]))?;
    let accounts = db.get_account_ids()?;
    ensure!(
        accounts.len() == 1,
        "expected exactly one disposable testnet account"
    );
    let destination = db
        .get_last_generated_address_matching(accounts[0], UnifiedAddressRequest::SHIELDED)?
        .context("shielded address not found")?;
    let proposal = propose_testnet_transfer(
        &mut db,
        accounts[0],
        &Address::Unified(Box::new(destination)),
        amount,
    )?;
    println!("Proposal created without signing or broadcasting");
    println!("Requested self-transfer zatoshis: {:?}", amount);
    println!("Proposal steps: {}", proposal.steps().len());
    for (index, step) in proposal.steps().iter().enumerate() {
        println!(
            "Step {} fee zatoshis: {:?}",
            index + 1,
            step.balance().fee_required()
        );
    }
    Ok(())
}
