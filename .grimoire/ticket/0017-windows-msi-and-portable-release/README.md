# Windows MSI and Portable Release

**Source:** [Spec 0023](../../spec/0023-windows-msi-and-portable-release.md), user-confirmed refinement on 2026-10-14, [ADR 0057](../../adr/0057-windows-installation-and-portable-isolation.md).
**Ticket folder:** `.grimoire/ticket/0017-windows-msi-and-portable-release/`
**Endpoint:** Execution through explicitly invoked `grimoire-loop`; implementation and partial verification exist. Full acceptance is not established, and ticket completion statuses remain unchanged pending the evidence listed below.

## Overview and Handoff

Deliver x64 MSI plus full portable ZIP from one versioned release payload. Both preserve the existing complete executable-relative ConPTY bundle, including all three host architectures, runtime license and version. MSI offers user/machine scope, same-scope upgrade and config-preserving uninstall; block visible opposite-scope install until explicit uninstall. ZIP ships an exe-adjacent marker selecting exe-adjacent `config.toml` without consulting home settings or creating/overwriting config. Stable `vMAJOR.MINOR.PATCH` tag must equal root Cargo version; local packaging and tag-triggered GitHub Release produce matching unsigned assets.

The user subsequently invoked `grimoire-loop`, authorizing scoped production, script, workflow, test and documentation changes. No commit, tag, push or live Release publication has been performed. ADR 0057 is Testing; the implementation and local automated verification are complete, with external acceptance still pending. The saved execution approach is `.grimoire/plans/0003-windows-msi-and-portable-release.md` (local ignored plan). Release-only static CRT is realized and verified in [ADR 0058](../../adr/0058-release-only-static-crt.md).

Explicit limitations: initial application architecture x64; no native ARM64/x86 app packages, prerelease version mapping or signing; install visibility does not enumerate every other account; portable isolation concerns configuration, not a no-trace sandbox for launched shells. Installer technology, marker filename and packaging script/asset names remain implementation details, not unconfirmed semantic assumptions.

### Specialist Knowledge Boundary

Consume the settled contract and verified sources. Organize, express, decompose and sequence existing choices only; do not invent or revise domain facts, requirements, constraints, acceptance semantics, architecture or unconfirmed assumptions. Trace meaningful choices to Spec 0023/ADR 0057 and their sources. Formatting or execution ordering does not authorize filling semantic gaps. If a new fact/decision is needed, suspend that part and return the gap to refinement for clarification and qualifying turn-batched recording before resuming; do not retrospectively legitimize invented choices or re-record processed knowledge. Return to the selected endpoint rather than launching downstream workflows.

## Delivery Surfaces

- `crates/harbor-config/src/lib.rs` and application startup: mode/path selection while reusing TOML semantics.
- Cargo release output, `crates/harbor-pty/build.rs`, ConPTY runtime contract and local packaging: shared matching binary/runtime source.
- ZIP directory marker and configuration instructions.
- MSI authoring, scoped files/entries, upgrade/uninstall and visible cross-scope detection.
- `.github/workflows/`: stable-tag build/check/publication alongside existing PR gates.
- README and package-specific evidence: local commands, Windows operation and actual downloaded assets.

Verified code/document paths are linked in the specification; their current behavior is not evidence that these tickets are implemented.

## Requirement Coverage

| Contract / acceptance | Owner | Integration |
| --- | --- | --- |
| R1 complete shared payload and local formats | T0001 ZIP; T0002 MSI | T0003 combined A1 and same-build publication |
| R2 scoped MSI lifecycle; A2–A6 | T0002 | T0003 downloaded A2/A3 shell launch |
| R3 portable config/ZIP; A7–A9 | T0001 | T0003 downloaded A7/A8 direct/moved launch |
| R4 stable tag/version and publication; A10 | T0003 | Consume both producer artifacts |
| R1/R4 failure outcomes; A11 | T0001/T0002 payload; T0003 version/publication | No silent ConPTY fallback or successful invalid release |
| R5 docs and consumer evidence | Each ticket for its surface | T0003 consolidated instructions and release evidence |

