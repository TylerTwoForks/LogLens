---
description: Phase 2 scope for authentication, organization isolation, and licensing
alwaysApply: false
---

# Phase 2: Auth, Org Isolation, and Billing

## Objective

Implement secure user identity, organization separation, and license entitlements.

## In Scope

- Auth provider integration and protected app routes.
- Org, membership, and role model (owner/admin/member/viewer).
- Org-scoped authorization in API layer.
- Individual and organization billing tiers.
- Entitlement checks for gated features.

## Out of Scope

- Full parser pipeline, advanced visualizations, and scale-out queueing.

## Exit Criteria

- Cross-org requests are denied by policy and tests.
- Billing status updates entitlements correctly.
- Role checks are enforced consistently in UI and API.
