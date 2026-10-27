//! Construct a transaction proposal from the real persisted testnet wallet.
//!
//! This module does **not** sign, persist a signed transaction, or broadcast.
//! It only selects confirmed shielded notes from a synchronized wallet DB.

use anyhow::{Result, anyhow, ensure};
use zcash_client_backend::{
    data_api::{
        InputSource,
        wallet::{ConfirmationsPolicy, propose_standard_transfer_to_address},
    },
    fees::StandardFeeRule,
    proposal::Proposal,
};
use zcash_keys::address::Address;
use zcash_protocol::{ShieldedPool, consensus::Network, value::Zatoshis};

use crate::storage::TestnetWalletDb;

/// The note reference remains the SDK backend's own type, preserving the
/// selected notes for LeakLens inspection and eventual gated construction.
pub type WalletTransferProposal =
    Proposal<StandardFeeRule, <TestnetWalletDb as InputSource>::NoteRef>;

/// Propose one testnet payment using the wallet's persisted account state.
///
/// This requires completed chain synchronization, commitment-tree witnesses
/// and spendable confirmed shielded notes. The recommended default
/// confirmation policy (3 trusted / 10 untrusted) is used.
///
/// No keys are accepted here. A spending key is required only at the
/// later, separately approved signing step.
pub fn propose_testnet_transfer(
    wallet_db: &mut TestnetWalletDb,
    account_id: zcash_client_sqlite::AccountUuid,
    destination: &Address,
    amount: Zatoshis,
) -> Result<WalletTransferProposal> {
    ensure!(
        amount.is_positive(),
        "payment amount must be greater than zero"
    );
    propose_standard_transfer_to_address::<_, _, std::convert::Infallible>(
        wallet_db,
        &Network::TestNetwork,
        StandardFeeRule::Zip317,
        account_id,
        ConfirmationsPolicy::default(),
        destination,
        amount,
        None,
        None,
        ShieldedPool::Orchard,
        None,
        None,
    )
    .map_err(|error| {
        anyhow!(
            "testnet proposal failed (wallet sync and funded shielded notes required): {error:?}"
        )
    })
}
