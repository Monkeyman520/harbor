# Windows MSI and Portable Release

**Spec ID:** 0023
**Status:** Draft
**Date:** 2026-10-14
**Sources:** User's Windows distribution refinement and explicit selections on 2026-10-14; [ADR 0057](../adr/0057-windows-installation-and-portable-isolation.md); [configuration context](../CONTEXT.md#toml-user-settings); [portable context](../CONTEXT.md#portable-distribution-mode); repository surfaces below.

## Intent and Settled Handoff

Make final Harbor releases usable on Windows through both an MSI installer and a fully portable ZIP. The user selected Spec → slice, not implementation or QA. Permission covers this specification, associated tickets, and qualifying Context/ADR records; no production, pipeline, Git configuration, staging or commit changes are authorized by this refinement.

Confirmed choices: initial x64 application packages; selectable MSI user/machine scope; no visible cross-scope coexistence or automatic migration; marker-identified full portable configuration; local packaging plus stable-tag GitHub Release publication; unsigned initial artifacts. Ordinary release requirements live here, while ADR 0057 preserves configuration and installation boundaries. No semantic assumptions or blocking questions remain. Concrete installer tooling, marker filename, script names, product identifiers and asset filenames are not mandated: implementations must satisfy this observable contract rather than treat those details as unresolved requirements.

## Verified Current Surfaces

- [README](../../README.md): Windows-first operation, complete adjacent ConPTY distribution, startup-only `~/.harbor/config.toml` settings; no auto-creation or hot reload.
- [ConPTY runtime contract](../../third_party/conpty/README.md): pinned Microsoft ConPTY `1.24.260710001`; target DLL plus x64/arm64/x86 hosts, LICENSE and VERSION. Its stated Windows 10 build 17763 baseline is a dependency baseline, not proof of Harbor's complete minimum OS support.
- [PTY build staging](../../crates/harbor-pty/build.rs): stages all required runtime files beside profile and test executables.
- [ConPTY loader](../../crates/harbor-pty/src/windows/conpty.rs): executable-relative `conpty/`; all hosts required; missing bundle errors, no inbox fallback, no cwd/PATH DLL search.
- [Settings loader](../../crates/harbor-config/src/lib.rs): `default_config_path`, `load`, `load_from_path`; defaults and diagnostics on missing/unreadable/invalid TOML, scalar and palette recovery already defined.
- [Package metadata and release profile](../../Cargo.toml), [Windows icon resource build](../../build.rs), [configuration example](../../config.example.toml).
- [Existing Windows quality workflow](../../.github/workflows/windows-quality-gates.yml): PR checks and pinned ConPTY restore, not a release publication pipeline. This specification adds release capability without claiming those gates establish installation acceptance.

## Requirements

### R1. Common Release Payload and Local Packaging

Provide a documented Windows local packaging entry point producing an x64 MSI and portable ZIP for the same release version from the same release-build application payload. Package identity must expose version and architecture without requiring installation to identify the download. Local packaging must not require GitHub publishing credentials.

Both formats must contain `harbor.exe` and the complete adjacent runtime layout:

```text
harbor.exe
conpty/
  conpty.dll                 # x64 application DLL
  x64/OpenConsole.exe
  arm64/OpenConsole.exe
  x86/OpenConsole.exe
  LICENSE
  VERSION
```

Preserve ConPTY pairing, version and redistribution license. Do not remove alternate host architectures because the application is x64; hosts are chosen for the native OS, including emulation. Validate required payload availability before returning successful distributable output. Preserve executable-relative loading and explicit errors for incomplete runtime bundles. Neither package may require Cargo, the repository checkout or its `third_party/` directory on the receiving machine.

The subsequent user-approved CRT policy requires release-only static CRT linkage for the x64 MSVC application, with actual PE import inspection before packaging, including `-SkipBuild`. Do not add an external VC++ runtime prerequisite or app-local CRT redistribution. Development linkage remains unchanged. See [ADR 0058](../adr/0058-release-only-static-crt.md).

