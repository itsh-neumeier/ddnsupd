# Installation and first steps

Release 0.1.2 provides Linux/amd64 images on GHCR. Use the base Compose file
for release images or the development override for source builds.
Compose 2.24.4+ is required for the test override's explicit list replacement.

Run the commands in [README](../../README.md). Store `.env` privately. Windows
chmod does not replace NTFS access controls; restrict access to your account.
Named volume `postgres-data` stores application state. The app and Go engine
run as UID/GID 10001; PostgreSQL runs as its own postgres user. App/engine
roots are read-only, with a small temporary filesystem and no capabilities.

Only port 8080 is published, on 127.0.0.1 by default. PostgreSQL and the engine
have no published ports. To use another local port, generate `.env` with
`python scripts/init-env.py --port 8081` before the first start. The external
URL must match the browser Origin exactly, including its port.

The application key requires exactly 64 hexadecimal characters. Generate it
with `scripts/init-env.py`. HTTP is supported on localhost and private LAN IP
addresses. Other origins require HTTPS.

## Reverse proxy / Zoraxy

Set `DDNSUPD_EXTERNAL_URL=https://dns.your-domain.example`, configure Zoraxy
to forward this origin to port 8080, preserve Host and Origin and terminate
TLS at the proxy. The configured HTTPS origin enables Secure session cookies.
If Zoraxy is on another host, choose an appropriate bind address and firewall
rule; do not expose the backend directly to the Internet. A proxy on the host
can use the default loopback mapping.

The application does not trust forwarded identity/IP headers and cannot
authenticate by proxy headers. No WebSocket connection is needed. Deploy at
the origin root; path-prefix hosting is not supported in this increment.

## Provider setup

Hetzner Cloud: use a project-scoped read/write token, preferably for a dedicated
DNS project. Legacy DNS tokens do not work. Enter the Cloud zone ID and name.
Cloudflare: use a scoped token with Zone Read and DNS Edit for the selected
zone; enter the existing zone and record IDs. Do not use a global API key.

Create the account under Providers. Tokens are encrypted and cannot be read
back. Replace them through Edit; an empty replacement keeps the existing token.
Replacing a token clears authentication suspension. Disabling the provider
stops its future updates. Local deletion is refused while records are bound.

Add a record with relative name (`home`, `@` or `*`), type and provider identity.
Keep new records paused while reviewing scope and IP source. For multiple
RRSet values, provide the exact previously
managed IP; an ambiguous binding is rejected. Do not run another writer on the
same RRSet without coordination. Deleting a local binding does not delete DNS.

For an active Hetzner Cloud binding, choose **Create missing DNS record** in the
record details and confirm **Create at Hetzner**. This queues creation of a
missing A or AAAA record in a primary zone using the configured IP source.
Cooldowns still apply. An existing record of that type is left untouched by
the creation job. Check **Activity** for the result.

Empty manual IP selects HTTPS discovery with a fallback for that family.
Dual-stack uses two records, A and AAAA. Private/CGNAT/reserved addresses are
rejected. Reachability, port forwarding and DS-Lite are not repaired by DDNS.
Stable IPv6 suffix handling is planned; do not assume a discovered temporary
IPv6 address is a stable host identity.

The Compose egress network assigns both IPv4 and IPv6. Automatic AAAA updates
still require the container to resolve AAAA answers and reach an IPv6 address
discovery service over HTTPS. If the host lacks public IPv6 connectivity, use
a stable public IPv6 address of the target as the manual IP.

## Discover and import existing records

In Providers, select an account and choose **Test access / load zones**, then
select a zone and load its records. Browse pages explicitly. This checks read
access only; it cannot certify DNS write permission. Select a public A/AAAA
value to create a paused local binding. No remote records are created.

Administrators can upload a ddns-updater `config.json` for that selected account
and zone. Review the preview, resolve conflicts and select the rows to import.
Choose A or AAAA explicitly for ambiguous `ipv4 or ipv6` entries. Source tokens
are ignored; the saved account credential is used. Provider TTL/proxy settings
are preserved. Confirmation requires review acknowledgement. All bindings stay
paused until explicitly activated in Records; import never enqueues DNS writes.

Previews expire after ten minutes, are single-use and become invalid on credential
rotation. Limits: 60-KiB UI file, 200 expanded entries, 20 provider pages/2000
address values, 30-second preview discovery and 30 provider pages/minute. Exceeding
limits fails explicitly. Cross-account/history/suffix import is still planned.

## Users and roles

Administrators can create users, roles and resource grants in Administration.
The role permits operations; the grant or record ownership determines scope.
The Viewer role never permits writes, even for a granted record. A record grant
does not reveal provider credentials. High-impact account/role/credential
administration is limited to built-in administrators in this increment.
User disable/delete, role editing and password reset are planned. A user can
change their own password with the current password; this revokes all sessions.

## Health, troubleshooting and recovery

`/healthz` reports the HTTP process; `/readyz` checks PostgreSQL. Record errors
are in Activity and do not cause a restart loop. Engine errors are classified
without provider response bodies. Authentication/permission failures suspend
the account; 429 respects Retry-After. Checks can remain queued until cooldown
expires. `GET /api/v1/openapi.json` needs an authenticated session.

If login/setup fails with forbidden, verify the exact external URL and browser
Origin. If no records update, check active flags, provider suspension, next
check time and the IP source. To see service health, run `docker compose ps`;
use `docker compose -p ddnsupd-dev -f docker-compose.yml -f docker-compose.dev.yml ps`
for the development instance. Logs use JSON and contain no request
bodies or tokens.

## Backup and recovery

Administrators can download an encrypted archive under **Administration →
Backup**. Store the file and passphrase separately. The archive includes
users, roles, grants, DNS bindings, providers and credentials, update history
and audit. Restore re-encrypts provider credentials with the destination key.

On a fresh installation select **Restore backup** during setup and provide
the destination setup token, backup file and passphrase. On an existing
installation the restore form in **Administration** replaces the current data
after explicit confirmation. Sign in as a backed-up user afterwards. Restored
bindings are paused until you verify their IP sources and reactivate them.
Existing sessions and pending jobs are discarded. Damaged files and wrong
passphrases fail before replacing data.

Archives support up to 16 MiB and currently require the same DDNSupd database
schema version. Configure ports, URL, database and engine access in the
destination's private runtime configuration. For larger installations, keep
a PostgreSQL dump and its matching application key as separate recovery material.

For daily backups add `-f docker-compose.backup.yml` to the Compose command.
Set `DDNSUPD_BACKUP_HOST_DIR` to a private directory and
`DDNSUPD_BACKUP_HOST_PASSPHRASE_FILE` to a protected passphrase file.
Container UID 10001 needs write access to the directory and read access to the
file. DDNSupd checks hourly for the current UTC day's backup and rotates seven
files. Copy important archives to separate storage.
