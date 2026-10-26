# Zecathon Submission Proof Checklist

**Deadline to recheck:** October 28, 2026, 23:59 UTC, as listed on <https://thezecathon.com/>. Convert to your local time before final submission and verify the live portal.

## Essential proof

- [ ] Repository is public and the exact submitted commit SHA is recorded.
- [ ] README states the actual implemented state without claiming funded or signed transactions.
- [ ] MIT license and attribution are present.
- [ ] `cargo +1.95.0 test --workspace` passes at the submitted SHA; save uncut output.
- [ ] `cargo +1.95.0 fmt --all -- --check` and strict Clippy pass; record versions.
- [ ] Typed SDK proposal adaptation is demonstrated by a test, not a hand-authored fake SDK object.
- [ ] Privacy rules demonstrate transparent destination, input, change, and Unified receiver selection.
- [ ] Multi-step, unsupported variant, stale proposal, and policy-change cases are covered.
- [ ] Gate denies incomplete proposals and invalid reused approvals in tested control flow.
- [ ] No real keys, wallet mnemonic, sensitive notes, or personal payment information are published.

## Two-minute video and demo

- [ ] Opening two-sentence pain/solution statement is comprehensible to a non-engineer.
- [ ] Show actual running tests or executable interface, clearly labeling which one.
- [ ] Separate definite public disclosures from uncertain amount/timing correlations.
- [ ] Show a failed/incomplete review and what the gate does.
- [ ] Explain current limitations: library-level preview, SDK fixtures, no funded/signing/broadcast proof unless subsequently verified.
- [ ] End on concrete Zcash relevance and repository URL.
- [ ] Review video at normal speed; no misleading splices or fabricated completion banners.

## Submission form

- [ ] Confirm the exact competition track and eligibility in the signed-in organizer portal.
- [ ] Enter project name, concise description, GitHub repository, demo link, and requested video.
- [ ] Use the organizer's current requirements for video length, visibility, screenshots, and team information.
- [ ] Submit before the server deadline; capture confirmation receipt and timestamp.

## Optional stretch proof (only after genuine execution)

- [ ] A wallet-generated proposal from funded testnet state.
- [ ] Full authorization/signing run through the mandatory gate.
- [ ] Network submission/confirmation, if appropriate; never imply mainnet activity without direct evidence.

**Release rule:** An unchecked essential item must be fixed or explicitly disclosed, not silently treated as complete.
