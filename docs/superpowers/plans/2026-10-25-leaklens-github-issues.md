# LeakLens Integrated Privacy Firewall Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a real Zcash wallet proposal privacy gate, reusable Rust analysis engine, and tested pre-authorization demonstration.

**Architecture:** Separate `leaklens-core`, `leaklens-zcash`, `leaklens-gate`, and `leaklens-demo` crates in one Cargo workspace. The wallet creates a proposal, the adapter extracts facts, the policy engine evaluates risks, and the gate binds user approval to exactly the authorized proposal before wallet signing.

**Tech Stack:** Rust, Cargo test, Zcash `librustzcash` (`zcash_client_backend`, pinned compatible release), thin CLI/TUI interface, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-10-25-leaklens-design.md`

## Global Constraints

- No mainnet spending or real user keys in fixtures.
- Rules LL-001 through LL-006 from spec; no fabricated deanonymization or privacy probability.
- No signing without a complete assessment and bound approval; incomplete inspection fails closed.
- SDK compatibility must be established in a compile spike before parallel implementation.
- All GitHub PRs target the repository default branch and use `Closes #N` at the end of their descriptions.
- Each agent gets a distinct git worktree and branch. No overlapping file ownership without an integration handoff.
- Tests first; run focused tests, then full suite; commit passing changes.

## Repository structure / ownership

| Path | Owner | Responsibility |
|---|---|---|
| `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml` | #1 | Workspace, dependencies and CI |
| `crates/leaklens-core/src/{lib,model,policy,fingerprint}.rs` | #2 | Normalized immutable facts, findings and deterministic policy |
| `crates/leaklens-zcash/src/{lib,adapter}.rs` | #3 | Typed SDK proposal extraction |
| `crates/leaklens-gate/src/{lib,gate}.rs` | #4 | Proposal-bound approval and authorization boundary |
| `crates/leaklens-demo/src/{main,display}.rs` | #5 | Wallet harness and user-facing preview |
| `tests/`, `README.md`, `docs/demo.md` | #6 | End-to-end verification, reproducible demo, delivery |

## Review Focus

1. Multi-step proposal where a later step has a transparent output: #2/#3 tests must not miss later steps.
2. Unified Address containing shielded receivers but selected transparent output: #2/#3 test the *chosen* pool.
3. Unknown future pool or unsupported proposal variant: #3/#4 tests return incomplete and deny authorization.
4. Same amount/recipient but changed input, change output, or fee after acknowledgment: #4 test rejects stale authorization.
5. Two simultaneous or repeated approvals: #4 tests show acknowledgments cannot authorize a different proposal or policy version.

---

## GitHub Milestone 0 — Foundation (serial prerequisite)

### Issue #1: Pin Zcash SDK and establish a real proposal compile/test harness

**Labels:** `foundation`, `sdk`, `critical-path`; **Depends:** none.

**Files:** Create `Cargo.toml`, `Cargo.lock`, `crates/leaklens-zcash/tests/sdk_smoke.rs`, `.github/workflows/ci.yml`; modify spec with exact SDK version if required.

**Interfaces:** Produce agreed SDK version, feature flags, `FeeRule` and `NoteRef` concrete test types, and an SDK-generated `Proposal` fixture constructor documented for downstream issues. Avoid defining other teams' crate APIs.

- [ ] Write `sdk_generates_real_shielded_proposal` smoke test using the official SDK; assert a constructed proposal has at least one step and inspectable payment pool.
- [ ] Run `cargo test --test sdk_smoke` and record the expected initial dependency/compiler failure.
- [ ] Pin compatible crates, feature flags and minimal wallet DB/test fixture; make the test pass without network broadcasting.
- [ ] Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` successfully.
- [ ] Commit `chore: establish Zcash SDK proposal harness` and merge into default branch to unblock the parallel wave.

**Acceptance:** CI compiles on a clean runner; the fixture is an actual SDK `Proposal`, not a mocked struct; versions are pinned.

## GitHub Milestone 1 — Parallel wave (agents 2–4)

### Issue #2: Implement normalized proposal model and privacy policy rules

**Labels:** `policy`, `parallel`; **Depends:** #1. **Owner:** Agent A, exclusive `leaklens-core`.

**Files:** Create `crates/leaklens-core/src/{lib,model,policy,fingerprint}.rs`, `crates/leaklens-core/tests/policy_rules.rs`.

**Interfaces:** Export `ProposalFacts { steps: Vec<StepFacts> }`, `StepFacts { payments: Vec<PaymentFacts>, has_transparent_inputs: bool, has_shielded_inputs: bool, change_pools: Vec<PoolKind> }`, `PaymentFacts { recipient_kind: RecipientKind, selected_pool: PoolKind, amount_zat: u64 }`, `PoolKind { Transparent, Sapling, Orchard, Ironwood, Unknown }`, `PrivacyAssessment { findings: Vec<Finding>, complete: bool, policy_version: u32 }`, and `assess(&ProposalFacts, &Policy) -> PrivacyAssessment`. Add clear derives/constructors and a stable finding code/severity type. Do not include private keys or viewing keys.

- [ ] Write failing tests: `transparent_destination_warns`, `transparent_input_warns`, `shielded_to_transparent_warns`, `unified_transparent_selection_warns`, `shielded_only_no_known_disclosure`, `later_step_risk_detected`, `unknown_pool_marks_incomplete`.
- [ ] Run `cargo test -p leaklens-core`; confirm red tests.
- [ ] Implement minimal rule engine covering LL-001 through LL-005, differentiating certain disclosure from possible correlation.
- [ ] Run `cargo test -p leaklens-core` and `cargo clippy -p leaklens-core --all-targets -- -D warnings`; expect pass.
- [ ] Commit `feat: evaluate Zcash proposal privacy risks`, open PR, finish description with `Closes #N`.

