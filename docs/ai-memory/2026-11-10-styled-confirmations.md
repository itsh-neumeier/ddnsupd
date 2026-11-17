# Styled delete confirmations

Record and provider deletion now use the shared CSS-styled application dialog
instead of browser confirmation popups. The dialog identifies the target and
uses the existing API error and busy states. No API or migration changes.
The app image built and isolated Chromium tests passed for canceling provider
deletion, canceling and confirming record deletion, and detecting browser popups.

The Windows IPv6 follow-up found link-local addresses but no visible global
IPv6 address or default route in the checked host session. The app's Docker
egress network still has IPv6 disabled.
