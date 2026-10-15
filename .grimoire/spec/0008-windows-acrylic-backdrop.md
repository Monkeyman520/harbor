# Windows Acrylic Backdrop

**Spec ID:** 0008
**Status:** Implemented
**Date:** 2026-08-25

**Superseded in part:** the earlier TransientWindow-first, 0.72-alpha contract in the acceptance text below is superseded by the unified backdrop tint chain (ADR 0026, amended by ADR 0028). The current four-tier chain and the configured `BACKGROUND` alpha are the reference; the earlier values are retained as historical design intent.
## Requirement

The Harbor main window on Windows must show a Windows Terminal-style Acrylic backdrop through default-background cells and the caption strip, while remaining readable for inverse and non-default cells.

## Solution

Acrylic is a Runtime Host / compositor concern, not a VT protocol feature. The Host creates the main window as an alpha-composited HWND, enables the OS acrylic material, and presents GPU frames whose default-background pixels do not occlude that material. Terminal continues to skip Default Background Cell quads and paints Inverse Default Cells so reverse video stays readable.

On Windows 11 build 22621 and later, the Host creates the main window with winit transparency, `BackdropType::TransientWindow` (`DWMSBT_TRANSIENTWINDOW`), and caption color `None` so Desktop Acrylic sits behind the entire window including the non-client strip. On earlier Windows versions, including Windows 10, the Host applies `SetWindowCompositionAttribute` with `ACCENT_ENABLE_ACRYLICBLURBEHIND` after the HWND exists, using a tint aligned to `harbor_config::BACKGROUND` at alpha 0.72. Mica (`DWMSBT_MAINWINDOW`) is not used.

The Host keeps system decorations. It leaves the window title `Harbor` for the taskbar and Alt-Tab, and uses `SetWindowThemeAttribute` with `WTNCA_NODRAWCAPTION` and `WTNCA_NODRAWICON` so DWM does not paint title text or the caption icon. Minimize, maximize, and close remain DWM System Caption Chrome; Harbor does not custom-draw or hit-test those buttons. Windows 10 Caption Degradation is accepted: if a theme still paints an opaque caption, the client area must still be acrylic and the limitation is documented.

The main-window wgpu surface selects a compositing-capable `CompositeAlphaMode`, preferring `PreMultiplied`, then `PostMultiplied`, then `Auto`, and never choosing `Opaque` when a compositing mode is advertised. The Host passes `BACKGROUND` with alpha 0.72 as `WinitFrameTarget` clear color (premultiplied at the clear boundary when the surface is premultiplied). Opaque GDI first-paint is skipped whenever the main window is translucent. Widget chrome must not emit an opaque full-window quad over the terminal.

`Color::to_rgba(Color::Default)` remains opaque white. Inverse Default Cells fill with that default foreground and glyph-paint `BACKGROUND` RGB at alpha 1. Non-default cell backgrounds stay fully opaque.

The paste confirmation window stays a separate opaque OS window with its existing surface alpha-mode selection and black clear. The standalone terminal winit/wgpu adapter stays opaque.

### Seams

| Seam | Connects | Expects | Provides |
| --- | --- | --- | --- |
| Main-window compositor policy | Runtime Host → Windows DWM / user32 | HWND, OS build, system decorations | Acrylic behind client and caption; undrawn title and icon; DWM caption buttons |
| Surface compositing alpha | Runtime Host / `GpuContext` → wgpu | Advertised `alpha_modes` | Compositing-capable `CompositeAlphaMode` for the main surface only |
| Frame clear color | Runtime Host → Runtime Frame Presentation | `BACKGROUND` at alpha 0.72, premultiplied when required | `LoadOp::Clear` that reveals acrylic through Default Background Cells |
| Default and inverse cell paint | `harbor-terminal` → `harbor_config::BACKGROUND` | `Color::Default` cells and inverse-default attributes | Skipped default-background quads; readable Inverse Default Cells |

## End-to-End Tests

### E2E: Windows 11 acrylic through empty cells

On Windows 11 build 22621+ with transparency effects enabled, desktop/other-window content behind Harbor appears as Acrylic through Default Background Cells, not opaque brown or Mica.

### E2E: Caption strip is acrylic without title or icon

The decorated main window shows an Acrylic Win11 caption without title/icon paint; DWM minimize/maximize/close work and taskbar/Alt-Tab still identify Harbor.

### E2E: Inverse and colored cells stay readable

Over Acrylic, Inverse Default Cells use opaque default-foreground fills with `BACKGROUND` glyphs; named/RGB backgrounds remain opaque/readable, without black holes.

### E2E: Windows 10 client acrylic with accepted caption degradation

