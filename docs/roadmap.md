# Roadmap and acceptance gates

All phases preserve an executable checkpoint. The MVP is a bounded first
vertical slice; full upstream parity is a later gate, not a label for a stub.
No live DNS changes are authorized. The current user decision fixes AGPL-3.0-only.

| Phase | Deliverable | Acceptance | Dependencies |
| --- | --- | --- | --- |
| 0 | Immutable reference analysis, 60-entry provider matrix, features, ADRs, license, UI rules, model and test plan | Catalog completeness and documentation links checked; architecture decisions recorded | Reference access |
| 1 | Rust workspace, Svelte SPA, PostgreSQL migrations, configuration, Compose, health, CI | Source stack builds/starts on a clean host; DB persistence/readiness tested | 0 |
| 2 | Token-protected one-time setup, login/logout/passwords, server permissions, roles, shell, DE/EN, themes | Setup races, CSRF, rate limits, session expiry and denied resource access tested | 1 |
| 3 | HTTPS v4/v6 discovery, manual IP, records, persistent scheduler/jobs, registry, Hetzner Cloud/Cloudflare, history | Mock end-to-end pass including no-change, drift, failures, cooldown and restart; live test zone requires authorization | 2 |
| 4 | Provider/zone discovery, credential rotation, record inspector, domains, preview/confirm import, search/filter/sort, bulk and safe export | Entire supported record lifecycle works through UI; imported records inactive | 3 |
| 5 | Metrics, record/provider diagnostics, IP timeline/charts, notifications with outbox/dedup/escalation | Fault injection gives correct status and bounded, scoped notifications | 3–4 |
| 6 | TOTP/reset hardening, audit retention, encrypted backup and restore wizard, key rotation, SSRF policy | Restore on a fresh stack with keys; reject damaged archives; permission/security checks pass | 2–5 |
| 7 | Remaining providers, docs, multiarch builds, browser/regression tests, release workflow | Per-provider mock evidence; authorized live tests separately labeled; reviewed release and publication approval | 0–6 |

## Increment order

1. Buildable service foundation and read-only health. Test migrations and health.
2. Authenticated empty portal. One-time setup needs a separate runtime token.
3. Create a credential, bind a provider zone and existing record, choose an IP
   source, enqueue a check, inspect history. No hidden JSON maintenance.
4. Handle no-change, conflicts, unavailable IPs, invalid credentials, rate limits,
   concurrent clicks and uncertain writes. Prove sibling record preservation.
5. Extend the provider/account UI and implement preview-only imports before
   any import can activate a record. Add other priority providers in the user's
   stated order after the two initial adapters. Legacy Hetzner is migration-only.
6. Complete monitoring, notifications, recovery and remaining hardening.
7. Expand parity in audited batches. Each batch adds descriptor, schema, mock
   fixtures, safe errors and capability constraints. No silent fallback provider.

## Product choices

Resolved: requested Rust/Svelte/PostgreSQL stack, Go targeted adaptation,
single installation with multiple users/sites, AGPL application, private engine,
five-minute default, API-driven UI, existing records bound explicitly first.

Open, without blocking ordinary local implementation:

- Which test zones and provider accounts may receive live changes? None supplied.
- Which real deployment URL/proxy networks will be used? Local loopback defaults
  apply until deployment is explicitly authorized.
- Which notification channel and retention policy should ship enabled? Keep
  channels disabled and preserve documented conservative defaults until configured.
- Which release/version and architectures should be published? No release or
  registry operation is authorized; prepare artifacts locally first.

## MVP gate

A fresh host runs a versioned published stack. A user completes setup, stores
a supported provider credential, binds an existing record, enables automatic
discovery/updates and reads results in the UI. No manual JSON changes are needed.
Mock acceptance is required before any authorized live test. Publication and
live verification remain explicit gates; a local build is not a released MVP.

See task memory for completed increments and actual test results.

## Current checkpoint (2026-11-03)

Phase 0 and the local amd64 technical foundation are complete. The first
vertical slice of phases 2–4 is executable and mock-tested. It includes local
authentication, basic roles/grants, credentials, existing-record bindings,
scheduled A/AAAA reconciliation, history and responsive portal. The complete
phase-2 identity scope and phase-4 management scope are not yet complete.
Read-only paginated discovery, paused binding selection and single-account/zone
preview/confirm imports are now implemented and mock-tested (ADR 0008).
Phases 5–7 remain partial or planned as the feature matrix states. Live
provider acceptance and production recovery/release gates remain open.
