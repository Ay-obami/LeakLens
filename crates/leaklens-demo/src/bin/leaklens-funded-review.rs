//! Interactive privacy review of a real, funded testnet wallet proposal.
//! This executable has no spending-key or broadcast functionality.
use anyhow::{Context, Result, ensure};
use leaklens_core::Policy;
use leaklens_demo::wallet_flow::{WalletReviewOutcome, review_wallet_proposal};
use leaklens_wallet::{proposal::propose_testnet_transfer, storage::open_testnet_wallet};
use std::{env, io, path::Path};
use zcash_client_backend::data_api::WalletRead;
use zcash_keys::{address::Address, keys::UnifiedAddressRequest};
use zcash_protocol::value::Zatoshis;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    ensure!(
        args.len() == 3,
        "usage: leaklens-funded-review WALLET_DB AMOUNT_ZATOSHIS"
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
        .context("shielded wallet address not found")?;
    let proposal =
        propose_testnet_transfer(&mut db, accounts[0], &Address::Unified(destination), amount)?;
    println!("REAL WALLET PROPOSAL — SELF-TRANSFER TEST ONLY");
    println!(
        "Amount: {:?} zatoshis; steps: {}",
        amount,
        proposal.steps().len()
    );
    for step in proposal.steps().iter() {
        println!(
            "Estimated fee: {:?} zatoshis",
            step.balance().fee_required()
        );
    }
    println!("No private key is loaded; this program cannot sign or broadcast.");
    let mut stdout = io::stdout().lock();
    let review = review_wallet_proposal(
        &proposal,
        &Policy::default(),
        io::stdin().lock(),
        &mut stdout,
    )
    .map_err(|error| anyhow::anyhow!("privacy review failed: {error:?}"))?;
    match review {
        WalletReviewOutcome::Abort => println!("User aborted; no transaction created"),
        WalletReviewOutcome::Revise => println!("Revision requested; no transaction created"),
        WalletReviewOutcome::Approved(_) => {
            println!("Review authorized; signing deliberately unavailable in this program")
        }
    }
    Ok(())
}
