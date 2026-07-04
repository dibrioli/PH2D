---
name: feedback-gizmo-verify-hit-target-before-transform-math
description: "Gizmo/manipulator \"transforms the wrong thing\" — log which target the grab RESOLVED to before auditing the transform math; it's usually the hit, not the math"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: c60d3dd2-e693-4dcc-94cb-e5f0a8d0153a
---

Multi-select gizmo rotation "rotated each sprite wrong / used another sprite's center" (2026-06-08). I burned ~10 round-trips auditing the rotation MATH (atan2 seam, in-place vs orbit, parenting, world↔local) — and the math was **correct the whole time**. The real bug was in the **hit-test**: `keyed_handle_id` mixed sprite bits into the canonical handle id with **XOR** (`canonical ^ bits ^ GOLDEN`). XOR is linear so GOLDEN cancels; two handles collide whenever `canonical_a ^ canonical_b == bits_a ^ bits_b`. With CONSECUTIVE rotate-corner ids (960..963) and CONSECUTIVE entity bits, 12 of 20 handles collided → `gizmo_hit_map.insert` overwrote → the grab resolved to whichever sprite painted LAST. Fix: `canonical ^ bits.wrapping_mul(GOLDEN)` (non-linear → 0 collisions). Confirmed by a multi-agent audit + numeric collision count.

**Why:** one log at PointerDown — "click_world, resolved target/entity_bits, and each selected sprite's distFromClick" — caught it on the FIRST try and proved the grabbed entity ≠ the nearest sprite. The transform logs only confirmed the (correct) math, round after round. Bench/math-green ≠ the right thing was selected. (Mirror of [[feedback-tool-unit-green-integration-dead]] and [[feedback-visual-bug-debug]].)

**How to apply:** For ANY gizmo / manipulator / widget that "transforms the wrong target," FIRST instrument the dispatch — log the resolved `entity_bits`/target (and the nearest-candidate distances) at grab — BEFORE touching the transform/apply math. Two reusable gotchas: (1) XOR-based id scramblers are linear and collide for consecutive `(id, bits)` pairs → use **multiplicative** mixing. (2) Two parallel `Vec`s indexed by position (here `extra_selection` bits vs `extra_views` positions) drift silently → store `(bits, payload)` pairs so identity travels with the payload.