### R2. MSI Installation and Lifecycle

One x64 MSI allows selection of current-user or all-users installation. User installation uses a user-owned directory and normally requires no elevation; machine installation uses Program Files and requires administrator authorization. Provide a Start Menu launch entry scoped to the selected installation, and a registered uninstall entry. Do not modify PATH by default.

Support upgrading an older stable release in the same scope while preserving user configuration and replacing the application/runtime as a matching release payload. Uninstallation removes installer-owned application files, runtime and launch/registration entries but preserves user configuration. Installed applications retain startup settings at `~/.harbor/config.toml`; MSI does not ship the portable marker.

If an opposite-scope installation is visible to the current user, block the new installation with an actionable explanation requiring explicit uninstall before switching. Do not silently migrate scope, remove the old installation, copy configuration or leave a second registered install. Detection does not promise enumeration of every other account's private per-user installations. This visibility limit must be documented rather than described as a global machine-wide coexistence guarantee.

### R3. Full Portable ZIP

The ZIP includes an unambiguous marker beside `harbor.exe`; marker presence selects portable mode even for direct exe launch. Configuration is `config.toml` beside that exe, not relative to the working directory. MSI contains no such marker. Preserve startup-only parsing semantics and reuse existing settings recovery: missing/unreadable/invalid TOML uses defaults with existing diagnostics, without falling back to the user's home configuration.

Startup must neither create nor overwrite configuration, nor read/write the existing home configuration in portable mode. Retain usable defaults when no portable config exists; document how to place/edit the supplied repository example as a portable config. Moving the extracted directory, including marker, configuration and complete runtime, to another writable location carries settings and retains operation. The directory can be read-only for configuration loading; startup must not depend on writing config files.

The ZIP requires neither MSI installation nor installer registration or elevation to run. This contract isolates configuration, not arbitrary external shell behavior, shell histories, operating-system caches or every existing diagnostic/logging surface. It does not introduce settings hot reload or a new preferences editor.

### R4. Stable Version Identity and GitHub Release

Retain a local packaging path and add tag-triggered GitHub Actions publication. Accepted release tags are `vMAJOR.MINOR.PATCH` stable versions, with numeric version equal to the root Harbor Cargo package version. Fail on mismatched versions or a version that cannot be represented by the chosen MSI mechanism; do not truncate, silently remap or publish artifacts labeled as a different version.

For an accepted tag, build the tag's source and generate both formats from the same release-build payload and pinned dependency sources. Run applicable build, payload and packaging checks before attaching both verified outputs to the corresponding GitHub Release. A failed required check must not result in publication of an accepted release's artifact set. Use the same packaging contract locally and in CI, not two divergent file lists.

Initial artifacts are unsigned. Document that Windows may present untrusted-publisher/SmartScreen prompts; this refinement does not claim bypass or reputation. Signing credentials or an external signing service are not prerequisites to this selected delivery. Alpha/beta/rc tag mapping is not covered.

### R5. Consumer Evidence and Documentation

Document local prerequisites and commands, supported x64 artifact selection, MSI scopes/upgrade/uninstall/switching, full portable identification/configuration, complete runtime layout, stable tag/version rules and unsigned limitations. Documentation must distinguish current support from requirements until implemented and verified.

Record real Windows package acceptance for installation, direct/extracted launch and interactive shell operation, not just archive existence or successful MSI compilation. Capture revision/tag, version, build profile, Windows version, scope, commands/actions, expected and observed results, and PASS/FAIL/NOT RUN/BLOCKED. A dependency's OS baseline is not a newly approved Harbor OS-support matrix.

## Solution and Necessary Seams

Reuse the existing pinned runtime staging and TOML parsing behavior. Extend startup path selection to distinguish a marker-identified portable directory from installed/user configuration. Add package construction and stable-tag publication around the same release payload; no installer tool has been selected by this requirements contract.

