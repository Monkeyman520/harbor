# Windows System-Native Font Backend

**Spec ID:** 0003
**Status:** Implemented
**Superseded in part:** the `HARBOR_FONT` path override is superseded by TOML `[font]` configuration in `harbor-config` (CONTEXT.md, “System Default Font Selection”); override requirements below are historical.
**Date:** 2026-07-30

## Requirement

On Windows, Harbor must select, resolve, measure, and rasterize terminal fonts through DirectWrite without retaining complete font-file copies in the Rust heap, while preserving primary-font terminal metrics and the existing WGPU glyph-atlas renderer.

## Solution

Replace the Windows `fontdb`/`fontdue` font path in `harbor-text` with a DirectWrite-backed font abstraction. DirectWrite owns primary-face selection, process-private loading of the optional `HARBOR_FONT` file, system fallback resolution, metrics, glyph identity, and grayscale glyph rasterization.

When `HARBOR_FONT` is absent, the backend delegates primary-face selection to Windows rather than consulting Harbor’s hard-coded font candidates. When it is present, the referenced font becomes the primary face; an invalid configured path is a startup error. Missing characters from either primary source are resolved through DirectWrite system fallback.

Primary-face metrics remain the sole source of terminal cell width, line height, ascent, underline, and strikethrough placement. Fallback faces cannot change grid geometry. Glyph resolution is cached, including unavailable results, and atlas identity distinguishes at least face, glyph, size, and style so multiple fallback faces cannot collide.

DirectWrite resources use explicit ownership and thread-lifetime rules compatible with application startup and terminal rendering. Discovery collections and rejected candidates are released after selection. Long-lived state contains only required native handles, resolution caches, and glyph-atlas data; it contains no complete font-file `Vec<u8>`.

`harbor-text` remains CPU-only and exposes atlas-compatible rasterized glyph data. `harbor-terminal` retains the existing WGPU texture, upload, vertex, and draw pipeline. Public `harbor-text` APIs may change, and all workspace consumers are migrated together.

The Windows implementation has no runtime fallback to `fs::read`, `fontdb`, or `fontdue`. Non-Windows builds are rejected at compile time for this delivery.

### Seams

| Seam | Connects | Expects | Provides |
| --- | --- | --- | --- |
| Windows font services | `harbor-text` → DirectWrite | A usable DirectWrite factory, system font collection, fallback service, and process-private custom-font references | Native face identities, primary metrics, fallback mappings, glyph IDs, and grayscale glyph images |
| Font configuration | Process environment → `harbor-text` | Optional `HARBOR_FONT` filesystem path | A process-private primary face or an explicit configuration error |
| Text rendering contract | `harbor-terminal` ↔ `harbor-text` | Stable primary metrics and rasterizable resolved glyphs | Atlas-compatible bitmap/placement data without changing terminal GPU ownership |

## End-to-End Tests

### E2E: Default Windows startup

On Windows without `HARBOR_FONT`, first-window Latin rendering uses a DirectWrite-selected primary face with positive, stable metrics and the existing WGPU atlas, never the legacy loader.

### E2E: Configured primary with system fallback

With a valid process-private `HARBOR_FONT` lacking CJK coverage, Latin uses that primary and CJK uses DirectWrite fallback; both retain primary-face cell geometry.

### E2E: Cached fallback resolution

Repeated rendering/redraw of a previously resolved missing-primary character reuses its face/glyph without font rescans, reopenings or reparsing, with stable atlas output.

### E2E: Invalid configured font

A missing, unreadable or unsupported `HARBOR_FONT` produces a clear startup configuration error rather than silently choosing a system default or legacy loader.

### E2E: Windows memory profile

In the documented Latin DHAT/Windows private-memory scenario, first-present and steady-dwell measurements show no full-font Rust allocation, global live-heap peak below 40 MiB, and lower peak/steady private memory than the pre-change baseline.

### E2E: Unsupported platform

Compiling `harbor-text` for a non-Windows target fails with an explicit unsupported-platform diagnostic, not a portable-backend fallback.

## Decisions

### DirectWrite owns the full Windows font path

DirectWrite replaces eager parsing of large CJK collections; file discovery alone would retain the dominant allocation. See [ADR 0012](../adr/0012-consolidate-render-ui-into-terminal.md).

### Keep CPU text and GPU rendering separated

See [ADR 0004](../adr/0004-widget-dependency-boundary.md), [ADR 0008](../adr/0008-widget-runtime-for-confirmation-window.md), [ADR 0011](../adr/0011-terminal-custompaint-gpu-injection.md), [ADR 0012](../adr/0012-consolidate-render-ui-into-terminal.md).

