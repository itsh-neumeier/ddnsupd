# Bootstrap and initial vertical slice

- Date: 2026-11-03
- Status: local testable MVP checkpoint ready; full product/release scope remains open
- Objective: analyze references, fix architecture and build a testable local MVP.

## Initial state and decisions

`D:\GitHub\ddnsupd` was empty and was not a Git repository. No remote or local
Git repository was created, no commit/tag/push/publication/deployment was made.
The original user specification is saved in `docs/product-specification.md`.
The user chose AGPL-3.0-only explicitly. Other normal architecture choices
follow the requested stack and are recorded in ADRs 0001–0007.

TalkOps commit: `00b968c0cb8b8d40312e52933d8c0144d1e36e34`.
ddns-updater commit: `7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2`.
Sources were read through the connected GitHub tools. The manifest records
68 file/directory sources and the 60-entry provider tree inventory. The source
factory, constants and docs disagree in several catalog details; the matrix
includes custom, Vercel, Netcup and the development example explicitly.

The official Hetzner status page confirms removal of the legacy API on
2026-05-27. Keep its import identifier but do not implement legacy updates.

## Implemented areas

- Phase 0 architecture, reference comparison, license provenance, provider
  catalog, ADRs, model, UI rules, roadmap and tests.
- Rust API/core/scheduler libraries, PostgreSQL migrations, local setup,
  Argon2id authentication, hashed sessions, exact Origin + CSRF checks,
  persistent brute-force counters, logout/password revocation.
- Built-in roles, custom-role/user creation, explicit record/provider grants.
  Resource scope is applied to record lists, counts and history.
- Authenticated encryption for provider tokens, write-only API, scoped existing
  record bindings, active state, manual source, fixed HTTPS discovery/fallback,
  persistent jobs/history/audit, coalesced manual requests and Retry-After.
- Go registry and two targeted MIT-attributed adapters with fixed endpoints,
  bounded I/O, safe errors, origin read-back and preservation of siblings.
- German/English responsive Svelte portal, themes, dashboard, provider forms,
  record inspector, user/role/grant forms and audit view.
- Source Docker builds, private engine/DB, non-root apps, read-only roots,
  isolated contract mock, API/browser tests and CI definitions.

Migration IDs: `20261103000001_initial`, `20261103000002_retry_state`.
Do not change these migrations after publication.

## Verification checkpoint

Passed before final boundary changes: Rust check, 4 core unit tests, Clippy
with warnings denied, Go race tests, Svelte check (0 errors/warnings), formatting,
frontend build, real app/engine Docker builds, Compose config and startup.
42 API integration checks passed. Browser create/edit/enqueue/login/logout,
locale/theme, 30 width/theme/view combinations, reduced motion and keyboard
dialog checks passed without runtime or CSP errors.

Final results: 49 API integration checks pass, plus a positive scheduled AAAA
smoke test after the final SQLx/runtime changes. All 11 Go race tests and vet
pass, including service authentication, strict JSON, error redaction, sibling
binding, IPv6, response limits and deadlines. Rust fmt, Clippy and the four core
tests pass on the final feature set. Frontend check/lint/build pass. Browser
E2E and all 30 responsive theme/view cases pass again on the final runtime.
The actual private Go service passes an authenticated registry and invalid-input
contract probe without external provider requests. actionlint passes the three
workflow files. Catalog/documentation/descriptor/OpenAPI checks and a generated
runtime-secret source scan pass. See `docs/verification.md` for precise coverage.

`npm audit` reports four low-severity dependency entries from one deepmerge
prototype-poisoning advisory (GHSA-gcqx-w48m-w2h4), in the Svelte build toolchain.
No corrected deepmerge release was available at inspection (`npm view`: 4.3.1).
Do not apply the suggested pre-release-era SvelteKit downgrade. The shipped
frontend is static and the app image has no Node runtime. Track the build
dependency and block unreviewed/untrusted build configuration. CI fails moderate
or higher npm findings. Raw RustSec flags optional inactive rsa 0.9.10; the
graph-aware check confirms zero active findings over 240 active packages/all
targets and will block if that dependency becomes active. SQLx default features
are off; migrations are read-only runtime files. Initial Debian slim images
had high OS findings; ADR 0007 switches Rust to pinned Debian-13 distroless and
Go to scratch plus CA bundle. Final Trivy 0.76.1 app/engine scans have zero
HIGH/CRITICAL findings. Rust crates and PostgreSQL have separate review scope.

The first attempt to run the expanded test suite was not executed because the
automatic approval reviewer hit its usage limit; after the user said Weiter,
the same reviewed test invocation was accepted. This was a review-service
failure, not a security rejection. No approval bypass was used.

## Runtime and limits

QA project is `ddnsupd-qa`, with `.local/qa.env`, loopback port 18080 and an
internal Python engine mock. Public echo hosts resolve to loopback in QA.
Only this newly created QA volume was reset to test fresh migrations. Browser
credentials/screenshots are in `.local/` and must never enter Git or build contexts.
No real provider credentials, authorized test zone or live DNS changes exist.

QA is now stopped, with fixture data retained. The real local `ddnsupd-dev`
stack is healthy at `http://localhost:18081`. Port 8080 was already allocated,
so only the generated `.env` port/origin were changed; other services were left
alone. The development database is empty and needs setup with the private
`.env` setup token. API and engine roots are read-only and run as UID/GID 10001.

The Go adapters are mock-verified, not live-certified. Remote record creation,
zone listing, import/export, TOTP/reset/OIDC/LDAP, DNS/consensus/suffix discovery,
notifications, metrics/charts, API tokens and encrypted web recovery remain
planned. User administration is create/list/grant only. High-impact management
is built-in-admin-only. Record search/filter works on the loaded page.
Provider APIs cannot participate in DB transactions; external RRSet writers
must be coordinated. Production release/recovery/multiarch/source/SBOM gates
are not complete. No GHCR release exists; use the source Compose override.

## Next actions

Continue the roadmap with provider/zone discovery, preview/confirm import,
full identity administration and recovery before release. Keep evidence claims
limited to actual tests. The local checkpoint is usable; it is not the complete
seven-phase product or an approved production release. Source/SBOM publication,
build-toolchain digest pinning, multiarch and PostgreSQL image review are open;
live testing, remote repo creation, publishing and production deployment remain
separately authorized user actions.
