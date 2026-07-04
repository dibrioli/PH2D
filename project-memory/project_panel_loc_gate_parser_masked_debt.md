---
name: project-panel-loc-gate-parser-masked-debt
description: panel LOC-cap gate parser miscounts comments → masks pre-existing oversized fns across panels; splitting a fn can trip false overruns
metadata: 
  node_type: memory
  type: project
  originSessionId: f72fd562-e393-4e8e-953f-5a10ba8f3d6c
---

The `architecture_panel_loc_cap` gate (crates/ph2d-editor-core/tests/) has a
naive brace-walk parser (`extract_fn_locs`) that tracks `'` / `"` / `{}` **even
inside `//` comments**. An odd apostrophe in prose ("doesn't", "sprite's")
leaves `in_char` stuck, so the walk overruns past the function's closing brace
into following functions.

**Consequence when splitting a panel fn:** a new helper with an odd-apostrophe
comment gets counted as hundreds of LOC (it swallows siblings) even though it is
really ~110. Discovered 2026-05-31 splitting `ph2d-panel-inspector/src/event.rs::apply_event_impl`
(real helpers <200 each, counted 371).

**Masked pre-existing debt:** the same bug HIDES real violations — when an
earlier sibling overruns it swallows the next fn so it's never counted. A
comment-aware parser fix (skip `//` and `/* */` in the walk) unmasks:
`ph2d-panel-inspector/src/paint.rs::paint_inspector` (~431), `.../sections/render_source.rs::paint_render_source_section` (~303),
`.../sections/transform.rs::paint_transform_section` (real 281 vs allowlisted 212),
`ph2d-panel-equalize-sizes/src/paint.rs::paint_body_sections` (~263),
`ph2d-panel-hierarchy/src/event.rs::apply_event` (~205). Also: `apply_event_impl`
was silently RED on origin/main (~493 LOC, no allowlist entry) — the gate had
been failing unnoticed (see [[feedback-full-gate-periodically]]).

**Why deferred (not fixed 2026-05-31):** the parser fix re-baselines EVERY
panel's allowance (all FN_OVERAGE_OK entries were tuned to the buggy counts) and
surfaces other owners' debt — foundational cross-panel churn, against
[[feedback-audit-scope-discipline]] to rush at a pause. Closed instead with an
honest allowlist entry for `apply_event_impl` (353). **Follow-up:** a deliberate
pass = comment-aware parser fix + re-baseline allowances + land the (ready)
per-cluster `try_*` split of apply_event_impl + split the surfaced fns.
