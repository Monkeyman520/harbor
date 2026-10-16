# Windows Installation and Portable Configuration Isolation

**Status:** Testing
**Date:** 2026-10-14
**Source:** Windows release refinement conversation on 2026-10-14; user selected x64, selectable per-user/per-machine MSI, full portable mode, local packaging plus tag-triggered GitHub Release, and Spec → slice.

## Context

Harbor currently reads startup settings from `~/.harbor/config.toml` ([README](../../README.md)). Distribution already requires an executable-relative complete ConPTY bundle, including the matching DLL, all three host architectures, version and license ([runtime contract](../../third_party/conpty/README.md)). An install-free archive that still uses home-directory configuration is not the full portable behavior selected by the user.

## Decision

- Provide selectable current-user and all-users MSI installation scopes. Per-user installation targets a user-owned location; per-machine installation targets Program Files and requires administrator authorization.
- Treat the portable distribution as an isolated configuration mode: configuration belongs to the portable directory and travels with it; it does not read or write the existing home-directory configuration. Merely omitting MSI registration is insufficient.
- Preserve the existing complete executable-relative ConPTY distribution contract in both installation and portable packaging. The initial application artifact architecture is x64; this does not authorize removing ARM64 or x86 OpenConsole hosts from its runtime bundle.
- Do not automatically switch MSI installation scopes or allow a new opposite-scope installation when an existing installation is visible to the current user. Require explicit uninstall before switching. Do not claim detection of every other account's per-user installation.
- Identify portable mode by a marker beside the executable, shipped only in the ZIP. Read `config.toml` beside that executable; do not auto-create or overwrite configuration. Missing or invalid configuration retains existing default/diagnostic behavior without consulting home-directory settings.

These boundaries are user-approved behavior with implementation and partial local verification. Full release acceptance remains pending. Artifact architecture and release automation details are ordinary delivery requirements and belong in the associated specification.

## Consequences

- Installed and portable distributions must remain distinguishable independently of the shell's current working directory.
- Portable settings must remain associated with the moved application directory rather than the Windows account's home directory.
- The selected lifecycle avoids implicit privilege-changing migration. Marker naming and installer tooling remain implementation details; mode identity and configuration isolation are fixed contracts.
- This record does not establish signing, live publication, clean-machine compatibility, or overall Windows acceptance. See the specification's execution evidence and outstanding decisions.