| Seam | Connects | Expects | Provides |
| --- | --- | --- | --- |
| Release payload | Cargo release output → MSI/ZIP packaging | One versioned x64 binary, complete matching adjacent ConPTY runtime | Shared distribution contents; format-specific marker and registration |
| Startup mode | Executable location/marker → settings loading | Portable marker presence independent of cwd | exe-adjacent config or existing home config, never cross-mode fallback |
| MSI lifecycle | Windows Installer → files/entries/settings boundary | Explicit selected scope and visible opposite-scope state | Scoped install/upgrade/uninstall without config loss or silent scope migration |
| Version identity | Tag + root Cargo version → packaging/publication | Equal stable version, MSI-representable identity | Consistent MSI/ZIP/Release version or failure before publication |
| Release automation | Tagged source + local packaging contract → GitHub Release | Applicable checks and both successful package outputs | Two matching release assets, no successful publication after required-check failure |

## End-to-End Acceptance

| ID | Given / action | Observable outcome |
| --- | --- | --- |
| A1 | Local Windows checkout restores pinned runtime and invokes documented release packaging | Both x64 formats have matching version and common binary/runtime payload; no GitHub credentials needed |
| A2 | Install MSI as current user without elevation, launch Start Menu entry from unrelated cwd | Scoped files/entries exist; Harbor opens a real shell, accepts a command and displays its response using bundled ConPTY |
| A3 | Install MSI for all users with authorized elevation and launch as a normal user | Program Files install and machine entries work; settings still belong to launching user's home; interactive shell works |
| A4 | Upgrade older stable MSI to newer MSI in each same scope with distinguishable user config | Only the intended upgraded installation remains; new executable/runtime work; settings remain unchanged |
| A5 | Uninstall each scope, then explicitly install the opposite scope | Old owned files and entries are removed, user config remains, the explicitly selected new installation works |
| A6 | With a visible opposite-scope installation, attempt new install in both directions | Clear block requiring uninstall; no automatic removal/migration or second installation; limitation to visible installs documented |
| A7 | Extract ZIP without installing and directly launch exe from unrelated cwd, with different home and portable settings | Marker selects exe-adjacent config; observed settings match portable values and existing home config is untouched |
| A8 | Move entire portable directory to a path containing spaces/non-ASCII and launch directly | Configuration and full ConPTY runtime still resolve beside exe; shell command round trip works |
| A9 | Portable config is absent, unreadable or malformed while home config contains valid distinctive values | Existing defaults/diagnostics, no home-setting substitution and no created/overwritten config; read-only readable config loads without config writes |
| A10 | Accepted stable tag agrees with Cargo version and CI packaging/checks succeed | Corresponding GitHub Release receives both matching assets from the same build; downloaded MSI and ZIP satisfy installation/portable smoke cases |
| A11 | Tag/version mismatch, MSI-unrepresentable version or required package/runtime check failure | Actionable failure, no accepted artifact publication; missing ConPTY is not silently replaced by inbox behavior |

## Verification Guidance

Inspect format payloads and installed layouts, then exercise actual Windows launch and a real shell command/response from the packaged binaries. Use two stable version fixtures for upgrades and isolated accounts/environments for installation scopes. Compare user configuration before/after uninstall and portable launch. Configuration regression coverage must exercise consumer-visible path precedence, missing-file/default behavior and cwd independence, preserving existing parsing semantics.

Publication may be verified in an isolated release environment with recorded actual outputs; do not label a YAML inspection or local mock as a successful GitHub Release. This document and ticket generation have not performed these implementation checks.

## Decisions, Limitations and Exclusions

- User confirmed all material refinement behavior above. [ADR 0057](../adr/0057-windows-installation-and-portable-isolation.md) is Testing: implementation and local automated acceptance exist, with ordinary-user evidence still pending. [ADR 0058](../adr/0058-release-only-static-crt.md) records the completed release-only static CRT policy.
- Marker filename, installer authoring technology and product identifiers are reversible implementation details, not additional confirmed architectural constraints.
- No native ARM64/x86 application artifacts, pre-release version mapping, automatic scope migration, global enumeration of other users' installs, or signing are promised in this delivery.
- Full portable means portable configuration and install-free distribution, not a newly mandated no-trace sandbox for processes Harbor launches.
- No implementation plan, production code, QA execution or deployment is authorized by the selected refinement endpoint. Execution should consume this contract through `grimoire-loop`.

