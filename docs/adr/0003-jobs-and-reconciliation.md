# ADR 0003: PostgreSQL jobs and reconciliation

- Status: Accepted for implementation
- Date: 2026-11-03

Keep the initial scheduler in the Rust API with a separate domain module.
Go remains a stateless provider executor. Persist jobs, attempts, due time,
lease, request identity, error class and result. Partial uniqueness coalesces
queued/running checks. Claim with SKIP LOCKED and lock the actual mutation
unit across network work. Use bounded operations and cancel on lost locks.

Manual checks need the same timing and concurrency policy as automatic checks.
On uncertain write outcomes, inspect provider state before another write.
Backoff has jitter and a ceiling. Authentication failures suspend an account;
rate limits respect Retry-After. Separate provider acceptance, verified state
and DNS propagation. Exactly-once provider writes are not promised.