On Windows 10 with transparency enabled, default client cells reveal Acrylic. An opaque themed caption is accepted/documented degradation, without crashes or custom buttons.

### E2E: Paste confirmation remains an opaque separate window

When both windows present, paste confirmation remains a separate opaque native window on its existing path, with unchanged confirm/cancel behavior and no Acrylic.

### E2E: Surface recovery still presents

After surface loss, outdated configuration, resize or temporary zero size, Runtime follows its existing recover/skip policy; recovered frames again reveal Acrylic through default cells.

## Decisions

### Acrylic on both Windows 10 and Windows 11, not Mica

See [ADR 0023](../adr/0023-windows-acrylic-backdrop-not-mica.md).

### System caption buttons with undrawn title and icon

See [ADR 0024](../adr/0024-system-caption-buttons-undrawn-title.md).

### Windows 10 opaque caption is documented degradation, not CSD

Document an opaque Win10 caption instead of using ADR 0023's former CSD escape: ADR 0024 forbids custom caption buttons. See [ADR 0023](../adr/0023-windows-acrylic-backdrop-not-mica.md), [ADR 0024](../adr/0024-system-caption-buttons-undrawn-title.md).

### Host owns compositor and surface config; Runtime owns present given a clear color

See [ADR 0015](../adr/0015-runtime-owned-frame-presentation.md), [ADR 0011](../adr/0011-terminal-custompaint-gpu-injection.md), [ADR 0012](../adr/0012-consolidate-render-ui-into-terminal.md).

### Paste confirmation and standalone host stay opaque

See [ADR 0007](../adr/0007-retain-separate-paste-confirmation-window.md), [ADR 0009](../adr/0009-app-cross-window-input-gate.md), [ADR 0021](../adr/0021-external-draw-scheduling-and-standalone-terminal-host.md).

### Hardcoded translucent `BACKGROUND` until user config exists

Historical preset: RGB `[0.36, 0.20, 0.08]`, alpha 0.72, with no TOML/runtime toggle in this slice; see the partial-supersession note. See [ADR 0023](../adr/0023-windows-acrylic-backdrop-not-mica.md).

## Test Plan

- **Integration tests:** Alpha-mode selection prefers a compositing-capable mode and never picks `Opaque` when alternatives exist; Inverse Default Cell fill and glyph color; Default Background Cell skip remains degenerate; `BACKGROUND` alpha is 0.72; existing GPU/surface recovery contracts stay green.
- **Manual tests:** Windows 11 22H2+ smoke that Acrylic is visible through empty cells and the caption strip; Windows 10 smoke that the client is Acrylic and that an opaque caption, if present, is the documented degradation; taskbar still says Harbor; min/max/close work; paste confirmation still presents; system Transparency effects off yields a defined non-crash appearance.
- **Performance thresholds:** Idle scheduling remains `Wait` with no continuous redraw loop introduced by Acrylic. Present and input-latency gates in `docs/validation.md` stay green.
- **Edge cases:** Maximize and restore; DPI change; zero-sized then drawable; surface lost/outdated; inverse on named vs default colors; Windows builds below 22621 taking the accent path; confirmation window open while the main window is Acrylic.

## Out of Scope

- User TOML, theme packs, settings UI, or a runtime Acrylic toggle.
- Custom blur radius, tint controls, or non-system blur materials.
- Acrylic, transparency, or alpha-mode changes on the paste confirmation dialog.
- Unix transparency.
- OSC 10/11/12 default-color protocol.
- Mica or TabbedWindow backdrops.
- Custom-drawn caption buttons, client-side decoration, or empty `with_title("")` that would blank the taskbar name.
- Changing `Color::to_rgba(Color::Default)` globally.
- Acrylic on the standalone terminal winit/wgpu adapter.
- Changing confirmation-window `alpha_modes[0]` selection unless a black/opaque regression is proven after the main-surface change.

## Future Evolution

- Land user-configurable background alpha and optional Acrylic disable in the TOML config issue.
- Revisit Windows 10 caption reliability if dogfood shows the opaque strip is common enough to need a non-CSD Host workaround.
- Offer Mica as an explicit appearance option only if product direction changes away from Windows Terminal-style glass.
- Extend Acrylic to Unix only after P8 host work exists; this spec is Windows-only.
- Re-evaluate undocumented accent policy if Microsoft removes `SetWindowCompositionAttribute` behavior.

## Historical Ticket IDs

| Historical ID | Surviving contract |
| --- | --- |
| 0008/T0003 (Inverse Default Cell paint) | [Solution](#solution), [inverse-cell E2E](#e2e-inverse-and-colored-cells-stay-readable), and [Test Plan](#test-plan) |
