# Windows MSI and Portable Release

**Source:** [Spec 0023](../../spec/0023-windows-msi-and-portable-release.md), [ADR 0057](../../adr/0057-windows-installation-and-portable-isolation.md), [ADR 0058](../../adr/0058-release-only-static-crt.md).
**Status:** Implementation and partial local verification exist; T0001–T0003 remain Todo pending full acceptance; their execution contracts survive in the spec and mappings below. No commit, tag, push or live Release was performed.

## Ticket Index

| Alias / ticket | Surviving contract / acceptance |
| --- | --- |
| [T0001 — portable configuration/ZIP](../../spec/0023-windows-msi-and-portable-release.md) | Spec R1/R3/R5; A1/A7–A9/A11 |
| [T0002 — selectable-scope MSI](../../spec/0023-windows-msi-and-portable-release.md) | Spec R1/R2/R5; A1–A6/A11 |
| [T0003 — stable-tag publication](../../spec/0023-windows-msi-and-portable-release.md) | Spec R1/R4/R5; A1/A10/A11 plus downloaded A2/A3/A7/A8 |

## Dependencies and Remaining Acceptance

T0001/T0002 are independent producers of the same R1 payload; coordinate one owner for shared packaging/runtime/version/marker/README edits. ZIP includes the marker, MSI excludes it. T0003 may draft automation but cannot finish without both actual producer formats and their acceptance; local and CI packaging must not diverge.

[Spec 0023's execution addendum](../../spec/0023-windows-msi-and-portable-release.md#execution-addendum-final-local-evidence-and-external-gates) retains the historical revision/environment, A1–A11 results, final hashes, static-CRT inspection and limitations. Do not repeat incident diaries here or promote old results to newer-tree acceptance; [Current Status](../../../docs/current-status.md#windows-distribution) records current revision-specific package checks and the unresolved GPU-test intermittency.

Remaining: genuine non-elevated user install and ordinary-user launch, authorized UAC machine install followed by ordinary-user launch, live stable-tag GitHub Release and downloaded MSI/ZIP smoke. Keep the `0.0.9` same-binary MSI fixture limitation explicit: installer-version mechanics do not prove historical application migration. No public publication or spec/ticket completion is claimed. New semantic decisions require clarification rather than invented requirements.
