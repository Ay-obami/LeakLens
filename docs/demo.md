# LeakLens — Two-Minute Demo Walkthrough

**Status:** Script and verification storyboard. The current `leaklens-demo` crate has a verified interactive executable using a non-spendable SDK fixture. Do not suggest a real spend/signature occurred.

## Preflight

1. Check out the reviewed commit; record `git rev-parse HEAD` and Rust toolchain version.
2. Run `cargo +1.95.0 test --workspace`, preserving its exit status.
3. Confirm tests use isolated, non-sensitive proposal fixtures and do not broadcast.
4. Choose a test fixture that represents shielded-only delivery and one with a transparent destination or change output.
5. Verify the preview correctly distinguishes public disclosure from possible linkage; keep the source test name and displayed rule code visible.

## Narrated storyboard (target 120 seconds)

| Time | On-screen evidence | Narration purpose |
|---|---|---|
| 0–15s | Title + one-sentence problem | A Zcash payment path can reveal fields the user expected to keep private |
| 15–35s | SDK proposal fixture and test invocation | Inspection occurs on an actual typed SDK proposal structure, before any hypothetical authorization |
| 35–60s | Transparent output test and `LL-001` | Show what is public: recipient and value |
| 60–75s | Shielded-funding crossing and `LL-003` | State correlation is possible, never proven identity tracing |
| 75–90s | Unified receiver selection and transparent change tests | Show selected pool and change are checked, not just address prefix |
| 90–105s | `LL-005` incomplete review / stale approval test | Explain why unsupported or changed proposals must not receive a clean approval |
| 105–120s | Architecture summary + repository/limitations | Explain what is implemented and what remains for production use |

## Example presentation language

“This test creates or supplies a structured Zcash SDK transaction proposal. LeakLens examines the selected output pool before authorization. When the proposal uses a transparent recipient, the review reports publicly observable payment information. A separate rule identifies a *possible* linkability signal when shielded funding crosses into public outputs. Neither rule claims to decrypt a shielded transaction or identify its owner.”

“The current prototype demonstrates the rule engine and pre-authorization review interface through tests. It has not demonstrated a funded network transaction or completed signing.”

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
- Explicit note of unimplemented funded/signing/broadcast milestones.

For a Zecathon submission, supply a repository, concise project description, functioning demo evidence, and the requested short video. Check the organizer's authenticated submission form for the current exact requirements and track before final submission: <https://thezecathon.com/>.
