# LeakLens Testnet Wallet Integration — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development. Tasks are split by nonoverlapping source ownership, reviewed and verified independently.

**Goal:** Have an actual persistent Zcash testnet wallet generate an SDK transaction proposal, present LeakLens findings, and reach the real SDK signing-and-persistence API only following verified consent.

**Architecture:** Add `leaklens-wallet` as the persistent, testnet-only SQLite wallet host using stable `zcash_client_sqlite=0.22.0` and `zcash_client_backend=0.24.0`. Keep review/approval in `leaklens-demo::wallet_flow`, reuse the immutable `PendingZcashReview` and fixed-expiry/fixed-OVK signer seam. Synchronization and actual signing must be separately demonstrated; no automatic broadcast.

**Tech Stack:** Rust 1.95.0; Zcash SDK 0.24; SQLite 0.22; ZIP-317; Rustfmt, Clippy, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-25-leaklens-design.md`.

## Global Constraints

- Testnet only; never open or mutate an existing mainnet wallet database.
- Do not generate, print, copy, export or persist user seed phrases in logs or repository.
- Do not sign merely because a proposal exists: inspection, preview, consent and fresh canonical fingerprint verification are required first.
- SDK signing always uses `OvkPolicy::Sender` and proposal-derived expiry.
- Do not broadcast by default; any eventual testnet broadcast requires independent approval.
- Do not substitute synthetic fixtures for funded-wallet integration when describing the result.
- Keep `zcash-devtool` as a reference only: current upstream source resolves backend 0.25 pre-release APIs incompatible with the pinned 0.24 signing call.
- Changes from parallel workers are staged on draft PR #23 and merge only after full workspace tests, Rustfmt, Clippy and independent security review.

## Review Focus

1. Unknown or unsynced wallet state: proposing a payment must return a clear error, never create fake spendable notes.
2. Invalid wallet DB path or migration: fail without overwriting/replacing another wallet.
3. Wrong user acknowledgment (transparent payment): signer callback never runs.
4. Stale or modified proposal bytes: callback cannot sign alternate proposal.
5. Wallet-derived spend keys and network parameters: remain in wallet host, never leak into policy engine or output.

---

### Task 1 — Persistent Testnet Wallet DB

**Files:** `crates/leaklens-wallet/src/{lib,storage}.rs`, `crates/leaklens-wallet/tests/storage.rs`, crate/root manifests.

**Interface:** `open_testnet_wallet(&Path) -> Result<TestnetWalletDb>`; `create_testnet_account(&mut TestnetWalletDb, name, &SecretVec<u8>, &AccountBirthday)`; `next_shielded_testnet_address(...)`.

- [x] Write storage persistence and invalid path tests.
- [x] Implement isolated storage adapter.
- [ ] Run focused test and fix pinned API compiler errors.
- [ ] Verify migrations do not overwrite existing data and secret never logs.

### Task 2 — Wallet-Origin Proposal Review

**Files:** `crates/leaklens-demo/src/wallet_flow.rs`, `crates/leaklens-demo/src/lib.rs`, `crates/leaklens-demo/tests/wallet_flow.rs`.

**Interface:** `review_wallet_proposal(&Proposal<StandardFeeRule, NoteRef>, &Policy, input, output) -> Result<WalletReviewOutcome<NoteRef>, WalletReviewError>` with Abort, Revise, and Approved(PendingZcashReview).

- [x] Write four contract tests.
- [x] Observe expected compile-red: E0432 missing `wallet_flow`, GitHub Actions run 40311112093.
- [ ] Implement minimal review adapter.
- [ ] Run focused and workspace tests; confirm callback receives exact borrowed proposal.

### Task 3 — Proposal Creation from Synced Wallet State

**Files:** `crates/leaklens-wallet/src/proposal.rs` and dedicated tests.

**Interface:** Given wallet DB, account ID, destination and amount, call official SDK proposal selection with ZIP-317 and nonzero confirmation policy. Never fabricate selected inputs.

- [ ] Write test proving unsynced wallet cannot yield a spendable proposal.
- [ ] Implement adapter for pinned stable SDK.
- [ ] Verify successful wallet-derived proposal if a synced funded testnet wallet becomes available.

### Task 4 — Network Sync

**Files:** `crates/leaklens-wallet/src/sync.rs` and tests.

- [ ] Write test for no-network and interrupted sync behavior.
- [ ] Port only compatible stable SDK lightwalletd sync operations.
- [ ] Persist chain state safely; explain testnet server metadata/privacy limitations.

### Task 5 — Guarded Signing Integration

**Files:** `crates/leaklens-wallet/src/send.rs` or `crates/leaklens-demo/src/wallet_runner.rs` and tests.

- [ ] Write test proving rejection does not invoke signer; consent passes identical proposal.
- [ ] Wire `Approved(PendingZcashReview)` to `execute_reviewed_zcash_signing!` with caller-owned wallet DB and keys.
- [ ] Validate error propagation and persisted transaction IDs; no broadcast side effect.
- [ ] Where funded testnet state is unavailable, honestly mark signing execution unverified.

### Task 6 — Security and Delivery

**Files:** integration README and `docs/verification.md`.

- [ ] Perform independent code/security review.
- [ ] Run `cargo +1.95.0 test --workspace`, Rustfmt, strict Clippy and GitHub Actions.
- [ ] Record actual boundary reached, source SHA, limitations and demo steps.
- [ ] Merge passing PR; only close Issue #5 if actual wallet-backed signing acceptance is proven.
