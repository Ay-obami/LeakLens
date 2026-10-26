//! Compile-only proof that the SDK signer accepts the concrete SQLite-backed
//! testnet wallet and its *actual* received-note identifier type.
//!
//! This function is never invoked; no keys, wallet database, or network
//! transaction are created by running this test binary.

use leaklens_core::Policy;
use leaklens_demo::{PendingZcashReview, execute_reviewed_zcash_signing};
use leaklens_wallet::storage::TestnetWalletDb;
use sapling::prover::{OutputProver, SpendProver};
use zcash_client_backend::data_api::{InputSource, wallet::SpendingKeys};
use zcash_protocol::consensus::Network;

type SqliteNoteRef = <TestnetWalletDb as InputSource>::NoteRef;

#[allow(dead_code, clippy::too_many_arguments)]
fn sqlite_wallet_signing_boundary_typechecks(
    pending: PendingZcashReview<'_, SqliteNoteRef>,
    active_policy: &Policy,
    wallet_db: &mut TestnetWalletDb,
    params: &Network,
    spend_prover: &impl SpendProver,
    output_prover: &impl OutputProver,
    spending_keys: &SpendingKeys,
) {
    // Rust typechecks this body during compilation without executing it.
    // The macro must accept the database's real NoteRef, not a synthetic u32.
    let _result = execute_reviewed_zcash_signing!(
        pending,
        active_policy,
        wallet_db,
        params,
        spend_prover,
        output_prover,
        spending_keys,
    );
}
