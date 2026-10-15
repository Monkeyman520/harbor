# Selectable-Scope MSI and Installation Lifecycle

**Ticket ID:** T0002
**Source:** [Spec 0023](../../spec/0023-windows-msi-and-portable-release.md), R1/R2/R5 and A1–A6/A11; [ADR 0057](../../adr/0057-windows-installation-and-portable-isolation.md).
**Status:** Todo

## Goal

Deliver one x64 MSI with explicit user/machine installation choice, working shell launch, same-scope upgrades and config-preserving uninstall, without implicit cross-scope migration.

## Affected Surfaces

- Windows installer authoring and local MSI packaging entry point consuming the R1 release payload.
- Cargo package/release identity and existing icon/resource build where consumed by packaging.
- Installed files, scoped Start Menu entry, uninstall registration, upgrade identity and opposite-scope detection.
- README/local installation instructions and Windows lifecycle evidence.

## Approach

Consume the settled spec and established runtime/configuration contracts without inventing new domain facts, acceptance semantics or architecture. Installer tooling and identifiers remain implementation choices. Use user-owned install location for current user; Program Files with administrator authorization for all users. Installed payload has no portable marker and retains home-directory settings.

Preserve the exact R1 matching binary/runtime payload and license/version files. Add the MSI capability to the shared local packaging entry point without a separate divergent runtime list. Support older-to-newer stable same-scope upgrade, config-preserving uninstall and scoped launch/uninstall entries; do not change PATH by default.

Block opposite-scope installation when an existing installation is visible to the current user. Explain that explicit uninstall is required; do not silently remove/migrate/copy config. Do not claim discovery of every other account's private user-level installation.

## Dependencies and Coordination

- **Blocked by:** None: R1 defines the payload and installed mode requires no portable marker. MSI work can consume ordinary release staging independently of portable settings changes.
- **Blocks:** T0003: publication requires a real MSI satisfying lifecycle acceptance.
- **Coordination:** T0001 shares packaging entry point, binary/runtime/version contract and README. Use one owner for shared edits; format-specific registration and marker differences must not change common file contents. Agree marker filename so MSI exclusion can be checked explicitly.

## Acceptance

- [ ] Documented local packaging produces a version/architecture-identifiable x64 MSI containing the complete R1 payload, without GitHub credentials; missing required runtime is an actionable packaging failure.
- [ ] Current-user installation normally runs without elevation, creates user-scoped launch/uninstall entries, and launches a real shell from unrelated cwd (A2).
- [ ] All-users installation requires authorized elevation into Program Files; a normal user can launch and complete a shell command/response using their own home settings (A3).
- [ ] Older-to-newer same-scope stable upgrade works in both scopes, leaves the intended new installation and matching runtime, and preserves distinguishable existing user settings (A4).
- [ ] Uninstall removes owned installed/runtime files and entries while retaining user config; subsequent explicit opposite-scope installation succeeds (A5).
- [ ] Opposite-scope attempts in both visible-install directions are blocked clearly without a second install, automatic removal, config copy or migration (A6).
- [ ] MSI carries no portable marker and makes no default PATH change.
- [ ] Document prerequisites, scopes, switching/visibility limitation and lifecycle actions; record Windows version, scope, version fixtures and real packaged shell/lifecycle evidence under R5.

## Out of Scope

Automatic scope migration, global enumeration of other users' installations, portable configuration implementation, GitHub publication, signing and non-x64 application packages.
