# Release-Only Static CRT

**Status:** Completed
**Date:** 2026-10-14
**Source:** User selected release-only static CRT during the Windows MSI/portable grimoire-loop execution checkpoint; [Spec 0023](../spec/0023-windows-msi-and-portable-release.md).

## Context

Native PE import inspection of the first x64 release found `VCRUNTIME140.dll` imported by `harbor.exe`. The pinned ConPTY DLL and its x64/ARM64/x86 hosts had no direct external VC runtime imports. Local launch on a machine with that runtime did not establish availability on clean Windows.

The user chose static CRT for release builds over app-local CRT redistribution or requiring users to install the VC++ runtime separately.

## Decision

- The Windows packaging release build uses Rust's `-C target-feature=+crt-static` for the x64 MSVC target, including its target dependencies. Normal development builds retain their existing linkage.
- Scope compiler environment changes to the packaging build and restore the caller's environment afterward. Preserve active environment rustflags, respecting Cargo's encoded rustflags precedence and ASCII `0x1f` separator.
- Inspect the actual release executable's PE import directory before packaging. Reject direct external versioned VC runtime imports, including when consuming an existing build through `-SkipBuild`; success must not depend solely on a compiler flag having been requested.
- Preserve the existing pinned ConPTY bundle unchanged. Do not add app-local CRT DLLs or an external VC++ runtime prerequisite to the selected distribution.

## Consequences

- Runtime linkage changes require rebuilding the release binary and both package formats, then refreshing affected startup/package evidence. Earlier dynamic-CRT artifacts are not the final selected release output.
- Static linkage can change binary size and requires rebuilding for CRT/compiler updates. It does not remove normal Windows system-library dependencies or expand the approved OS-support matrix.
- This decision does not waive signing limitations, unrelated workspace failures, normal-user/manual interaction evidence, or live publication acceptance.

## Sources

- [Rust Reference: static and dynamic C runtimes](https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes) documents `crt-static` and recommends inspecting the resulting binary.
- [Cargo environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html) documents environment rustflags and the encoded argument separator.
