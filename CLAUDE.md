# DDNSupd contributor rules

DDNSupd is a self-hosted DNS management platform. Read `docs/architecture.md`,
the ADRs, `docs/roadmap.md` and relevant task memory before a major change.
The product specification is `docs/product-specification.md`.

- Preserve user changes. Check branch, status and diff when Git is available.
- Do not create remote repositories, publish images, deploy to production or
  change real DNS without the user's explicit authorization. Live DNS tests
  require an explicitly supplied and authorized test zone.
- Code, comments and technical documentation use clear English. User guides
  and UI strings have English and German versions. Keep translation keys equal.
- Rust: edition 2024, forbid unsafe, typed library errors. Use Axum, Tokio and
  runtime SQLx queries; test database queries against PostgreSQL.
- Frontend: SvelteKit static SPA, Svelte 5, TypeScript, Tailwind. Follow
  `docs/ui-guidelines.md`. Do not add another UI framework.
- Keep provider HTTP code in `provider-engine/`. Rust owns authorization,
  scheduling, history and encrypted credentials. The engine has no DB access.
- All API mutations need server authorization, resource checks and audit.
  Cookie requests also need CSRF checks. Never return saved credentials.
- Do not log request bodies, cookies, provider responses or secret URLs.
- Migration names use UTC `YYYYMMDDHHMMSS_description.sql`. Never modify a
  published migration. Check IDs before integration.
- Provider status is evidence-based: implementation, mock tests and live tests
  are distinct. Update `docs/provider-parity.json` and its table together.
- Use Conventional Commits and Semantic Versioning when Git is initialized.
- Parallel work, if requested, needs separate branches and worktrees. Do not
  introduce agents or concurrent checkout edits without an explicit request.
- Run relevant tests. Report passed, failed and unavailable checks separately.
- Save task decisions, evidence, limitations and next actions in `docs/ai-memory/`.
- Production Compose uses published versioned images without `build:`.
  Source builds belong in the development override. Publication needs approval.