## Execution Addendum: Final Local Evidence and External Gates

The user explicitly invoked `grimoire-loop` after refinement. Scoped implementation and local automated verification are complete. This is not full release acceptance and does not authorize a commit, tag, push or public Release.

**Snapshot:** `eed3494` plus the local implementation and test changes; no actual `v0.1.0` Git tag was created. Cargo/application/MSI version: `0.1.0`. Build: locked x64 MSVC release with static CRT. Runtime: pinned ConPTY `1.24.260710001`. Windows reported `10.0.22621.0`; the actual cmd banner reported `10.0.22621.4317`. Windows PowerShell: `5.1.22621.4249`. WiX/UI: `5.0.2`. Local .NET SDK `8.0.426` was restored under ignored `target/`, without changing global tooling.

### Criterion-to-Evidence Check

| Criterion | Observed local evidence | Limit / verdict |
| --- | --- | --- |
| A1 | MSI and ZIP produced from one matching release payload; x64/ProductVersion, complete pinned runtime and import checks passed. All seven installed common payload files matched the release hashes; ZIP contents matched those same release bytes | PASS locally; not downloaded-asset evidence |
| A2/A3 | Real setup GUI completed scope → license → destination → Install → Finish in both scopes. Native MSI registration was context 2 for user and context 4 for machine. Directories were user Apps and native Program Files. Actual advertised Start Menu entries loaded home settings and adjacent ConPTY; targeted window keys executed `echo 638240` and produced a standalone response | Current token was elevated. Genuine non-elevated user install, UAC authorization and normal-user machine/portable launch remain NOT RUN |
| A4 | Both quiet and real-GUI same-scope `0.0.9 → 0.1.0` upgrades passed in both scopes, removed old registration and left the matching current payload. Quiet lifecycle checks preserved a distinctive temporary home config | PASS installer mechanics. The old MSI is an explicitly labeled version fixture using the same application binary, not a historical release or proof of historical application migration |
| A5 | Both scoped uninstalls removed owned application/runtime, shortcut and registration state while preserving home configuration. Explicit uninstall followed by opposite-scope installation passed | PASS local lifecycle; all owned test installations were removed |
| A6 | Both visible opposite-scope attempts failed with exit 1603 and the explicit uninstall-before-switching message, without removal or a second install | PASS locally; other accounts' private installs remain outside the selected visibility boundary |
| A7 | Actual extracted GUI launch from unrelated cwd selected read-only portable settings despite distinctive valid home settings; loaded its own adjacent ConPTY and left home/portable config hashes unchanged | PASS configuration isolation; no no-trace claim |
| A8 | The complete portable directory moved to a spaced/non-ASCII path and still loaded its config/runtime. Targeted Win32 key messages executed a real shell command and returned standalone `638240` in both original and moved locations | PASS automated GUI/shell round trip; no global keyboard injection used |
| A9 | Actual missing, malformed and ACL-denied portable config each started the default shell in a visible GUI, with bundled ConPTY and no distinctive home/portable fixture-shell substitution. Missing config stayed absent; existing config and home hashes stayed unchanged. The denied-read fixture really rejected reads and its ACL was restored. Read-only readable config also loaded | PASS tested boundaries; 17 deterministic configuration tests also passed |
| A10 | Source-reviewed stable-tag workflow uses the shared packager, read-only package job and dependent write-enabled draft/upload/publish job; both assets are required and published releases are not replaced | NOT RUN: no live tag, GitHub Actions Release or downloaded-asset acceptance |
| A11 | Real entry-point checks rejected mismatched/prerelease tags, all MSI numeric overflow boundaries, missing/mismatched ConPTY and the earlier dynamic-CRT input. ZIP path checks, embedded MSI cabinet streams and actual late-scope native costing regression passed | PASS local failure boundaries; remote failure/publication behavior remains unexecuted |

