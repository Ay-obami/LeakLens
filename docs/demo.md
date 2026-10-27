# LeakLens — Two-Minute Demo Walkthrough

**Status (October 27, 2026):** Script and verification storyboard. LeakLens has both a fixture-only non-signing demo and a separate **funded Zcash testnet** flow. The latter signed and relayed an NU7 Ironwood self-transfer, mined in block 4,546,012 and observed with three confirmations. Present the fixture behavior and the real on-chain evidence distinctly. Never display wallet recovery material.

## Preflight

1. Check out the reviewed commit; record `git rev-parse HEAD` and Rust toolchain version.
2. Run `cargo +1.95.0 test --workspace`, preserving its exit status.
3. Confirm tests use isolated, non-sensitive proposal fixtures and do not broadcast.
4. Choose a test fixture that represents shielded-only delivery and one with a transparent destination or change output.
5. Verify the preview correctly distinguishes public disclosure from possible linkage; keep the source test name and displayed rule code visible.

## Narrated storyboard (target 120 seconds)

| Time | On-screen evidence | Narration purpose |
|---|---|---|
| 0–12s | Title + plain-language pain/solution | A proposed Zcash payment can select a public route. LeakLens reveals that before wallet signing. |
| 12–32s | Run fixture `leaklens-demo -- --transparent` | Show actual `LL-001` and `LL-002` public-disclosure warnings. Label this as a non-signing SDK fixture. |
| 32–47s | Enter `CONTINUE` where `ACKNOWLEDGE` is required | Demonstrate that the privacy gate denies incomplete acknowledgment. |
| 47–60s | Run fixture `leaklens-demo -- --shielded` | Show the narrower message: no known exposure under implemented rules, not an anonymity guarantee. |
| 60–83s | Display a **redacted** testnet wallet balance and actual proposal/fee output | Identify 1 TAZ funded Ironwood account, 0.01 TAZ self-transfer and 0.0001 TAZ fee, without showing wallet paths or keys. |
| 83–107s | Show recorded, verified signing-gate output and transaction receipt | Explain `CONTINUE` plus exact-amount `SIGN` authorization, NU7 branch check, accepted relay and mined block 4,546,012; label evidence replay if not a continuous live recording. |
| 107–120s | GitHub repository, limits, and result | Testnet-only prototype, not automatic integration into other wallets; link to reproducible verification evidence. |

## Example presentation language

“This test creates or supplies a structured Zcash SDK transaction proposal. LeakLens examines the selected output pool before authorization. When the proposal uses a transparent recipient, the review reports publicly observable payment information. A separate rule identifies a *possible* linkability signal when shielded funding crosses into public outputs. Neither rule claims to decrypt a shielded transaction or identify its owner.”

“Beyond the demonstration fixtures, we funded a disposable Zcash testnet wallet, constructed a shielded self-transfer through the actual SDK, completed LeakLens review and exact-amount signing authorization, and relayed a NU7 transaction mined at block 4,546,012. That demonstrates one controlled wallet integration on testnet—not a universal privacy guarantee or production integration.”

## Verified CLI demonstration

The two paths below were run on the Ubuntu development laptop on October 26, 2026. They construct valid SDK proposal *structures* but do not use spendable wallet funds.

### Path A — transparent payment triggers warnings

```bash
cargo +1.95.0 run --bin leaklens-demo -- --transparent
```

Expected visible findings:

- `LL-002 [PUBLIC DISCLOSURE]`: transparent funding inputs and their public history are observable.
- `LL-001 [PUBLIC DISCLOSURE]`: transparent recipient and payment amount are public.

Enter `REVISE` to abort. Repeat and enter `ACKNOWLEDGE` to reach `NO-OP CALLBACK REACHED — NOT SIGNING OR BROADCASTING`. This is a demo of risk acknowledgment, not payment authorization.

### Path B — shielded-only proposal

```bash
cargo +1.95.0 run --bin leaklens-demo -- --shielded
```

The fixture produces `No known exposure was flagged by this policy`. The wording is deliberately narrower than an anonymity guarantee. Enter `CONTINUE` to reach the same NO-OP callback, or `REVISE` to abort.

Both paths fingerprint the complete standard-proposal SDK Protobuf and check it before entering the protected callback. They do not sign, persist, or broadcast a transaction.

## Evidence to capture

- Commit SHA and matching PR/merge status.
- Full `cargo +1.95.0 test --workspace` summary and any failing tests.
- At least one real SDK proposal adaptation test, with fixture provenance explained.
- Public-output and incomplete-review findings in actual output.
- The separate funded/testnet evidence: proposal and fee, gated signing result, NU7 relay acceptance, transaction ID `6620a249d10c8ecc0f54e5932f9309ce683cc5c5f4fde3a14cbb6cfa644c25e8`, mined block 4,546,012, three observed confirmations.
- Explicit limitations: fixture demo never signs; live testnet flow uses one disposable wallet; production wallets and mainnet are not integrated.
- Keep all wallet database paths, seed, identity, and any spending/viewing keys out of the video.

For a Zecathon submission, supply a repository, concise project description, functioning demo evidence, and the requested short video. Check the organizer's authenticated submission form for the current exact requirements and track before final submission: <https://thezecathon.com/>.
