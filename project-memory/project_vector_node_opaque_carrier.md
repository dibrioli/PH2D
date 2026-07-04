---
name: project-vector-node-opaque-carrier
description: "Vector geometry nodes ride an opaque CookValue channel, NOT Stream/Column — and the spec pseudocode was aspirational"
metadata: 
  node_type: memory
  type: project
  originSessionId: d8d0ccb6-f213-4b49-9d9d-48b60304c8bd
---

Vector graph nodes (`ph2d-node-vector-*`) emit/consume a `VectorNetwork` through the
nodegraph's **type-erased opaque channel** `CookValue::Opaque(Arc<dyn Any+Send+Sync>)`
(ADR-0058-amendment-1, 2026-06-03), via the glue crate `ph2d-vector-graph`
(`VectorEvalExt::{emit_network, input_network}` + `VECTOR_PORT` = `Domain::Vector`/`Clock::Static`).
The substrate `ph2d-nodegraph` stays zero-deps/domain-agnostic; it never names `VectorNetwork`.
Params are `f32`-only — `kind`/`sides`/`turns` ride as discriminant/count via `param_as_count`.
The frozen caps `NodeOp=2/OpResolver=1/NodeManifest=8` are **untouched** (the carrier lives in
ungated cook internals: `EvalCtx`, `CookValue`, the `Domain` enum).

**Why:** `02_geometry_graph.md` §2.2.1 pseudocode (`eval(ctx) -> Result<VectorNetwork>`,
`Output::path`, `Param::enum_var`, `Clock::None`, `param_f32`) was **aspirational and never
built** — none of it existed in the real frozen contract (eval emits a `Stream` of
`Scalar/Vec2/Vec3`). The W3 handoff's "just wrap the generators as a node" framing missed that
the node *substrate* didn't exist. T3.2 nearly got built against a phantom API.

**How to apply:** for T3.3 (boolean) / T3.4 (offset) and the W4 fan-out, build against the real
substrate (read ADR-0058-amendment-1 + `ph2d-vector-graph` + `ph2d-node-vector-source`), not the
spec's illustrative code. General lesson (reinforces [[feedback-no-industrial-claims-without-verification]]
and [[feedback-tool-unit-green-integration-dead]]): a normative spec's code blocks can drift from
the frozen contract — diff against the actual types before implementing.
