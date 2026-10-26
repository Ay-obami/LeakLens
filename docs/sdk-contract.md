# LeakLens SDK contract (Issue #1)

- Toolchain: Rust 1.95.0, edition 2024.
- Pin: `zcash_client_backend = 0.24.0` with `orchard` and `transparent-inputs`.
- Matching crates: `zcash_protocol = 0.10.5`, `zcash_primitives = 0.30.1`, `zcash_address = 0.13.0`, `zip321 = 0.9.0`, `orchard = 0.15.0`, `zip32 = 0.2`.
- Source SDK proposal type: `zcash_client_backend::proposal::Proposal<FeeRuleT, NoteRef>`.
- Inspect all `proposal.steps()`. For each `Step`, inspect `transaction_request()`, `payment_pools()`, `transparent_inputs()`, `shielded_inputs()`, `balance().proposed_change()`, and `prior_step_inputs()`.
- Selected destination value pool is `step.payment_pools()[payment_index]`, **not** the address prefix.
- Normalized domain types owned by `leaklens-core` (Issue #2): `ProposalFacts`, `StepFacts`, `PaymentFacts`, `PoolKind`, `PrivacyAssessment`, and `assess(&ProposalFacts, &Policy)`.
- Gate identity (Issue #4): `ProposalFingerprint([u8;32])` must bind **all relevant SDK proposal fields**, not only `ProposalFacts`. A canonical, versioned serialization strategy is still open pending actual compile/test and must not be falsely marked verified.
- Test fixture `sample_shielded_proposal` is intended to exercise the SDK validated proposal constructor. It uses deterministic *non-spendable zero-value* data; it is not evidence of wallet funding, signing, or network broadcasting.

## Security rule

Any missing or incompatible field mapping returns inspection incomplete; do not silently infer a shielded payment from a Unified Address prefix.
