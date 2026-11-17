"""Render the human-readable provider matrix from its JSON source."""
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
catalog = json.loads((root / "docs/provider-parity.json").read_text(encoding="utf-8"))
lines = ["# Provider parity", "", "Status at 2026-11-03. Frozen source inventory: [qdm12/ddns-updater](" + catalog["upstream"]["inventory_url"] + ").", "", "The source has 60 registry entries, including the development example. The inventory comes from constants, the factory and the provider directory. Vercel/custom occur in source; Netcup is absent from ProviderChoices() but present in the factory.", "", "Statuses: **Unterstützt**, **In Entwicklung**, **Geplant**, **Nicht relevant**, **Technisch eingeschränkt**. Mock tests and live verification are separate. No live integration is verified. The two implemented adapters remain In Entwicklung until their broader provider integration scope is complete.", "", "The JSON file is authoritative. Run `python scripts/render-provider-matrix.py` after a catalog change. See [the feature matrix](feature-parity.md) for bounded implemented behavior.", "", "| ID | Provider | Priority | DDNSupd status | Verification | Notes |", "| --- | --- | --- | --- | --- | --- |"]
for p in catalog["providers"]:
    priority = p["priority"] or "—"
    lines.append(f"| {p['id']} | [{p['name']}]({p['upstream_url']}) | {priority} | {p['status']} | {p['verification']} | {p['notes']} |")
(root / "docs/provider-parity.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
