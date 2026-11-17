"""End-to-end API tests against a disposable, network-isolated Compose stack.

Run only with docker-compose.test.yml; the fixture engine cannot contact DNS.
This script creates test accounts. It requires an empty test database.
"""
import argparse
import concurrent.futures
import http.cookiejar
import json
from pathlib import Path
import secrets
import subprocess
import time
import urllib.error
import urllib.request

parser = argparse.ArgumentParser()
parser.add_argument("--env-file", default=".local/qa.env")
parser.add_argument("--project", default="ddnsupd-qa")
args = parser.parse_args()
env = dict(line.split("=", 1) for line in Path(args.env_file).read_text().splitlines() if "=" in line and not line.startswith("#"))
base = env["DDNSUPD_EXTERNAL_URL"]
compose = ["docker", "compose", "--env-file", args.env_file, "-p", args.project, "-f", "docker-compose.yml", "-f", "docker-compose.dev.yml", "-f", "docker-compose.test.yml"]
checks = 0

def expect(condition, message):
    global checks
    assert condition, message
    checks += 1
    print("PASS", message)

class Client:
    def __init__(self):
        self.opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
        self.csrf = ""
    def request(self, path, method="GET", data=None, headers=None):
        hdr = {"Origin": base, "X-CSRF-Token": self.csrf}
        if data is not None:
            hdr["Content-Type"] = "application/json"
        hdr.update(headers or {})
        # Docker Desktop publishes IPv4. Keep the configured Origin while avoiding
        # slow localhost IPv6 connection fallback in this non-browser client.
        request = urllib.request.Request(base.replace("localhost", "127.0.0.1") + "/api/v1" + path, data=json.dumps(data).encode() if data is not None else None, method=method, headers=hdr)
        try:
            response = self.opener.open(request, timeout=15)
        except urllib.error.HTTPError as error:
            response = error
        raw = response.read()
        try:
            value = json.loads(raw) if raw else None
        except ValueError:
            value = raw.decode()
        return response.status, value
    def login(self, username, password):
        status, result = self.request("/auth/login", "POST", {"username": username, "password": password})
        assert status == 200, (status, result)
        self.csrf = result["csrf_token"]

def wait_for(predicate, message, timeout=35):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if predicate():
            expect(True, message)
            return
        time.sleep(.5)
    raise AssertionError(message)

def sql(statement):
    return subprocess.check_output(compose + ["exec", "-T", "postgres", "psql", "-U", "ddnsupd", "-d", "ddnsupd", "-tAc", statement], text=True).strip()

def record_payload(provider, name="home", active=True):
    return {"provider_id": provider, "zone_remote_id": "testzone", "zone_name": "example.org", "name": name, "record_type": "A", "remote_record_id": "fixture", "manual_ip": "8.8.8.8", "managed_ip": "1.1.1.1", "active": active, "interval_seconds": 60, "site": "Isolated test"}

admin = Client()
expect(admin.request("/records")[0] == 401, "unauthenticated access denied")
expect(admin.request("/setup")[1]["needs_setup"], "fresh database requires setup")
password = secrets.token_urlsafe(24)
setup_input = {"username": "qa-admin", "password": password}
expect(admin.request("/setup", "POST", setup_input, {"X-Setup-Token": "wrong"})[0] == 403, "setup requires runtime token")
with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
    statuses = list(executor.map(lambda _: Client().request("/setup", "POST", setup_input, {"X-Setup-Token": env["DDNSUPD_SETUP_TOKEN"]})[0], range(2)))
expect(sorted(statuses) == [200, 409], "concurrent setup creates exactly one administrator")
admin.login("qa-admin", password)
me = admin.request("/auth/me")[1]
expect(me["admin"] and "credentials.manage" in me["permissions"], "administrator has server permissions")
expect(admin.request("/auth/logout", "POST", headers={"X-CSRF-Token": ""})[0] == 403, "missing CSRF rejected")
expect(admin.request("/auth/logout", "POST", headers={"Origin": "https://foreign.invalid"})[0] == 403, "foreign origin rejected")
expect(admin.request("/records/00000000-0000-0000-0000-000000000000/update")[0] == 405, "GET cannot enqueue updates")

secret = "fixture-token-" + secrets.token_hex(20)
status, result = admin.request("/providers", "POST", {"kind": "cloudflare", "name": "Isolated provider", "token": secret})
expect(status == 201, "create encrypted provider credential")
provider = result["id"]
expect(secret not in json.dumps(admin.request("/providers")[1]), "stored credentials never returned")
expect(admin.request("/provider-registry")[1]["items"][0]["capabilities"]["read_record"], "registry advertises actual implemented capabilities")
expect(sql("SELECT bool_and(octet_length(ciphertext)>40) FROM provider_credentials") == "t", "credentials stored as authenticated ciphertext")
payload = record_payload(provider)
payload["manual_ip"] = "10.0.0.1"
expect(admin.request("/records", "POST", payload)[0] == 400, "private update address rejected")
status, result = admin.request("/records", "POST", record_payload(provider))
expect(status == 201, "create active managed record through API")
record = result["id"]
def latest(record_id):
    return admin.request("/history?record_id=" + record_id)[1]["items"]
