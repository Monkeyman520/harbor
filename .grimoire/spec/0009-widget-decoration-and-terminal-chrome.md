# Widget Decoration and Terminal Chrome

**Spec ID:** 0009
**Status:** Implemented
**Date:** 2026-08-26

**Superseded in part:** the earlier 16dp inset and 12dp/25%/4dp/12dp preset in the acceptance text below are superseded by the current Terminal Window Inset (4dp) and Terminal Decoration Preset (8dp radius, 10%-opaque shadow, 1dp offset and blur) recorded in CONTEXT.md. The earlier values are retained as historical design intent.
## Requirement

Harbor must provide Flutter-style, reusable widget box decoration and use it to render the main terminal with anti-aliased rounded corners and an outer shadow while preserving layout, input, external-draw, and Acrylic behavior.

## Solution

`harbor-widget` exposes a general-purpose `DecoratedBox` wrapper with immutable value types implementing `Default` plus fluent builders modeled after Flutter: `BoxDecoration`, `BorderRadius`, `Border`, `BoxShadow`, and `ClipBehavior`. `BoxDecoration` supports an optional color, one uniform border, uniform or per-corner radii, and an ordered list of outer shadows. Values use logical pixels and do not alter measurement or layout bounds.

Decoration paints in the stable order shadow → background → child → border. Shadow lists paint in list order from back to front, with the first item at the bottom. A decoration with no color emits no background fill. Radius, border width, and blur radius are finite and non-negative; spread radius may be negative; non-finite values are invalid.

`ClipBehavior` supports `None`, `HardEdge`, and `AntiAlias`, defaulting to `None`. When clipping is enabled, the rounded shape constrains child painting and hit testing but does not clip the box's own shadows. Ancestor clips remain authoritative. Rounded clipping applies to normal widget primitives and `Primitive::External`, so the Terminal continues to render through `CustomPaint` rather than gaining terminal-specific styling or rendering APIs.

The main Runtime Host composes `TerminalWidgetBridge` inside `DecoratedBox` within the existing 16dp Terminal Window Inset. The product preset uses a 12dp radius, 25%-opaque black outer shadow, `(0dp, 4dp)` offset, 12dp blur, zero spread, no fill, and `ClipBehavior::AntiAlias`. The absent fill preserves Default Background Cell transparency and the existing Windows Acrylic backdrop.

### Seams

| Seam | Connects | Expects | Provides |
| --- | --- | --- | --- |
| Terminal decoration composition | Runtime Host → `harbor-widget` | Root `TerminalWidgetBridge`, existing 16dp inset, and the product decoration preset | A reusable `DecoratedBox` composition without Terminal-specific visual API |
| Rounded external-draw clipping | `harbor-widget` frame encoding ↔ `CustomPaint` / Terminal external draw | Paint-ordered `Primitive::External`, allocation geometry, ancestor clips, and selected `ClipBehavior` | Rounded child clip for terminal pixels while preserving external draw identity, scheduling, and render-pass ownership |

## End-to-End Tests

### E2E: Main terminal displays the product decoration

At first drawable main-window presentation, the historical preset produces a 12dp anti-aliased outline and 25%-opaque black shadow, offset 4dp downward with 12dp blur, without changing allocation or 16dp inset. See the current-preset supersession note.

### E2E: Rounded clipping applies to Terminal CustomPaint

Terminal content/scrollbar reaching a corner is clipped to the historical 12dp rounded shape through bridge/CustomPaint; the outer shadow remains visible but ancestor clips still truncate overflow.

### E2E: Rounded clipping governs pointer targeting

With `AntiAlias`, a pointer inside the terminal rectangle but outside its rounded shape does not target Terminal; input inside the shape is unchanged.

### E2E: Acrylic remains visible

Over Windows Acrylic, default terminal cells stay unfilled; explicit backgrounds, inverse cells, configured borders and shadows remain readable.

### E2E: Decoration remains reusable and optional

A decorated non-terminal child follows its configured color, border, radii, shadows and clipping without changing size; an undecorated sibling keeps its existing behavior.

### E2E: Degenerate and invalid decoration values are bounded

Zero radius/blur and negative spread collapsing shadow bounds produce no spurious pixels. The public contract rejects non-finite values without panics or invalid GPU geometry.

## Decisions

### Use a generic DecoratedBox rather than Terminal-specific properties

See [ADR 0003](../adr/0003-widget-independent-pipeline.md), [ADR 0011](../adr/0011-terminal-custompaint-gpu-injection.md).

