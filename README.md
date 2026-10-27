# LeakLens

**A pre-authorization privacy review for Zcash transaction proposals.**

LeakLens examines the destination pool, transparent inputs, shielded inputs, change pools, and available proposal metadata before a wallet proceeds with authorization. It presents concrete public disclosures separately from *possible* privacy correlations, so users can reconsider a payment route before committing to it.

> **Development status (October 27, 2026):** LeakLens is a hackathon prototype with a fixture-only, non-signing demo **and a separate real Zcash testnet wallet flow**. The wallet flow used an independently funded disposable 1 TAZ account, reviewed and signed a 0.01 TAZ shielded self-transfer through the authorization gate, and broadcast an NU7-compatible transaction confirmed in testnet block 4,546,012. See [verification evidence](docs/verification.md). This is not a production wallet or a guarantee of anonymity. Never use real funds.

## What LeakLens checks

| Code | Signal | Interpretation |
|---|---|---|
| `LL-001` | Payment selects a transparent output | Recipient address and output value are publicly visible |
| `LL-002` | Transparent funding input | Its public UTXO history remains observable |
| `LL-003` | Shielded funding plus a transparent output | Amount/timing may invite correlation; no identity link is proven |
| `LL-004` | Unified Address actually selects a transparent receiver | Unified formatting alone did not ensure shielded delivery |
| `LL-005` | Unrecognized or contradictory proposal facts | The review is incomplete and must not be treated as a pass |
| `LL-007` | Transparent change output | Change recipient/value would be public |

The policy is deliberately conservative. “No findings” means none of these *implemented* rules found a known exposure in the inspected proposal—not that the payment is anonymous, untraceable, or safe against every adversary.

## Architecture

1. `leaklens-zcash`: translate official `librustzcash` proposal structures into explicitly typed facts.
2. `leaklens-core`: run deterministic privacy rules without handling spending keys.
3. `leaklens-gate`: bind a review/acknowledgment to the exact proposal and policy version; reject mismatched or incomplete decisions.
4. `leaklens-wallet`: persist a **testnet-only** account, sync compact blocks, construct SDK-native transactions, and expose separate proposal and broadcast CLIs.
5. `leaklens-demo`: provide the non-signing SDK-fixture demo **and** a guarded wallet signer that requires review and exact-amount confirmation before accessing locally encrypted recovery material.

The intended wallet integration point is **after proposal generation but before authorization**. The harness must route every authorization path through the gate for that guarantee to hold; installing a library alone does not intercept other wallets.

## Verify locally

Prerequisites: Rust toolchain compatible with the repository's pinned SDK. The current test invocation on the development machine is:

```bash
cargo +1.95.0 test --workspace
```

Also run, with the toolchain configured for this checkout:

```bash
cargo +1.95.0 fmt --all -- --check
cargo +1.95.0 clippy --workspace --all-targets -- -D warnings
```

These are **verification commands, not a statement that this checkout currently passes**. Record fresh command output and exit status before claiming green CI.

Two interactive, non-signing CLI paths have been verified on the Ubuntu development laptop:

```bash
cargo +1.95.0 run --bin leaklens-demo -- --shielded
cargo +1.95.0 run --bin leaklens-demo -- --transparent
```

The shielded fixture presents no findings under the implemented rules and accepts `CONTINUE` or `REVISE`. The transparent fixture displays `LL-002` (public funding inputs) and `LL-001` (public recipient and value), requiring `ACKNOWLEDGE` to proceed or `REVISE` to cancel. Both reach at most a **NO-OP CALLBACK**—neither signs, funds, nor broadcasts a payment.

The independent, **testnet-only** wallet flow uses these binaries (paths are caller-owned; no seed is committed):

```text
leaklens-wallet-balance WALLET_DB
leaklens-wallet-propose WALLET_DB AMOUNT_ZATOSHIS
leaklens-wallet-sign WALLET_DB SEED_AGE AGE_IDENTITY AMOUNT_ZATOSHIS
leaklens-wallet-broadcast HTTPS_TESTNET_LIGHTWALLETD WALLET_DB APPROVED_TXID
```

Signing is interactive and distinct from broadcasting. The broadcaster verifies
that the endpoint advertises testnet, that the signed transaction has the
current consensus branch, and that its expiry has sufficient headroom. Only
the disposable testnet wallet has been exercised, not a production integration.

See [Demo walkthrough](docs/demo.md), [Privacy model](docs/privacy-model.md), [verified testnet evidence](docs/verification.md), and [Submission proof checklist](docs/submission-checklist.md).

## Security boundaries

- No seed phrase, private key, or viewing-key upload is required for policy analysis.
- Zcash's consensus validation and zero-knowledge proof machinery remain the wallet/SDK's responsibility.
- An incomplete review fails closed within the controlled demo gate; LeakLens cannot prevent bypass through an unmodified external wallet.
- No mainnet spending or broadcast should occur in tests or recorded demonstrations.

## License

MIT. See `LICENSE`.
