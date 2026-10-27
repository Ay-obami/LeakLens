# LeakLens web

Responsive Vite/TypeScript presentation and privacy-review sandbox for the Rust LeakLens project.

```sh
npm ci
npm test
npm run build
npm run dev
```

The public sandbox uses **synthetic, non-spendable proposal facts**. Its built-in browser policy mirror is only an educational demonstration; it does not produce wallet-approved or SDK-validated facts and is not a signing or authorization endpoint.

Set `VITE_LEAKLENS_API_URL` to the Rust API's HTTPS Railway origin (`https://*.up.railway.app`) to use `/api/v1/assess`; otherwise the frontend runs the local browser simulation and labels the source in results. If a configured Rust backend is unreachable, the review displays a prominent **BACKEND UNAVAILABLE** alert and clearly identifies the local-only fallback. User-submitted facts are **not** authenticated wallet facts even when assessed by the Rust backend. This frontend deliberately contains **no seed-entry, wallet-import, signing or broadcast controls**.

The `vercel.json` CSP allows API traffic only to Railway-generated `*.up.railway.app` domains. If deploying the API on a custom domain, update `connect-src` in `vercel.json` to that explicit HTTPS origin before redeploying. There are no third-party fonts, scripts, trackers or other runtime asset dependencies.

For Vercel, choose `web` as the root directory, framework preset **Vite**, build `npm run build`, and output `dist`.
