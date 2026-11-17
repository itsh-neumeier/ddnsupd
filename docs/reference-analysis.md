# Reference analysis

Observed on 2026-11-03 through the connected GitHub API. The immutable source
links and directory hashes are in [reference-snapshot.json](reference-snapshot.json).
These are source observations, not evidence that either project was executed.

| Reference | Commit | License | Role |
| --- | --- | --- | --- |
| [TalkOps](https://github.com/itsh-neumeier/talkops/commit/00b968c0cb8b8d40312e52933d8c0144d1e36e34) | `00b968c0cb8b8d40312e52933d8c0144d1e36e34` | AGPL-3.0-only | Management/UI reference |
| [ddns-updater](https://github.com/qdm12/ddns-updater/commit/7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2) | `7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2` | MIT | Provider reference |

## TalkOps observations

README, AGENTS, CLAUDE, architecture, UI guidelines and release workflow were
read. Selected ADRs cover PostgreSQL queue (0005), credential encryption
(0008), encrypted recovery (0021) and fine-grained roles (0029).

- Rust/Axum/Tokio/SQLx control plane, PostgreSQL 17 and static SvelteKit SPA.
- `web/package.json` at this snapshot uses SvelteKit 3, Svelte 5 and Tailwind 4.
  Do not assume the older SvelteKit 2 configuration when implementing the UI.
- Domain libraries are separate from HTTP handlers. DB jobs avoid a broker.
- `CLAUDE.md` is the shared instruction source. AGENTS points to it. Worktree,
  PR, release and task-memory conventions support sequential or parallel tools.
- `web/src/app.css`, root layout, login route, `lib/api.ts` and `lib/roles.ts`
  show teal/slate themes, API/session separation and permission display helpers.
- ADR 0029 is more current than the older role summary in architecture.md:
  custom permissions replace simple role ordering. Server checks are authoritative.
- Recovery ADR 0021 treats missing deployment secrets and keys as incomplete
  recovery. It avoids Docker socket access and validates archives before restore.
- TalkOps uses host networking for telephony and excludes Docker Desktop from
  deployment support. This is a domain constraint, not a DDNS requirement.

UI principles adopted: one topbar, responsive navigation, one fixed footer,
container-query content grids, table column priorities, URL-addressable
inspector, icon consistency, separate loading/empty/error states, DE/EN keys,
keyboard focus and reduced-motion handling. These will be implemented for DDNS.

## ddns-updater observations

| Area | Source and behavior | DDNSupd implication |
| --- | --- | --- |
| Providers | `internal/provider/provider.go`: factory switch and interface with Update and HTML | Adapt provider code behind a new management-independent interface |
| Inventory | 60 directories/constants/factory entries; example is a scaffold | Track all 60 explicitly; 59 are production candidates |
| Catalog mismatch | custom and Vercel occur in source, README is incomplete; ProviderChoices omits Netcup/custom | Do not derive parity from README alone |
| Configuration | `internal/params/json.go`: settings array, owner/host legacy mapping, comma domains, eTLD+1 parsing | Import with per-entry diagnostics; do not silently collapse records |
| Scheduling | `internal/update/service.go`: ticker, forced pass, global cooldown, one-hour abuse pause | Add persistent jobs and per-record/account timing |
| Startup | Service.Start launches run; run waits for ticker/force | Do not claim this loop itself performs an immediate startup pass |
| Discovery | `pkg/publicip` and config/pubip.go: HTTP and DNS fetchers, family-specific lists, timeouts | Reuse concepts, add explicit public-address policy and later consensus |
| Retries | getip.go retries discovery three times; service.go retries resolution five times | Classify retries; do not imply exponential backoff exists here |
| Comparison | Non-proxied records compare DNS results; proxied records compare local history | Read provider origin state for proxy drift detection |
| IPv6 | update/ipv6.go composes prefix/suffix with bit handling | Preserve semantics with boundary tests and explicit import conversion |
| Updates | update/update.go invokes Provider.Update and appends IP/time history | Separate attempt, provider verification and propagation states |
| Hetzner Cloud | setrecord.go sends set_records with a single new value | Adapt before use: preserve other RRSet values and comments or reject unsafe ambiguity |
| Cloudflare | provider.go reads origin content, identifies matching record and updates/creates | Use explicit record binding; preserve unrelated metadata and reject ambiguous records |
| Persistence | JSON records have domain/owner/ips; host compatibility; IP events ordered by time | Import legacy history as provenance, never as a new successful write |
| Health | health/check.go accepts prior IPs within TTL and skips proxy DNS comparison | Separate process readiness from record correctness and propagation |
| HTTP API | server/handler.go has GET /, GET /update and static resources under ROOT_URL | Not a CRUD management API; never expose mutating GET in DDNSupd |
| Notifications | Shoutrrr client receives success and abuse messages | Add rule scope, deduplication, outbox and escalation |
| Containers | Scratch, UID/GID 1000, healthcheck command, many architecture targets | Reuse minimal/unprivileged concept; own runtime and support tests |
| Backup | README describes optional periodic data-directory backup | PostgreSQL + application key require a new recovery design |

All relevant upstream features are tracked in [feature-parity.md](feature-parity.md).
Upstream source or upstream tests do not certify DDNSupd support.

## Current provider service availability

The pinned upstream still contains a legacy Hetzner provider and a warning
that shutdown is forthcoming. The official status page states that the old
console and API were removed on **2026-05-27**. Keep the import identifier,
but disable live legacy writes and guide migration to Hetzner Cloud with a
new token. Old tokens are not Cloud API tokens.

Sources: [Hetzner shutdown notice](https://status.hetzner.com/de/incident/c2146c42-6dd2-4454-916a-19f07e0e5a44),
[Hetzner integration differences](https://docs.hetzner.com/networking/dns/migration-to-hetzner-console/features-and-differences/).
This external fact is separate from the frozen upstream feature inventory.

## License and maintenance

The user selected AGPL-3.0-only for DDNSupd. MIT provider adaptations retain
the upstream copyright and permission notice, exact origin and modification
description. TalkOps concepts are implemented independently; any later direct
code reuse needs explicit file provenance and retained notices. AGPL network
source duties are part of distribution planning, see
[GNU license explanation](https://www.gnu.org/licenses/license-list.html#AGPLv3.0).
Dependency license review and an SBOM remain release gates.

Upstream refreshes are reviewed changes to pinned commit, catalog and adapter
tests. No build fetches mutable master. Do not auto-enable a new provider just
because its directory exists upstream.
