# Architecture

## Implementation Status

- Current shell behavior is primarily in-process while crate boundaries define isolation contracts.
- `pd-browser` includes worker runtime spawning scaffolding (`boot_with_runtime`).
- `pd-ipc` includes framed message encoding plus local transport endpoints for integration paths.

## Process Model

PixelDust is designed around strict process separation.

- Browser process: coordinates tabs, policies, permissions, and navigation.
- Renderer process: parses HTML/CSS, runs scripts, computes layout, and emits frames.
- Network process: handles DNS, HTTP, TLS, caching, and connection pooling.
- Storage process: manages cookies, cache, and persistent site data.

This separation minimizes blast radius for renderer compromise and supports policy enforcement at the browser boundary.

## Pipeline

1. Browser receives navigation request.
2. Network fetches response under `pd-security` and `pd-privacy` policies.
3. Renderer builds DOM (`pd-html` + `pd-dom`) and styles (`pd-css`).
4. Layout computes box tree (`pd-layout`).
5. Render stage emits draw commands/frame (`pd-render`).
6. Browser shell presents output.

## Crate Ownership

- `pd-browser`: global orchestration and process startup.
- `pd-renderer`: high-level render pipeline orchestration.
- `pd-net`: transport, protocol, and request lifecycle.
- `pd-storage`: partitioned persistence primitives.
- `pd-security`: hardening, sandbox requirements, and policy validation.
- `pd-privacy`: anti-tracking/fingerprinting controls and defaults.

## Security and Privacy Design Principles

- Deny by default.
- Least privilege per process.
- Data minimization and partitioning.
- Privacy features enabled by default, opt-out only.
- Explicit policy objects passed to subsystems (no hidden globals).
