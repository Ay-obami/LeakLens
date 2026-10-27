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

## Live funded-signing and NU7 broadcast checkpoint — 2026-10-26

A separate, disposable testnet wallet received a shielded 1.00000000 TAZ
funding note in block 4,544,325. On-chain receipt was verified by SDK wallet
synchronization. A 0.01 TAZ shielded self-transfer was proposed, passed the
interactive privacy gate and the exact-amount second consent, and was signed
and stored locally with SDK-generated transaction ID
`1b69288d5e91a57a038c3d5c48728cb8f760644feb2670f581ea61a83807e337`.
No secret or recovery material was placed in this repository.

The first signed transaction (`b00c6900b9f55a1b9d05fa6103d7972d5a6ff8030033de584cad73cd0b4f53bd`)
expired unbroadcast; wallet synchronization reconciled the reserved funds
back to 1.00000000 spendable TAZ. The standalone broadcaster refused that
expired transaction (`expiry 4,544,405` vs tip `4,544,670`).

The replacement transaction was rejected by public testnet relay at height
4,544,671 (`-25`, incorrect consensus branch ID), **not accepted or mined**.
The pinned `zcash_protocol 0.10.5` library selects `Nu6_3` on testnet height
4,544,675 because its network activation table still has no NU7 activation.
NU7 activated on public testnet at **height 4,465,026** in October 2026.
At this earlier checkpoint, the pre-migration broadcaster **failed closed before
network submission** when the tip was on NU7. A compatible SDK upgrade and
fresh signature were therefore required; the next section records the upgrade.
Never alter a signed transaction's branch ID or claim a rejected broadcast succeeded.

## NU7-compatible live testnet relay — 2026-10-27

The isolated SDK migration (PR #24, commit `5d00570`) updated to
`zcash_protocol 0.11.0-pre.1` and its compatible wallet SDK components.
Its consensus table recognizes NU7 activation at public testnet height
4,465,026; a regression test covers the activation boundary. The upgraded
Rust wallet binaries compiled locally, the workspace tests and strict Clippy
passed, and GitHub Actions passed.

A mode-0600 copy of the funded wallet database was migrated successfully
without exposing recovery material. After bounded lightwalletd synchronization,
its sole testnet account showed 100,000,000 spendable Ironwood zatoshis.
The wallet derived a 1,000,000-zatoshi shielded self-transfer proposal with
10,000-zatoshi fee, completed the LeakLens pre-signing privacy review and both
interactive authorization steps, and locally signed transaction
`6620a249d10c8ecc0f54e5932f9309ce683cc5c5f4fde3a14cbb6cfa644c25e8`.

At network tip 4,546,008 the standalone broadcaster verified that this
transaction and the expected next-block consensus branch were both `Nu7`.
The TLS testnet lightwalletd relay returned success (`error_code == 0`).
After wallet synchronization, the wallet database recorded the transaction
as mined at Zcash testnet height **4,546,012** (well before expiry 4,546,041).
A follow-up sync at tip 4,546,014 showed **3 confirmations** and reconciled
98,990,000 spendable Ironwood zatoshis, 1,000,000 pending-spendability
self-transfer zatoshis, and the 10,000-zatoshi fee.
The original funded wallet database remained untouched during this test.

## Unverified and still required

- [x] A funded, wallet-backed Zcash proposal and complete signing execution through the gate.
- [x] SDK update for NU7 testnet and fresh, compatible signing.
- [x] Testnet relay accepted the signed NU7 transaction.
- [x] Verify the transaction is mined in public testnet block 4,546,012.
- [ ] An actually recorded two-minute video and its final accessible link.
- [ ] Final hackathon submission URL, submitted source SHA, and confirmation receipt.

**Important:** The synthetic fixture-based demo remains a separate no-op path.
The live wallet workflow is now proven through testnet mining; additional
confirmation depth, the recorded demo, and final submission evidence remain outstanding.
