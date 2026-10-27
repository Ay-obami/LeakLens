# LeakLens Hosted Demo Release Implementation Plan

> **For agentic workers:** Apply the Superpowers test-driven development and verification workflows to each behavior change.

## Goal and constraints

Ship a public, usable hackathon demonstration on Vercel + Railway without exposing or migrating any testnet spending key. The public application is a non-signing privacy preview using deterministic SDK fixtures and explicitly user-supplied simulation facts. Actual funded signing remains on the local machine until a separate, authenticated, narrowly scoped signer is designed and authorized.

## Design choice

Selected from prior discussion: separate static Vercel web frontend and Railway-hosted, stateless Rust policy API. This fits a low-cost public demo, preserves the existing deterministic policy library and avoids making the Zcash funded wallet a public signing endpoint. A browser-only demonstration would not demonstrate the backend; a fully hosted funded signer introduces disproportionate custody/security risk and paid persistent requirements.

## Independent deliverables

1. **Public web (`web/`)**: responsive landing page, tangible privacy-assessment flow, fixture selection, transparent warnings and explicit simulation/non-signing disclosure. Unit tests and production static build.
2. **Policy API (`crates/leaklens-api/`)**: health, synthetic fixtures, bounded proposal-facts assessment, versioned JSON result, fail-closed validation, no spending material. Rust unit/integration tests.
3. **Hosting (`deploy/`)**: Railway Dockerfile and Vercel deployment instructions with environment variables, resource limits, health checks, key-handling boundaries and rollback guidance.
4. **Security review (`docs/deployment-security-review.md`)**: cloud threat model, hosting acceptance checks and explicit limitations of simulated facts.
5. **Integration (`Cargo.toml`, root README and CI)**: add API workspace member, run formatting/tests/clippy/build and check frontend/API contract; then publish a safe branch/PR if repository authentication permits.

## Acceptance criteria

- Public HTTP endpoints never sign, broadcast, read seed files or expose wallet database paths.
- A transparent fixture produces public-disclosure findings and a shielded fixture produces no implemented warnings; unknown/incomplete inputs fail closed.
- The UI plainly distinguishes simulation facts from verified SDK-derived proposals.
- Frontend builds; all runnable project tests pass; Rust checks are run in an environment with the pinned toolchain.
- Deployment instructions describe actual limits of Railway Free rather than promising always-on hosting.
- No private keys, wallets, auth material or funded database content are committed or uploaded.
- Final release report distinguishes deployed, deployment-ready and blocked items.
