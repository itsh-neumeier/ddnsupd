# Provider engine contract

Private HTTP JSON protocol version 1. Rust authenticates each request with a
separate Bearer service token. The service rejects oversized bodies and
unknown fields. Health is the only unauthenticated endpoint. No public port.

`POST /v1/reconcile` accepts provider ID, credential token, explicit zone ID,
relative record name, type A/AAAA, desired public address, previously managed
address and dry-run flag. Cloudflare also needs the immutable record ID.
The API validates grants and zone/name before sending. The engine validates
again. Endpoints are fixed in production; test clients inject a transport.

Registry descriptors expose ID, display name, credential schema, supported
types/families, least-privilege instructions, discovery/read/write capabilities,
TTL policy, limits, options and documentation. Provider schemas are versioned.

Reconciliation reads current provider state, rejects ambiguous ownership,
checks the expected managed value, preserves other RRSet values and metadata,
then writes only when needed. It verifies the result. A dry run performs no
write. It returns before/after values, changed, verified and propagation state.
DNS propagation remains unknown for proxy records.

Errors are stable codes: invalid_input, unsupported, auth, permission,
rate_limit, conflict, not_found, unavailable, timeout, uncertain and internal.
Responses contain safe descriptions only, optional retry-after seconds and
correlation ID. Provider bodies, HTTP headers, URLs with secrets and raw errors
never cross into API responses or logs. Network work has a deadline. The
caller cancels on lease/lock loss and reconciles after uncertain outcomes.

Connection tests use read-only provider operations and never call a generic
update endpoint as a credential test. Providers without non-mutating tests
must report that limitation. Missing records are not silently created by
reconcile; creation is a separate reviewed action.
