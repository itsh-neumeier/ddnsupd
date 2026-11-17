# Confirmed Hetzner record creation

The active Hetzner binding inspector now offers a CSS-styled confirmation for
creating a missing A or AAAA record. The API requires both create and update
permissions, resource scope and CSRF. It persists a distinct create job,
respects cooldowns and audits the request and verified creation. The Go engine
checks the primary zone and reads the RRSet before issuing a create request.
Only a missing RRSet can be created. Normal checks, dry runs and imports do not
create records. The creation job leaves an existing RRSet alone.

The isolated stack passed 58 API integration checks and the browser regression.
Go mock tests, race detector and vet passed. Rust tests, Clippy and formatting,
Svelte checks and static build, and the OpenAPI and provider matrix checks passed.
No database migration is required. No live creation was requested during tests.
