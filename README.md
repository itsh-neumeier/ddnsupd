# DDNSupd

[English](docs/en/installation.md) · [Deutsch](docs/de/installation.md)

Self-hosted Dynamic DNS management with a Rust API, Svelte web portal,
PostgreSQL and a private Go provider engine. Licensed **AGPL-3.0-only**.

**Current release:** 0.1.0 for Linux/amd64. Setup, authentication, scoped
permissions, provider credentials, scheduled A/AAAA reconciliation, history,
discovery, imports and encrypted recovery are included. Hetzner Cloud and
Cloudflare adapters have mock verification. See the provider matrix for scope.

## Portainer and release images

Repository URL: `https://github.com/itsh-neumeier/ddnsupd`

Repository reference: `refs/tags/v0.1.0`

Compose path: `docker-compose.portainer.yml`

Set the private environment variables and the external URL before deploying.
The stack includes IPv4/IPv6 egress. Follow the [Portainer guide](docs/de/portainer.md).

For a regular Compose deployment, generate the private `.env` and run
`docker compose up -d --wait`. Images are
`ghcr.io/itsh-neumeier/ddnsupd:0.1.0` and
`ghcr.io/itsh-neumeier/ddnsupd-worker:0.1.0`.

## Run from source

Requires Docker Engine with Compose 2.24.4 or later, or Docker Desktop, and
Python 3 for first configuration. No local Rust, Go or npm installation is
needed. Run in this directory:

```sh
python scripts/init-env.py
docker compose -p ddnsupd-dev -f docker-compose.yml -f docker-compose.dev.yml config --quiet
docker compose -p ddnsupd-dev -f docker-compose.yml -f docker-compose.dev.yml up -d --build --wait
```

Open [http://localhost:8080](http://localhost:8080). Read `DDNSUPD_SETUP_TOKEN`
from your private `.env` and create the first administrator. Use a username
with lowercase letters, digits, `.`, `_`, `-` or `@`, and a password with at
least 12 characters. Setup closes after the first account is created.
To migrate an existing installation, choose **Restore backup** on this screen.

The environment generator refuses to overwrite an existing file. Keep it and
the PostgreSQL volume together: losing or replacing `DDNSUPD_SECRET_KEY`
prevents credential decryption. Do not use test credentials in a real instance.

The base Compose file uses versioned GHCR images. The development override
builds the same services from source.

## Use the portal

1. Connect a Hetzner Cloud or Cloudflare account with a scoped API token.
2. Add a local binding to an **existing** A or AAAA record. Enter the provider
   zone ID, canonical zone name, relative name and, for Cloudflare, record ID.
3. Select a manual public IP or leave it empty for family-specific HTTPS
   discovery. Set the interval (60–86400 seconds) and site.
4. Review the binding, then enable automatic checks. An enabled record can
   change real DNS as soon as it is due. Use only authorized zones.
5. Open the inspector, enqueue a check and read its history. Manual checks
   respect the next due time and provider rate limits. Origin verification
   does not certify global propagation.

The engine preserves unrelated Hetzner RRSet members/comments and updates
only Cloudflare origin content. It rejects ambiguous or stale bindings instead
of silently replacing other values. A single RRSet needs one coordinated
writer: external writers are outside PostgreSQL locks and provider APIs do not
provide a distributed transaction with DDNSupd.

## Verification

The isolated test stack uses a Python contract mock with no external network.
The API has only a loopback-published test port; its public IP echo names map
to loopback to test discovery failure without external requests.

```sh
python scripts/check-docs.py
python scripts/init-env.py --output .local/qa.env --port 18080
docker compose --env-file .local/qa.env -p ddnsupd-qa -f docker-compose.yml -f docker-compose.dev.yml -f docker-compose.test.yml up -d --build --wait
python -u tests/integration.py
```

These integration tests require an empty, disposable test DB. They create
accounts and fixtures and restart only the named QA application. Browser
tests use the generated `.local/qa-credentials.json`, never production secrets.
See [tests and developer commands](docs/en/development.md) and
[CI](.github/workflows/ci.yml). The real Go adapter tests use injected HTTP
transports and cannot contact DNS providers.

## Scope and remaining work

- Implemented: setup/login/logout/password change, four built-in roles,
  custom-role creation and explicit grants; encrypted write-only provider
  tokens; record bindings/edit/delete/enable; persistent checks/history/audit;
  read-after-write adapters; read-only zone/record discovery, paused binding
  selection and staged configuration import; DE/EN, themes, tables and inspector.
- Partial: user administration (create/list/grants), provider permissions,
  dashboard, OpenAPI contract, IP discovery and registry. Import targets one
  supported account/zone and existing records. Search/filter uses the loaded page.
- Planned: TOTP/reset/OIDC/LDAP, remote record creation for more providers,
  complete import/export, IPv6 suffixes, DNS/consensus/custom IP sources, bulk
  actions, notification channels, Prometheus, charts, API tokens and complete
  provider parity.
- Recovery: encrypted downloads, optional daily rotation and setup/admin
  restore are available. See [backup instructions](docs/en/installation.md#backup-and-recovery).
- Not released: multiarch validation, final security
  review, approved GHCR publication and production deployment.

The complete status is in the [feature matrix](docs/feature-parity.md),
[60-entry provider matrix](docs/provider-parity.md), [roadmap](docs/roadmap.md)
and [task memory](docs/ai-memory/2026-11-03-discovery-import.md).
Actual check results and security limits are in the [verification record](docs/verification.md).

## Architecture and provenance

[Architecture](docs/architecture.md) · [Reference analysis](docs/reference-analysis.md) ·
[ADRs](docs/adr/0001-management-stack.md) · [API contract](docs/openapi.json) ·
[Test plan](docs/test-plan.md) · [Contributor rules](CLAUDE.md)

Reference snapshots: TalkOps `00b968c0cb8b8d40312e52933d8c0144d1e36e34`,
ddns-updater `7d6fd785f78317fc8bc53e6b245a1d33e43f3ad2`.
The latter still contains legacy Hetzner code, but Hetzner removed the old API
on 2026-05-27. That entry remains migration-only, not supported for updates.
See the official evidence in the reference analysis.

Provider adaptations retain their [MIT notice](provider-engine/LICENSE.upstream).
Other source and license details are in [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
