# Stable-Tag Release Publication and Integrated Distribution Acceptance

**Ticket ID:** T0003
**Source:** [Spec 0023](../../spec/0023-windows-msi-and-portable-release.md), R1/R4/R5 and A1/A10/A11; downloaded-artifact reruns of A2/A3/A7/A8. [ADR 0057](../../adr/0057-windows-installation-and-portable-isolation.md) constrains installer/portable behavior.
**Status:** Todo

## Goal

Publish matching unsigned x64 MSI and portable ZIP from a stable version tag through GitHub Actions, reusing documented local packaging and proving the downloaded distribution works.

## Affected Surfaces

- Tag-triggered `.github/workflows/` release automation and publication permissions.
- Root Cargo version, local Windows packaging entry point, pinned ConPTY restore and common release payload.
- GitHub Release assets and README/release usage documentation.
- Recorded integrated package/publication evidence, without claiming unrelated terminal acceptance.

## Approach

Consume the settled spec, its verified source links and T0001/T0002 interfaces. Organize and integrate them without inventing requirements, architecture, domain facts or acceptance semantics. If a needed semantic decision is missing, return it to refinement rather than implement an unconfirmed assumption.

Accept stable `vMAJOR.MINOR.PATCH` tags matching the root Cargo package version. Reject mismatches and versions unrepresentable by the selected MSI mechanism instead of truncation or remapping. Build tagged source, restore pinned dependencies and invoke the shared local package construction path so both formats originate from the same release build. Perform applicable build/payload/package checks before publishing both assets to the corresponding GitHub Release. Required-check failures must not publish an accepted artifact set.

Retain unsigned artifacts and disclose untrusted-publisher/SmartScreen risks. Do not require signing credentials or claim reputation warnings are eliminated. Keep the existing Windows PR checks; this ticket adds release automation, not a replacement claim for those gates.

## Dependencies and Coordination

- **Blocked by T0001:** Must have the portable ZIP and verified marker/configuration behavior to publish and validate the selected full portable outcome.
- **Blocked by T0002:** Must have the real MSI and verified scoped lifecycle to publish and validate installer assets.
- Workflow drafting can occur against R1/R4 while producer work is underway; complete publication and downloaded-artifact verification cannot finish before both producers.
- **Coordination:** Consume the same entry point, payload contract, marker convention and version identity; consolidate documentation changes with producer owners rather than introduce a CI-only packaging convention.

## Acceptance

- [ ] One documented local packaging invocation produces both version-matched x64 formats from the same binary/runtime release payload without GitHub credentials (A1).
- [ ] A matching stable tag builds its source and attaches both checked artifacts to the corresponding GitHub Release through actual Actions publication (A10).
- [ ] Tag/version mismatch, MSI-unrepresentable identity and missing required payload/package check failure produce actionable failure before accepted artifact publication (A11).
- [ ] Downloaded MSI passes user and elevated machine install/normal-user shell launch; downloaded ZIP passes direct launch from unrelated cwd and directory move with independent config. Capture actual artifact version and command/response evidence, not only archive or YAML inspection.
- [ ] Local and CI packaging reuse one runtime/payload contract and preserve pinned dependency pairing/license/version.
- [ ] README covers prerequisites and commands, artifact choice, stable tag/version rules and unsigned limitations, linking producer installation/portable guidance.
- [ ] Evidence distinguishes PASS/FAIL/NOT RUN/BLOCKED for real Windows package behavior and actual GitHub publication, recording tag/revision/version/build/Windows/actions. No mock publication is labeled a live release pass.

## Out of Scope

Code signing infrastructure, pre-release MSI mappings, native ARM64/x86 application packages, rewriting producer installation/configuration semantics and unrelated terminal feature acceptance.
