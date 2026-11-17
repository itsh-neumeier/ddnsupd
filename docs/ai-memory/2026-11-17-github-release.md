# GitHub, GHCR and Portainer

The user authorized repository creation, upload and image/release publication.
Verified account itsh-neumeier, created public repository itsh-neumeier/ddnsupd
and pushed main. The standalone Portainer file uses IPv6 egress, configurable
host binding and a required browser origin.

The v0.1.0 attempt stopped on a zlib finding. Updated the pinned distroless
digest, verified zero HIGH/CRITICAL base findings and published v0.1.1 with
source, SBOMs and checksums after successful GitHub CI. Packages were private
at anonymous-access verification; the user was given package-settings links.
The available connector and documented APIs cannot change package visibility.

The user's Portainer environment exposed an invalid nonhex application key,
a loopback-only bind and a private LAN HTTP URL. Generated a valid private
ENV file under .local, preserving the supplied database credentials. Version
0.1.2 adds actionable key errors and supports HTTP on private LAN addresses.
The user's exact private origin passed a fresh standalone-stack startup test.
All four CI jobs passed for 92e275f before tagging v0.1.2.

GHSA-fc5q-39xc-vjpf has no patched picomatch release at the time of inspection.
It affects trusted build globs and is absent from the shipped runtime. The
npm audit wrapper reports this exact advisory with an expiry of 2026-12-17
and blocks other moderate-or-higher findings. The exception is documented in
release notes. No runtime secrets are committed.

Release v0.1.2 and its two images, source archive, SBOMs and checksums were
published successfully. Anonymous image pulls still returned 404 at the
final check. Public package visibility therefore remains a user settings
action, or Portainer requires authenticated registry access.
