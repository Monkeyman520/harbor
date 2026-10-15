# Terminal Render Consolidation

**Spec ID:** 0002
**Status:** Implemented
**Date:** 2025-07-28

**Architecture evolution:** The consolidation remains current; later boundary changes are defined by Specs 0005 and 0012 and ADR 0031.
## Requirement

Merge `harbor-render` GPU rendering into `harbor-terminal`, making the terminal a self-contained engine (state + parsing + wgpu rendering + PTY I/O) that renders into the widget tree via the existing `CustomPaint` escape hatch, with zero orchestration in the app layer. Delete `harbor-render` and `harbor-ui` crates. Minimize `harbor-parser`'s public API to three items (`VtHandler`, `Params`, `Parser`).

## Solution

### Architecture

```
harbor-parser (VtHandler + Params + Parser)
     └─► harbor-terminal (Screen + wgpu rendering + PTY I/O)
           └─► CustomPaint widget (harbor-widget) ──► app (single root assignment)
```

`harbor-terminal` owns the screen grid, the VT parser bridge, all wgpu rendering pipelines (glyph atlas, cursor quad, scrollbar, selection highlight, background, decoration), and an internal I/O thread for PTY read. It does NOT own `GpuContext`, `wgpu::Device`, or a window — those are injected at render time by `CustomPaint`.

`CustomPaint` wraps a `Terminal`. During `build()`, it registers the terminal's `ExternalDrawFn` into `BuildCx`. `Runtime` stores a `HashMap<ExternalDrawId, Box<ExternalDrawFn>>` and invokes the correct handler when it encounters `Primitive::External` during `encode()`. No per-frame callback parameter is needed; `encode()` signature simplifies.

### Seams

| Seam | Connects | Expects | Provides |
|------|----------|---------|----------|
| Parser protocol | `harbor-terminal` → `harbor-parser` | `VtHandler` trait impl | Parsed VT actions via `Parser::advance()` |
| PTY input | `harbor-terminal` → OS PTY | `impl Read + Send + 'static` | Byte stream consumed by internal reader thread |
| PTY output | `harbor-terminal` → OS PTY | `impl Write + Send + 'static` | Keystroke bytes written synchronously |
| CustomPaint registration | `CustomPaint` → `Runtime` | `BuildCx::register_external_draw(id, fn)` | Handler stored in Runtime's `HashMap<id, fn>` |
| External draw callback | `Runtime::encode` → `Terminal` | `ExternalDrawFn` signature: `(id, rect, &mut RenderPass)` | Terminal renders into the widget's allocated rect |
| Widget input | `Runtime` → `CustomPaint` → `Terminal` | `UiEvent` via `drain_external_input` | Terminal processes keyboard/mouse events |
| GPU injection | `CustomPaint` → `Terminal` | `&GpuContext` at render time | Terminal accesses atlas, pipelines, device/queue transiently |
| App widget setup | App → `harbor-widget` | `Runtime::set_root(CustomPaint::new(terminal.draw_id()))` | Terminal is a widget in the tree |

## End-to-End Tests

### E2E: Terminal renders via widget tree

A Terminal created with PTY handles and mounted as `CustomPaint::new(draw_id)` renders PTY output `b"\x1b[31mHello\x1b[0m"` as red Hello through its registered handler on the next `Runtime::encode()`.

### E2E: Keystroke reaches PTY

With the terminal widget focused, pressing `A` routes through Runtime → queued CustomPaint input → App drain → Terminal, writing `b"a"` to the PTY.

### E2E: Terminal resize propagates

Resizing an 80×24 terminal changes the CustomPaint allocation and grid size; subsequent output uses the new geometry without losing scrollback.

### E2E: Multiple CustomPaint widgets coexist

Two Terminals with distinct draw IDs render in their own widget allocations and receive input according to hit testing.

## Decisions

### CustomPaint registers handler during build, not app at encode time

