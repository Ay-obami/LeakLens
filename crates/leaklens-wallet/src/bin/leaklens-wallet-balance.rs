//! Read-only, pool-aware Zcash testnet wallet balance report.
//! No spending keys are loaded or transactions constructed.
use anyhow::{Context, Result, ensure};
use leaklens_wallet::storage::open_testnet_wallet;
use std::{env, path::Path};
use zcash_client_backend::data_api::{WalletRead, wallet::ConfirmationsPolicy};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    ensure!(args.len() == 2, "usage: leaklens-wallet-balance WALLET_DB");
    let db = open_testnet_wallet(Path::new(&args[1]))?;
    let summary = db
        .get_wallet_summary(ConfirmationsPolicy::default())?
        .context("wallet has no synchronized summary yet")?;
    println!("Wallet accounts: {}", summary.account_balances().len());
    for balance in summary.account_balances().values() {
        println!(
            "Ironwood spendable zatoshis: {:?}",
            balance.ironwood_balance().spendable_value()
        );
        println!(
            "Ironwood pending zatoshis: {:?}",
            balance.ironwood_balance().value_pending_spendability()
        );
        println!(
            "Orchard spendable zatoshis: {:?}",
            balance.orchard_balance().spendable_value()
        );
        println!(
            "Sapling spendable zatoshis: {:?}",
            balance.sapling_balance().spendable_value()
        );
    }
    Ok(())
}
