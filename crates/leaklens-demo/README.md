# LeakLens SDK Fixture Demonstration

Run from the integrated Cargo workspace:

```bash
cargo run -p leaklens-demo --bin leaklens-demo
```

The demonstration uses a deterministic, **SDK-validated but unfunded** zero-value Zcash proposal. It passes the real typed proposal through the SDK adapter and privacy policy engine, displays findings, requests a user decision, and exercises `GateReview` with a harmless callback that prints **NOT SIGNING**.

This is **not a wallet integration**, signature, transaction broadcast, or proof of deployable pre-signing enforcement. The demo marker used for the gate fingerprint is not a complete canonical proposal encoding. Do not reuse this marker or the callback for a real transaction. Before real authorization, an SDK integration must provide a complete canonical representation of the immutable proposal, enforce the gate as the only authorization path, and keep all changes locked until signing finishes.

Tests:

```bash
cargo test -p leaklens-demo
```

Expected controls: type `CONTINUE` to reach the non-signing callback for the supplied shielded fixture; any other input cancels. Other policy-fixture tests demonstrate the `ACKNOWLEDGE`, `REVISE`, and fail-closed paths without a wallet signing key.
