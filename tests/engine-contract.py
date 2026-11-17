"""Read-only/invalid-input contract probe against the actual private Go engine.

No accepted provider reconciliation is requested. This cannot mutate real DNS.
"""
import argparse
import json
from pathlib import Path
import urllib.error
import urllib.request

parser = argparse.ArgumentParser()
parser.add_argument("--env", default="/config/.env")
parser.add_argument("--engine", default="http://ddnsupd-worker:8090")
args = parser.parse_args()
env = dict(line.split("=", 1) for line in Path(args.env).read_text().splitlines() if "=" in line and not line.startswith("#"))
def call(path, token, body=None):
    request = urllib.request.Request(args.engine+path, data=json.dumps(body).encode() if body is not None else None, headers={"Authorization":"Bearer "+token,"Content-Type":"application/json"})
    try:
        response = urllib.request.urlopen(request, timeout=5)
    except urllib.error.HTTPError as error:
        response = error
    raw = response.read()
    return response.status, json.loads(raw) if raw else None
assert call("/v1/providers", "invalid")[0] == 401
status, registry = call("/v1/providers", env["DDNSUPD_ENGINE_TOKEN"])
assert status == 200 and {item["id"] for item in registry["items"]} == {"hetznercloud","cloudflare"}
status, result = call("/v1/reconcile", env["DDNSUPD_ENGINE_TOKEN"], {"provider":"not-supported","token":"fixture-not-real","zone_id":"fixture","zone_name":"example.org","name":"@","record_id":"fixture","record_type":"A","desired":"8.8.8.8","managed":"","dry_run":True})
assert status == 422 and result["code"] == "unsupported"
assert call("/v1/discover", "invalid", {"operation":"zones"})[0] == 401
status, result = call("/v1/discover", env["DDNSUPD_ENGINE_TOKEN"], {"provider":"not-supported","token":"fixture-not-real","operation":"zones","page":1})
assert status == 422 and result["code"] == "unsupported"
print("PASS actual private engine: service authentication, typed registry and reconcile/discovery invalid-provider rejection; no external DNS requests")
