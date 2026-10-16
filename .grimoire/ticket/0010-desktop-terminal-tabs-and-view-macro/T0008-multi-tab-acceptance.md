# Multi-Tab Resize, Scheduling, and Acceptance

**Ticket ID:** T0008
**Source:** [Spec: 0010-desktop-terminal-tabs-and-view-macro](../../spec/0010-desktop-terminal-tabs-and-view-macro.md)
**Status:** Todo

## Goal

The complete multi-tab slice has executable evidence for all-session resize, active-only input/render scheduling, deterministic cleanup, macro equivalence, Windows desktop behavior, and standard quality gates.

## Execution

Verify the integrated widget layout/keying, interaction, scrolling, focus, theme, macro and post-layout contracts. Broadcast each distinct valid clamped TerminalSize once after layout; test active/inactive output, blink, synchronized output, activation, close, failure handling and surface recovery. Prove reader/PTY/bridge/schedule cleanup, macro/handwritten equivalence and compile-fail diagnostics; run workspace, Windows and performance/resource gates below.

## Blocked by

- [Spec 0010 Terminal allocation propagation](../../spec/0010-desktop-terminal-tabs-and-view-macro.md#terminal-allocation-propagation) — post-layout allocation feedback (historical T0005, retired).
- [Spec 0010 tab/session contract](../../spec/0010-desktop-terminal-tabs-and-view-macro.md#application-model-and-boundaries) — Integrated acceptance must verify independent resources, tab-qualified events, active-only registrations, stale-event safety, and lifecycle cleanup.
- [Spec 0010 Product Behavior](../../spec/0010-desktop-terminal-tabs-and-view-macro.md#product-behavior) — integrated product UI (historical T0007, retired).

## Blocks

- (none)

## Acceptance

- [ ] Every live tab receives each changed rows/columns value once; inactive tabs do not remain at stale window geometry.
- [ ] Zero size/minimize emits no invalid PTY resize, and restore/maximize/DPI transitions recover correctly.
- [ ] Only the active terminal receives input, paints, and contributes visible cursor scheduling.
- [ ] Sustained inactive output is ingested without hidden terminal draws or cursor-driven continuous Poll.
- [ ] Activation replaces external draw/schedule registration before presentation; close removes it with no stale callback.
- [ ] Paste confirmation blocks terminal and tab-changing commands until resolved.
- [ ] Final-tab close shuts down the session and closes Harbor.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --workspace`, `python scripts/check_docs.py`, and applicable checklist tooling pass.
- [ ] Windows smoke covers shell/Vim/less/output in multiple tabs, shortcuts, pointer selection, scrollback, Acrylic, resize, 100/150/200% DPI, minimize/restore, and surface recovery.
- [ ] Quiet-runtime and active-frame measurements remain within existing validation/performance policy; no leak or work proportional to hidden terminal cell count is introduced.

## Out of Scope

- Deferred feature additions from the source spec; failures discovered here become focused follow-up tickets rather than silent acceptance exceptions.
