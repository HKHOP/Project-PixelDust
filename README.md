# PixelDust Browser

PixelDust is a Rust browser project built from scratch with no Chromium, WebKit, or Firefox engine code.

## Current State

PixelDust is now beyond a skeleton:

- Desktop shell app (`eframe`) with navigation UI and history controls.
- Hardened HTTP/1.1 + TLS fetch path with redirect handling and response decoding in `pd-net`.
- HTML/CSS parsing and rendering path in the app shell (`simple_html`) with image and script loading.
- JavaScript execution sandbox baseline via `pd-js` (Boa runtime with limits).
- Workspace lints and CI workflow for `fmt`, `check`, `clippy`, and `test`.
- Multi-process runtime wiring in app entrypoint (`--pd-role` worker roles).
- Runtime supervision primitives in `pd-browser` (worker health snapshots + restart on exit).
- Typed IPC message framing in `pd-ipc` (`Ping`, `Pong`, `HealthCheck`, `HealthReport`, `Shutdown`).
- Persistent partitioned storage baseline in `pd-storage` (file-backed key/value by top-level site).
- Privacy tracker host block helper in `pd-privacy` and subresource block enforcement in app shell.

## Goals

- Fast: predictable performance by isolating hot paths and minimizing cross-process overhead.
- Secure: process isolation, strict sandbox defaults, and deny-by-default policies.
- Privacy-focused: tracker blocking, partitioned storage, and anti-fingerprinting defaults.
- Open source ready: modular workspace with clear ownership boundaries.

## Workspace Layout

- `apps/pixeldust-browser`: executable shell entrypoint.
- `crates/pd-browser`: browser process coordinator.
- `crates/pd-renderer`: renderer process pipeline.
- `crates/pd-net`: network stack boundary.
- `crates/pd-storage`: cookies/cache/local storage subsystem.
- `crates/pd-security`: sandbox and security policy.
- `crates/pd-privacy`: privacy policy and anti-tracking defaults.
- `crates/pd-ipc`: process channel contracts.
- `crates/pd-dom`: DOM data model.
- `crates/pd-html`: HTML parser boundary.
- `crates/pd-css`: CSS parser/style model.
- `crates/pd-layout`: layout engine boundary.
- `crates/pd-render`: paint/compositor boundary.
- `crates/pd-js`: JavaScript runtime boundary.
- `crates/pd-core`: shared error/result primitives.

## Build

```bash
cargo check
cargo run -p pixeldust-browser
```

## Next Milestones

1. Continue extracting parser/style/layout/render logic from `apps/pixeldust-browser/src/simple_html.rs` into `pd-html`/`pd-css`/`pd-layout`/`pd-render`.
2. Replace worker idle loops with real pipe/IPC command handlers and typed request/response routing.
3. Add runtime crash telemetry (crash reason persistence + restart backoff policy).
4. Add fuzzing for HTML/CSS/network parsers and malformed response handling.
5. Tighten browser behavior conformance tests (redirects, caching, script execution, form handling).
