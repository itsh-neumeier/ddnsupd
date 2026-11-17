"""Check frozen provider coverage, evidence paths and local documentation links."""
import json
from pathlib import Path
import re

root = Path(__file__).resolve().parents[1]
catalog = json.loads((root / "docs/provider-parity.json").read_text(encoding="utf-8"))
snapshot = json.loads((root / "docs/reference-snapshot.json").read_text(encoding="utf-8"))
statuses = {"Unterstützt", "In Entwicklung", "Geplant", "Nicht relevant", "Technisch eingeschränkt"}
providers = catalog["providers"]
assert len(providers) == len({item["id"] for item in providers}), "Duplicate provider IDs"
assert {item["source_path"] for item in providers} == {item["path"] for item in snapshot["provider_inventory"]}, "Frozen source inventory differs from parity catalog"
assert len(providers) == 60, "The frozen inventory has 60 entries"
for item in providers:
    assert item["status"] in statuses, item["id"]
    assert re.fullmatch(r"[0-9a-f]{40}", item["source_tree_sha"]), item["id"]
    assert catalog["upstream"]["commit"] in item["upstream_url"], item["id"]
    if item["status"] == "Unterstützt" or item["verification"] != "not_tested":
        assert item["tests"], f"Missing evidence: {item['id']}"
    for test in item["tests"]:
        assert (root / test).is_file(), test
for repo in snapshot["repositories"]:
    assert re.fullmatch(r"[0-9a-f]{40}", repo["commit"])
assert all(item["read"] for item in snapshot["sources"]), "Unfetched reference in manifest"
registry = json.loads((root / "provider-engine/registry.json").read_text())
assert {item["id"] for item in registry["items"]} <= {item["id"] for item in providers}
required = {"id", "name", "authentication_schema", "record_types", "ip_versions", "required_permissions", "capabilities", "ttl_policy", "options", "rate_limit", "error_classes", "documentation", "verification", "live_verified"}
for item in registry["items"]:
    assert required <= item.keys(), f"Incomplete descriptor: {item['id']}"
    assert not item["live_verified"], "Live verification needs a separate reviewed evidence record"

checked = 0
for file in [root / "README.md", root / "AGENTS.md", root / "CLAUDE.md", root / "THIRD_PARTY_NOTICES.md", *(root / "docs").rglob("*.md")]:
    text = file.read_text(encoding="utf-8")
    # Do not interpret fenced source examples as links.
    text = re.sub(r"```[\s\S]*?```", "", text)
    for link in re.findall(r"\[[^\]]*\]\(([^)]+)\)", text):
        if link.startswith(("https://", "http://", "#", "mailto:")):
            continue
        target = link.split("#", 1)[0]
        if target:
            assert (file.parent / target).exists(), f"Broken link in {file.relative_to(root)}: {target}"
            checked += 1
openapi = json.loads((root / "docs/openapi.json").read_text())
assert openapi["openapi"] == "3.1.0"
assert len(openapi["paths"]) >= 17
ids = [op["operationId"] for methods in openapi["paths"].values() for op in methods.values()]
assert len(ids) == len(set(ids)), "Duplicate OpenAPI operation IDs"
print(f"PASS: {len(providers)} frozen providers, complete descriptors, {checked} local documentation links and {len(ids)} OpenAPI operations")
