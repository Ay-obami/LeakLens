# LeakLens — Two-Minute Demo Walkthrough

**Status:** Script and verification storyboard. Capture only steps actually implemented and reproduced. The current `leaklens-demo` crate exposes a library function `preview_review`, not a confirmed interactive executable. Do not stage terminal screenshots or voiceover suggesting a real spend/signature occurred.

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

## CLI demonstration gate

The present build is a **library-level** demonstration. If an executable is subsequently added, update this document with the precise `cargo run -p leaklens-demo -- <documented subcommand>` invocation **only after running it**. Until then, use targeted tests and source-backed preview output; do not invent interactive commands.

## Evidence to capture

- Commit SHA and matching PR/merge status.
- Full `cargo +1.95.0 test --workspace` summary and any failing tests.
- At least one real SDK proposal adaptation test, with fixture provenance explained.
- Public-output and incomplete-review findings in actual output.
- Explicit note of unimplemented funded/signing/broadcast milestones.

For a Zecathon submission, supply a repository, concise project description, functioning demo evidence, and the requested short video. Check the organizer's authenticated submission form for the current exact requirements and track before final submission: <https://thezecathon.com/>.
