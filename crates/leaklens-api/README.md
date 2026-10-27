# LeakLens public API

This is an intentionally **read-only** HTTP service for the LeakLens privacy
policy. It contains no wallet database, spending-key loader, signing endpoint,
or transaction broadcaster.

## Run

After adding `crates/leaklens-api` as a workspace member:

```sh
LEAKLENS_ALLOWED_ORIGIN=https://your-frontend.vercel.app PORT=8080 cargo run -p leaklens-api
```

`LEAKLENS_ALLOWED_ORIGIN` accepts one HTTPS browser origin (or an HTTP
`localhost` origin for development). If unset, no cross-origin browser access
is allowed. The API still permits direct HTTP clients.

## Endpoints

- `GET /health` — `{ "status": "ok" }`.
- `GET /api/v1/fixtures` — SDK-backed, non-spendable `shielded` and
  `transparent` fixture fact sets with privacy assessments.
- `POST /api/v1/assess` — assess **user-supplied** facts; alias
  `POST /api/v1/review`.

Example request:

```json
{
  "steps": [
    {
      "payments": [
        {"recipient_kind": "unified", "selected_pool": "transparent", "amount_zat": 25000}
      ],
      "has_transparent_inputs": false,
      "has_shielded_inputs": true,
      "has_prior_shielded_funding": false,
      "change_pools": []
    }
  ]
}
```

For each result, `source` is either `sdk_fixture` or
`user_supplied_facts`. A user-supplied preview is **not an SDK-validated
transaction proposal and never authorizes a transaction**. The response
contains `policy_version`, `complete`, and `findings` with the rule code,
severity, certainty, indices, and explanation.

The API rejects unknown JSON fields, invalid pool kinds, overly large
requests (32 KiB), more than 16 steps, and more than 128 payments or change
outputs per step. Both requests and responses must stay free of spending
keys, seed phrases, and transaction authorizations.

Eight assessments (including SDK fixture generation) can execute concurrently
per server process. Excess requests receive HTTP `429` and `Retry-After: 1`;
the queue does not grow without bound. This protects CPU capacity, but is **not**
an account/IP rate limiter and does not defend against distributed abuse.
Production operators should also enable edge rate limiting or an API gateway.
Responses include `nosniff`, `no-referrer`, and restrictive framing/content
security headers.
