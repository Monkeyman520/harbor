# Configuration Hot Reload: Remaining Delivery

**Source:** [Spec 0024](../../spec/0024-watched-reload-and-launch-settings.md), [spec 0023](../../spec/0023-atomic-manual-configuration-reload.md), [ADR 0062](../../adr/0062-watched-reload-and-new-session-launch-settings.md), [#155](https://github.com/wangjq4214/harbor/issues/155).
**Status:** In Progress — T0001–T0003 Done; only [T0004](./T0004-package-integration-and-evidence.md) remains open. Local tickets, not GitHub sub-issues.

## Ticket Index

| Alias / ticket | Contract and status |
| --- | --- |
| [T0001 — manual/font acceptance](../../spec/0023-atomic-manual-configuration-reload.md) | Done; spec 0023 R1–R8/E1–E11 and 0024 R1/R6. Automated/scoped native results plus prior user-confirmed manual acceptance, not full package acceptance. |
| [T0002 — session environment/directory](../../spec/0024-watched-reload-and-launch-settings.md) | Done; spec 0024 R2–R4/E1–E7/E12, including startup, native child settings, fallback, lost-directory failure and confidentiality. Automated/native-seam results plus user-confirmed testing. |
| [T0003 — watched reload](../../spec/0024-watched-reload-and-launch-settings.md) | Done; spec 0024 R4/R5/E8/E9/E11 and manual/save ordering with existing live fields. Controlled/filesystem/native-window results. |
| [T0004 — package integration/evidence](./T0004-package-integration-and-evidence.md) | Todo; all-field E10/R6, inherited manual outcomes and remaining integrated human/remote acceptance. Combined automated/native checks passed; not Done. |

## Preserved Slice Acceptance

These are historical recorded results, not checks rerun by document maintenance. Detailed evidence is deliberately local-only/Git-ignored at the user's request; it was not inspected here.

- **T0001 (2026-10-15):** Baseline `bb307d812a27919291792c32a58038bf94f3ba07`; Windows 26300.9457, ConPTY 1.24.260710001, Vulkan/RX 7900 XT, PowerShell 7.6.6, nvim 0.12.5. All five gates passed (2449 workspace passed / 0 failed / 10 ignored); 8 focused reload tests included preparation rejection, native later-session failure, failed compensation, reader resume/retry. Desktop observations covered cmd/PowerShell pending input and unchanged cmd PIDs, Consolas/Cascadia Mono/repeated sizes, active/inactive/new tabs, nvim redraw/retained primary history, invalid-font recovery, R-to-G rebind, new-session shell, clipboard restart feedback and retained-text copy. Raw ConPTY ReadFile probes received exactly `7871` for x/q during successful/rejected reload, no reload-chord bytes. E1/E2/E3/E5/E8 had automated PASS plus scoped desktop observations; E4/E6/E7/E10/E11 automated PASS (E7 native reader/PTY); E9 deterministic lifecycle plus scoped native chord PASS, not every-clause E2E proof. The user separately confirmed prior manual IME, physical Backspace, OSC/reset/startup fallback, held-key rebind and scoped backdrop/DPI checks and requested completion on the unchanged baseline. No new prior-run environment/measurements or broad compatibility claim. Local records: `docs/verification/manual-configuration-reload.md`, `target/manual-reload-acceptance/`.
- **T0002 (2026-10-15):** Baseline `da1eaeb` plus changes committed as `2488f96`; five gates passed. Agent evidence was automated/native-seam; interactive checks were NOT RUN in that record. The user subsequently confirmed testing and requested Done; no new agent-run UI check is claimed. Local record: `docs/verification/session-launch-settings-evidence.md`.
- **T0003:** Baseline `2488f965b1efdc482ba0335b34adb137366b3f4f` plus watcher diff; Windows 26300.9457. Default-on content polling uses 200 ms samples / 300 ms settling (not a latency promise), a latest-observation mailbox, payload-free wake and shared strict current-file transaction; manual reload remains independent. Five controlled/lifecycle and three real-filesystem tests passed; native saves/replacements/bursts, rejection/suppression, fault feedback/correction and real transaction current/future resource/child continuity passed. All five gates passed (2464 workspace passed / 0 failed / 10 ignored); two feedback-order defects were fixed before verification, with only a nonfatal incremental-cache note. Synthetic desktop typing was unreliable: native footer/font observations are distinct from automated PTY/input continuity. Local record: `docs/verification/watched-configuration-reload-evidence.md`.

T0001/T0002 completion is partly user-reported; T0003's waiver of a tracked/retrievable report did not itself waive T0004 requirements. T0004 records its own later execution-specific local-evidence waiver and remaining human cases. Slice processes/config were cleaned up; no remote mutation, ignore-policy change, force-add or full-package closure was claimed.

## Dependencies and Execution Boundary

T0001 provides manual/font acceptance, T0002 native launch behavior and T0003 observation; all feed T0004. T0002/T0003 used PR #201 or its successor independently of each other's completion or T0001 evidence work; only a specific defect creates a new blocker. Coordinate shared coordinator/feedback/fixture edits; both triggers use one acceptance/factory owner. No new reflow prerequisite.

T0004 must establish fresh integrated evidence rather than assume slice passes compose. Preserve environment/directory in scope. #172's font exclusion, PR #201 scope and #155 delivery/sub-issue links require separate remote-action authorization; no closure is claimed. Do not change Git policy, stage/commit or treat local-only evidence as a remote link.