### Quality and Repaired Defects

- `cargo fmt --all -- --check` passed. `cargo clippy --locked --all-targets --all-features -- -D warnings` passed.
- `cargo test --locked --workspace` completed successfully, including the 17 config cases, all 16 live ConPTY resize cases, the renamed-dependency consumer fixture and doctests. The final run used process-local `CARGO_NET_OFFLINE=true` after a Windows Schannel download failure. The exact missing `yoke-derive 0.8.4` archive came from the official crate endpoint and matched its existing fixture lockfile SHA256 before entering the normal Cargo cache; versions/configuration were not changed to bypass the failure.
- The earlier 14 resize failures were producer-startup failures: matching END inside echoed inline commands did not establish output. Complete-line handshakes and RAII temporary `.ps1` producers now drive the same VT source through the already-working file mode. All original resize/input assertions remain; terminal production behavior was not changed.
- Real package runs repaired .NET Framework ZIP backslash paths/assembly loading, invalid MSI UI action placement and shortcut keypath ownership, and an external cabinet discarded with staging. Cabinets are now physically embedded and regression-checked.
- Full wizard navigation exposed an additional machine-scope defect missed by quiet installs: the initial per-user session had cached scope-redirected standard folders. The MSI now obtains the physical native Program Files root through a declarative 64-bit Windows registry search and refuses an unavailable machine root. Headless costing failed before the fix and passed afterward; rendered machine destination, real GUI installation and GUI upgrades subsequently passed. See [single-package authoring](https://learn.microsoft.com/en-us/windows/win32/msi/single-package-authoring) and [registry-search schema](https://docs.firegiant.com/wix/schema/wxs/registrysearch/).
- Earlier failed GUI-driving/timeout probes are not passing acceptance results. A later non-invasive visible-button/static-control observer completed both GUI upgrade routes. The owned timed-out transaction was allowed to finish and explicitly uninstalled; no service, account or system policy was changed.

### Final Artifact Binding

Both files remain under `target/dist/`:

| Artifact | SHA256 |
| --- | --- |
| `harbor-0.1.0-windows-x64.msi` | `8fae51a317092e305abb970925826297562accb12dd73f2dcdf22ec7f9eaa5b8` |
| `harbor-0.1.0-windows-x64-portable.zip` | `60725f1751f9b203d9819a9c6fbec741f069987601bf86d90b2fcd0bbac69c6b` |

Independent PE inspection found no direct external versioned VC runtime imports in `harbor.exe`, `conpty.dll` or any of the three OpenConsole hosts. The observed development test executable still imports `VCRUNTIME140.dll`. The release build respected encoded rustflags precedence and restored both caller compiler environment variables. This realizes ADR 0058 without adding app-local CRT DLLs or an external VC++ runtime prerequisite; it does not certify a clean-machine OS matrix.

Raw MSI logs, native sequence/costing inspection, command transcripts, GUI captures, desktop-token evidence and artifact/import JSON remain under ignored `target/windows-release-evidence/`. Throwaway automation scripts and extracted test payloads are not production deliverables.

### Remaining External Acceptance

All same-user Explorer tokens inspected were elevated at integrity level 12288. Additional shell/broker inspection found only high-integrity tokens or low-integrity AppContainers, not an accessible non-elevated medium-integrity non-AppContainer token. No accounts, credentials, UAC policy or global tool settings were changed to manufacture ordinary-user evidence.

Required next evidence is ordinary-user Windows installation/launch, authorized machine elevation followed by ordinary-user launch, and an authorized stable-tag Release plus downloaded MSI/ZIP smoke cases. Preserve the explicit old-version fixture limitation when assessing real historical upgrades. Until the required evidence exists, tickets remain unchanged and this spec is not Implemented. No qualifying project-knowledge update is pending: ADR 0057 is Testing and ADR 0058 is Completed.

