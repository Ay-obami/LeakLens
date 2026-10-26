# LeakLens — Reproducible Verification Record

**Recorded:** 2026-10-26 (Zecathon build period)  
**Verified implementation commit:** `76dd680e7d5e0acaca59dab96226a4d80c54b325`  
**Repository:** <https://github.com/Ay-obami/LeakLens>  
**Toolchain:** `rustc 1.95.0 (59807616e 2026-04-14)`, Ubuntu.

This is a snapshot of the implementation's **test and demonstration evidence**, not a claim of real funded transaction signing.

## Checks executed

| Verification | Result |
|---|---|
| `cargo +1.95.0 test --workspace` | PASS — 49 runnable tests across the workspace; one optional signer usage doctest explicitly ignored |
| `cargo +1.95.0 fmt --all -- --check` | PASS |
| `cargo +1.95.0 clippy --workspace --all-targets -- -D warnings` | PASS |
| GitHub Actions Rust verification | PASS on [SDK mutation PR #21](https://github.com/Ay-obami/LeakLens/pull/21), workflow run [40307743725](https://github.com/Ay-obami/LeakLens/actions/runs/40307743725) |

The SDK signer typecheck harness is compiled by workspace tests but **is never called**. Its compilation confirms SDK function signature compatibility only, not a successful signature, spend or broadcast.

## Wallet integration sprint — 2026-10-26

[Draft PR #23](https://github.com/Ay-obami/LeakLens/pull/23) introduced a
persistent testnet SQLite wallet (`zcash_client_sqlite 0.22.0`), caller-owned
account import, shielded-only addresses, actual SDK wallet-backed transfer
proposal selection, and an explicit approval boundary around the exact proposal.

**Verified commit:** `e4a1f0139cb47cb0b9902f8d0792aa8b164bd06f`  
**GitHub Actions:** [40313168982](https://github.com/Ay-obami/LeakLens/actions/runs/40313168982)

- PASS: Rustfmt, entire workspace test suite, strict Clippy.
- PASS: persistent wallet database tests and unfunded proposal rejection.
- PASS: reviewed SDK fixture cannot enter guarded execution without the appropriate approval.
- NOT VERIFIED: concrete SQLite note-reference signer execution or funded testnet send.
- NOT VERIFIED: network synchronization, subtree roots, reorg recovery, broadcast and confirmation.

The signer macro remains typechecked generically, but it is not yet exercised
with actual SQLite wallet state. This is an incremental integration milestone;
Issue #5 must remain open until a real wallet-backed end-to-end path is verified.

## Live CLI commands tested on Ubuntu

```bash
cargo +1.95.0 run --bin leaklens-demo -- --transparent
cargo +1.95.0 run --bin leaklens-demo -- --shielded
```

Observed transparent fixture output includes `LL-002` (observable public funding inputs) and `LL-001` (public recipient and value). `ACKNOWLEDGE` is required to reach `NO-OP CALLBACK REACHED — NOT SIGNING OR BROADCASTING`; entering `CONTINUE` instead is not accepted.

The shielded-only fixture reports no *known* exposure under the implemented rules and accepts `CONTINUE` to the same non-signing callback. This result is **not** a guarantee of anonymity.

## Additional security proof

- SDK-native proposal canonicalization is deterministic for the same typed proposal.
- Amount, destination, and fee/balancing-change modifications produce different proposal fingerprints.
- A destination change after approval is rejected before invoking the protected callback.
- Incomplete proposals and missing warning acknowledgment fail closed in tested flows.
- The optional signer seam fixes `OvkPolicy::Sender` and uses the proposal-derived expiry (`None` override).

## Unverified and still required

- [ ] A funded, wallet-backed Zcash proposal and complete signing execution through the gate.
- [ ] Testnet broadcast or confirmation (if attempted, never use production funds).
- [ ] An actually recorded two-minute video and its final accessible link.
- [ ] Final hackathon submission URL, submitted source SHA, and confirmation receipt.

**Important:** The repository contains real SDK proposal structures and an optional compile-checked signing integration, but no end-to-end funded wallet send has been demonstrated. The live CLI uses synthetic unspendable fixtures and a no-op callback. Do not present test or compile evidence as a completed network transaction.
