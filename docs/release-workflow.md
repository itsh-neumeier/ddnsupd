# Development and release workflow

Use CLAUDE.md for shared instructions and task memory for handoff. One task
owns a checkout. Concurrent work, when requested, uses separate branches and
worktrees. Check target branch and migration IDs rather than assuming main.
Use Conventional Commits and focused PRs when a repository exists.

## First source upload

Check `git status`, inspect the initial commit and verify that generated
runtime secrets, local test data and archives are ignored. Use an empty GitHub
repository so the initial commit can be pushed directly. Choose the repository
visibility before uploading.

```sh
python scripts/check-docs.py
python scripts/check-runtime-secrets.py
git remote add origin <repository-url>
git push -u origin main
```

CI validates the source after the push. Release and image publication are
separate operations.

Development builds are local and do not publish. Production Compose references
versioned GHCR images with no mandatory source build. A dedicated release step
checks the final combined revision, versions, lockfiles, migrations, translations,
all required CI results and the source/license bundle. Tagging/pushing images,
creating a remote repository and production deployment require user authorization.

Tag pushes matching `v*` run the publication workflow. It checks for a successful
Validate run on the same commit, builds and scans both images, publishes
versioned and latest GHCR tags, and creates a release with source and SBOMs.

Release gates: Rust fmt/clippy/tests, API auth/grant/job integration tests,
Go provider mock tests and vet/race, frontend lint/check/build, Compose config,
Docker build/smoke, browser E2E, dependency and image scans, backup recovery.
amd64 and arm64 support are claimed only for successful tested build artifacts.
Do not push a publication tag as a test command.

Handoff includes scope, branch/commit/PR if present, migration IDs, tests passed
and not run, provider evidence, known limitations, next action and task memory path.
