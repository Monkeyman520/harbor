# Lifecycle Cleanup and Conformance Evidence

**Ticket ID:** T0004
**Source:** [Spec: 0007-synchronized-output-mode](../../spec/0007-synchronized-output-mode.md)
**Status:** Todo

## Goal

RIS and PTY/session close reliably release synchronized-output suppression, and the protocol checklist records evidence for the completed `?2026` behavior.

## Execution

Clear RIS and PTY EOF/session-close state exactly once, preserving DECSTR and query status. The matched bridge must propagate released eligibility; Runtime must cancel stale recovery demand and permit ordinary presentation without violating drawable-surface policy. Test cleanup end-to-end after a prior forced recovery. Check protocol entries only after automated batch, nesting, recovery and lifecycle evidence exists.

## Blocked by

- [Spec 0007 Solution](../../spec/0007-synchronized-output-mode.md#solution) — shared presentation-eligibility contract (historical T0001, retired).
- [Spec 0007 synchronized batch contract](../../spec/0007-synchronized-output-mode.md#solution) — clears the concrete nested state and validates normal release.
- [Spec 0007 E2E](../../spec/0007-synchronized-output-mode.md#e2e-an-unclosed-batch-receives-bounded-recovery-presents) — cancellation of completed recovery behavior (historical T0003, retired).

## Blocks

- None.

## Acceptance

- [ ] RIS clears every active synchronization depth and permits ordinary presentation.
- [ ] DECSTR leaves an active synchronization depth and its query status unchanged.
- [ ] PTY EOF/session close clears synchronization and a surviving terminal view can redraw normally.
- [ ] Cleanup cancels pending recovery scheduling without a stale or busy wake.
- [ ] End-to-end tests cover reset and session close after active synchronization, including recovery state.
- [ ] `docs/protocol/checklist.md` checks only `?2026` items supported by the completed automated evidence.

## Out of Scope

- Changes to alternate-screen, reflow, wide-cell, mouse, or focus semantics.
- Windows manual smoke execution itself; this ticket supplies the protocol evidence that unblocks it.
- New synchronized-output protocol families or diagnostics beyond the accepted checklist evidence.
