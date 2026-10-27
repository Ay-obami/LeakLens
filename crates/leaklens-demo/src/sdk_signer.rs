//! Opt-in execution of an approved proposal with the real librustzcash signer.
//!
//! This integration deliberately does not provide an automatic signing command:
//! the host wallet must supply its existing database, chain parameters, provers,
//! spending keys. No key is created or loaded
//! here. `create_proposed_transactions` persists signed transaction(s) in the
//! wallet database; it does **not** broadcast them to the Zcash network.

/// Invoke the NU7-compatible `zcash_client_backend` signer **only** after the
/// [`PendingZcashReview`](crate::PendingZcashReview) has passed its gate.
///
/// The macro expands at the wallet integration call site. Database-specific
/// associated error types are inferred, while the SDK's two otherwise
/// unconstrained proposal-selection error types are pinned to `Infallible`.
///
/// All expressions are evaluated at most once. Crucially, the signer receives
/// `reviewed_proposal`, a reference from the consumed pending review; callers
/// cannot substitute another proposal after acknowledgment. This is an
/// *opt-in, side-effecting* operation: an approved call signs and persists.
/// The outgoing viewing key policy is fixed to `OvkPolicy::Sender`: the sender
/// can recover external shielded outputs. Callers cannot silently substitute
/// a custom viewing key or discard policy after the proposal review.
///
/// ```ignore
/// // Production wallet integration only, after presenting findings and asking
/// // the user for explicit consent via pending.review_mut().acknowledge().
/// let signed_ids = leaklens_demo::execute_reviewed_zcash_signing!(
///     pending, &active_policy, &mut wallet_db, &network_params,
///     &clock, &mut rng, &spend_prover, &output_prover, &spending_keys,
/// )?;
/// // The wallet still must explicitly retrieve and broadcast these txs.
/// ```
#[macro_export]
macro_rules! execute_reviewed_zcash_signing {
    (
        $pending:expr, $active_policy:expr, $wallet_db:expr, $params:expr,
        $clock:expr, $rng:expr,
        $spend_prover:expr, $output_prover:expr, $spending_keys:expr $(,)?
    ) => {{
        let pending_review = $pending;
        let active_policy = $active_policy;
        let wallet_db = $wallet_db;
        let params = $params;
        let clock = $clock;
        let rng = $rng;
        let spend_prover = $spend_prover;
        let output_prover = $output_prover;
        let spending_keys = $spending_keys;
        pending_review.execute(active_policy, |reviewed_proposal| {
            // InputsErrT and ChangeErrT are unconstrained by the SDK's
            // parameters: they represent errors of proposal *creation*,
            // which this execution path does not perform. Pin them to
            // Infallible so the compiler does not hit E0282/E0283.
            ::zcash_client_backend::data_api::wallet::create_proposed_transactions::<
                _,
                _,
                ::core::convert::Infallible,
                _,
                ::core::convert::Infallible,
                _,
            >(
                wallet_db,
                params,
                clock,
                rng,
                spend_prover,
                output_prover,
                spending_keys,
                ::zcash_client_backend::wallet::OvkPolicy::Sender,
                reviewed_proposal,
                // Expiry is derived from the already-reviewed proposal.
                // A caller-controlled override would change transaction
                // semantics after fingerprint authorization.
                None,
            )
        })
    }};
}
