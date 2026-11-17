# Verification plan

All automated provider tests use injected transports/local fixtures. CI must
have no production tokens and cannot reach live provider mutation endpoints.
Treat mock-tested and live-tested as separate evidence.

| Area | Required evidence |
| --- | --- |
| Catalog | Exact frozen inventory IDs, no omissions, valid statuses, source links; support requires test paths |
| Database | Migration from empty DB, constraints, setup race, grants, job uniqueness, restart recovery |
| Authentication | Argon2id, wrong password, bounded login attempts, expiry, logout, changed role, last admin |
| Authorization | Table of operation × role × own/granted/foreign resources; direct API requests, list/count/history/export |
| CSRF | Reject missing/wrong token, foreign Origin and mutating GET; secure cookie policy |
| Secrets | Roundtrip, tamper, wrong key/associated data, no secret readback/logs, rotation version |
| IP | Both families, invalid/private/CGNAT/documentation/mapped values, timeout, oversized body, no-source no-write, suffix boundaries |
| Provider adapters | Origin read, no-change, update/re-read, mismatch, multiple RRSet values and metadata preserved, malformed/oversized response, auth/403/429/5xx, timeout/action |
| Scheduler | Effective interval, cooldown, manual coalescing, provider-level lock, lease loss, crash after send, reconcile before retry |
| Import | Current and legacy host/owner, comma domains, family semantics, unknown providers/keys, credential redaction, stale previews and default inactive |
| UI | Setup/login, CRUD/check/history, themes/locales, mobile inspector/focus, loading/empty/errors; no console/CSP errors |
| Compose | config validation, non-root/read-only app, private DB/engine, health, persistence, source build and smoke tests |
| Recovery | Encrypted archive, key inventory, corruption/path/size rejection, isolated fresh restore, sessions revoked, interrupted restore |
| Release | Exact lockfiles/toolchains, fmt/clippy/lint/check/tests/build/E2E/scans, SBOM/license inventory, version/tag/image consistency |

No check is marked passed from source inspection. Missing tools and unavailable
services are recorded in the task summary. Product support is promoted only
after tests run on the exact implementation. Release artifacts require another
verification pass when runtime or lockfiles change.
