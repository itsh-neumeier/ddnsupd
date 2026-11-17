# ADR 0008: Read-only provider discovery and staged imports

- Status: Accepted for implementation
- Date: 2026-11-03

Add private Go discovery operations for zones and existing A/AAAA records.
Use fixed official API hosts, validated zone IDs/names, GET requests only,
bounded bodies, explicit 50-item provider pagination and read-only scope
verification. Zone reads do not prove DNS write permission. Rust applies
provider grants, CSRF, safe errors, audit and a persisted 30-page/minute budget.
Rate-limit responses block subsequent provider access until Retry-After.

Import initially targets one configured account/zone for Hetzner Cloud or
Cloudflare. Analyze at most 200 source entries, with a 60-KiB UI file limit and
the API body limit. Inspect at most 20 provider pages/2000 address values with
a 30-second overall deadline; exceeding bounds fails, never silently truncates.
The source is parsed as data. Source credentials and arbitrary URL/command
options do not enter a normalized draft. Unsupported options block their rows.

Resolve existing remote record identities/values before preview. Missing or
ambiguous matches, suffixes not implemented, outside-zone names and duplicate
bindings are explicit conflicts. Upstream `ipv4 or ipv6` requires a selected
family; it never silently becomes dual stack. Preserve provider TTL/proxy
settings and show a warning on differences. Source credentials are ignored;
the selected account's existing encrypted token is used.

Store only the normalized plan and digest, bound to user, provider and credential
version, for ten minutes. Confirmation requires explicit row selection, digest
and acknowledgement. Lock the draft/provider, reject stale versions/expiry or
replay, recheck database uniqueness through constraints and insert atomically.
All imported bindings are inactive; no jobs or remote writes occur during import.
Credential rotation invalidates staged plans. Remote changes after preview are
reconciled against the saved managed value when a user later enables the binding.

Full cross-provider import, updates.json history, suffix support, credential
migration and remote record creation remain separate increments.

Official API references: [Cloudflare zones](https://developers.cloudflare.com/api/resources/zones/methods/list/),
[Cloudflare records](https://developers.cloudflare.com/api/resources/dns/subresources/records/methods/list/),
[Hetzner Cloud zones/RRSets and pagination](https://docs.hetzner.cloud/reference/cloud).
