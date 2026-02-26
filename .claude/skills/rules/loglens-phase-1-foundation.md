---
description: Phase 1 scope for monorepo foundation and API contract baseline
alwaysApply: false
---

# Phase 1: Monorepo Foundation

## Objective

Stand up a stable monorepo with Next.js frontend, Rust backend integration, and PostgreSQL database. 
This will use Docker Compose to spin up the front end/backend/and postgresql database. 

## In Scope

- Monorepo layout for `apps/web`, `apps/api`, and Rust parser crates.
- Base Next.js app shell and Rust API service.
- Health/version endpoints and typed API client wiring.
- Shared contract workflow (OpenAPI or equivalent).
- Basic lint, test, and CI checks.
- PostgreSQL DB should be persistent.
- PostgreSQL migration implementation.
- Makefile with local commands: build backend, build frontend, build DB, migrate, build all, tear down all.

## Exit Criteria

- Web can call API successfully via typed client.
- Contract mismatches are detectable in CI.
- Local dev flow runs both services reliably.
- Root `dev/lint/test/check` works.
- CI smoke runs install + lint + tests.
- Fresh-clone validation completed.