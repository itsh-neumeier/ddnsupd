"""Round-trip recovery between isolated QA projects with different application keys."""
import argparse
import http.cookiejar
import json
from pathlib import Path
import secrets
import subprocess
import urllib.error
import urllib.request

parser = argparse.ArgumentParser()
parser.add_argument("--source-project", default="ddnsupd-qa")
parser.add_argument("--target-project", default="ddnsupd-qa-restore")
parser.add_argument("--source-env", default=".local/qa.env")
parser.add_argument("--target-env", default=".local/qa-restore.env")
args = parser.parse_args()


class Instance:
    def __init__(self, env_file, project):
        self.env = dict(line.split("=", 1) for line in Path(env_file).read_text().splitlines() if "=" in line)
        self.origin = self.env["DDNSUPD_EXTERNAL_URL"]
        self.compose = ["docker", "compose", "--env-file", env_file, "-p", project,
                        "-f", "docker-compose.yml", "-f", "docker-compose.dev.yml", "-f", "docker-compose.test.yml"]
        self.client = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
        self.csrf = ""

    def request(self, path, data=None, *, headers=None, binary=False):
        request = urllib.request.Request(
            self.origin.replace("localhost", "127.0.0.1") + "/api/v1" + path,
            data=json.dumps(data).encode() if data is not None else None,
            headers={"Origin": self.origin, "Content-Type": "application/json", "X-CSRF-Token": self.csrf, **(headers or {})},
        )
        try:
            response = self.client.open(request, timeout=60)
        except urllib.error.HTTPError as failure:
            response = failure
        with response:
            body = response.read()
            return response.status, body if binary else json.loads(body)

    def sql(self, statement):
        return subprocess.check_output(self.compose + [
            "exec", "-T", "postgres", "psql", "-X", "-v", "ON_ERROR_STOP=1",
            "-U", "ddnsupd", "-d", "ddnsupd", "-tAqc", statement
        ], text=True).strip()

    def login(self, fixture):
        status, result = self.request("/auth/login", {key: fixture[key] for key in ("username", "password")})
        assert status == 200
        self.csrf = result["csrf_token"]


source = Instance(args.source_env, args.source_project)
target = Instance(args.target_env, args.target_project)
assert source.origin != target.origin and source.env["DDNSUPD_SECRET_KEY"] != target.env["DDNSUPD_SECRET_KEY"]
fixture = json.loads(Path(args.source_env).with_name("qa-credentials.json").read_text())
source.login(fixture)
assert target.request("/setup") == (200, {"needs_setup": True})

# Exercise the route-specific upload limit, beyond the regular 64 KiB API limit.
source.sql("INSERT INTO audit_logs(id,action) SELECT gen_random_uuid(),'backup.fixture.'||i FROM generate_series(1,1000) i")
passphrase = secrets.token_urlsafe(24)
assert source.request("/backup", {"passphrase": passphrase}, headers={"X-CSRF-Token": ""})[0] == 403
status, archive = source.request("/backup", {"passphrase": passphrase}, binary=True)
assert status == 200 and archive.startswith(b"DDNSUPD-BACKUP\x01") and len(archive) > 32768
assert b"backup.fixture" not in archive
setup_headers = {"X-Setup-Token": target.env["DDNSUPD_SETUP_TOKEN"]}
payload = {"archive": archive.hex(), "passphrase": passphrase, "confirm": True}
assert target.request("/setup/restore", payload)[0] == 403
assert target.request("/setup/restore", {**payload, "confirm": False}, headers=setup_headers)[0] == 400
assert target.request("/setup/restore", {**payload, "passphrase": secrets.token_urlsafe(24)}, headers=setup_headers)[0] == 422
assert target.request("/setup/restore", {**payload, "archive": (archive[:-1] + bytes([archive[-1] ^ 1])).hex()}, headers=setup_headers)[0] == 422
assert target.request("/setup") == (200, {"needs_setup": True})
print("PASS setup authorization, confirmation, wrong passphrase and damaged archive")
assert target.request("/setup/restore", payload, headers=setup_headers)[0] == 200
assert target.request("/setup/restore", payload, headers=setup_headers)[0] == 409
assert target.request("/setup") == (200, {"needs_setup": False})
for table in ("users", "roles", "providers", "dns_zones", "dns_records", "provider_permissions", "record_permissions"):
    assert source.sql(f"SELECT count(*) FROM {table}") == target.sql(f"SELECT count(*) FROM {table}"), table
assert target.sql("SELECT count(*) FROM audit_logs WHERE action LIKE 'backup.fixture.%'") == "1000"
assert int(target.sql("SELECT count(*) FROM update_history")) > 0
assert target.sql("SELECT count(*) FROM dns_records WHERE active") == "0"
for table in ("user_sessions", "update_jobs", "import_previews"):
    assert target.sql(f"SELECT count(*) FROM {table}") == "0", table
assert source.sql("SELECT md5(ciphertext::text) FROM provider_credentials ORDER BY provider_id LIMIT 1") != target.sql(
    "SELECT md5(ciphertext::text) FROM provider_credentials ORDER BY provider_id LIMIT 1"
)
target.login(fixture)
provider = next(row for row in target.request("/providers")[1]["items"] if row["name"] == "Isolated provider")
assert target.request(f"/providers/{provider['id']}/discover", {"operation": "zones"})[0] == 200
print("PASS fresh restore with a different key, login, paused records, history and provider decryption")

target.sql("INSERT INTO audit_logs(id,action) VALUES(gen_random_uuid(),'backup.replace.fixture')")
assert target.request("/backup/restore", {**payload, "confirm": False})[0] == 400
assert target.sql("SELECT count(*) FROM audit_logs WHERE action='backup.replace.fixture'") == "1"
assert target.request("/backup/restore", payload)[0] == 200
assert target.request("/auth/me")[0] == 401
assert target.sql("SELECT count(*) FROM audit_logs WHERE action='backup.replace.fixture'") == "0"
target.login(fixture)
print("PASS confirmed replacement invalidates old sessions and restores backed-up accounts")
