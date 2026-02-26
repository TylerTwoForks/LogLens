---
description: Phase 3 scope for upload ingest, parser output model, and in-process jobs
alwaysApply: false
---

# Phase 3: Ingest, Parse, and Job Execution

## Objective

Ship the core upload-and-parse flow with no durable raw-log retention.

## In Scope

- Single and batch upload endpoints.
- Job table lifecycle: queued/running/done/failed.
- In-process async job runner with bounded concurrency.
- Parser output normalized event model and derived aggregates.
- Progress/status APIs for frontend polling or streaming.

## Out of Scope

- Redis queueing, advanced chart UX, and enterprise hardening.

## Exit Criteria

- One-or-many log uploads process successfully.
- Raw log content is not retained as durable app data.
- Job progress and completion are visible and reliable.
