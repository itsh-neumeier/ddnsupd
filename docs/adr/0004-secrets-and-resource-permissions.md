# ADR 0004: Secrets and resource permissions

- Status: Accepted for implementation
- Date: 2026-11-03

Use Argon2id password hashes, hashed opaque session/API tokens, cookie CSRF
protection and server-side permission checks. Roles define operations; grants
define resources. A provider grant never exposes its credentials. Only admin
can change roles or delegate privileges. Guard the last administrator.

Encrypt provider secrets with XChaCha20-Poly1305 and a fresh random nonce.
Associate ciphertext with provider identity and key version. Store the master
key outside PostgreSQL. Saved secrets are write-only through the API. Require
explicit replacement, audit changes and never echo provider error bodies.

Fixed provider/discovery endpoints are used first. Arbitrary URLs and webhook
targets require a later connection-time SSRF policy, not a URL prefix check.