### Delegate default selection and missing-glyph fallback to Windows

See [ADR 0012](../adr/0012-consolidate-render-ui-into-terminal.md).

### Preserve terminal geometry from the primary face

See [ADR 0012](../adr/0012-consolidate-render-ui-into-terminal.md).

### Deliver a Windows-only backend without a legacy fallback

Reject non-Windows compilation and fail explicitly if no usable DirectWrite face exists; no portable-loader fallback.

### Permit coordinated workspace API changes

Migrate all workspace callers together; immutable `FontBook` plus `char`-only atlas identity cannot represent native fallback ownership reliably. See [ADR 0004](../adr/0004-widget-dependency-boundary.md), [ADR 0012](../adr/0012-consolidate-render-ui-into-terminal.md).

### ADR cross-check

- ADRs 0001 and 0006 are superseded and introduce no active constraint.
- ADRs 0002, 0003, 0005, 0007, 0009, 0010, and 0013 concern widget reactivity/rendering, windowing/input, parser API, or PTY I/O and do not conflict with this requirement.
- ADRs 0004 and 0008 are satisfied because `harbor-text` remains a shared CPU core without `wgpu`, `winit`, or `harbor-widget` dependencies.
- ADRs 0011 and 0012 are satisfied because `harbor-terminal` continues to own GPU text rendering and consumes `harbor-text` glyph-atlas output.

## Test Plan

- **Integration tests:** Exercise DirectWrite factory creation, system primary selection, process-private `HARBOR_FONT` loading, primary metrics, system fallback across multiple faces, unavailable-result caching, stable glyph identity, grayscale rasterization, and the `harbor-terminal` atlas contract. Windows-dependent tests must distinguish platform integration tests from deterministic backend-policy tests.
- **Manual tests:** Run cold Latin startup, sustained Latin output, first and repeated CJK rendering, symbols/emoji with available grayscale outlines, a primary font with built-in CJK coverage, an invalid `HARBOR_FONT`, resize/DPI changes, and repeated terminal redraws. Confirm that default appearance may follow Windows rather than the former hard-coded candidate order.
- **Performance thresholds:** Under the documented Latin DHAT workload, the reference-machine global live-heap peak must remain below 40 MiB; Windows font initialization must make zero calls to the legacy `fs::read`/`fontdb`/`fontdue` path and allocate no buffer equal to a complete font file; peak and steady Windows private memory must be lower than a pre-change run using the identical executable scenario. First-present and first-fallback latency are recorded but have no hard gate in this spec.
- **Edge cases:** Missing or malformed configured files; TTC/collection inputs; a primary face missing Latin or CJK characters; unavailable fallback faces; repeated unresolved characters; spaces and zero-area glyphs; multiple characters resolving to different faces; DPI/font-size changes; atlas rebuild/eviction; DirectWrite initialization failure; and transfer or recreation of native resources across the startup/render thread boundary.

## Out of Scope

- CoreText, Fontconfig, FreeType, or any other non-Windows backend.
- Retaining the portable `fontdb`/`fontdue` implementation as a runtime or compile-time fallback.
- The reference document’s “lazy `fontdue` fallback first” delivery path; this spec intentionally proceeds directly to DirectWrite.
- Complex-script shaping, ligatures, grapheme-cluster layout, bidi layout, and changes to terminal cell-width policy.
- Full-color emoji rendering or changes from the existing grayscale atlas format.
- Dynamic atlas sizing, vertex scratch-buffer reuse, atlas eviction redesign, or replacement of the WGPU text pipeline.
- Installing fonts or mutating the user/system font collection.
- Asynchronous missing-glyph resolution and placeholder/redraw coordination.
- Preserving the exact default font family or appearance produced by the former hard-coded candidate order.

## Future Evolution

- Add platform backends such as CoreText and Fontconfig/FreeType behind the same text-backend contract.
- Add color-glyph atlas support if system emoji fallback requires color presentation.
- Add shaping and grapheme-aware resolution if terminal requirements expand beyond one-scalar glyph lookup.
- Revisit dynamic atlas allocation and reusable upload buffers after DirectWrite memory measurements isolate the remaining costs.
- Consider sharing immutable DirectWrite factories or face caches across multiple terminal/widget runtime instances if profiling shows duplicated native state.
- Re-evaluate asynchronous fallback only if measured first-use latency is unacceptable and redraw/cancellation semantics are specified.
