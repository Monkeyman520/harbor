# Color Emoji and Whole Sequence Presentation

**Source:** [Spec 0020](../../spec/0020-color-emoji-sequence-presentation.md), [ADR 0054](../../adr/0054-color-emoji-offscreen-presentation.md), [issue #181](https://github.com/wangjq4214/harbor/issues/181)
**Ticket folder:** `.grimoire/ticket/0015-color-emoji-sequence-presentation/`

## Overview

Deliver complete supported color/ZWJ emoji presentation without changing retained Unicode, model-assigned width, copy or neighboring cells. Use DirectWrite whole-unit shaping and reused Direct2D offscreen rasterization, then transfer tiles to a separate bounded lazy color atlas while retaining the ordinary R8 fast path. Uniformly downscale oversized artwork, position it inside assigned cells and finally clip. Unsupported presentation stays visible and explicitly distinct from complete sequence rendering.

The architecture and fit policy were confirmed in refinement. These tickets are Todo, not implementation or Windows support claims. They do not authorize general shaping, new width/copy semantics, shared-texture interoperation or resolution of Spec 0015's historical no-wrap discrepancy.

## Delivery Surfaces

- `harbor-text`: sequence requests, font-session/fallback identities, Windows DirectWrite/Direct2D resources, presentation classification and bounded native caches; preserve existing scalar consumers.
- `harbor-terminal::render`: bounded CPU/GPU color atlas, pixel/blend contract, fitted/clipped geometry, full/incremental updates and projection invalidation; preserve the GPU-independent engine.
- Actual host font/DPI/appearance transitions in the terminal renderer/application bridge; no new settings hot-reload feature is required.
- Focused backend/atlas/geometry/damage tests, Windows runtime and comparable performance evidence under `docs/verification/`, and truthful support documentation.

## Dependencies

| Producer | Consumer | Why consumer cannot complete first |
| --- | --- | --- |
| T0001 | T0002 | The renderer must consume the actual backend presentation/tile contract and unsupported outcomes to verify end-to-end composition and invalidation. Synthetic geometry tests can be prepared earlier. |
| T0002 | T0003 | Final Windows/clipboard and after-change performance acceptance must exercise the integrated renderer, caches and host transitions. Baseline capture and procedure preparation can start before integration. |

There is no blocking edge on #153's historical evidence gap or #176's remaining acceptance ticket: their existing text/width and engine/update contracts are invariants to preserve, not work newly assigned here.

## Coordination

T0001/T0002 coordinate request identity, raster bounds, pixel/color-space/alpha and generations under Spec 0020; sequence shared `harbor-text` edits and preserve widget scalar consumers. Capture the ordinary baseline before fast-path edits; rerun affected final evidence after changes. Establish actual font/OS/native capability rather than assuming support; new material choices return to refinement.

## Recommended Order

1. Capture the simple monospace baseline before changing its path; prepare Windows fixtures/procedures from T0003 without claiming that ticket complete.
2. T0001 delivers the independently testable native presentation API and bounded resources.
3. T0002 integrates that result with the terminal atlas, geometry and resource-change/damage paths as one coherent unit.
4. T0003 completes integrated Windows, clipboard, performance and quality-gate evidence.

Backend and synthetic renderer test preparation can overlap with coordinated interfaces. Shared files alone are merge risks, not additional semantic blockers. No separate global atlas refactor or implementation-plan stage is required by this set.

## Requirement Coverage

T0001 owns native whole-unit presentation, classification, composition and bounded resources (R1/R3/R4/R6). T0002 owns renderer text/width/neighbor authority, fit/fallback, atlas/invalidation and compatibility (R1-R6). T0003 verifies integrated Windows/performance outcomes; all three record focused tests and honest evidence under R7.

## Ticket Index

| Ticket | File | Outcome |
| --- | --- | --- |
| T0001 | [T0001-native-emoji-sequence-presentation.md](./T0001-native-emoji-sequence-presentation.md) | Complete sequence-aware native presentation and bounded raster/cache resources are independently verifiable. |
| T0002 | [T0002-terminal-color-atlas-and-invalidation.md](./T0002-terminal-color-atlas-and-invalidation.md) | Terminal color drawing, fit/fallback and dirty/resource transitions work together without changing model/copy semantics. |
| T0003 | [T0003-windows-and-performance-acceptance.md](./T0003-windows-and-performance-acceptance.md) | Named supported/unsupported Windows results, clipboard correctness, comparable costs and quality gates are honestly evidenced. |
