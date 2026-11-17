# Provider discovery and staged configuration import

- Date: 2026-11-03
- Status: implemented and locally verified; production/live gates remain open
- User request: continue autonomous development from the local MVP checkpoint.

Read AGENTS/CLAUDE, bootstrap memory, architecture/ADRs/UI rules and roadmap.
The directory still has no Git repository; no branch, commit, remote, publication
or live DNS action was created. Preserve the existing `.env`, keys and volumes.

## Scope and changes

Extend the two existing adapters with read-only zone/record discovery, pagination,
safe permission errors, read-only connection test and binding selection in UI.
Add a pure Rust upstream config parser and user-bound preview/confirm API. New
table migration: `20261103000003_import_previews`. Normalized drafts contain no
source credentials, expire after ten minutes and are single-use. Imported records
are always paused. ADR 0008 records scope, limits and safety decisions.

Files: Go discovery module/tests and HTTP handler; core config_import module;
API routes/management.rs and shared engine client config; Svelte discovery/import
components and DE/EN strings; isolated engine fixtures, management integration
tests and browser workflow; OpenAPI and CI updates.

## Verification

Rust workspace tests (eight core tests), formatting and Clippy with warnings
denied pass. Go tests (14 total) pass with race detector and vet. Svelte check
has zero errors/warnings; formatting and static build pass. A fresh isolated
`ddnsupd-qa-discovery` database passed all 49 baseline API checks, the IPv6
runtime smoke and 30 new management checks. Browser discovery, paused binding,
two-page import matching, review acknowledgement and confirmation pass with the
existing CRUD/locale/keyboard and 30 responsive theme/view checks; no runtime/CSP
errors. The discovery-import screenshot was visually inspected. OpenAPI checks
cover 28 operations; 60 provider descriptors and documentation links pass.
Actionlint passes all three workflows. Runtime-secret source scan passes.
Both final runtime images pass Trivy HIGH/CRITICAL scans (zero findings); Rust
dependencies are checked separately. See verification.md for image identities.
The actual private Go engine authentication/registry and unsupported discovery/
reconcile probes pass without outbound provider calls.

An attempted QA-volume reset was rejected by automatic approval review because
volumes must be preserved. No deletion occurred. The final regression instead
used a separate fresh project and retained both QA volumes. Both QA projects
are now stopped. Management tests accept DDNSUPD_QA_PROJECT; baseline integration
accepts --project. Canonical Origin is preserved when Python connects over IPv4.

Development stack `ddnsupd-dev` was upgraded with the new migration/images and
is healthy at http://localhost:18081. Pre-upgrade counts were zero users,
providers and records. Runtime configuration, keys and volumes are unchanged.
No Git initialization, remote, publication or deployment occurred.

## Boundaries and next actions

No real credentials/test zone supplied; no external DNS changes authorized.
Only fixed official API endpoint reads are implemented; local validation uses
injected Go transports and the isolated Python engine. Read access never claims
write permission. Matrices, architecture/model, ADR, bilingual installation,
developer instructions, roadmap, changelog and verification reflect this scope.
Keep full cross-provider/history/suffix/credential import and remote creation
marked open. Next management work includes scoped safe export, server-side
filter/sort and fuller user lifecycle; recovery/notifications/release remain open.