### Preserve the CustomPaint external-draw boundary

See [ADR 0005](../adr/0005-custom-paint-provider-by-id.md), [ADR 0011](../adr/0011-terminal-custompaint-gpu-injection.md), [ADR 0015](../adr/0015-runtime-owned-frame-presentation.md).

### Separate decoration, clipping, and layout semantics

See [ADR 0002](../adr/0002-signal-pull-model.md), [ADR 0003](../adr/0003-widget-independent-pipeline.md).

### Use Flutter-style value types with a bounded first-version surface

See [ADR 0003](../adr/0003-widget-independent-pipeline.md).

### Preserve transparent terminal composition

See [ADR 0005](../adr/0005-custom-paint-provider-by-id.md), [ADR 0023](../adr/0023-windows-acrylic-backdrop-not-mica.md).

### Apply a concrete product preset at the Runtime Host root

See [ADR 0011](../adr/0011-terminal-custompaint-gpu-injection.md), [ADR 0015](../adr/0015-runtime-owned-frame-presentation.md), [ADR 0023](../adr/0023-windows-acrylic-backdrop-not-mica.md).

## Test Plan

- **Integration tests:** Verify public builder defaults and validation; uniform and per-corner radius geometry; multi-shadow ordering; stable shadow `SceneItem` identity across retained updates, including when zero-alpha or collapsed shadows are filtered as ineffective entries; shadow/background/child/border scene order; unchanged decorated-child layout; rounded hit testing; ancestor-clip intersection; `Primitive::External` clipping for `None`, `HardEdge`, and `AntiAlias`; no-fill decoration preserving transparent pixels; retained-scene updates when decoration values change.
- **Manual tests:** Launch Harbor on Windows 11 and Windows 10; inspect corners and shadow at 100%, 150%, and 200% DPI; resize, maximize, restore, and move between monitors; exercise text selection, scrollbar dragging, wheel input, focus, typing, cursor blink, synchronized output, and paste confirmation; verify Acrylic remains visible through default cells and the confirmation window remains unchanged.
- **Performance thresholds:** Preserve the existing sub-2ms 80×24 terminal frame target at p95 on the reference development GPU; decoration must not introduce a continuous redraw deadline; steady-state frames with unchanged decoration must reuse retained scene data and perform no per-frame heap allocation proportional to terminal cell count.
- **Edge cases:** Zero-sized and fully clipped allocations; radius larger than half the box extent; independently large corner radii; zero-alpha and empty shadow lists; zero blur; negative spread collapsing shadow bounds; multiple overlapping shadows; fractional logical coordinates and DPI scales; ancestor rectangular and rounded clips; surface loss/recovery; opaque explicit terminal backgrounds; pointer positions exactly on rounded boundaries.

## Out of Scope

- Inner/inset shadows.
- Per-edge border colors or widths; the first version supports a uniform `Border::all` equivalent only.
- Gradients, background images, shape decorations other than rounded rectangles, or decoration-driven layout/padding.
- Flutter `AntiAliasWithSaveLayer`, arbitrary path clipping, filters, blend modes, or a general offscreen compositing API.
- Terminal-owned decoration fields or changes to `harbor-terminal`'s internal text, background, cursor, selection, scrollbar, or scheduling pipelines.
- Appearance changes to the standalone terminal adapter or paste confirmation window.
- User-configurable terminal radius, shadow, border, or Acrylic settings.
- Custom caption chrome, Mica, or changes to Windows compositor policy.
- Allowing a widget's own shadow to escape an ancestor clip.

## Future Evolution

- Add per-edge borders when a concrete widget requires asymmetric outlines.
- Add inner shadows or arbitrary shape decoration only after a compositing model and performance budget are specified.
- Add gradients and images as new decoration inputs without changing the Terminal boundary.
- Introduce user-configurable terminal decoration after the configuration schema and live-update policy are defined.
- Re-evaluate save-layer anti-aliasing if visual evidence shows direct anti-aliased rounded clipping is insufficient.
- Generalize rounded clip representation for nested transformed clips when non-axis-aligned transforms enter the Widget Runtime.

## Historical Ticket IDs

| Historical ID | Surviving contract |
| --- | --- |
| 0009/T0003 (Layered Outer Shadows) | [Solution](#solution) and [Test Plan](#test-plan), including stable shadow SceneItem identity when ineffective entries are filtered. |