**Acceptance:** Pure deterministic core, no network, all steps assessed, conservative unknown handling.

### Issue #3: Adapt actual `librustzcash` proposals to the privacy model

**Labels:** `adapter`, `parallel`; **Depends:** #1, consumes #2 public contract. **Owner:** Agent B, exclusive `leaklens-zcash`.

**Files:** Create `crates/leaklens-zcash/src/{lib,adapter}.rs`, `crates/leaklens-zcash/tests/adapter_real_proposals.rs`.

**Interfaces:** Export `extract_proposal_facts<FeeRule, NoteRef>(proposal: &Proposal<FeeRule, NoteRef>) -> Result<ProposalFacts, AdapterError>` with the exact generic bounds required by pinned SDK; consume #2 model contract without editing core. Ensure all steps, destinations, chosen pools, transparent funding inputs, and change pools map into normalized facts.

- [ ] Write failing SDK-backed tests: `extracts_shielded_destination`, `extracts_transparent_destination`, `detects_unified_selected_pool`, `extracts_transparent_input`, `extracts_every_step`, `unknown_pool_rejected` (or a documented equivalent case if API prevents construction).
- [ ] Run `cargo test -p leaklens-zcash`; confirm red tests.
- [ ] Implement mapping through official typed accessors (`steps`, `payment_pools`, `transparent_inputs`, `shielded_inputs`, balance); avoid guessing based on address prefix.
- [ ] Run `cargo test -p leaklens-zcash` and clippy; expect pass.
- [ ] Commit `feat: inspect SDK transaction proposals`, open PR with `Closes #N`.

**Acceptance:** Tests operate on SDK-generated proposals; unsupported data never silently maps to 'safe'.

### Issue #4: Implement proposal-bound authorization gate

**Labels:** `security`, `parallel`; **Depends:** #1, consumes #2 public contract. **Owner:** Agent C, exclusive `leaklens-gate`.

**Files:** Create `crates/leaklens-gate/src/{lib,gate}.rs`, `crates/leaklens-gate/tests/authorization.rs`.

**Interfaces:** Export `GateDecision { Allow, RequireAcknowledgment, DenyIncomplete }`; `prepare_decision(facts: &ProposalFacts, assessment: &PrivacyAssessment, fingerprint: ProposalFingerprint) -> GateDecision`; `authorize(current_fingerprint: ProposalFingerprint, current_policy_version: u32, approval: &Approval) -> Result<(), GateError>`. `ProposalFingerprint` is computed over a canonical/versioned SDK proposal encoding by integration code, not lossy `ProposalFacts`. Gate has no direct signing-key access.

- [ ] Write failing tests: `warned_proposal_requires_ack`, `clean_complete_proposal_can_pass`, `incomplete_fails_closed`, `changed_recipient_rejects`, `changed_inputs_rejects`, `changed_change_or_fee_rejects`, `changed_policy_version_rejects`, `approval_not_reusable_for_other_proposal`.
- [ ] Run `cargo test -p leaklens-gate`; confirm red tests.
- [ ] Implement immutable approval binding, domain-separated/versioned fingerprint comparison, and fail-closed gate.
- [ ] Run `cargo test -p leaklens-gate` and clippy; expect pass.
- [ ] Commit `feat: bind privacy approval to exact proposal`, open PR with `Closes #N`.

**Acceptance:** Every transaction-building/signing path in the demo must go through the gate; demonstrate denial for stale and incomplete assessments.

## GitHub Milestone 2 — Integration (after #2–#4)

### Issue #5: Build the controlled wallet flow and pre-signing privacy preview

**Labels:** `integration`, `demo`, `critical-path`; **Depends:** #2, #3, #4. **Owner:** Agent D.