wait_for(lambda: latest(record) and latest(record)[0]["outcome"] == "updated", "scheduled update persists verified history")
expect(admin.request("/records")[1]["items"][0]["managed_ip"] == "8.8.8.8", "managed state follows verified engine result")
expect(admin.request("/dashboard")[1]["total_records"] == 1, "dashboard counts use server resource scope")
automatic = record_payload(provider, "no-ip-source")
automatic["manual_ip"] = ""
automatic_id = admin.request("/records", "POST", automatic)[1]["id"]
wait_for(lambda: latest(automatic_id) and latest(automatic_id)[0]["error_code"] == "ip_unavailable", "failed discovery prevents provider update")

jobs = [admin.request(f"/records/{record}/update", "POST")[1]["job_id"] for _ in range(5)]
expect(len(set(jobs)) == 1, "repeated manual checks coalesce to one persistent job")
expect(len(latest(record)) == 1, "manual requests do not bypass cooldown")
sql(f"UPDATE dns_records SET next_check=now() WHERE id='{record}'; UPDATE update_jobs SET due_at=now() WHERE record_id='{record}' AND state='queued'")
wait_for(lambda: latest(record)[0]["outcome"] == "no_change", "no-change reconciliation is recorded separately")

other_provider = admin.request("/providers", "POST", {"kind": "hetznercloud", "name": "Foreign provider", "token": "fixture-other-token"})[1]["id"]
other_record = admin.request("/records", "POST", record_payload(other_provider, "foreign", False))[1]["id"]
viewer_password = secrets.token_urlsafe(24)
viewer_id = admin.request("/users", "POST", {"username": "qa-viewer", "password": viewer_password, "role_id": "00000000-0000-0000-0000-000000000004"})[1]["id"]
viewer = Client(); viewer.login("qa-viewer", viewer_password)
expect(viewer.request("/records")[1]["total"] == 0, "global read permission does not grant foreign resources")
admin.request("/grants", "POST", {"user_id": viewer_id, "record_id": record, "enabled": True})
expect(viewer.request("/records")[1]["total"] == 1, "resource grant scopes records")
expect(all(item["record_id"] == record for item in viewer.request("/history")[1]["items"]), "history obeys record grants")
expect(viewer.request(f"/records/{other_record}/update", "POST")[0] == 403, "viewer cannot mutate foreign record")
expect(viewer.request(f"/records/{record}/update", "POST")[0] == 403, "viewer cannot mutate granted record")
expect(viewer.request("/providers", "POST", {"kind": "cloudflare", "name": "Denied", "token": "denied"})[0] == 403, "viewer cannot store provider credentials")
admin.request("/grants", "POST", {"user_id": viewer_id, "record_id": record, "enabled": False})
expect(viewer.request("/records")[1]["total"] == 0, "grant revocation applies to existing sessions")

role = admin.request("/roles", "POST", {"name": "Restricted QA", "permissions": ["dashboard.read"]})
expect(role[0] == 201, "custom roles use permission catalog")
limited_password = secrets.token_urlsafe(24)
limited_id = admin.request("/users", "POST", {"username": "qa-limited", "password": limited_password, "role_id": role[1]["id"]})[1]["id"]
admin.request("/grants", "POST", {"user_id": limited_id, "record_id": record, "enabled": True})
limited = Client(); limited.login("qa-limited", limited_password)
expect(limited.request("/records")[0] == 403, "resource grant cannot bypass missing role permission")

failed_provider = admin.request("/providers", "POST", {"kind": "cloudflare", "name": "Invalid credentials", "token": "invalid-test-token"})[1]["id"]
failed_record = admin.request("/records", "POST", record_payload(failed_provider, "bad-auth"))[1]["id"]
wait_for(lambda: latest(failed_record) and latest(failed_record)[0]["error_code"] == "auth", "authentication failure is classified")
expect(any(p["id"] == failed_provider and p["suspended"] for p in admin.request("/providers")[1]["items"]), "invalid credentials suspend provider instead of retrying forever")
admin.request(f"/providers/{failed_provider}", "PATCH", {"name": "Rotated credentials", "enabled": True, "token": "fixture-rotated-token"})
sql(f"UPDATE dns_records SET next_check=now() WHERE id='{failed_record}'")
wait_for(lambda: latest(failed_record)[0]["outcome"] == "updated", "credential rotation resumes suspended provider")

