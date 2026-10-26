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

The signer macro is now compile-tested with the concrete SQLite wallet and
its real received-note reference type. This checks compatibility but never
executes a signer or handles spending keys.

**Second verified integration commit:** `b16ddc60acd817b492f7bca8dc8b61aa7d82755a`  
**CI:** [40314477035](https://github.com/Ay-obami/LeakLens/actions/runs/40314477035)
— Rustfmt, all workspace tests, and strict Clippy PASS.

The compiled sync module performs TLS lightwalletd testnet checks, bounded
compact-block fetch/cache/scan, and Sapling/Orchard subtree-root insertion.
However, **live testnet synchronization is not yet demonstrated**. Chain
reorganizations are reported as failures rather than safely recovered, and
the lightwalletd operator learns connection IP and query metadata.
 This is an incremental integration milestone;
Issue #5 must remain open until a real wallet-backed end-to-end path is verified.

## Live disposable testnet wallet checkpoint — 2026-10-26

**Environment:** Authorized Ubuntu laptop; dedicated verification clone and
private wallet directory. **Network:** Zcash testnet via a TLS lightwalletd.

- The wallet initializer compiled and connected to `https://testnet.zec.rocks:443`.
- An age-encrypted, 32-byte random recovery seed was created locally, without
  writing plaintext to disk or printing key material. The age identity,
  encrypted seed and SQLite databases are stored in a private `0700` directory.
  Individual file permissions were verified/secured at `0600`.
- The wallet was initialized with a tree-state birthday 100 blocks behind the
  tip, and a shielded-only `utest1...` receiving address was generated.
- The real lightwalletd synchronizer reported **tip 4,544,285** and
  **100 compact blocks scanned** on its first successful bounded run.
- The TLS connection fix was committed as
  `411027a4a7228bb089067436d19708864f414ec4`.
- No testnet funds have been received, no wallet-backed funded proposal was
  constructed, and no signing or broadcasting was performed.

The local encryption identity must be backed up securely by the owner before
funding. Disclosure or loss of that identity together with the encrypted seed
can compromise recoverability; never commit these files to Git or send them to
support services. Live sync success does not establish reorg-recovery quality.

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
