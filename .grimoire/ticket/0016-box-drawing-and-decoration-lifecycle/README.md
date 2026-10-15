# Box Drawing and Decoration Lifecycle

**Source:** [Spec 0022](../../spec/0022-box-drawing-and-decoration-lifecycle.md), [ADR 0056](../../adr/0056-terminal-owned-procedural-box-drawing.md), [Issue #193](https://github.com/wangjq4214/harbor/issues/193), and the user-approved refinement scope of 2026-10-11.

## Ticket Index

| ID | Surviving contract and evidence |
| --- | --- |
| T0001 — Procedural Box Drawing | [Spec 0022 R1/R2 and box-specific R4](../../spec/0022-box-drawing-and-decoration-lifecycle.md#requirements); [focused CPU/GPU evidence](../../../docs/procedural-box-drawing-evidence.md). The redundant execution ticket is retired. |
| T0002 — Decoration Metric Lifecycle | [Spec 0022 R3 and decoration/transition-specific R4](../../spec/0022-box-drawing-and-decoration-lifecycle.md#r3-decoration-metric-and-viewport-invariants), [Spec 0018](../../spec/0018-modern-sgr-decorations.md); [native changed-metrics and GPU evidence](../../../docs/decoration-metric-lifecycle-evidence.md). The redundant execution ticket is retired. |
| T0003 — Windows Integrated Acceptance | [Execution contract and closure limits](./T0003-windows-integrated-acceptance.md): named workload, combined native-resource replacement, actual cross-monitor evidence and fresh combined-tree gates. |

## Evidence Boundary

T0001/T0002 records establish focused implementation evidence, not integrated desktop acceptance. T0003 consumes their fixtures/results and must preserve source/width/copy, ordinary/shared text and modern SGR behavior while verifying the combined tree. Its user-confirmed Done status does not replace missing detailed evidence.

Native font smoke requires DirectWrite/wgpu replacement with demonstrably changed metrics, not user-configuration hot reload. Actual cross-monitor moves remain distinct from simulated scale changes. Required unavailable cases stay NOT RUN/BLOCKED; completion by deferral needs an explicit authorized scope decision. No broader support or Issue #193 closure is inferred.