rate_record = admin.request("/records", "POST", record_payload(other_provider, "rate-limited"))[1]["id"]
wait_for(lambda: latest(rate_record) and latest(rate_record)[0]["error_code"] == "rate_limit", "provider rate limit is classified")
expect(sql(f"SELECT blocked_until>now()+interval '8 minutes' FROM providers WHERE id='{other_provider}'") == "t", "Retry-After blocks the provider account")
paused_payload = record_payload(provider, "paused-record", False)
paused_id = admin.request("/records", "POST", paused_payload)[1]["id"]
admin.request(f"/records/{paused_id}/update", "POST")
wait_for(lambda: latest(paused_id) and latest(paused_id)[0]["outcome"] == "skipped", "paused records never write even for manual requests")
expect(admin.request(f"/providers/{provider}", "DELETE")[0] == 409, "provider in use cannot be deleted")
expect(admin.request(f"/records/{paused_id}", "DELETE")[0] == 204, "local record binding can be deleted")
expect(sql("SELECT count(*) FROM update_history WHERE fqdn='paused-record.example.org' AND record_id IS NULL") == "1", "deleting a binding retains history")

creation_provider = admin.request("/providers", "POST", {
    "kind": "hetznercloud", "name": "RRSet creation fixture", "token": "fixture-creation-token"
})[1]["id"]
creation_payload = record_payload(creation_provider, "missing-aaaa")
creation_payload.update(record_type="AAAA", manual_ip="2606:4700:4700::1111", managed_ip="", remote_record_id="")
creation_record = admin.request("/records", "POST", creation_payload)[1]["id"]
creation_path = f"/records/{creation_record}/create-missing"
wait_for(lambda: latest(creation_record) and latest(creation_record)[0]["error_code"] == "not_found", "ordinary check does not create a missing AAAA")
expect(admin.request(creation_path, "POST", headers={"X-CSRF-Token": ""})[0] == 403, "remote creation requires CSRF")
expect(viewer.request(creation_path, "POST")[0] == 403, "viewer cannot request remote creation")
expect(admin.request(f"/records/{record}/create-missing", "POST")[0] == 409, "remote creation is restricted to Hetzner")
status, creation_job = admin.request(creation_path, "POST")
expect(status == 202, "explicit request queues remote creation")
job_id = creation_job["job_id"]
expect(sql(f"SELECT trigger FROM update_jobs WHERE id='{job_id}'") == "create_missing", "creation intent is recorded durably")
expect(admin.request(creation_path, "POST")[1]["job_id"] == job_id, "repeated creation requests coalesce")
sql(f"BEGIN; UPDATE dns_records SET next_check=now() WHERE id='{creation_record}'; UPDATE update_jobs SET due_at=now() WHERE id='{job_id}'; COMMIT")
wait_for(lambda: latest(creation_record)[0]["trigger"] == "create_missing" and latest(creation_record)[0]["outcome"] == "updated", "confirmed missing AAAA is created and verified")
expect(sql(f"SELECT count(*) FROM audit_logs WHERE object_id='{creation_record}' AND action='dns.created'") == "1", "verified creation is audited")

durable_job = admin.request(f"/records/{record}/update", "POST")[1]["job_id"]
subprocess.run(compose + ["restart", "ddnsupd"], check=True, stdout=subprocess.DEVNULL)
def ready():
    try:
        return urllib.request.urlopen(base + "/readyz", timeout=3).status == 200
    except Exception:
        return False
wait_for(ready, "application restarts with persistent database")
expect(sql(f"SELECT count(*) FROM update_jobs WHERE id='{durable_job}'") == "1", "queued job survives application restart")
expect(admin.request("/auth/me")[0] == 200, "session survives process restart")

new_password = secrets.token_urlsafe(24)
expect(viewer.request("/auth/password", "POST", {"current": viewer_password, "password": new_password})[0] == 204, "password change requires current password")
expect(viewer.request("/auth/me")[0] == 401, "password change revokes existing sessions")
expect(secret not in json.dumps(admin.request("/audit")[1]), "audit contains no provider secret")
expect(admin.request("/openapi.json")[1]["openapi"] == "3.1.0", "versioned API contract available")
expect(admin.request("/auth/logout", "POST")[0] == 204, "logout succeeds")
expect(admin.request("/auth/me")[0] == 401, "logout invalidates session")

for _ in range(11):
    status, _ = Client().request("/auth/login", "POST", {"username": "qa-wrong", "password": "wrong-password"})
expect(status == 429, "login brute-force limit enforced")

fixture_path = Path(args.env_file).parent / "qa-credentials.json"
fixture_path.write_text(json.dumps({"url": base, "username": "qa-admin", "password": password}), encoding="utf-8")
fixture_path.chmod(0o600)
print(f"{checks} integration checks passed. Browser-only test credentials: {fixture_path}")
