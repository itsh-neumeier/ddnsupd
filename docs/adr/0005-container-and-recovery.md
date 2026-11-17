# ADR 0005: Bridge networks and complete recovery

- Status: Accepted for implementation
- Date: 2026-11-03

Use Compose bridge networks instead of TalkOps host networking. Publish only
the web/API port, on loopback by default. Keep DB on an internal network;
provider engine has controlled outbound access and no host port. Use multi-stage
builds, non-root app users, dropped capabilities and read-only roots.

Production Compose has versioned image references and no build keys. Development
uses an override. Do not claim GHCR availability until approved publication.
Test amd64 and arm64 when available; a multiarch manifest requires actual builds.

Complete recovery needs PostgreSQL, runtime configuration and matching
credential keys. An encrypted archive has integrity metadata and a separate
recovery password. Restore validates the full archive before changing state,
stops writers and clears old sessions. No Docker socket access is required.
Backup implementation and recovery tests are tracked separately in the roadmap.
