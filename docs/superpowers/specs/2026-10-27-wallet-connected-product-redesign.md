# LeakLens wallet-connected product redesign — design specification

Date: 2026-10-27
Decision owner: Ayobami
Status: approved direction, implementation follows

## Intent and user outcome
Replace the educational-only landing/sandbox with a polished Zcash testnet product that a hackathon judge can independently explore. The first screen must show a compelling value proposition and an obvious **Launch App** CTA. The app must offer a genuine Zcash wallet connection, real balances/addresses/activity, and an actionable send flow that gives understandable pre-authorization privacy guidance. Preserve the separate Rust SDK's authoritative, proposal-bound local signing gate, and explain the distinction honestly.

## Success criteria
1. A new visitor understands the product and clicks Launch App without reading a technical manual.
2. App shows clear disconnected, extension-missing, connecting, syncing, connected, error, and transaction progress states.
3. The Noir Wallet *testnet* extension connects after explicit user approval; the app can read authorized wallet addresses, balances and activity. No fabricated wallet data.
4. User enters a recipient and amount, sees a pre-send analysis and fee/amount context, and explicitly chooses to continue, edit or cancel. Sending invokes the wallet extension's own approval; no backend custody.
5. The preview plainly labels what is inferred from requested recipient/funding choices versus what has actually been verified from an SDK proposal. Never claim exact outputs or privacy pass if the external wallet's proposal is not available.
6. Local Rust wallet proposal / review / sign / broadcast remain testnet-only with no remotely exposed keys or signing. Document how to reproduce the stronger gate.
7. Responsive desktop/mobile UI, meaningful accessibility, honest loading/error feedback, coherent elegant brand, copy that non-engineers can understand, and tests for transaction guardrails.

## Non-goals
- Deploying funded wallet keys, viewing keys, seed phrases, age identities, SQLite databases, or a signing service to Render/Vercel.
- Claiming Noir exposes an unsigned proposal or that LeakLens can block transactions initiated outside LeakLens.
- Mainnet payments, anonymous cloud-sponsored transactions, phantom wallet support, or a universal Zcash wallet standard.
- Telling visitors a shielded transaction is completely anonymous.

## Approaches considered
A. **Chosen:** Vercel web app integrates Noir Wallet SDK for a real browser-connected testnet UX. Existing Rust policy API supports informational review. Keep local Rust signer as authoritative demonstration and clearly mark the guarantee boundary. Advantage: judge can interact on hosted URL. Tradeoff: external wallet SDK currently exposes sendTransaction but not an inspectable proposal; user-approved send remains Noir-mediated.
B. Native Tauri desktop client with existing Rust wallet. Stronger exact-proposal review and single-client ownership, but installation burden and more distribution/security work.
C. Browser-hosted keys and own WebAssembly wallet. Low installation friction but high key management, proving, synchronization, and risk scope; rejected for this release.

## Product architecture
- Vite/TypeScript app deployed at existing Vercel domain; same origin routes / (marketing) and /app (wallet application) through history fallback.
- Wallet adapter isolates `@noir-wallet/sdk` interactions: detected provider, explicit connect, balances, addresses, transaction history, getMaxTransfer, sendTransaction, and wallet events. Testnet extension is distinct from mainnet; installation guide must be clear and direct.
- Privacy estimator runs local deterministic address/funding analysis and uses the existing Rust policy API only for explicitly self-reported facts. UI must label **preflight estimate**, not **verified SDK proposal**; if unsupported or inconclusive, warn rather than pass.
- Send flow: validate testnet address and monetary amount -> confirm wallet connected/synced -> show requested destination/funding source, privacy estimate and boundaries -> request wallet approval -> display real returned txid and activity; cancel never calls sendTransaction.
- Rust authoritative flow remains local via `leaklens-wallet-*` CLI programs; no browser bridge to key material in this release. Evidence/docs link shows verified testnet transfer separately.
- Render API remains stateless non-signing; no changes to public signing surface.

## Information architecture and visual direction
- Home: compact nav (Product, How it works, Security, GitHub, Launch App); bold concise hero with authentic app dashboard preview, visible real-product affordances, problem/solution, 3-step flow, factual trust/limits, CTA footer.
- App: left navigation and/or mobile tabs (Overview, Send, Receive, Activity), top network status and wallet action. Dashboard displays balances, wallet identifier, quick actions, recent transfers and protection summary.
- Send: recipient, amount, optional memo (only shielded), source selection if wallet supports, Review button; review summary presents a clear colored outcome with expandable technical evidence, fee, user decision, and distinct confirmation.
- Receive: reveal/copy wallet-provided address with privacy type and QR only if reliable; do not invent wallet data.
- Activity: wallet-provided history, status, txid links if available; explicit empty state.
- Design language: warm off-white / near-black, cobalt or vivid lime accent restrained, high-quality typography, depth through borders and spacing, decisive typography, real interface screenshots/components rather than arbitrary hero illustrations. Clear keyboard focus, labels, aria-live, readable mobile width.

## Security and reliability
- Require explicit opt-in for wallet connect and send; extension handles all signing; no seed entry in web UI.
- Avoid sending full addresses, memos, payment amounts or wallet balances to Render by default. Prefer local review. If public API is used for manually entered facts, disclose and minimize.
- CSP and allowed extension origins must be compatible while preventing third-party injection; do not weaken without testing.
- Abort on unsupported network, invalid addresses, errors, locked/disconnected wallet, insufficient funds, unknown exact route; status must be truthful.
- Keep all tracked and untracked user files; feature-branch PR, tests, typecheck, build, manual browser verification before production promotion.
- Unfunded testnet setup instructions, judge-friendly trial path, no requirement to receive funds to explore non-signing UI.

## Acceptance tests
- Fresh visitor: home -> Launch App -> extension missing guide -> connect -> real wallet balance.
- Wallet testnet signed in: receive address, activity, send review and wallet approval with testnet funds.
- Canceled wallet approval -> no tx broadcast claim, recovery CTA.
- Failed/locked/syncing wallet -> no fake zero balances or misleading success.
- Transparent recipient/source -> prominent disclosure warning; shielded source/destination -> scoped non-guarantee.
- Invalid address/amount and missing wallet -> submit disabled with understandable reason.
- Mobile narrow viewport, keyboard tab, reduced-motion preference.
- Existing Rust cargo workspace tests, web tests and build pass; deployment smoke checks report exact limitations.

## Research bases
- Zcash librustzcash 0.24 wallet API documentation and proposal/sign separation.
- Zcash ZIP 316 receiver selection.
- Noir Wallet SDK current GitHub README: provider, balances, history, sendTransaction, separate testnet extension, no exposed proposal method.
- Zodl wallet published user-facing privacy/shielding patterns.
- Nielsen Norman usability heuristics.
