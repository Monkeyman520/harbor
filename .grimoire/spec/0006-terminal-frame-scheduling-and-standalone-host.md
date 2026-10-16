# Terminal Frame Scheduling and Standalone Host

**Spec ID:** 0006
**Status:** Implemented
**Superseded in part:** the public standalone winit/wgpu host requirement (Solution, its decision, and the standalone E2E) is superseded by a newer decision not recorded in this repository; the retired ticket 0006/T0003 was marked Superseded.
**Date:** 2026-09-11

## Requirement

Terminal cursor blinking must continue while idle through deadline-driven redraws in both widget-hosted and direct winit/wgpu rendering modes, without Terminal owning window, surface, submission, or presentation resources.

## Solution

Expose a host-neutral Terminal Frame Demand that reports whether a frame is immediately needed and the earliest next cursor-blink deadline. Terminal input and cursor movement reset the blink cycle to its visible phase and update that demand.

For widget hosting, extend the `CustomPaint` external-provider contract with a scheduling callback. The Runtime collects frame demands before choosing its idle policy, merges the earliest deadline into its own effects, and lets the feature-gated winit integration and `Runtime Frame Scheduler` retain sole ownership of redraw requests, `WaitUntil`, surface acquisition, submission, and presentation. A non-drawable surface suppresses timer wakes; becoming drawable causes one recovery frame and recomputes future demand.

For direct rendering, provide a public feature-gated standalone winit/wgpu host adapter outside the Terminal core. It consumes the same demand, translates deadline expiry into winit control flow and redraw requests, and invokes Terminal only with a host-owned render pass. This preserves the core `harbor-terminal` boundary while allowing consumers to use Terminal without `harbor-widget`.

### Seams

| Seam | Connects | Expects | Provides |
| --- | --- | --- | --- |
| External draw scheduling | Terminal Widget Bridge ↔ harbor-widget Runtime | A pullable Terminal Frame Demand for each registered external draw | A Runtime-owned earliest deadline and redraw effect before idle wait selection |
| Direct terminal host | Standalone winit/wgpu adapter ↔ Terminal | Frame demand plus a host-owned wgpu render pass | Direct event-loop scheduling and terminal presentation without a Widget Runtime |

## End-to-End Tests

### E2E: Idle cursor blinks inside a widget frame

A drawable, visible blinking widget-hosted cursor with no PTY/input activity alternates at successive deadlines: Runtime requests one frame per phase edge and waits between deadlines, without polling.

### E2E: Terminal input resets widget-hosted blinking

Keyboard input or cursor-position-changing output during a hidden widget-hosted blink phase makes the next frame visible immediately and starts a fresh cycle.

### E2E: Idle cursor blinks in the standalone host

Historically, a drawable idle standalone winit/wgpu host presents the same blink visibility transition through its own path when the deadline expires; see the partial-supersession note.

### E2E: Non-drawable surface suspends and recovers scheduling

In either host, zero-size, minimized or otherwise non-drawable surfaces suppress timer-driven frames; restoration requests one recovery frame and resumes deadlines from current terminal state.

## Decisions

### Use one host-neutral frame-demand contract

See [ADR 0020](../adr/0020-host-neutral-terminal-frame-scheduling.md), [ADR 0011](../adr/0011-terminal-custompaint-gpu-injection.md).

### Make Runtime the widget-hosted scheduling authority

See [ADR 0021](../adr/0021-external-draw-scheduling-and-standalone-terminal-host.md), [ADR 0015](../adr/0015-runtime-owned-frame-presentation.md).

### Keep direct winit support outside the Terminal core

Historical direct-mode contract: a feature-gated companion host satisfies ADR 0021 without adding winit to the core. The partial-supersession note above remains unresolved. See [ADR 0021](../adr/0021-external-draw-scheduling-and-standalone-terminal-host.md), [ADR 0012](../adr/0012-consolidate-render-ui-into-terminal.md).

### Suspend timer wakes for non-drawable surfaces

See [ADR 0021](../adr/0021-external-draw-scheduling-and-standalone-terminal-host.md), [ADR 0015](../adr/0015-runtime-owned-frame-presentation.md).

## Test Plan

- **Integration tests:** Cover callback registration and deadline merging in `harbor-widget`; cover Terminal Frame Demand transitions for hidden/visible phases and reset events; cover the direct adapter's `WaitUntil`/redraw behavior without requiring a real GPU surface where possible.
- **Manual tests:** Run the application in widget mode and the standalone sample/host, leave each idle for several cycles, type while the cursor is hidden, minimize and restore the window, and verify the cursor resumes correctly.
- **Performance thresholds:** With blinking as the only activity, each host must wait between phase edges and request no more than one redraw per elapsed blink edge; it must not enter continuous `Poll` solely to blink the cursor.
- **Edge cases:** Steady cursor styles, terminal-hidden cursor, multiple external draw providers, delayed event-loop wakeups that cross multiple phases, zero-sized surfaces, surface loss/reconfiguration, and input arriving at a deadline boundary.

## Out of Scope

- Changing cursor style, blink interval configuration, or terminal protocol semantics.
- Moving window, surface, device, queue, or fatal-error ownership from the Runtime Host.
- Replacing terminal input routing, PTY invalidation, damage tracking, or paste-confirmation behavior.
- Generalizing external scheduling callbacks to unrelated animation providers beyond the contract required by Terminal.
- Adding non-winit standalone platform adapters.

## Future Evolution

- Generalize the proven external draw scheduling contract to other deadline-driven custom renderers when a second concrete provider exists.
- Add direct host adapters for additional platform event loops without changing Terminal Frame Demand.
- Revisit blink configuration and focus-state behavior if user settings or accessibility requirements require them.
- Re-evaluate scheduling granularity if terminal decorations or other animations add independent deadlines.

## Historical Ticket IDs

| Historical ID | Surviving contract |
| --- | --- |
| 0006/T0001 | [Host-neutral Frame Demand and blink-reset contract](#solution) |
