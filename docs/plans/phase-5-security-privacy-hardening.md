# Phase 5: Security & Privacy Hardening — Implementation Plan

## Current State Summary

### What's Already Solid
- RBAC with cross-org access prevention via `organization_memberships`
- Timing-safe session signature verification (HMAC-SHA256)
- HttpOnly/SameSite cookies for session tokens
- Raw log content already dropped (migration 0004)
- 30-day data retention TTL with `expires_at` index
- Request semaphore limiting (4 concurrent parse jobs)
- Proper 401 vs 403 status codes
- ON DELETE CASCADE for data cleanup

### Gaps to Close

| Area | Current State | Target |
|------|--------------|--------|
| Authentication | Email-only, no passwords | Argon2 password hashing, register/login endpoints |
| Password reset | No mechanism | DB-stored reset tokens, reset endpoints (no email yet) |
| Rate limiting | None on any endpoint | `tower-governor` on auth + global API limits |
| Input validation | Minimal manual header checks | Payload validation, email/password strength rules |
| Request limits | No max body/upload size | Enforced size limits on all endpoints |
| Audit logging | None | Auth events, org/permission changes, no raw content |
| Security headers | None | CSP, X-Frame-Options, X-Content-Type-Options |
| Tenant boundary tests | No dedicated tests | Integration tests covering cross-org access |

## Work Order

### 1. Password-Based Authentication (highest priority)
- **Migration:** Add `password_hash TEXT` column to `app_users`
- **API:** `POST /v1/auth/register` — email + password, argon2 hashing
- **API:** `POST /v1/auth/login` — validate credentials, return session
- **API:** `POST /v1/auth/logout` — invalidate session
- **Web:** Update login/register forms to collect password
- **Deps:** Add `argon2` crate to `Cargo.toml`

### 2. Password Reset Flow
- **Migration:** Create `password_reset_tokens` table (token, user_id, expires_at)
- **API:** `POST /v1/auth/forgot-password` — generate token, store in DB
- **API:** `POST /v1/auth/reset-password` — validate token, update password
- **Note:** No email delivery required yet — token returned in response for dev/testing

### 3. Rate Limiting & Abuse Protection
- **Deps:** Add `tower-governor` crate
- **Auth endpoints:** Strict limits (e.g., 5 login attempts/min per IP)
- **Global API:** Moderate limits (e.g., 100 req/min per user)
- **Ingest:** Separate limits for upload endpoints

### 4. Request Validation & Size Limits
- **Body size:** Enforce max request body (e.g., 50MB for uploads, 1MB for JSON)
- **Input validation:** Email format, password strength (min length, complexity)
- **Payload validation:** Structured validation on ingest endpoints

### 5. Audit Logging
- **Events to log:** login, logout, failed login, register, password reset, role changes, org create/delete, member add/remove
- **Exclusions:** Never log raw log content, passwords, or session tokens
- **Storage:** `audit_events` table with (id, actor_id, event_type, metadata_json, created_at)
- **Structured:** Use `tracing` spans for request correlation

### 6. Security Headers Middleware
- `Content-Security-Policy` — restrict script/style sources
- `X-Frame-Options: DENY`
- `X-Content-Type-Options: nosniff`
- `Strict-Transport-Security` (for production HTTPS)
- `Referrer-Policy: strict-origin-when-cross-origin`

### 7. Tenant Boundary & Auth Tests
- Cross-org access denied tests (user A cannot see org B data)
- Role permission matrix tests (viewer cannot manage members, etc.)
- Invalid/expired session rejection tests
- Rate limit trigger tests

## Exit Criteria (from phase rule)
- [ ] Tenant boundary and authz tests pass
- [ ] Privacy controls are validated and documented
- [ ] Operational diagnostics are sufficient for beta support

## Architecture Notes
- All auth logic lives in `apps/api/src/main.rs` (monolithic — consider splitting into modules as part of this work)
- Web auth in `apps/web/lib/auth.ts` and `apps/web/app/api/auth/`
- Session format: `base64url(JSON) + HMAC-SHA256 signature`, 7-day TTL
- Auth subject derived as `user_{SHA256(email)[:24]}`
