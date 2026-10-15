# Portable Configuration and ZIP

**Ticket ID:** T0001
**Source:** [Spec 0023](../../spec/0023-windows-msi-and-portable-release.md), R1/R3/R5 and A1/A7–A9/A11; [ADR 0057](../../adr/0057-windows-installation-and-portable-isolation.md).
**Status:** Todo

## Goal

Deliver an x64 portable ZIP that runs directly after extraction with the complete pinned ConPTY payload and directory-local configuration, independent of cwd and existing home settings.

## Affected Surfaces

- Startup settings selection and its consumers, anchored in `crates/harbor-config/src/lib.rs` and application startup.
- Windows local packaging scripts/entry point, Cargo release output and existing `crates/harbor-pty/build.rs` staging contract.
- ZIP payload, executable-adjacent marker, portable configuration guidance and README.

## Approach

Consume the settled spec and existing source choices; do not invent or revise requirements, architecture or acceptance semantics. Use a marker beside the exe to select `config.toml` in the same directory; preserve existing startup-only TOML parser/defaults/diagnostics. Do not consult home configuration or write/create configuration in portable mode. Without the marker, retain existing installed/user settings behavior. Marker filename is an implementation detail coordinated with T0002/T0003.

Construct the ZIP from the shared versioned x64 release payload defined by R1, including all three OpenConsole host architectures and runtime LICENSE/VERSION. Validate required payload before packaging reports success. Provide/document the ZIP capability of the local entry point; T0002 adds the MSI capability and T0003 verifies the complete two-format contract. Do not solve distribution by changing the ConPTY loader to use cwd/PATH or an inbox fallback.

## Dependencies and Coordination

- **Blocked by:** None: Spec R1 fixes the common payload contract. No need to wait for MSI lifecycle implementation.
- **Blocks:** T0003: complete release publication requires the actual portable artifact and verified configuration behavior.
- **Coordination:** Share the local entry point, version identity and marker filename with T0002/T0003. Assign one owner for shared script/README edits; do not duplicate the binary/runtime file list per format. Shared files are merge risks, not semantic blockers.

## Acceptance

- [ ] Local Windows packaging produces a version/architecture-identifiable ZIP from release output with the exact complete R1 runtime payload, without publishing credentials or receiver-side Cargo/repository dependencies.
- [ ] Direct exe launch with marker uses exe-adjacent `config.toml` from unrelated cwd; contrasting valid home settings are neither substituted nor modified (A7).
- [ ] Moving the complete directory, including settings and marker, to a spaced/non-ASCII path preserves observed settings and a real bundled-ConPTY shell command/response (A8).
- [ ] Missing/unreadable/malformed portable config retains defaults/diagnostics, never home fallback; startup does not create/overwrite config, and readable read-only config works without configuration writes (A9).
- [ ] Marker-absent startup retains existing user configuration behavior; existing TOML recovery semantics remain intact.
- [ ] Missing required ConPTY payload prevents successful packaging and does not enable inbox fallback (ZIP portion of A11).
- [ ] README documents marker/config placement, use of the existing example, startup-only parsing and complete directory movement. Record real packaged Windows shell evidence and meaningful configuration precedence regressions under R5.

## Out of Scope

MSI registration/lifecycle, GitHub publication, native ARM64/x86 application packages, hot reload, preferences UI, signing and a no-trace shell/process sandbox.
