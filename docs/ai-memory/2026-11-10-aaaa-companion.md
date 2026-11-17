# AAAA companion record

The record inspector offers an AAAA creation shortcut for A records when the
user can create records and select the source provider. The existing creation
dialog copies provider, zone name and ID, relative name, interval and site.
The new draft defaults to AAAA and remains paused. IP fields and the Cloudflare
record ID start empty because they belong to the new address record.

The app image passed Svelte check, formatting and static build. The browser
regression passed against the isolated QA engine, covering the copied fields,
blank IP fields, paused default and reset of the ordinary creation dialog.
There is no migration or backend change.

The shared native dialog now uses explicit viewport centering. Chromium
verified both axes at 1440×1000 and 360×560. The updated local app is healthy.
The IPv6 discovery investigation found the existing Docker egress bridge has
IPv6 disabled and no IPv6 default route. The Windows probes also failed to
resolve the IPv6 echo endpoints. No provider credentials or DNS records were
changed during diagnosis.
