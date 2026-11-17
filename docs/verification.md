# Local verification record

For 0.1.2, the full npm build graph reports GHSA-fc5q-39xc-vjpf in picomatch
4.0.7. No patched upstream version is available at 2026-11-17. A bounded
build-only exception expires on 2026-12-17. The shipped application has no
Node runtime. See [the npm audit wrapper](../scripts/audit-web.mjs) and
[the upstream advisory](https://github.com/advisories/GHSA-fc5q-39xc-vjpf).

On 2026-11-17 two isolated QA databases with different runtime encryption
keys verified setup authorization and confirmation, wrong passphrase and
tampering rejection, uploads above the normal API body limit, database and
credential recovery, paused records and session invalidation on replacement.
The browser downloaded an encrypted archive. The Compose schedule generated
an encrypted daily file on the initialized target.

Date: 2026-11-03. Scope: development amd64 build on Docker Desktop. No remote
GitHub CI run, live provider test or production deployment is claimed.

| Check | Result and scope |
| --- | --- |
| Reference/catalog checks | 60 frozen provider entries, complete descriptors, local links and 28 OpenAPI operations validated |
| Rust | Workspace builds; 8 core unit tests pass; fmt and Clippy with warnings denied pass |
| Go | 14 tests pass with race detector; vet passes; no live transport in unit tests |
| Frontend | Svelte check: 0 errors/0 warnings; Prettier and static build pass |
| API integration | 49 checks pass against isolated Python contract engine, including setup race, CSRF, grants/revocation, crypto storage, scheduled update, cooldown, rate-limit, suspension/rotation, restart and retained history |
| Discovery/import | 30 additional checks pass: read-only pagination/grants, redaction, family/conflict handling, expiry, credential rotation, atomic rollback, concurrent consumption, paused/no-job imports and persisted page budget |
| IPv6/final runtime smoke | Scheduled AAAA job, history and persisted DB/session access pass after SQLx and runtime changes |
| Browser | Discovery, paused binding, paginated import preview/review/confirm; provider/record creation, edit, enqueue, locale/logout; 30 viewport/theme/view cases; dialog Escape/reduced motion; no runtime/CSP errors |
| Compose | Base/development/test config checks pass; fresh technical stack and existing migrations start with healthy services |
| Runtime images | App uses pinned Debian-13 distroless; Go uses scratch + CA certificates; UID/GID 10001 and binary healthchecks |
| Actual private engine contract | Service authentication, registry response and invalid-provider rejection pass without external DNS requests |
| Image scans | Trivy 0.76.1: zero HIGH/CRITICAL findings in final app OS packages and final Go binary image. Rust crates are checked separately, not detected by the OS image scan |
| Rust dependencies | Graph-aware RustSec check: zero active findings over 240 active packages/all targets; one inactive optional RSA advisory retained transparently |
| npm dependencies | Four low-severity entries from one build-tool deepmerge advisory; no patched upstream version at inspection. Moderate-or-higher CI threshold; no Node runtime in the shipped app |
| Workflow syntax | actionlint 1.7.7 passes all three workflow files; workflows are prepared, not run on GitHub |
| Runtime secrets | Generated local runtime values absent from source files; `.env` and `.local` excluded from Git/build contexts |

Previous bootstrap image IDs (local only; superseded by the discovery increment):

- App: `sha256:4a41d7465099099b55ac04e52e200b387f7cbc4cf917e0e6d8eef36a8cb4e1d2`, 59,095,322 bytes.
- Engine: `sha256:9b8019205d24c7ec65b0e971e20e2100c2b2c5b71d667a5daf754ecfaca3bb3b`, 10,390,216 bytes.

Discovery/import images (local only):

- App: `sha256:1bee3c8c44cec61695d8b702cceaac839b3378829c10e545c4fd86b8639a618d`, 59,809,333 bytes.
- Engine: `sha256:5ab50719653af9c77a4e2e06a783cfc6d9bea8a298a36d4a5b9820e07d635396`, 10,416,108 bytes.

Full local scan JSON, image archives and browser screenshots are under `.local/`.
They are not publication artifacts. Initial Debian slim reports are retained
as `app-scan.initial.json` and `engine-scan.initial.json`; they caused the runtime
reduction in ADR 0007. The final scans do not suppress those OS findings: the
unneeded affected packages are absent from the final images.

## Dependency finding limits

Raw `cargo audit` still flags RUSTSEC-2023-0071 (`rsa 0.9.10`) from optional SQLx
MySQL lock entries. `cargo tree -i rsa` has no active path. The wrapper checks the
locked active graph across all targets and does not ignore by advisory ID. If
RSA becomes active, that same advisory automatically blocks CI. SQLx default
features are disabled and migration files are packaged read-only at runtime.

The npm finding is GHSA-gcqx-w48m-w2h4 in deepmerge 4.3.1, inherited through the
Svelte build tooling. Do not apply the audit suggestion to downgrade to obsolete
pre-release Svelte packages. Track an upstream fix; build trusted source only.

## Not verified / open release gates

- Real provider credentials, account permissions, authorized test zones and live
  DNS propagation. Mocks do not certify a provider account.
- Positive live public-IP echo behavior on the user's IPv4/IPv6 network. The
  failure/no-write path and public-address validation are tested locally.
- Full crash/lease-loss fault injection and coordination with external RRSet writers.
- TOTP/reset, complete cross-provider/history/suffix imports, notifications, API tokens, metrics, retention and complete
  backup/key recovery on a fresh host.
- PostgreSQL deployment image review, complete license/SBOM/source publication,
  pinned build toolchain digests, arm64 builds and approved GHCR publication.

The clean development instance is at `http://localhost:18081`, project
`ddnsupd-dev`. It has no users or provider credentials; use the private `.env`
setup token. The default 8080 port was already allocated. QA is a separate
project on 18080 and is stopped after verification; its fixture data is retained.
The discovery regression run used a fresh `ddnsupd-qa-discovery` project with
the existing QA env and isolated contract engine, retaining the prior QA volume.
The development DB and existing runtime keys were preserved during the upgrade.
