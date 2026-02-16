# Threat Model (Initial)

## High-Risk Threats

- Renderer RCE from malformed web content.
- Cross-site data leakage via shared state.
- Network MITM downgrade or invalid cert acceptance.
- Persistent tracking through fingerprinting and third-party storage.

## Base Mitigations in Structure

- Dedicated process roles defined via `pd-ipc`.
- Security policy object in `pd-security` with sandbox and strict TLS defaults.
- Privacy policy object in `pd-privacy` with tracker/cookie protections enabled.
- Storage partitioning defaults in `pd-storage`.

## Gaps (To Implement)

- Real OS sandbox integration.
- Memory-safe but robust parser implementations against adversarial inputs.
- Full certificate validation and protocol hardening in network stack.
- Site isolation across renderer instances.
- Content Security Policy enforcement.

## Security Bar

No feature ships without:

1. Threat analysis updates.
2. Negative tests (malformed input / adversarial cases).
3. Policy review for privacy and security impact.
