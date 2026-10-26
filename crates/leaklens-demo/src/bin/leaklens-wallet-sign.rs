//! Explicitly approved, testnet-only wallet signing; never broadcasts.
//! A real SDK proposal is reviewed in memory before the seed is decrypted.
use anyhow::{Context, Result, ensure};
use leaklens_core::Policy;
use leaklens_demo::wallet_flow::{WalletReviewOutcome, review_wallet_proposal};
use leaklens_wallet::{
    proposal::propose_testnet_transfer, setup::decrypt_age_testnet_seed,
    storage::open_testnet_wallet,
};
use secrecy::ExposeSecret;
use std::{
    env,
    io::{self, Write},
    path::Path,
};
use zcash_client_backend::data_api::{Account, WalletRead, wallet::SpendingKeys};
use zcash_client_sqlite::util::SystemClock;
use zcash_keys::{
    address::Address,
    keys::{UnifiedAddressRequest, UnifiedSpendingKey},
};
use zcash_proofs::prover::LocalTxProver;
use zcash_protocol::{consensus::Network, value::Zatoshis};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    ensure!(
        args.len() == 5,
        "usage: leaklens-wallet-sign WALLET_DB SEED_AGE AGE_IDENTITY AMOUNT_ZATOSHIS"
    );
    let amount_u64: u64 = args[4].parse().context("invalid zatoshi amount")?;
    let amount = Zatoshis::from_u64(amount_u64).context("invalid amount")?;
    let mut db = open_testnet_wallet(Path::new(&args[1]))?;
    let accounts = db.get_account_ids()?;
    ensure!(
        accounts.len() == 1,
        "expected exactly one disposable testnet account"
    );
    let account = accounts[0];
    let destination = db
        .get_last_generated_address_matching(account, UnifiedAddressRequest::SHIELDED)?
        .context("no shielded address")?;
    let proposal =
        propose_testnet_transfer(&mut db, account, &Address::Unified(destination), amount)?;
    println!("Testnet self-transfer proposal: {amount_u64} zatoshis");
    for step in proposal.steps() {
        println!(
            "Estimated fee: {:?} zatoshis",
            step.balance().fee_required()
        );
    }
    let policy = Policy::default();
    let review = review_wallet_proposal(&proposal, &policy, io::stdin().lock(), &mut io::stdout())
        .map_err(|error| anyhow::anyhow!("privacy review failed: {error:?}"))?;
    let pending = match review {
        WalletReviewOutcome::Approved(pending) => pending,
        WalletReviewOutcome::Abort | WalletReviewOutcome::Revise => {
            println!("No signing performed");
            return Ok(());
        }
    };
    println!(
        "SECOND CONFIRMATION REQUIRED. Signing persists a transaction locally; it does not broadcast."
    );
    print!("Type SIGN {amount_u64} to authorize SDK signing, or anything else to abort: ");
    io::stdout().flush()?;
    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation)?;
    if confirmation.trim() != format!("SIGN {amount_u64}") {
        println!("Signing aborted; no seed decrypted");
        return Ok(());
    }
    let seed = decrypt_age_testnet_seed(Path::new(&args[2]), Path::new(&args[3]))?;
    ensure!(
        db.validate_seed(account, &seed)?,
        "encrypted seed does not match wallet account"
    );
    let metadata = db.get_account(account)?.context("account disappeared")?;
    let index = metadata
        .source()
        .key_derivation()
        .context("account is not seed-derived")?
        .account_index();
    let usk = UnifiedSpendingKey::from_seed(&Network::TestNetwork, seed.expose_secret(), index)
        .map_err(|error| anyhow::anyhow!("cannot derive wallet key: {error:?}"))?;
    let keys = SpendingKeys::from_unified_spending_key(usk);
    let prover = LocalTxProver::bundled();
    let mut rng = rand::rng();
    let ids = leaklens_demo::execute_reviewed_zcash_signing!(
        pending,
        &policy,
        &mut db,
        &Network::TestNetwork,
        &SystemClock,
        &mut rng,
        &prover,
        &prover,
        &keys
    )
    .map_err(|error| anyhow::anyhow!("approved SDK signing failed: {error:?}"))?;
    for id in ids.iter() {
        println!("Signed and stored testnet transaction: {id}");
    }
    println!("NOT BROADCAST. Sending is a distinct, unimplemented operation.");
    Ok(())
}
