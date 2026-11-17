# ADR 0001: Rust management plane and static SvelteKit UI

- Status: Accepted for implementation
- Date: 2026-11-03

Use Rust 2024, Axum, Tokio, SQLx and PostgreSQL. Serve a static SvelteKit /
Svelte 5 / TypeScript / Tailwind SPA from the API. Keep domain modules separate
from HTTP. Use versioned migrations and OpenAPI. This matches the requested
stack and the relevant TalkOps architecture. A Node runtime is not needed in
production. PostgreSQL also holds jobs; no Redis or separate message broker.
