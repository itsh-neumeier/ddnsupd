# ADR 0002: Targeted Go provider adaptation

- Status: Accepted for implementation; adapter verification remains separate
- Date: 2026-11-03

Options: wrap the upstream daemon and files, rewrite all providers in Rust,
or adapt the Go provider implementations behind a private contract.

Select targeted Go adaptation. The daemon's configuration and forced-update
route do not provide management CRUD or resource scope. Go internal packages
cannot be consumed as a stable library by an unrelated module; its Provider
interface also includes HTML. A full Rust rewrite duplicates all provider work.

Start with Hetzner Cloud and Cloudflare. Retain MIT attribution and exact
provenance. Adapt request construction and semantics, remove UI dependencies,
and test the new interface independently. Keep all other providers in the
parity matrix. No implicit parity claim follows from the source inventory.

Rust owns the DB, scheduler and authorization. The Go service has fixed
provider endpoints, bounded bodies/timeouts, authentication on its private
network and no published port. It never accepts arbitrary provider base URLs
in production. Use injected transports for mocks rather than user URLs.

Read origin state and preserve unrelated RRSet members. Upstream Hetzner's
single-value set_records behavior must not be reused without this change.
Provider-specific least-privilege and read/list capabilities are explicit;
DynDNS-only adapters must report zone listing as unavailable.
