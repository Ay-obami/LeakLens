# Wallet-Connected Product Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship polished web-first LeakLens with a real Noir Wallet testnet connection and truthful pre-send privacy review while preserving the local Rust authoritative signing flow.

**Architecture:** Vite SPA separates marketing and /app routes. Browser wallet adapter talks directly to injected Noir provider; pure preflight analyzer runs locally with explicit epistemic limits. Separate existing Rust SDK gate remains local and noncustodial. Vercel hosts web and Render hosts only stateless informational API.

**Tech Stack:** Vite 7, TypeScript 5.9, Noir Wallet SDK, CSS, Rust workspace, Vercel, Render.

**Spec:** docs/superpowers/specs/2026-10-27-wallet-connected-product-redesign.md

## Global Constraints
- Zcash **testnet only** on hosted product, no mainnet sends.
- Never store/expose seeds, keys, wallet database on Vercel or Render; no public signing endpoint.
- Never claim Noir provides a canonical unsigned SDK proposal. Label preflight as an estimate; exact Rust signing gate remains separate.
- Keep user's existing Rust wallet and release deployment intact until candidate is tested.
- Responsive accessible UI with connected/disconnected/loading/error states and real empty states.

## Review Focus
1. Mainnet Noir extension mistakenly connected: reject or require testnet-specific provider evidence and never send with unverified network.
2. Untrusted recipient and amount: strict address network validation and decimal parsing before review/send.
3. Wrong destination after review: bind requested recipient/amount/source/memo to review state; any edits invalidate review.
4. Wallet disconnect mid-flow: invalidate pending review and reset send action.
5. Failed user authorization: display a canceled/failed state, never a txid or success.

---

### Task 1: Baseline, branch, and wallet integration contract
**Files:** web/package.json, web/src/wallet.ts, web/src/wallet.test.ts
**Interfaces:** export `WalletClient`, `WalletConnection`, `WalletBalances`, `WalletTx`, `getBrowserWallet()` and `detectWalletKind()` with explicit testnet source.
- [ ] Test missing provider, refused connection, unlocked connected read paths, account-change updates and unavailable testnet build.
- [ ] Verify failing tests.
- [ ] Add pinned Noir SDK dependency and typed adapter; no seed handling.
- [ ] Run test/build; commit.

### Task 2: Pure privacy preflight, inputs and review authorization
**Files:** web/src/preflight.ts, web/src/preflight.test.ts, web/src/transaction.ts, web/src/transaction.test.ts
**Interfaces:** `analyzePreflight(request): PreflightResult`; `parsePaymentDraft(draft): Result`; `makeReviewToken`; `samePayment`.
- [ ] Tests for shielded vs transparent Zcash address, UA ambiguity, known vs unknown funding, invalid network address, decimal fee/balance, edit-after-review, and false clean claim.
- [ ] Verify tests fail before implementing.
- [ ] Implement deterministic client-side privacy estimate (not canonical proposal assertion), strict network-aware validation and send-state token.
- [ ] Run tests; commit.

### Task 3: Premium public landing page and app shell
**Files:** web/src/main.ts, web/src/site.ts, web/src/style.css, web/index.html
**Interfaces:** `renderHome`, `renderAppShell` with routing on pathname and link navigation.
- [ ] Verify first-load / and /app behavior and meaningful CTA with manual browser test.
- [ ] Replace old sandbox hero with compact branded positioning, real app preview, evidence/capabilities, security and launch.
- [ ] Build responsive desktop/mobile app nav, overview, send, receive, activity tabs; keyboard and empty states.
- [ ] Run web tests/build and visual viewport check; commit.

### Task 4: Wire real Noir Wallet and payment workflow
**Files:** web/src/app.ts, web/src/wallet.ts, web/src/preflight.ts, web/src/style.css
**Interfaces:** connect, refresh, disconnect, preflight review, send via wallet approval; transaction list and receive address.
- [ ] Add mock adapter tests for connect and send cancellation, stale review, wallet error, connect-to-testnet guidance.
- [ ] Implement UI with wallet-derived data only; no fake balances, status or txids.
- [ ] Ensure user approves wallet popup and gets a receipt only after SDK txid returned.
- [ ] Verify tests/build and manual browser flows; commit.

### Task 5: Documentation and authoritative Rust gate alignment
**Files:** web/README.md, README.md, docs/demo.md and project evidence docs
**Interfaces:** standalone testnet Noir setup and CLI exact-proposal signing walkthrough.
- [ ] Check rust SDK safety boundary and actual CLI command paths.
- [ ] Document two paths honestly (browser preflight, local exact SDK proposal gate).
- [ ] Update user guide, judge walkthrough, no fabricated claims; Rust tests; commit.

### Task 6: E2E, accessibility, release and production smoke checks
**Files:** web tests/config if needed, deployment docs, release PR
**Interfaces:** release verification and rollback.
- [ ] Run `npm test`, `npm run build`, `cargo +1.95.0 test --workspace` on reviewed commit.
- [ ] Manually navigate at laptop/mobile widths; exercise extension-missing, wallet connected via installed testnet extension if available.
- [ ] Verify staging environment, production project/team/commit, then merge/deploy only after gates; do not claim wallet e2e without actual wallet.
- [ ] Smoke test live home, /app, backend /health, browser error console and security CSP; report remaining constraints.