**Files:** Create `crates/leaklens-demo/src/{main,display}.rs`, `crates/leaklens-demo/tests/demo_flow.rs`.

**Interfaces:** Consume core `assess`, SDK `extract_proposal_facts`, and gate APIs. No direct SDK signing invocation outside the checked integration path. CLI commands: `prepare`, `inspect`, `revise`, `acknowledge`, `authorize` (exact CLI ergonomics may be simplified while keeping the behaviors).

- [ ] Write failing tests: `warns_before_authorize`, `revised_proposal_reassessed`, `unknown_proposal_never_authorized`, `cannot_sign_without_gate`, `display_shows_public_vs_possible_risks`.
- [ ] Run `cargo test -p leaklens-demo`; confirm red tests.
- [ ] Implement wallet-owned proposal creation, full inspection, preview, acknowledgment, fresh fingerprint check and call to existing wallet authorization API; no broadcast.
- [ ] Run focused tests and `cargo test --workspace`; expect pass.
- [ ] Commit `feat: integrate privacy checks before wallet authorization`, open PR with `Closes #N`.

**Acceptance:** A working actual proposal-to-authorization path is shown; if cryptographic proving needs extra runtime resources, explicitly distinguish reached authorization gateway from completed signing.

## GitHub Milestone 3 — Release verification (after #5)

### Issue #6: Add end-to-end regression matrix and reproducible two-minute demo

**Labels:** `qa`, `submission`, `critical-path`; **Depends:** #5. **Owner:** Agent E.

**Files:** Create `tests/end_to_end.rs`, `docs/demo.md`; modify `README.md` and `.github/workflows/ci.yml` only through integration owner's coordination.

**Interfaces:** Treat other crates as public consumers; no ad hoc changes to core behavior. Report missing integration contracts to owning agent.

- [ ] Write failing end-to-end test cases covering transparent output, transparent input, multi-step output, Unified receiver selection, unknown pool, tampering, and reapproval.
- [ ] Run `cargo test --workspace`; record red cases.
- [ ] Fix test harness/doc wiring, escalate genuine production failures to owners, rerun until green.
- [ ] Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; capture exact results.
- [ ] Record two-minute demonstration of real proposal inspection, warning, revision, and approval; document setup and limitations; commit `docs: verify LeakLens end-to-end demo` and open PR with `Closes #N`.

**Acceptance:** Fresh clone reproducibility, public repository, truthful demo, no secret keys in output, clear source/license/limitations and a working submission link.

## Concurrent agent execution schedule

| Wave | Parallel workers | Merge barrier |
|---|---|---|
| 0 | Foundation owner only: #1 | SDK fixture and public core model contract agreed |
| 1 | Agent A #2; Agent B #3; Agent C #4 | Each has isolated worktree, passing focused tests, reviewed PR |
| 2 | Agent D #5; Agent E may prepare test design and demo script without editing dependent code | All of #2–#4 merged |
| 3 | Agent E #6; Agent D handles integration fixes | Full green CI; all PRs merged; submission-ready |

Parallel execution is not a promise of safe simultaneous merging: assign nonoverlapping file ownership, synchronize against the default branch after every barrier, and let one integrator own dependency/lockfile changes.

## GitHub issue publication requirements

- Create repository (or use exact repository user identifies), verify default branch and permissions.
- Create milestone names above, labels, then six issues in order; replace placeholders `#N` in each PR description with real issue IDs.
- Issue body contains Goal, Files, Interfaces, step checklist, Acceptance, Dependencies; assign each to the intended owner/agent workstream.
- Branch naming: `feat/leaklens-issue-N-short-name`; one PR per issue, reviewed and merged at the appropriate barrier rather than accumulating completed PRs.
- Do not claim issues or GitHub artifacts exist before verifying links from the GitHub API.

## Risk and deadline controls

- **Highest risk:** SDK compilation/proposal construction and signing integration. Time-box #1 to the first work period. If it fails, report exact blocker; do not substitute a fake proposal while claiming real integration.
- **Second:** Canonical fingerprint. Verify the SDK's stable versioned proposal serialization can encode all approved fields. A lossy fingerprint is a security defect.
- **Third:** Library API may change at a pinned pre-release; lock dependency versions and toolchain in #1.
- Freeze feature scope as soon as #5 demonstrates the real signing boundary; spend remaining time on regression tests and demo.

## Self-review mapping

Spec goal real wallet proposal: #1, #3, #5. Pure deterministic rules and correct warnings: #2. Strict approval binding and fail-closed semantics: #4 and #5. User preview: #5. Tests, reproducibility and submission: #6. All six risk rules and five review-focus edge classes have named test owners. No duplicate primary file ownership in parallel wave.
