"""Generate a new local configuration without printing secret values."""
import argparse
from pathlib import Path
import secrets

parser = argparse.ArgumentParser()
parser.add_argument("--output", default=".env")
parser.add_argument("--port", type=int, default=8080)
args = parser.parse_args()
path = Path(args.output)
path.parent.mkdir(parents=True, exist_ok=True)
content = f"""DDNSUPD_VERSION=0.1.1
DDNSUPD_BIND=127.0.0.1
DDNSUPD_PORT={args.port}
DDNSUPD_EXTERNAL_URL=http://localhost:{args.port}
POSTGRES_PASSWORD={secrets.token_hex(24)}
DDNSUPD_SECRET_KEY={secrets.token_hex(32)}
DDNSUPD_SETUP_TOKEN={secrets.token_hex(32)}
DDNSUPD_ENGINE_TOKEN={secrets.token_hex(32)}
"""
with path.open("x", encoding="utf-8", newline="\n") as file:
    file.write(content)
path.chmod(0o600)
print(f"Created {path}. Keep this file private; do not regenerate it for an existing database.")
