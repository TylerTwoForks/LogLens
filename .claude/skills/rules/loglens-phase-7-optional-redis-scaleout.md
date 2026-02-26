---
description: Phase 7 scope for optional Redis-backed distributed job processing
alwaysApply: false
---

# Phase 7: Optional Redis Scale-Out

## Objective

Introduce distributed queue-backed workers only when in-process jobs are no longer enough.

## Trigger Conditions

- Sustained job backlogs under expected load.
- Need for multiple API/worker instances.
- Restart-related in-flight job loss is unacceptable.
- Retry/backoff and dead-letter handling are required.

## In Scope

- Queue adapter behind existing job-runner abstraction.
- Retry, backoff, and dead-letter policies.
- Worker scaling and queue observability.
- Load/failover tests for resilience validation.

## Exit Criteria

- Throughput and reliability improve over in-process baseline.
- Queue operations are observable and supportable.
