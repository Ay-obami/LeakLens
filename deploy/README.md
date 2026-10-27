# Deploy LeakLens public demo (Vercel + Railway)

This deployment publishes the **non-signing privacy-analysis API only**. Never
deploy the disposable funded wallet, SQLite account files, encrypted seed, AGE
identity, viewing keys, or CLI signing/broadcast binaries to this public server.

## Architecture

- Vercel: `web/` frontend (Vite static build), public URL and TLS.
- Railway: `leaklens-api` Rust HTTP service, public **read-only** policy review.
- No wallet database is needed for the public API. Do not attach a volume merely
  to keep secrets on a hosted server. If later adding a separate private wallet
  service, document and review its threat model before deployment.
- External Zcash testnet lightwalletd is used by the **local**, separately
  authorized wallet demo. The hosted public API does not need network access
  to wallet RPC and cannot spend funds.

## Railway deployment

1. Select **New Project → Deploy from GitHub repo**, connect `Ay-obami/LeakLens`.
2. Use repo root as build context and set Railway config file to
   `deploy/railway.json` (or set Dockerfile path to `deploy/Dockerfile`).
3. Deploy the `leaklens-api` service. The Dockerfile copies only its compiled
   binary into the runtime image.
4. Ensure `PORT` is set by Railway (the API defaults to 8080), and configure
   `LEAKLENS_ALLOWED_ORIGIN` to the exact Vercel HTTPS origin after deployment.
   Never use a wildcard
   origin on an endpoint that might later gain authority to sign.
5. Generate a Railway public domain. Verify `GET /health` returns a successful
   response. Test fixture listing and a benign review, then malformed and
   oversized inputs; confirm no route signs or broadcasts.
6. Record the public API URL, the release commit, Railway health-check result,
   and RAM/CPU usage in the submission evidence.

### Free-tier budget and capacity

Railway's current Free plan includes **$1/month** of usage credit after the
initial **$5/30-day** trial, with up to **1 vCPU, 0.5 GB RAM**, and a **0.5 GB
volume**. Free usage is a metered allowance, **not unlimited uptime**. Check
[pricing](https://docs.railway.com/pricing/plans) and
[volumes](https://docs.railway.com/volumes/reference) before deploying.

The public API should be benchmarked below 0.5 GB RSS, and a budget alert
should be enabled if available. If the free allowance expires, the service
may stop; do not describe the deployment as permanently available.

The 0.5 GB persistent volume is unnecessary for stateless privacy checking.
Wallet sync and proof generation may exceed the Free tier, so do not promise
that the existing testnet signer can run here without measurement.

## Vercel deployment

1. Import the same GitHub repo as a **separate Vercel project**.
2. Set Root Directory to `web` and Framework Preset to **Vite**.
3. Build command `npm run build`, output directory `dist`.
4. Set `VITE_LEAKLENS_API_URL=https://YOUR-API.up.railway.app` (the origin,
   without a trailing slash). See `web/README.md` for details.
5. Deploy a preview, verify health, mobile UI and read-only review end-to-end,
   and then promote to production.

Do not put private credentials in `VITE_` variables: they are published to the
browser bundle. Frontend calls should never send seed phrases or keys.

## Release gate

Before calling the deployment ready:

1. `cargo +1.95.0 fmt --all -- --check`
2. `cargo +1.95.0 test --workspace`
3. `cargo +1.95.0 clippy --workspace --all-targets -- -D warnings`
4. `cargo +1.95.0 build --release -p leaklens-api --bin leaklens-api`
5. Run `npm ci`, `npm test` (if defined) and `npm run build` in `web/`.
6. Container-build `deploy/Dockerfile` from **repo root**, run it with a
   temporary port, and verify `/health`, API review, and error responses.
7. Inspect the image for absence of wallet CLI binaries, local databases,
   `.age` files, key material, and source code.
8. Confirm hosted demo cannot cause spending, regardless of input.
9. Record public frontend and API URLs; run the browser demo; add a short
   public demo video and submission details before the deadline.

### Future isolated signing service

For a later private deployment, use a **separate service and image**, distinct
credentials and network policy, encryption backed by a key-management system,
authenticated and rate-limited user commands, authorization challenges bound
to the transaction, and explicit testnet-only enforcement. Prefer user-owned
browser/external wallet signing for multi-user use. Never expose one funded
wallet's signing power to anonymous visitors or migrate the existing local
seed without a deliberate operator decision.
