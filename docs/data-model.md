# Data model

PostgreSQL is authoritative. UUID primary keys and UTC timestamptz are used.
The migration files describe implemented tables; this document also lists
planned tables and constraints. Planned entities are not runtime guarantees.

| Entity | Purpose and constraints |
| --- | --- |
| users | Unique normalized username, Argon2id hash, active flag, role reference; protect last admin |
| roles / permissions / role_permissions | Built-in and custom roles; immutable administrator; catalog keys enforced on server |
| user_sessions | Hashed random token, user, CSRF token, expiry; revoke on password/reset/restore |
| api_tokens | Planned: hashed token, explicit scopes, expiry, rotation/revocation |
| providers | Type, display name, enabled state, failure classification, credential version |
| provider_credentials | Authenticated encrypted value, nonce, key ID, provider-bound associated data |
| dns_zones | Provider, immutable remote ID, canonical DNS name; unique provider + remote ID |
| dns_records | Zone, owner/name, type, immutable remote binding, prior managed IP, interval, TTL, owner, site, active state, next check |
| provider_permissions / record_permissions | Explicit user-resource grants; separate from global operation permissions |
| sites | Planned location labels and future discovery agents |
| ip_sources | Planned configurable family-specific discovery with policy, timeout and suffix |
| update_jobs | Record, state, due/lease, attempt, trigger, configuration version; one queued/running check per record |
| update_history | Append-only attempt outcome, before/after, trigger, duration, safe error; imported provenance separate |
| ip_history | Planned discovery timeline, family, source, observed time and site |
| notification_channels / rules / outbox | Planned scoped destinations, events, retry/deduplication and escalation |
| audit_logs | Actor, operation, object identity, safe metadata, timestamp; never credential contents |
| system_settings | Versioned typed configuration; secret values belong in encrypted storage |
| import_previews | Expiring user/provider/credential-version-bound normalized plan and digest; single-use atomic confirmation; no source credentials |
| provider_discovery_limits | Persisted per-provider fixed-window page budget for read-only discovery |

Unique active job constraints coalesce repeated manual clicks. Record/provider
deactivation is rechecked immediately before network work. Locks are scoped
to provider account/zone/RRSet, not merely record ID. Jobs store the credential
and configuration version so stale work cannot apply after edits.

All record list/count/export/history queries apply the same resource predicate.
Foreign keys prevent a record from being bound to a zone of another provider.
Delete cascades must not remove audit history; store an object snapshot without
secrets. Provider deletion with bound records is rejected. Credential replacement
increments its version and clears an authentication suspension explicitly.
