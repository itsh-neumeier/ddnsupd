# Developer checks

The initial workspace had Docker and Python, but no local Rust/Go/npm tools.
Use containers for repeatable checks. CLI examples below use POSIX paths;
PowerShell can use `--mount type=bind,source=D:\GitHub\ddnsupd,target=/src`.

```sh
docker build -f docker/rust-tools.Dockerfile -t ddnsupd-rust-tools:local .
docker run --rm -v "$PWD:/src" -w /src ddnsupd-rust-tools:local cargo fmt --all --check
docker run --rm -v "$PWD:/src" -w /src ddnsupd-rust-tools:local cargo clippy --workspace --all-targets --locked -- -D warnings
docker run --rm -v "$PWD:/src" -w /src ddnsupd-rust-tools:local cargo test --workspace --locked
docker run --rm -v "$PWD/provider-engine:/src" -w /src golang:1.26-trixie go test -race ./...
docker run --rm -v "$PWD/provider-engine:/src" -w /src golang:1.26-trixie go vet ./...
docker run --rm -v "$PWD/web:/src" -w /src node:22-trixie npm ci
docker run --rm -v "$PWD/web:/src" -w /src node:22-trixie npm run check
docker run --rm -v "$PWD/web:/src" -w /src node:22-trixie npm run lint
docker run --rm -v "$PWD/web:/src" -w /src node:22-trixie npm run build
```

Run API integration after starting the disposable QA stack as in README.
Then install the pinned browser package and run it on the private QA network:

Run `python tests/smoke.py` and `python -u tests/management.py` before browser
tests. For an additional fresh QA project, preserve existing volumes and use
`integration.py --project <name>` and the same `DDNSUPD_QA_PROJECT` environment
variable for management.py. Stop the earlier stack to free its test port; use
the new project's control network for the browser container. Never reset a real
installation to satisfy the empty-fixture requirement.

```sh
docker run --rm -v "$PWD/tests/browser:/src" -w /src node:22-trixie npm ci
docker run --rm --network ddnsupd-qa_control -v "$PWD:/src" -w /src/tests/browser -e DDNSUPD_UPSTREAM=ddnsupd mcr.microsoft.com/playwright:v1.63.0-noble node e2e.mjs
```

The node TCP forwarder preserves the configured localhost Origin inside the
browser container; it does not bypass CSRF. Screenshots are saved under
`.local/screenshots/`. Test credentials and runtime configuration are ignored
by Git and excluded from Docker contexts. Do not run integration.py against a
real installation; it creates fixtures and restarts the named QA application.

CI has no publication permissions. The release preparation workflow builds
candidate archives only. A protected, explicitly authorized GHCR publication
step, multiarch verification and SBOM/source bundle are still release work.
