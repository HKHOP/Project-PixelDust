# Roadmap

## Phase 0: Foundation (Completed)

- Workspace crate boundaries and policy objects.
- Security/privacy defaults and validation.
- Shared error/result primitives and lint policy baseline.

## Phase 1: Headless Fetch + Parse (Completed Baseline)

- URL parsing, request building, and HTTP contracts in `pd-net`.
- HTTP/1.1 transport with chunked transfer and content-encoding decode.
- Strict TLS policy and rustls backend integration.
- In-app HTML/CSS parsing path used by the shell.

## Phase 2: First Render (Completed Baseline)

- App shell viewport rendering for block/inline content.
- CSS cascade and style application subset.
- Image decode path (PNG/JPEG/WebP/SVG) with texture upload.

## Phase 3: JavaScript Runtime Baseline (In Progress)

- Script execution via `pd-js` with runtime limits.
- Host document shims (`document`, `location`, `getElementById`).
- DOM event-script dispatch path in shell.
- Next: broader DOM/event loop compatibility.

## Phase 4: Multi-Process Runtime + Shell (In Progress)

- Browser runtime launch scaffolding in `pd-browser::boot_with_runtime`.
- IPC framing and local channel transport primitives in `pd-ipc`.
- Existing desktop shell UI for navigation and diagnostics.
- Next: wire role-specific worker entrypoints and supervision policy.

## Phase 5: Harden + Open Source Launch (Planned)

- Fuzzing for parser/network boundaries.
- Reproducible release builds and platform CI matrix.
- Public release cadence, contribution guidelines, and governance.
