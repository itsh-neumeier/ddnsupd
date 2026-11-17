"""Report RustSec findings against the actual locked workspace feature graph.

Cargo.lock also records optional SQLx MySQL dependencies that the PostgreSQL
application does not compile. Never suppress by advisory ID: a finding becomes
blocking automatically if the affected package enters the active graph.
"""
import json
import re
import subprocess

tree = subprocess.check_output(["cargo", "tree", "--workspace", "--locked", "--target", "all", "--prefix", "none", "--format", "{p}", "--edges", "normal,build,dev"], text=True)
active = set()
for line in tree.splitlines():
    match = re.match(r"([^ ]+) v([^ (]+)", line)
    if match:
        active.add(match.groups())
result = subprocess.run(["cargo", "audit", "--json"], text=True, capture_output=True)
if result.returncode not in (0, 1):
    print(result.stderr)
    raise SystemExit(result.returncode)
try:
    report = json.loads(result.stdout)
except ValueError:
    print(result.stderr)
    raise SystemExit("RustSec report could not be read")
if not report.get("database"):
    raise SystemExit("Missing advisory database evidence")
blocking = []
excluded = []
for finding in report.get("vulnerabilities", {}).get("list", []):
    package = finding["package"]
    label = f"{finding['advisory']['id']}: {package['name']} {package['version']}"
    if (package["name"], package["version"]) in active:
        blocking.append(label)
    else:
        excluded.append(label)
for item in excluded:
    print("INACTIVE OPTIONAL DEPENDENCY:", item)
for item in blocking:
    print("BLOCKING ACTIVE DEPENDENCY:", item)
if result.returncode and not report.get("vulnerabilities", {}).get("list"):
    print(result.stderr)
    raise SystemExit("Audit failed without a classified vulnerability report")
print(f"RustSec: {len(blocking)} active findings; {len(excluded)} inactive optional findings; {len(active)} active packages checked across all targets")
raise SystemExit(bool(blocking))
