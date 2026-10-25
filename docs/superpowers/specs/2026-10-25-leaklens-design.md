# LeakLens — Integrated Zcash Transaction Privacy Firewall

**Status:** User-approved architecture, October 25, 2026.

## Intent

Prevent accidental exposure during Zcash payments by inspecting the wallet's *actual* transaction proposal prior to authorization. Explain observable privacy risks and allow a user to revise or explicitly acknowledge them.

## Goals

- Thin Rust adapter to the official `zcash_client_backend` proposal interface.
- Pure, deterministic policy engine with traceable findings.
- A controlled wallet harness with a pre-authorization gate and understandable preview.
- Proposal-bound approval: editing any materially relevant proposal content invalidates the approval.
- Tests with real SDK-generated proposals and a demonstration of an approved signing path.

## Non-goals

- Deanonymization of shielded transactions; privacy probability scores; custody of funds.
- Handling seed phrases or signing keys in the rules engine.
- Patching multiple consumer wallets or promising universal interception.
- Mainnet fund movement in the demo.

## Constraints and behavior

- Inspect all proposal steps, including transparent inputs, destination pools, and change outputs.
- Findings distinguish **certain public disclosure**, **possible correlation**, and **inspection incomplete**.
- Explicit warnings: LL-001 transparent destination, LL-002 transparent funding inputs, LL-003 shielded-to-transparent crossing, LL-004 Unified Address selecting a transparent pool, LL-005 missing/unsupported analysis, LL-006 changed proposal after approval.
- Never label a payment 'private' merely because its address has a Unified prefix.
- If inspection fails, fail closed within the controlled signing path; report the limitation and never emit a clean bill of health.
- The authorization gate verifies a canonical, domain-separated proposal fingerprint and policy version immediately before continuing; a stale acknowledgment cannot be reused.
- Keep wallet signing keys in the wallet's existing boundary; the analyzer receives no key material.
- User may acknowledge a known warning, but not bypass an error preventing reliable inspection.
- No unsafe inference that visible amount/timing implies a proven common owner.
- Submission target: October 28, 2026 at 23:59 UTC. Prioritize integrated, tested functionality over appearance.

## Selected approach and alternatives

Selected: a separate LeakLens Rust crate integrated into a controlled `librustzcash` wallet workflow through a thin adapter. This preserves a reusable policy engine and produces a real pre-signing demo.

Rejected: standalone checker (cannot gate the authorization path); large fork of a mature wallet (higher regression and integration burden within the deadline).

## Planned interface and boundaries

`leaklens-core` owns normalized, versioned proposal facts, `PrivacyAssessment`, `Finding`, `PolicyVersion`, and `assess(&ProposalFacts, &Policy) -> PrivacyAssessment`.

`leaklens-zcash` extracts normalized facts from a typed `zcash_client_backend::proposal::Proposal<FeeRule, NoteRef>` and computes a fingerprint covering all security-relevant serialized proposal fields. Do not define a hash over `Debug` formatting or arbitrary JSON field order. SDK type and serializer details require confirmation in the initial SDK spike.

`leaklens-gate` binds the generated assessment, proposal fingerprint, and acknowledgment to the exact proposal delivered to the wallet's construction/signing API. The gate must be the only invocation path in the demo harness.

`leaklens-demo` provides a preview showing selected payment pool, exact public disclosures, possible linkability, and explicit revise/acknowledge decisions.

## Acceptance proof

Tests demonstrate real SDK proposals for shielded-to-shielded, shielded-to-transparent, transparent input, Unified Address with selected transparent pool where supported, multi-step proposals, malformed/unsupported data, and stale approvals. A test proves the signing operation cannot be reached through the harness when inspection fails or authorization is absent. No mainnet broadcast required.

## Open implementation judgments

- Select and pin a mutually compatible SDK/crate toolchain after running a compile spike.
- Confirm whether the controlled harness can exercise transaction *construction* without network broadcast in the available environment; if not, provide an honest testnet-only dependency explanation.
- Select a CLI/TUI preview first if a graphical integration would put the core demonstration at risk.
