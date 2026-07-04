---
name: project_node_effect_pure_for_renderer_consumed
description: "Vector/geometry nodes consumed by the renderer MUST be Effect::Pure, not Stateful (spec pseudocode is wrong)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 9a06dd51-73b4-4fc3-abe4-4ed101c3ae24
---

Any `ph2d-node-*` whose output the renderer cooks (pull side) **must declare
`Effect::Pure`**, even when the spec pseudocode says `Effect::Stateful // cached
by hash`. In this substrate `Effect::Stateful` means "writes SimWorld, push
side" and such nodes are **never driven by the presentation `Cook`** (the
membrane, ADR-0030 — see `effect.rs`/`cook.rs`). A Stateful node feeding the
renderer would be invisible → the `source→node→render` smoke is silently dead.

The `Cook` already memoizes a `Pure` node by `(input revisions + param hash)` —
that IS the ADR-0058 §2.2.2 "cache by `(input_a_hash, input_b_hash, op)`". You do
NOT implement a separate content cache for W3; the per-instance memo covers it
(content-addressed LRU 50MB is a T5.2 follow-up).

**Why:** `vector.boolean` (T3.3) spec + handoff both said `Effect::Stateful`;
shipping that would have made the node dead in-product (passes unit tests, 100%
dead in the smoke — [[feedback_tool_unit_green_integration_dead]]).
**How to apply:** for a renderer-consumed node use `Effect::Pure` + `Clock::Static`
(mirror `vector.source`); rely on the Cook memo for caching. Build against the
real substrate, not the aspirational pseudocode ([[project_vector_node_opaque_carrier]]).
The exact-boolean engine uses the `linesweeper` crate (jneem/linesweeper,
kurbo-native, MIT/Apache, in-tree via peniko's kurbo 0.13) — the spec-named
engine in `16_referencias.md`. The `VectorNetwork⇄kurbo::BezPath` conversion +
the Linesweeper/kurbo-stroke calls live in the **shared satellite crate
`ph2d-vector-kurbo`** (the only crate that imports kurbo+linesweeper) — reuse it
for the rest of the geometry fan-out (offset done; outline-stroke / roughen next)
instead of re-implementing the conversion.
