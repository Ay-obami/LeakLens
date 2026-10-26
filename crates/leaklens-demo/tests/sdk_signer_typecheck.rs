//! Compile-only compatibility check for the real librustzcash v0.24 signer.
//!
//! Deliberately not invoked by the test runner: it requires an external
//! funded wallet DB, existing keys and Sapling provers. Rust still typechecks
//! the generic body during `cargo test --no-run` / `cargo test`.

use leaklens_core::Policy;
use leaklens_demo::{PendingZcashReview, execute_reviewed_zcash_signing};
use sapling::prover::{OutputProver, SpendProver};
use zcash_client_backend::{
    data_api::{WalletCommitmentTrees, WalletWrite, wallet::SpendingKeys},
    wallet::OvkPolicy,
};
use zcash_protocol::consensus::Parameters;

#[allow(dead_code, clippy::too_many_arguments)]
fn real_sdk_signer_signature_must_compile<DbT, ParamsT, N>(
    pending: PendingZcashReview<'_, N>,
    active_policy: &Policy,
    wallet_db: &mut DbT,
    params: &ParamsT,
    spend_prover: &impl SpendProver,
    output_prover: &impl OutputProver,
    spending_keys: &SpendingKeys,
    ovk_policy: OvkPolicy,
) where
    DbT: WalletWrite + WalletCommitmentTrees,
    ParamsT: Parameters + Clone,
{
    let _result = execute_reviewed_zcash_signing!(
        pending,
        active_policy,
        wallet_db,
        params,
        spend_prover,
        output_prover,
        spending_keys,
        ovk_policy,
    );
}

// This intentionally defines no runnable tests. The body above is typechecked
// by the compiler but never instantiated or executed.
