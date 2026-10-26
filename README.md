# LeakLens

**A pre-authorization privacy review for Zcash transaction proposals.**

LeakLens examines the destination pool, transparent inputs, shielded inputs, change pools, and available proposal metadata before a wallet proceeds with authorization. It presents concrete public disclosures separately from *possible* privacy correlations, so users can reconsider a payment route before committing to it.

> **Development status (October 26, 2026):** The project is a hackathon prototype. The current demo crate exposes a library-level `preview_review` interface and tests, not a released CLI or consumer-wallet integration. SDK proposal fixtures are structural; a funded transaction, completed authorization/signature, and network broadcast have **not** been demonstrated. Do not use the prototype to protect real funds.

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
4. `leaklens-demo`: present a review with the current `preview_review` library API.

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

There is currently **no verified `leaklens-demo` command-line binary**. Do not use `cargo run -p leaklens-demo` as a submission instruction until an executable target and documented flags exist.

See [Demo walkthrough](docs/demo.md), [Privacy model](docs/privacy-model.md), and [Submission proof checklist](docs/submission-checklist.md).

## Security boundaries

- No seed phrase, private key, or viewing-key upload is required for policy analysis.
- Zcash's consensus validation and zero-knowledge proof machinery remain the wallet/SDK's responsibility.
- An incomplete review fails closed within the controlled demo gate; LeakLens cannot prevent bypass through an unmodified external wallet.
- No mainnet spending or broadcast should occur in tests or recorded demonstrations.

## License

MIT. See `LICENSE`.
