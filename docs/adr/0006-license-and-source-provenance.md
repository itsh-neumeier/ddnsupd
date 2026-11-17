# ADR 0006: AGPL application and retained MIT provider notices

- Status: Accepted by the user
- Date: 2026-11-03

The user selected AGPL-3.0-only for DDNSupd. Keep the full license in LICENSE.
MIT-licensed ddns-updater adaptations retain their original notice in
`docs/licenses/ddns-updater-MIT.txt` and in the provider engine distribution.
Track source commit and changes per adapted file. Review dependency licenses
and source availability before release. Reference analysis alone does not
mean TalkOps application code was copied.
