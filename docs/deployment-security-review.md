# LeakLens hosted-demo security review

**Review date:** 2026-10-27

**Target:** public Vercel UI + Railway Rust policy API, Zcash testnet only
**Decision:** allow **non-signing, non-custodial public preview** after the release gates below. Do **not** expose funded signing or broadcast from a public endpoint.

## Trust boundaries

| Boundary | Trusted input | Prohibited input / action |
| --- | --- | --- |
| Browser -> review API | Explicitly entered, bounded, normalized proposal facts or synthetic fixtures | Seed phrases, spending keys, viewing keys, wallet databases, full transaction notes, authenticated signing claims |
| API -> `leaklens-core::assess` | Parsed facts with enforced size and enum constraints | Treating caller-supplied facts as proof of what an external wallet will actually sign |
| SDK adapter -> authorization gate | Facts and canonical bytes derived from the *same immutable* SDK proposal | Mixing user-entered facts with an unrelated executable proposal |
| Gate -> signer | Single-use approved review, current policy, identical canonical proposal, explicit wallet consent | A server-side `/sign` or `/broadcast` endpoint available to anonymous demo visitors |
| Wallet -> lightwalletd | Network-identified TLS **testnet** service, verified branch and expiry | User-selected arbitrary endpoints, mainnet fallback, bypassing explicit broadcast approval |

The public policy checker is informational. It **does not intercept an external wallet's payment**, and it must not promise to block wallet transactions that never invoke the LeakLens gate. The checker's input is supplied by the browser, not independently verified against an SDK proposal; label results accordingly.

## Existing code observations

1. The gate (`crates/leaklens-gate/src/lib.rs`) computes the policy assessment itself, denies incomplete data, compares the entire policy and a domain-separated fingerprint of canonical bytes before invoking a signing callback. Its docs explicitly require a locked/immutable SDK proposal through signing and acknowledge that alternate wallet paths can bypass the gate.
2. The local signing binary (`crates/leaklens-demo/src/bin/leaklens-wallet-sign.rs`) is interactive, expects exactly one testnet account, requires review and a separate exact-amount `SIGN ...` confirmation, and only decrypts `age` recovery material **after** both confirmations. It uses Zcash testnet consensus parameters.
3. The broadcast binary (`crates/leaklens-wallet/src/bin/leaklens-wallet-broadcast.rs`) reads a locally stored approved TXID, requires HTTPS, checks the remote chain name is `test`, verifies the current consensus branch and transaction expiry, then relays a transaction. Signing and broadcasting remain distinct operations.
4. The privacy model (`docs/privacy-model.md`) correctly distinguishes observable transparent disclosures from *possible* correlations; it does not claim to deanonymize shielded transfers.
5. `crates/leaklens-core::assess` operates on normalized facts. A public JSON API must cap payload size, step count, payment count, and string lengths **before** materializing unconstrained vectors. The current core crate is not a network input validator.

## Release-blocking gates for the hosted policy API

- [ ] **No secrets in deployment:** never commit, copy, or mount the funded Ubuntu wallet's encrypted seed, `age` identity, spending key, SQLite wallet backup, or recovery material into Railway or Vercel. Existing testnet wallet stays local unless a new, separately authorized, tightly limited service is designed and reviewed.
- [ ] **No signing endpoints:** public API contains only health, fixture metadata, and deterministic assessment; no shell execution, seed decryption, wallet initialization, proposal persistence, signer invocation, or transaction broadcast reachable from HTTP.
- [ ] **Request validation:** reject unknown enum variants as incomplete or return a clear client error; reject oversized bodies, excessive steps/payments/change pools, malformed JSON, and non-finite/invalid amounts without panics or excessive allocation. Reject HTTP methods other than documented ones.
- [ ] **Privacy:** disable request-body logging, tracing of user-entered addresses/payment amounts, analytics capture of proposal content, and query-string submission of private details. Error messages contain neither submitted JSON nor paths, environment variables, keys, or panic backtraces.
- [ ] **Browser and transport:** HTTPS for deployment; explicitly scoped CORS origins (not wildcard for any future credentialed endpoint), restrictive CSP on the UI, no remote script injection, no `innerHTML` on untrusted values, and no client-side third-party tracking of form contents.
- [ ] **Truthful UX:** always show `fixture` / `manual preview` status, pool type, findings' certainty, incomplete/fail-closed states, and a prominent disclaimer that a clean preview is not a guarantee of anonymity or proof a wallet's final transaction matches the preview.
- [ ] **Availability:** limit request concurrency and rate, set reasonable timeouts, expose an unauthenticated health endpoint that never discloses secrets or wallet state, and test deployment within Railway Free's 512 MiB RAM and $1/month post-trial usage allowance.
- [ ] **Persistence:** the public analyzer is stateless and must start with no persistent volume. If a separate wallet service is ever added, its database must use a persistent mount with monitored capacity, encryption/backup strategy, and an independent threat review.
- [ ] **CI:** run Rust tests, formatting, strict Clippy, frontend typecheck/lint/tests/build, and HTTP smoke tests on the exact release commit. Confirm all endpoints reject negative cases and `/health` returns safe minimal metadata.
- [ ] **Supply chain:** lock dependencies, pin the Rust toolchain, avoid build-time execution of untrusted network scripts, configure secret scanning, and verify generated frontend files do not embed private environment values.

## Test matrix

| Scenario | Required result |
| --- | --- |
| Shielded-only fixture | No implemented public-disclosure findings; do **not** display `100% private` |
| Transparent recipient | `LL-001` warning for publicly visible recipient and value |
| Transparent inputs | `LL-002` warning about exposed input history |
| Shielded -> transparent route | `LL-003` as **possible correlation**, never definite linkage |
| Unified address selecting transparent receiver | `LL-004` warning |
| Transparent change | `LL-007` warning |
| Missing/contradictory/unknown facts | `LL-005`, `complete=false`, no claim of safety |
| Empty proposal | `LL-005`, `complete=false` |
| Huge/malformed request | Bounded 4xx; no panic or data echo |
| `GET /sign`, `POST /broadcast`, arbitrary paths | No execution; 404/405 |
| Restart stateless API | Same deterministic results; no wallet data created |

## Deferred: publicly usable signing

Cloud-hosting the funded testnet signer is a *different risk profile* from deploying an analyzer. A safe future implementation needs an explicit wallet ownership model (prefer user-local/external wallet signing), authenticated per-user sessions, anti-CSRF, replay protection, transaction amount/destination caps, rate limits, spend budgets, per-account isolation, auditing without sensitive logs, key-management design, emergency disable, and independent assessment of browser-to-wallet trust. Even a disposable *testnet* faucet-style signer must not grant anonymous callers unrestricted spends.

## Go/no-go

The Vercel + Railway launch can be described as **public interactive policy-preview deployment** once its validation, privacy, security, functionality and resource gates have been verified. It must **not** be described as a hosted funded-wallet flow until a separate, authenticated, authorized testnet signing service has actually been deployed and exercised end to end. Keep the independently verified local testnet signing/confirmation proof as a separate demonstration and explain the distinction to Zecathon judges.
