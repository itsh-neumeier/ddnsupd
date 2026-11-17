# Backup, recovery and first source upload

Manual export and both restore entry points are implemented. The v1 archive
uses Argon2id and XChaCha20-Poly1305 and contains a consistent snapshot of
application data and its source credential key. Restore requires the same
schema, imports atomically and rekeys credentials to the target's key.
Sessions, queued jobs, import drafts and rate windows are omitted. Restored
bindings are paused, and an advisory lock waits for in-flight DNS work.
No migration was required. Runtime environment is configured separately.

The optional Compose override writes one encrypted snapshot per UTC day
using seven rotating slots in a private directory. This is opt-in.

Verified 58 API regression checks, the full isolated browser suite with
backup download, 8 Rust tests, Clippy, Svelte checks and image builds.
The end-to-end backup test recovered across different keys, rejected missing
authorization, wrong passphrase and tampering, and verified ordinary restore
replacement and session invalidation. The scheduled override generated
an encrypted file in QA. The source is prepared for a GitHub first push.
