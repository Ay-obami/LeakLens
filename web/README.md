# LeakLens — Wallet-connected web app

The web application contains two public routes:

- `/`: product website
- `/app`: real Zcash **testnet** wallet UX using Noir Wallet's browser extension

## Run locally

```sh
npm ci
npm test
npm run build
npm run dev
```

Visit `http://localhost:5173/app`. Install the official **Noir Wallet testnet build**, which is separate from the mainnet extension: <https://github.com/NoirWallet/noir-wallet/releases>. Download the release ending in `-testnet.zip`, unpack it, then load it via Chrome Extensions → Developer mode → Load unpacked. Create your own disposable testnet account in the wallet. Fund it with testnet TAZ to submit a real transaction.

The wallet interface reads connected accounts, addresses, balances, and transaction history directly from Noir. Every actual payment requires Noir's own wallet approval. There is no seed import, wallet custody, or remote signing endpoint in LeakLens's frontend or Render API.

## Critical privacy boundary

**The browser preflight is not the exact signed transaction.** Noir Wallet's currently documented SDK offers `sendTransaction` but no API for exporting its internal unsigned proposal. LeakLens therefore estimates risk using the recipient type and declared funding source, and explicitly says final input selection, output routing, change, fee and wallet transaction construction cannot be attested by LeakLens. Review the Noir approval itself. A Unified Address may select different receivers.

The stronger, authoritative policy gate lives in the separate Rust CLI, which checks the *actual* `librustzcash` proposal and binds review to the signing callback. It is not exposed on the public server. See [SDK signer integration](../docs/sdk-signer-integration.md) and [testnet demo](../docs/demo.md).

**Do not paste recovery phrases, spending keys, viewing keys, or wallet backups into this website or any hosted API.** Never confuse a demonstration risk estimate with a guarantee of anonymity.

## Deployment

Vercel project root: `web`, Vite framework, output `dist`. `vercel.json` rewrites `/app` and keeps restrictive response headers. The separate Render Rust backend remains non-signing and is not required for the live wallet operations.

The page uses only local CSS and packaged scripts; wallet data are not posted to our backend.
