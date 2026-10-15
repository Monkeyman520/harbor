# Terminal/Widget Boundary Migration

**Spec ID:** 0005
**Status:** Implemented
**Date:** 2026-08-15

## Requirement

Harbor must remove the `harbor-terminal` → `harbor-widget` dependency while preserving terminal behavior through a root-level bridge Component.

## Solution

`harbor-terminal` becomes widget-framework-independent while remaining a wgpu/winit terminal renderer for this phase. It owns a `RenderTarget` constructed from physical allocation and surface geometry, plus a `TerminalEvent` family containing the terminal's required keyboard, IME, pointer, and focus semantics. Its public rendering and input APIs accept only these terminal-owned types; no public terminal API exposes `ExternalDrawId`, `ExternalDrawContext`, `UiEvent`, or other `harbor-widget` types. The existing viewport model is refactored around the terminal-owned render target rather than duplicated.

The root crate adds `TerminalWidgetBridge: Component`. It owns the widget-facing external-draw identifier and composes `CustomPaint`; its render handler converts widget draw geometry into `RenderTarget` before calling `Terminal::render`, and its input adapter converts routed `UiEvent` values into `TerminalEvent` before calling `Terminal::handle_event`. It retains the existing frame-scoped GPU-access discipline required by the `'static` external-draw handler.

The App remains the Runtime Host: it constructs and owns the terminal, drains deferred external input, applies the cross-window input gate, and determines lifecycle and error policy. Once input is permitted and the external-draw identifier matches, the App delegates adaptation and delivery to `TerminalWidgetBridge`. The bridge does not move Host policy, PTY lifecycle, or window/GPU ownership into either crate.

### Seams

| Seam | Connects | Expects | Provides |
| --- | --- | --- | --- |
| Terminal boundary | `TerminalWidgetBridge` ↔ `harbor-terminal` | `RenderTarget`, `TerminalEvent`, and frame-scoped render access | Widget-independent terminal rendering and terminal input handling |
| External paint integration | `TerminalWidgetBridge` ↔ `harbor-widget` | `CustomPaint`, `ExternalDrawId`, `ExternalDrawContext`, deferred `UiEvent`, and external-draw callback registration | Widget paint-order/clipping integration and mapped terminal input |
| Host input policy | App ↔ `TerminalWidgetBridge` | Terminal lifecycle, drained external input, and cross-window gate result | Delivery only of permitted, identifier-matched terminal events |

## End-to-End Tests

### E2E: Terminal renders through the bridge

With App owning Terminal and mounting its bridge, output invalidation or allocation changes invoke CustomPaint → bridge → terminal `RenderTarget`, preserving allocation, paint order, clipping and resize behavior.

### E2E: Permitted terminal input preserves semantics

With bridge CustomPaint focused and the gate open, deferred/drained keyboard, IME, pointer, wheel and scrollback input maps to `TerminalEvent`, preserving PTY bytes, scrolling, focus and redraw effects.

### E2E: Cross-window gate remains authoritative

With paste confirmation open, App blocks terminal keyboard input and new pastes but permits existing scrollback, output and rendering; the bridge cannot bypass the gate.

### E2E: Terminal crate is widget-independent

`harbor-terminal` builds/tests without a `harbor-widget` dependency, exposes only terminal-owned render/input boundary types, and retains parser, PTY and wgpu/winit functionality.

## Decisions

### Use terminal-owned render and input contracts

Retain wgpu/winit for this phase. This explicitly scoped 5.1 exception conflicts with ADR 0012's no-winit rule; defer UI-neutral core extraction rather than silently resolving it. See [ADR 0012](../adr/0012-consolidate-render-ui-into-terminal.md), [ADR 0011](../adr/0011-terminal-custompaint-gpu-injection.md).

### Use a root-level bridge Component

Put the bridge in the root, the current lifecycle owner and consumer of both crates; no new crate or exposed `AnyView`. See [ADR 0005](../adr/0005-custom-paint-provider-by-id.md), [ADR 0011](../adr/0011-terminal-custompaint-gpu-injection.md).

### Keep App-owned policy outside the bridge

See [ADR 0009](../adr/0009-app-cross-window-input-gate.md), [ADR 0015](../adr/0015-runtime-owned-frame-presentation.md).

### Preserve behavior, not obsolete widget-typed signatures

Source-incompatible terminal APIs are allowed; rendering, resize, input, scrollback, cursor, PTY, layout, clipping and paint order remain compatibility obligations. See [ADR 0005](../adr/0005-custom-paint-provider-by-id.md), [ADR 0011](../adr/0011-terminal-custompaint-gpu-injection.md).

### Defer cursor deadline scheduling

Do not add blink deadlines, scheduler changes or external-paint deadline providers during this dependency migration. See [ADR 0015](../adr/0015-runtime-owned-frame-presentation.md).

## Test Plan

- **Integration tests:** Cover `ExternalDrawContext`→`RenderTarget` and `UiEvent`→`TerminalEvent` mappings, identifier matching, allowed versus gated event forwarding, and terminal PTY/scrollback behavior after mapped input. Add a Cargo dependency/build assertion proving `harbor-terminal` does not depend on `harbor-widget`.
- **Manual tests:** Run an interactive shell; type normal and modified keys, IME text, wheel-scroll and scrollback keys; resize the window; open paste confirmation and verify gate behavior; verify cursor shape, visibility, and current blink behavior are unchanged.
- **Performance thresholds:** Preserve the existing sub-2 ms 80×24 terminal frame target; introduce no recurring redraw, extra full-scene rebuild, or additional terminal-to-widget dependency.
- **Edge cases:** Zero-size or clipped allocations; pointer events not recognized by terminal input; focus gain/loss; unmatched external-draw identifiers; event drain while the gate is active; layout/DPI changes; absent terminal renderer or unavailable frame-scoped GPU access.

## Out of Scope

- Cursor blink deadline scheduling, Runtime scheduler changes, and external-paint deadline providers.
- Splitting a fully UI-neutral `terminal-core` from a wgpu renderer or removing winit from the terminal crate.
- A new `harbor-terminal-widget` crate or a public reusable bridge API.
- Changing `CustomPaint` deferred-input semantics or adding a generic external input-handler registration mechanism.
- Altering parser behavior, PTY I/O strategy, terminal rendering behavior, widget event routing, GPU/window ownership, or cross-window policy.
- New GPU integration-test infrastructure.

## Future Evolution

- Add external-paint deadline providers when the cursor blink fix is implemented without an App-specific scheduler path.
- Extract `terminal-core` and `terminal-wgpu` when true UI independence or an additional rendering host is required; resolve the ADR 0012 winit-dependency conflict at that time.
- Create a dedicated bridge crate only when a second application or reusable consumer needs the same Component.
- Generalize deferred external input handlers only when a bridge must autonomously receive widget input without App-mediated Host policy.

## Historical Ticket IDs

| Historical ID | Surviving contract |
| --- | --- |
| 0005/T0001 | [Terminal-owned `RenderTarget` and `TerminalEvent` boundary](#solution) |
| 0005/T0002 | [Root-level bridge Component and external-paint integration](#use-a-root-level-bridge-component) |
