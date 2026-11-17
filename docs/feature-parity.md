# Feature parity and product status

Status: development checkpoint, 2026-11-03. **Unterstützt** means the stated bounded behavior has implementation and local test evidence. It does not certify a live provider account or a production release. Other statuses are In Entwicklung, Geplant, Nicht relevant and Technisch eingeschränkt.

Upstream evidence is frozen in [reference analysis](reference-analysis.md) and [source snapshot](reference-snapshot.json). Provider-by-provider coverage is [separate](provider-parity.md). The full user scope remains in [the product specification](product-specification.md).

| ID | Capability | Status | Scope / difference | Evidence |
| --- | --- | --- | --- | --- |
| F01 | IPv4 and A records | Unterstützt | Existing records, fixed/manual IP, validation and reconciliation | [provider-engine/engine_test.go](../provider-engine/engine_test.go), [tests/integration.py](../tests/integration.py) |
| F02 | IPv6 and AAAA records | Unterstützt | Public IPv6 validation, Cloudflare adapter mock and scheduled AAAA contract path verified | [provider-engine/engine_test.go](../provider-engine/engine_test.go), [tests/smoke.py](../tests/smoke.py) |
| F03 | Dual-stack and ipv4-or-ipv6 import semantics | In Entwicklung | Separate A/AAAA bindings; bounded import requires explicit single-family selection for ambiguous entries | [tests/management.py](../tests/management.py) |
| F04 | HTTPS family-specific discovery and fallback | In Entwicklung | Fixed v4/v6 sources implemented; failure/no-write path tested; positive echo path not yet certified | [tests/integration.py](../tests/integration.py) |
| F05 | DNS public IP discovery | Geplant | Retain upstream DNS discovery in the parity backlog | Pending |
| F06 | Configurable discovery URLs | Geplant | Connection-time SSRF policy required before enabling arbitrary destinations | Pending |
| F07 | Multiple-source consensus | Geplant | Fallback is not source agreement | Pending |
| F08 | Timeouts and bounded provider bodies | Unterstützt | Fixed endpoints, per-request client timeout and bounded responses | [provider-engine/engine_test.go](../provider-engine/engine_test.go) |
| F09 | Manual IP override | Unterstützt | Public address of record family only | [tests/integration.py](../tests/integration.py) |
| F10 | IPv6 suffix/prefix composition | Geplant | Preserve bit-level upstream semantics during import and worker extension | Pending |
| F11 | CGNAT/private/reserved address rejection | Unterstützt | Conservative public address policy; reachability is not guaranteed | [crates/ddnsupd-core/src/lib.rs](../crates/ddnsupd-core/src/lib.rs), [provider-engine/engine_test.go](../provider-engine/engine_test.go) |
| F12 | Provider registry and credential schemas | Unterstützt | Two implemented adapters advertise capabilities; complete inventory tracked separately | [provider-engine/engine_test.go](../provider-engine/engine_test.go), [tests/integration.py](../tests/integration.py) |
| F13 | All upstream providers | Geplant | 60 registry entries tracked; example excluded; no implicit imported support | Pending |
| F14 | Hetzner legacy updates | Technisch eingeschränkt | Service API removed 2026-05-27; migration identification retained | Pending |
| F15 | Actual provider/origin comparison | Unterstützt | Cloudflare origin read, not public proxy IP; Hetzner RRSet read | [provider-engine/engine_test.go](../provider-engine/engine_test.go) |
| F16 | Avoid unchanged updates | Unterstützt | No-change checks are distinct from successful writes | [provider-engine/engine_test.go](../provider-engine/engine_test.go), [tests/integration.py](../tests/integration.py) |
| F17 | Sibling record and metadata preservation | Unterstützt | Hetzner preserves sibling values/comments; Cloudflare PATCH only content | [provider-engine/engine_test.go](../provider-engine/engine_test.go) |
| F18 | Provider record creation | In Entwicklung | Confirmed creation of missing A/AAAA RRSets in Hetzner primary zones. Ordinary checks only update existing records. | [tests/integration.py](../tests/integration.py), [provider-engine/engine_test.go](../provider-engine/engine_test.go) |
| F19 | Global/provider/record interval hierarchy | In Entwicklung | Five-minute default and per-record interval implemented; configurable hierarchy pending | [tests/integration.py](../tests/integration.py) |
| F20 | Startup check and automatic scheduling | Unterstützt | Due records queued at initial loop, with persistent state | [tests/integration.py](../tests/integration.py) |
| F21 | Manual check and cooldown | Unterstützt | POST queues/coalesces checks; no cooldown bypass | [tests/integration.py](../tests/integration.py) |
| F22 | Persistent jobs and restart recovery | Unterstützt | Queued jobs survive process restart; expired leases requeue | [tests/integration.py](../tests/integration.py) |
| F23 | Exponential backoff and jitter | In Entwicklung | Persistent failure count and bounded jitter implemented; complete progression tests pending | Pending |
| F24 | Rate-limit / Retry-After handling | Unterstützt | Classified error and account blocking; fixed quota is not assumed | [provider-engine/engine_test.go](../provider-engine/engine_test.go), [tests/integration.py](../tests/integration.py) |
| F25 | Authentication failure suspension | Unterstützt | Stops automatic retries until credential replacement | [tests/integration.py](../tests/integration.py) |
| F26 | Concurrent writer and crash-after-send handling | In Entwicklung | DB/provider locks, leases and read-before-write implemented; fault-injected lease-loss acceptance pending | Pending |
| F27 | Persistent update/IP history | Unterstützt | PostgreSQL attempt history with before/after, trigger, duration and safe error | [tests/integration.py](../tests/integration.py) |
| F28 | Legacy updates.json history import | Geplant | Preserve domain/owner/ips and legacy host mapping; imported provenance distinct | Pending |
| F29 | Propagation-aware status and TTL health | In Entwicklung | Origin verified status exists; public DNS/TTL observation not implemented | Pending |
| F30 | Liveness and readiness | Unterstützt | HTTP process and DB readiness separated; Compose health checks pass | [tests/integration.py](../tests/integration.py) |
| F31 | Per-record/provider health | In Entwicklung | Check outcomes, suspension and account blocking shown; richer diagnostics pending | Pending |
| F32 | Prometheus metrics | Geplant | Authenticated metrics and counters to be added | Pending |
| F33 | Healthchecks.io heartbeat | Geplant | Upstream integration retained in backlog; notification equivalent requires explicit implementation | Pending |
| F34 | Shoutrrr notifications | Geplant | Upstream feature not yet adapted | Pending |
| F35 | SMTP/webhooks/notification rules | Geplant | Scoped outbox, retry, SSRF policy, deduplication and escalation required | Pending |
| F36 | JSON configuration import / legacy owner-host | In Entwicklung | Single configured Cloudflare/Hetzner Cloud account/zone; preview/confirm, legacy names, conflicts and paused atomic imports; history/suffix/full parity pending | [tests/management.py](../tests/management.py), [tests/browser/e2e.mjs](../tests/browser/e2e.mjs) |
| F37 | CSV/JSON export without secrets | Geplant | Safe resource-scoped export pending | Pending |
| F38 | Periodic upstream data backup equivalent | In Entwicklung | Optional daily encrypted backups, seven rotating slots | [Compose override](../docker-compose.backup.yml) |
| F39 | Encrypted complete backup/restore | In Entwicklung | Manual export, setup and admin replacement, integrity and schema checks, credential rekeying | [tests/backup.py](../tests/backup.py) |
| F40 | Container runtime / persistent volumes | Unterstützt | Source stack runs locally with DB volume and non-root apps | [tests/integration.py](../tests/integration.py) |
| F41 | Versioned images without local production build | In Entwicklung | Compose references prepared; no published release exists | Pending |
| F42 | amd64 / arm64 / other upstream architectures | In Entwicklung | amd64 locally built; arm64 not tested; additional upstream targets require scope decision | Pending |
| F43 | Standalone binaries / AUR / Kubernetes manifests | Nicht relevant | Current product distribution contract is Docker Compose; document scope rather than imply parity | Pending |
| F44 | ROOT_URL/path-prefix hosting | Technisch eingeschränkt | Serve at origin root; path-prefix deployment not supported yet | Pending |
| F45 | Reverse proxy / secure session policy | In Entwicklung | Exact configured Origin, HTTPS secure cookies and ignored forwarded identity; real Zoraxy deployment unverified | Pending |
| F46 | Setup / local login / logout | Unterstützt | Runtime-token setup closes atomically; hashed sessions expire | [tests/integration.py](../tests/integration.py) |
| F47 | Password changes / reset / TOTP | In Entwicklung | Current-password change and revocation tested; reset and TOTP pending | [tests/integration.py](../tests/integration.py) |
| F48 | Brute-force protection / cookie CSRF | Unterstützt | Persistent username/global attempt bounds; exact Origin and per-session token | [tests/integration.py](../tests/integration.py) |
| F49 | Built-in roles / custom role creation / grants | Unterstützt | Operation and resource scope independently enforced | [tests/integration.py](../tests/integration.py) |
| F50 | User and role lifecycle administration | In Entwicklung | Create/list and grants exist; edit/disable/delete and full role lifecycle pending | Pending |
| F51 | API tokens / scopes / rotation | Geplant | Cookie sessions exist; do not imply API-token support | Pending |
| F52 | OIDC / LDAP/AD | Geplant | Optional identity extensions after local authentication hardening | Pending |
| F53 | Encrypted credential storage and replacement | Unterstützt | Provider-bound AEAD, fresh nonce, write-only reads, replacement clears suspension | [crates/ddnsupd-core/src/lib.rs](../crates/ddnsupd-core/src/lib.rs), [tests/integration.py](../tests/integration.py) |
| F54 | Master-key rotation | Geplant | Version field exists; rotation procedure and recovery test pending | Pending |
| F55 | Mutating API and DNS audit | Unterstützt | Explicit actions audited without secrets; DNS attempts recorded in history | [tests/integration.py](../tests/integration.py) |
| F56 | Full API OpenAPI / pagination / filters | In Entwicklung | Versioned OpenAPI file and offset pages exist; response detail/generation and server filtering pending | Pending |
| F57 | Responsive portal / themes / DE-EN | Unterstützt | Actual API views, 30 responsive theme checks and language persistence | [tests/browser/e2e.mjs](../tests/browser/e2e.mjs) |
| F58 | Record inspector and history view | Unterstützt | URL-selected inspector, edit/check controls and activity view | [tests/browser/e2e.mjs](../tests/browser/e2e.mjs) |
| F59 | Dashboard counts and recent activity | Unterstützt | Resource-scoped counts and real recent events; no synthetic operational data | [tests/integration.py](../tests/integration.py), [tests/browser/e2e.mjs](../tests/browser/e2e.mjs) |
| F60 | Public IP timeline and charts | Geplant | No dashboard graph implied by history table | Pending |
| F61 | Provider connection/permission test and zone discovery | In Entwicklung | Paginated read-only access test and zone/record discovery for two adapters; write permission requires authorized live verification | [provider-engine/discovery_test.go](../provider-engine/discovery_test.go), [tests/management.py](../tests/management.py) |
| F62 | Zone/domain overview, wildcard discovery | In Entwicklung | Paginated zone/A/AAAA selection and paused binding/import exist; dedicated domain overview pending | [tests/browser/e2e.mjs](../tests/browser/e2e.mjs) |
| F63 | Search/sort/filter/bulk actions | In Entwicklung | Search/status filter on current loaded page; server sort/filter and bulk actions pending | Pending |
| F64 | External authenticated IP agents | Geplant | Resource-scoped agent contract and token support required | Pending |
| F65 | Structured logs and safe errors | Unterstützt | JSON operational logs and classified provider errors, no raw secrets | [provider-engine/engine_test.go](../provider-engine/engine_test.go), [tests/integration.py](../tests/integration.py) |
| F66 | Retention, scheduled cleanup and policy | Geplant | No automatic history retention yet; session/attempt cleanup implemented | Pending |
| F67 | CI, security scans and release workflow | In Entwicklung | Local checks and workflow definitions exist; GitHub runs/publication/complete release gates pending | Pending |
| F68 | Custom HTTPS updater | Geplant | Upstream custom retained in inventory; disabled until SSRF and secret-URL controls exist | Pending |