See [ADR 0005](../adr/0005-custom-paint-provider-by-id.md), [ADR 0011](../adr/0011-terminal-custompaint-gpu-injection.md).

### Terminal does not own GpuContext

See [ADR 0011](../adr/0011-terminal-custompaint-gpu-injection.md).

### Parser exposes exactly three items

See [ADR 0010](../adr/0010-parser-minimal-public-api.md).

### Synchronous PTY I/O with std traits

See [ADR 0013](../adr/0013-synchronous-pty-io.md).

### Delete harbor-ui; paste confirmation becomes widgets

Delete `harbor-ui`; use Widget UI or a terminal-internal modal. The manual test below still requires no separate window, conflicting with Spec 0001 and ADRs 0007/0009; this cleanup does not resolve that discrepancy. See [ADR 0012](../adr/0012-consolidate-render-ui-into-terminal.md).

### Component trait from harbor-render dissolves

The public `harbor-render::Component` (`prepare`/`draw`/`resize`) dissolves: Terminal was its only consumer; render components become internal. See [ADR 0012](../adr/0012-consolidate-render-ui-into-terminal.md).

### ADR-0006 Superseded

Historically, this spec supersedes ADR 0006's App-owned input semantics: App drains external input while Terminal interprets it. See [ADR 0006](../adr/0006-custom-paint-input-provider.md).

## Test Plan

- **Integration tests:**
  - `harbor-terminal`: bootstrap Terminal with mock `Read`/`Write` handles; feed bytes, verify screen state; send keystrokes, verify bytes appear on write handle.
  - `harbor-widget`: verify CustomPaint registers handler via BuildCx; verify Runtime invokes correct handler for Primitive::External; verify multiple CustomPaint widgets with distinct IDs route correctly.
  - Cross-crate: Terminal + Parser integration with incremental byte feed and screen snapshot comparison.
- **Manual tests:**
  - Full PTY session: launch shell, observe rendering, type commands, verify output.
  - Window resize: drag corner, verify grid re-allocation and scrollback.
  - Paste multi-line text, verify confirmation dialog appears as widget (not separate window).
- **Performance thresholds:**
  - Parse + screen update < 500µs per typical 4KB PTY chunk.
  - GPU upload: dirty-row incremental path used for ≥90% of frames during interactive use.
  - Frame encode (wgpu draw calls) < 2ms for 80×24 grid.
- **Edge cases:**
  - PTY read returns 0 bytes (EOF) — Terminal shuts down reader thread.
  - PTY write pipe full — write blocks; callers on main thread must tolerate brief blocking.
  - Alt-screen switch mid-batch — parser returns partial consume; Terminal handles the split.
  - Font fallback for missing glyphs — Text component renders replacement character.
  - Zero-size window — Terminal skips render, does not crash.

## Out of Scope

- Async PTY I/O. Terminal uses blocking `Read`/`Write` with an internal thread.
- Multi-window support beyond widget tree split panes.
- GPU backend abstraction (wgpu is the only backend; `backend-dx12`/`backend-vulkan` features from harbor-render are removed).
- Font loading — stays in `harbor-text`, which `harbor-terminal` depends on.
- Parser feature negotiation (e.g., SGR sequences not yet implemented). Parser emits actions; Screen handler implements or ignores.
- `harbor-pty` crate — may be simplified or kept as a PTY-spawning helper; refactoring it is out of scope.
- Widget-based paste confirmation dialog implementation details — only the architectural decision (no separate `harbor-ui`) is in scope.

## Future Evolution

- When widget framework supports native sub-windows, paste confirmation could become a widget-managed overlay instead of a winit window.
- When GPU resource lifetime management grows complex, Terminal's pipeline ownership could be extracted into a `TerminalGpu` sub-struct with explicit init/shutdown.
- If additional external draw providers emerge (image preview, web view), the `HashMap<id, fn>` pattern naturally extends — no spec changes needed.
- If PTY write latency becomes measurable under heavy paste, consider a buffered write path with `BufWriter`.
