# Configurable Keybindings and UI-Wins Conflict Policy

**Status:** Implementing
**Date:** 2026-09-17

## Context

Harbor UI actions (tab management, clipboard paste, selection copy, and future scroll/zoom/search hooks) share key chords with terminal applications running in the PTY. The repository previously had hardcoded tab shortcuts in `harbor-app::ui`, hardcoded paste detection in `src/dialog.rs`, and selection-gated copy in `harbor-terminal`. Issue #104 requires a configurable binding table with an explicit conflict policy between UI actions and terminal PTY input. While this record and implementation establish the keybinding architecture, UI-wins conflict resolution, and deterministic test evidence, final issue acceptance remains blocked by upstream P6 dependencies (IME preedit rendering and candidate-window positioning, and final SGR mouse stabilization).

## Decision

Establish an explicit **UI Wins** conflict policy:
1. When an active Harbor UI action binding matches an incoming key chord, the UI consumes the event and executes the action. UI-only chords are guaranteed never to leak protocol markers or bytes to the PTY.
2. Configurable UI actions comprise application-root tab management (`NewTab`, `CloseTab`, `NextTab`, `PreviousTab`, `SelectTab(1..9)`) and Host-owned clipboard paste (`Paste`). Tab shortcuts are installed in the widget root, while `PasteController` matches the configured paste chord at the window-event layer.
3. Duplicate chord assignments resolve by dislodging the previous action, maintaining a strict 1:1 chord-to-action mapping.
4. Existing selection-gated copy (`Ctrl+C` copies with selection; passes to PTY as ETX without selection) and primary-screen scrollback keys remain preserved as terminal-owned input behavior.
5. All unhandled key chords pass through to the terminal PTY.

Provide a minimal binding table in `harbor-config` with in-code defaults and optional string overrides in `~/.harbor/config.toml` under `[keybindings]`.

Rejected alternatives:
- Per-binding policy configuration (e.g. configuring `ui-wins` vs `terminal-wins` per action in TOML) is rejected as unnecessary complexity.
- Terminal-wins default for UI actions is rejected because it breaks standard desktop shortcuts when the terminal is focused.
- Moving binding policy or terminal semantics into `harbor-widget` is rejected per the runtime host boundary.
- Speculative Zoom, Find, or screen-gated scroll bindings in config are rejected; Zoom and Search have no backing UI or rendering implementation in the repository today.
## Consequences

- UI actions and chords are centralized in `harbor-config` and customizable via `config.toml`.
- The conflict policy is simple and explicit: registered UI bindings win over PTY input.
- UI-only chords never produce accidental PTY injection.
- Upstream dependency note: Issue #104 remains blocked by upstream P6 items (IME preedit rendering and candidate-window positioning, and SGR mouse stabilization). This record establishes the binding architecture and conflict contract, while end-to-end issue closure is deferred to the P6 exit gate.
- Existing paste, tab, and selection-copy semantics remain intact at their host/terminal owners.
