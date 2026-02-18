# LogLens Phase 1 Foundation

This repository implements the active Phase 1 scope:

- Monorepo layout with `apps/web`, `apps/api`, and Rust parser crates.
- Next.js web shell calling the Rust API through a typed client.
- PostgreSQL-backed Rust API with migrations and health/version endpoints.
- Shared OpenAPI contract generation and CI mismatch checks.
- Root `dev/lint/test/check` workflows, Docker Compose, and CI smoke job.

## Layout

- `apps/web`: Next.js frontend shell.
- `apps/api`: Rust API service.
- `packages/api-client`: Typed TypeScript API client generated from OpenAPI.
- `crates/parser-core` and `crates/parser-metrics`: parser foundation crates.
- `contracts/openapi.json`: committed API contract.

## Local Setup

1. Install toolchain pinned in `mise.toml`.
2. Install dependencies:
   - `pnpm install`
3. Generate contract and typed client:
   - `pnpm run contract:generate`

## Required Root Commands

- `pnpm run dev`: starts web/API/DB via Docker Compose.
- `pnpm run lint`: TypeScript checks + Rust fmt/clippy.
- `pnpm run test`: API client tests + Rust tests.
- `pnpm run check`: contract sync checks + lint + tests.

## Make Targets

- `make build-backend`: build API image only.
- `make build-frontend`: build web image only.
- `make build-db`: start PostgreSQL only.
- `make migrate`
- `make build-all`: build and start DB + API + web.
- `make up-all`: preferred full-stack build + up target.
- `make redeploy-db`: rebuild + redeploy DB only.
- `make redeploy-api`: rebuild + redeploy API only.
- `make redeploy-web`: rebuild + redeploy web only.
- `make logs-db`: follow PostgreSQL logs.
- `make logs-api`: follow API logs.
- `make logs-web`: follow web logs.
- `make tear-down-all`

## Notes

- PostgreSQL persistence is provided by Docker volume `loglens_postgres_data`.
- Contract mismatch detection is enforced by:
  - Rust -> OpenAPI sync check.
  - OpenAPI -> typed client sync check.