## Dependencies

| Producer | Consumer | Why completion cannot proceed first |
| --- | --- | --- |
| T0001 | T0003 | Actual full portable artifact and verified settings isolation are required for publication/download acceptance |
| T0002 | T0003 | Actual MSI and verified installation lifecycle are required for publication/download acceptance |

T0001 and T0002 are parallel-with-coordination, not blocked by each other. Their common payload is already defined in Spec R1. T0003 may draft automation against that contract, but cannot finish publishing/verifying both formats without producers.

## Coordination

| Tickets | Risk | Strategy |
| --- | --- | --- |
| T0001, T0002 | Shared local packaging entry point and runtime file list | One owner for shared edits; consume the same R1 release payload, format-specific marker/registration only |
| T0001, T0002, T0003 | Mode marker and version identity drift | Agree concrete implementation names once; portable ZIP includes marker, MSI excludes it; preserve one version/build source |
| All | README and evidence claims | Own surface-specific instructions; consolidate without changing settled acceptance or claiming checks not run |

## Recommended Order

Implement T0001 and T0002 in parallel if shared-edit ownership is explicit; serial execution is also safe. Complete their Windows package acceptance, then finish T0003 integrated local packaging, actual stable-tag publication and downloaded-artifact verification. These are execution tickets, not a separate implementation plan.

## Ticket Index

| Ticket | File | Outcome |
| --- | --- | --- |
| T0001 | [T0001-portable-configuration-and-zip.md](./T0001-portable-configuration-and-zip.md) | Isolated marker-selected portable configuration and complete ZIP |
| T0002 | [T0002-selectable-scope-msi.md](./T0002-selectable-scope-msi.md) | User/machine MSI with upgrade/uninstall and visible cross-scope protection |
| T0003 | [T0003-stable-tag-release-publication.md](./T0003-stable-tag-release-publication.md) | Shared local two-format packaging and checked stable-tag GitHub Release |

## Execution Evidence and Remaining Acceptance

- Source snapshot: `eed3494` plus local changes. Final MSI/ZIP hashes and criterion-specific evidence are bound in [Spec 0023](../../spec/0023-windows-msi-and-portable-release.md).
- Both real setup GUI routes completed installation in the selected native context (user 2, machine 4), at user Apps/native Program Files. Both quiet and GUI same-scope upgrades removed the old registration; both visible cross-scope attempts were blocked and owned uninstalls preserved config. The old `0.0.9` MSI is an explicit installer-version fixture using the same application binary, not a historical release.
- Actual advertised Start Menu launches loaded home settings and the adjacent runtime; targeted window keys produced standalone `echo 638240` responses in both installed scopes. Portable original/moved spaced/non-ASCII directories also passed real GUI command round trips, read-only config loading and isolation from distinctive valid home settings.
- Actual absent, malformed and ACL-denied portable config all recovered to default shell behavior without home fallback or config writes. Own ACLs and test installations were restored/removed.
- Formatting, all-target/all-feature Clippy and the complete locked workspace suite passed. The final workspace run used offline dependency resolution after a checksum-verified missing crate was fetched into the normal cache. The 14 earlier resize failures were repaired at producer startup with file-backed fixtures and complete END-line handshakes; all 16 original scenarios passed without changing terminal production behavior or expected geometry.
- Real wizard navigation found and repaired a cached per-user Program Files path after machine-scope selection. A permanent actual-MSI costing regression covers the late scope change; cabinet embedding and complete runtime/version failure boundaries also passed.
- Remaining: genuine non-elevated install/normal-user launch and UAC evidence; live stable-tag GitHub Release and downloaded-asset validation. Keep the version-fixture limitation explicit. No public release was created and no ticket/spec completion is claimed.

