# ADR 0007: Minimal images for application runtimes

- Status: Accepted for implementation
- Date: 2026-11-03
- Amends: ADR 0005 runtime image detail; service boundaries do not change.

Local Trivy 0.76.1 scans of the initial Debian slim app/engine images reported
high-severity distribution package advisories without fixed versions in that
snapshot. Many affected packages (mount/login/Perl/SQLite and related utilities)
are not needed by either binary. Do not add broad vulnerability exclusions.

Keep Debian 13 build tooling. Run Rust in the official `cc-debian13:nonroot`
distroless image, pinned by digest. It supplies the required glibc/libgcc
runtime with package metadata, without a shell/package manager. Compile Go
with CGO disabled and run it in scratch with the build-stage CA bundle. Both
continue to run as UID/GID 10001 and have binary-based healthcheck commands.
The application has no shell/subprocess runtime dependency.

Rebuild and check both images, fresh/existing migrations, real-engine startup,
QA browser/API paths and scans after this change. Distroless does not prove
absence of vulnerabilities. RustSec, Go binary and OS scans have different
coverage. Preserve this distinction in evidence. PostgreSQL remains its own
official image and still needs deployment-specific update/recovery review.

Source: [official Distroless images and Rust usage](https://github.com/GoogleContainerTools/distroless/blob/main/README.md).
