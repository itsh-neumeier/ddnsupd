"""Non-destructive final QA smoke and IPv6 job check after integration.py."""
import http.cookiejar
import json
from pathlib import Path
import time
import urllib.error
import urllib.request
import uuid

fixture = json.loads(Path(".local/qa-credentials.json").read_text())
opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))
csrf = ""
def request(path, method="GET", body=None):
    headers = {"Origin": fixture["url"], "X-CSRF-Token": csrf}
    if body is not None:
        headers["Content-Type"] = "application/json"
    r = urllib.request.Request(fixture["url"].replace("localhost","127.0.0.1")+"/api/v1"+path, method=method, headers=headers, data=json.dumps(body).encode() if body is not None else None)
    with opener.open(r, timeout=10) as response:
        return json.load(response)

csrf = request("/auth/login", "POST", {"username": fixture["username"], "password": fixture["password"]})["csrf_token"]
assert request("/auth/me")["admin"]
providers = request("/providers")["items"]
provider = next(p for p in providers if p["name"] == "Isolated provider")
record = request("/records", "POST", {"provider_id":provider["id"],"zone_remote_id":"testzone","zone_name":"example.org","name":"ipv6-"+uuid.uuid4().hex[:8],"record_type":"AAAA","remote_record_id":"fixture-v6","manual_ip":"2606:4700:4700::1111","managed_ip":"","active":True,"interval_seconds":300})
for _ in range(50):
    history = request("/history?record_id="+record["id"])["items"]
    if history and history[0]["outcome"] == "updated":
        assert history[0]["new_ip"] == "2606:4700:4700::1111"
        break
    time.sleep(.5)
else:
    raise AssertionError("IPv6 scheduler path did not complete")
assert request("/dashboard")["total_records"] > 0
print("PASS final SQLx/migration runtime smoke, persisted session/data access, IPv6 scheduled update and history")
