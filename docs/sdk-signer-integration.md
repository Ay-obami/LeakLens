# Optional real SDK signing seam

This file accompanies `src/sdk_signer.rs` and must be integrated by adding
`mod sdk_signer;` to the `leaklens-demo` crate root. It targets the pinned
`zcash_client_backend = 0.24.0` dependency with `orchard` enabled.

1. Produce a funded, wallet-backed `Proposal<StandardFeeRule, N>` using the
   wallet's normal input-selection path. The deterministic SDK fixture is
   **not** funded and cannot be used for a real send.
2. Inspect it with `PendingZcashReview::inspect(&proposal, policy.clone())`.
3. Show the findings, enforce explicit acknowledgement where required, and
   do not offer a way to skip the review in the host wallet's send path.
4. After approval, invoke `execute_reviewed_zcash_signing!` with the pending
   session, current policy, the wallet's existing database, network parameters,
   Sapling provers, `SpendingKeys`, `OvkPolicy`, and optional expiry height.
5. The macro rechecks the canonical proposal and policy through the consumed
   session, then directly invokes
   `zcash_client_backend::data_api::wallet::create_proposed_transactions`.
   Its `Result` is wrapped in `GateRunError<SDK error>`.

## Important boundaries

- The SDK call **constructs, proves, signs, and persists** transactions in the
  wallet database. It does **not broadcast** them. Broadcast is a separate
  deliberate wallet action after checking the SDK result.
- The macro does **not** create keys or pass them to a service; the wallet must
  already have access to its own spending keys.
- Review protects the exact borrowed proposal. It does not prevent a host
  wallet from bypassing LeakLens elsewhere unless the host makes this gate the
  **only** transaction-construction/signing path.
- A funded database, witnesses, valid proving parameters, synced chain state,
  and a wallet's normal error handling are required. This repository's fixture
  alone cannot prove successful signing.
- Compatibility is scoped to SDK v0.24.0: later SDK versions add a `Clock`
  and RNG to this function's parameters; upgrading requires a deliberate edit.
- This integration has no automated real-wallet signing test. In addition to
  existing gate tests, it requires wallet integration tests on a testnet
  database before a production deployment.
