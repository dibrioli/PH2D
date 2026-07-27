---
name: feedback_green_composed_gates_can_hide_an_unproven_connector
description: "User says a feature has no effect but your gates are green — do NOT conclude \"perception\"; drive the REAL end-to-end gesture, a chain proven piece-by-piece can still have a dead connector"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 299e185e-eeb8-4b7e-8d3a-858f1557aa9f
  modified: 2026-07-26T14:16:34.515Z
---

When the user reports a feature "has no effect" / "muda nada" and your gates are
GREEN, do **not** conclude it's perception. Drive the **real end-to-end gesture**
(the actual UI click through the real dispatch), not a composition of piece-wise
gates.

**Why:** `Interp::Nearest` (2026-07-25) — I proved pick→`SetInterp` (gate on
`intents_for_pick`), `SetInterp`→`key.interp` (core), and `sample_keys`→midpoint
step (golden), then concluded end-to-end correctness *by composition* and told
Enio it was perception. It was **dead in the UI**: the connector between "menu row
clicked" and `intents_for_pick` was `timeline_segment::apply`, which enumerated the
leaf ids by hand (`Hold|Linear|Custom|Rove`) and forgot `Nearest` → the click hit
`return false`, no pick, no change. Every gate skipped that connector; the seam gate
enumerated the SAME incomplete list, green over broken product.

**How to apply:** a gate that tests the function *beneath* the UI does not prove
the UI. For any pick/click/menu feature, one gate MUST drive the real
`Click(id)`→handler and assert the observable effect (the pick parked / the intent
emitted). And derive "which rows are handled" from the TABLE the menu paints, never
a hand-list — that is [[feedback_a_condition_that_enumerates_its_readers_rots]].
Related: [[reference_topic_repro_discipline]] (cursor real · não-repro ≠ fix, here
inverted: a false *no-bug*).
