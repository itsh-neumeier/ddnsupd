"""Ensure generated runtime values are absent from the source tree."""
from pathlib import Path
import os

root = Path(__file__).resolve().parents[1]
values = []
for env in [root / ".env", root / ".local/qa.env"]:
    if env.exists():
        for line in env.read_text(encoding="utf-8").splitlines():
            if "=" in line:
                key, value = line.split("=", 1)
                if key in {"POSTGRES_PASSWORD", "DDNSUPD_SECRET_KEY", "DDNSUPD_SETUP_TOKEN", "DDNSUPD_ENGINE_TOKEN"} and value:
                    values.append(value.encode())
excluded = {".git", ".local", ".reference", "node_modules", "target", ".svelte-kit", "build", "__pycache__"}
checked = 0
for directory, directories, files in os.walk(root):
    directories[:] = [name for name in directories if name not in excluded]
    for name in files:
        if name == ".env":
            continue
        file = Path(directory) / name
        relative = file.relative_to(root)
        data = file.read_bytes()
        assert not any(value in data for value in values), f"Runtime secret found in {relative}"
        checked += 1
print(f"PASS: {checked} source files checked for generated runtime values; no values printed")
