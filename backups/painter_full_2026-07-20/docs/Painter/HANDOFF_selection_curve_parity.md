# HANDOFF — Selection curve = IDENTICAL to the stroke Shape curve system

> **STATUS 2026-07-03 — LANDED (local, pending Enio smoke).** Unified via the shared `CurveModel`
> (`crates/ph2d-tool-painter/src/tool/paint/curve_model.rs`): the pure editing ops (hit / insert / drag /
> delete / set-kind / select / from_fit) now live in ONE place, owned by BOTH the stroke `CurveEditor`
> (`self.paint.curve`, which embeds `model: CurveModel` and delegates) AND `SelectionShape::Freehand { model, u }`.
> Identical behaviour by construction. Delivered: Convert now **fits** to sparse anchors (all cases; Ellipse
> keeps its 4-arc, Polygon keeps sharp vertices); all 5 handle kinds + aligned/symmetric mirroring; right-click
> handle-kind menu (reuses `ContextMenuKind::CurvePointHandle`); click-to-insert; Delete; selected-anchor
> highlight + selected-only tangents in the overlay (ad-hoc all-handle drawing retired); isolation preserved
> (selection never touches `self.paint.curve`). Gates: **358 tool-painter lib tests green** (incl. 76 curve +
> the 3 new selection e2e seam tests driving the real events), shell + tool clippy clean, fmt, all files < 600
> LOC. NOT committed (main branch; awaiting Enio's manual smoke, then commit/ship). Original spec below.

---


> **Owner mandate (Enio, 2026-07-03):** "Por várias vezes eu disse que deveria ser como nas shapes do
> stroke." The selection **Convert to Curve** editor must be an *equivalent, identical* version of the
> stroke Shape **Curve** editor — same fit algorithm, same handle kinds, same right-click menu, same
> insert/delete/select, same look. The current selection curve is a **partial reimplementation** and is
> rejected. This handoff is the spec to make them one system.

---

## 0. TL;DR for the next agent

Do **NOT** keep polishing the parallel `selection_curve_gizmo.rs`. **Unify** the two systems by extracting the
stroke Curve editor's pure editing core into a shared model that BOTH the stroke slot (`self.paint.curve`) and
the selection curve use. Sharing **algorithms** (free fns in `curve_handle` / `curve_geom` / `curve_tangent` /
`ph2d_painter_brush::fit_curve`) is *explicitly allowed* — the ADR-0103 "isolation" rule only forbids reusing
the stroke's live **state slot** `self.paint.curve` for selection (so a selection edit can't corrupt a
half-open brush shape). Identical *code paths* are the goal.

Acceptance = a converted selection curve behaves pixel-for-gesture like a stroke Free-Hand/converted curve:
sparse anchors, all 5 handle kinds, right-click handle-kind menu, click-to-insert, Delete-to-remove, selection
highlight, aligned/symmetric tangent mirroring.

---

## 1. The reference system (stroke Curve) — files + APIs to reuse

All in `crates/ph2d-tool-painter/src/tool/paint/`:

- **`curve.rs`** — `CurveEditor { points, handles, kinds: Vec<HandleKind>, selected: Option<usize>, closed,
  editing, seed }` + `CurveOverlay`. `curve_select_point_at(pos, tol)`, `curve_delete_selected()`,
  `set_curve_handle_kind(wire)`, click-to-insert-on-curve, drag-point, drag-tangent.
- **`curve_handle.rs`** — `enum HandleKind { Free, Aligned, Symmetric, Vector, Auto }`, `from_wire`/`to_wire`,
  `is_manual()`, `mirror_mode()`, and `rebuild(points, kinds, handles, closed)` — recomputes the *derived*
  kinds (Auto/Vector) after any structural edit while preserving the *manual* ones (Free/Aligned/Symmetric).
- **`curve_tangent.rs`** — `tangent_hit(...)`, `mirror_tangent(...)`, `build_tangents(...)`: the tangent-handle
  hit-test + the aligned/symmetric opposite-handle follow.
- **`curve_geom.rs`** — `flatten_spine(points, handles, closed, out)` (also closed-seam), `simplify_curve(...)`,
  point insert/nearest/hit helpers.
- **`ph2d_painter_brush::fit_curve(&[[f32;2]], error)`** — the Schneider fit that turns a dense polyline into a
  sparse anchor+handle curve. **THIS is what keeps the point count low.**
- **`curve_commit.rs`** — `commit_open_shape()` / cancel (not needed for selection, which has no bake — the
  selection list IS the truth — but the pattern is here).

**Shell (stroke curve interaction) to mirror** in `shells/desktop/src/input_dispatch/`:
- `painter_canvas_input.rs::painter_curve_open_point_menu(px,py)` — right-click a control point → opens
  `ContextMenuKind::CurvePointHandle`. It calls `curve_select_point_at` then `open_context_menu`.
- `painter_canvas_input.rs::painter_curve_delete_selected_point()` (Delete key, `keyboard.rs:155`).
- `render_loop/mod.rs:1426` — the menu selection routes to `painter.set_curve_handle_kind(kind)`.
- Overlay: `render_loop/painter_bridge_curve_overlay.rs` (draws anchors/handles/selection the stroke way).

---

## 2. What the current selection curve does WRONG (reject + replace)

Current partial impl: `selection_curve_gizmo.rs` (+ `SelectionShape::Freehand { points, handles, u }`,
`selection_edit.rs::selection_convert_to_curve/selection_simplify_curve`, shell
`painter_bridge_selection_gizmos.rs`).

| Defect (Enio) | Cause | Fix |
|---|---|---|
| **"muitos pontos"** (dense curves) | `selection_convert_to_curve` copies raw lasso `points` verbatim, and the Raster/multi path uses `traced_curve_points` (a dense contour trace). No fit. | Convert must run `fit_curve` on the flattened outline for EVERY case (like stroke Free Hand). Ellipse can keep its exact 4-arc; everything else fits to sparse anchors. |
| **"handles não têm todos os tipos"** | `SelectionShape::Freehand` stores only `handles`, no per-anchor `kinds`. Drag is naive independent (Free only). | Add `kinds: Vec<HandleKind>` to the Freehand (or a full `CurveModel`). Reuse `curve_handle::rebuild` + `curve_tangent::mirror_tangent` so Aligned/Symmetric/Vector/Auto all work. |
| **"menu suspenso do botão direito não foi implementado"** | No right-click handler for the selection curve. | Add the selection analogue of `painter_curve_open_point_menu` → `ContextMenuKind::CurvePointHandle` → a new `set_selection_curve_handle_kind`. Reuse the SAME menu kind + dispatch. |
| **"algoritmo não é tão bom"** | `apply_curve_drag` moves points/handles by a raw delta; no insert-on-curve, no delete, no selection, no derived-kind rebuild. | Port the full stroke editing loop (select / insert / delete / drag-point / drag-tangent / rebuild). |

---

## 3. Recommended architecture — ONE curve core, two owners

**Extract** the stroke Curve editor's *pure* editing state + ops into a reusable struct (proposed
`curve_model.rs`):

```
pub(crate) struct CurveModel {
    pub points: Vec<[f32;2]>,
    pub handles: Vec<[[f32;2];2]>,
    pub kinds: Vec<HandleKind>,
    pub selected: Option<usize>,
    pub closed: bool,
}
impl CurveModel {
    fn from_fit(polyline: &[[f32;2]], closed: bool) -> Self;      // fit_curve + Aligned/Auto seed
    fn hit(&self, pos, tol) -> Option<CurveHit>;                  // anchor | in | out | on-curve-insert
    fn drag(&mut self, hit, from, to);                            // uses curve_tangent::mirror_tangent
    fn insert_at(&mut self, pos);                                 // curve_geom insert
    fn delete_selected(&mut self) -> bool;
    fn set_kind(&mut self, wire) -> bool;                         // then curve_handle::rebuild
    fn spine(&self) -> Vec<[f32;2]>;                              // curve_geom::flatten_spine
}
```

- **Stroke** (`CurveEditor` in `self.paint.curve`) wraps a `CurveModel` + its draw-phase/seed/editing extras.
- **Selection** stores a `CurveModel` inside `SelectionShape::Freehand` (replace the bare `points/handles/u`
  with the model + keep `u` only if the transform-box path still needs it — see §5). `recompose_selection_mask`
  rasterizes `model.spine()`.

Both call the identical `CurveModel` methods ⇒ identical behavior by construction. This is the padrão-ouro
move and what the owner has asked for repeatedly. If a full extraction is too large in one pass, the
*fallback* is to make the selection code call the existing stroke free-fns (`curve_handle::rebuild`,
`curve_tangent::*`, `curve_geom::*`, `fit_curve`) directly on the Freehand's `points/handles/kinds` — same
result, more duplication. Prefer the extraction.

---

## 4. Task list (in order)

1. **Data model.** Add `kinds: Vec<HandleKind>` (+ `selected`) to `SelectionShape::Freehand` (or embed
   `CurveModel`). Update every constructor/match: `selection_input.rs` (lasso), `selection_edit.rs`
   (convert/simplify), `selection_gizmo.rs` (transform of freehand carries kinds), `selection_shapes.rs`
   (rasterize unchanged — uses spine), snapshot/undo (`shape_snapshot`/`selection` snapshot include kinds).
   The **raw lasso** stays "not a converted curve" (see §5 switch).
2. **Convert = fit.** In `selection_convert_to_curve`, run `fit_curve` on the flattened outline for the
   Freehand/Raster/multi cases (Ellipse keeps its 4-arc). Seed kinds = `Aligned` (or `Auto`) like Free Hand.
   Result: sparse anchors. `selection_simplify_curve` already fits — align it to the same seed.
3. **Editing ops.** Replace `selection_curve_gizmo.rs` with the `CurveModel`-backed hit/drag/insert/delete/
   select/set-kind, reusing `curve_tangent`/`curve_handle`/`curve_geom`. Wire into `selection_gizmo_pointer`
   (Down = select+grab, Move = drag with mirror, Up = commit one undo entry; click-near-curve = insert).
4. **Right-click menu.** Add `App::painter_selection_curve_open_point_menu` (mirror of
   `painter_curve_open_point_menu`) → `ContextMenuKind::CurvePointHandle`; route its pick to a new
   `PainterTool::set_selection_curve_handle_kind(wire)`. Register the secondary-Down handler in
   `input_dispatch.rs` (the `PointerButton::Secondary, Down` arm) BEFORE the generic context menu, gated on
   "selection edit mode + converted curve + hit a control point".
5. **Delete key.** `keyboard.rs` — Delete removes the selected selection-curve anchor (mirror
   `painter_curve_delete_selected_point`).
6. **Overlay.** Draw anchors/handles/selection using the SAME visual language as
   `painter_bridge_curve_overlay.rs` (selected anchor highlighted, handle kinds' cues if any). Keep the
   per-gizmo fluorescent accent. Retire the ad-hoc drawing added to `painter_bridge_selection_gizmos.rs`.
7. **Tests.** Port the stroke curve tests' intent to selection: fit produces few anchors; each HandleKind
   drags correctly (aligned mirrors, symmetric reflects, vector/auto rebuild); insert/delete/select; the menu
   sets the kind. Delete the current `selection_curve_gizmo::tests` (they lock the inferior behavior) and the
   `converted_selection_curve_is_point_editable` test — rewrite against the unified behavior.

---

## 5. Design decisions to preserve (Chesterton's fences)

- **Isolation (ADR-0103 Am.2 v2):** selection editing must NOT read/write `self.paint.{curve,ellipse,line,
  polygon}` or `stroke_method`. Sharing pure fns / a stateless `CurveModel` is fine; sharing the live slot is
  not. Keep the selection curve's state inside the `selection_shapes` list.
- **Raw lasso vs converted:** the owner ratified "points editable ONLY after Convert to Curve" (earlier in this
  thread). So a raw lasso `Freehand` keeps the **transform box** gizmo; a **converted** one shows the point
  editor. Current switch = `is_converted_curve` (handles present). Preserve this two-mode behavior — with a
  `CurveModel` the switch becomes "does the Freehand carry a model?" (raw lasso = polyline only).
- **Selection is closed:** selection curves are always closed loops (`closed: true`) — the mask is the filled
  region. `flatten_spine(..., closed=true, ...)` and `fit_curve` closed handling apply.
- **No bake:** unlike the stroke curve (which bakes pixels on commit), the selection curve's "commit" is just
  one structural undo entry over the `selection_shapes` mutation + `recompose_selection_mask`. Don't route it
  through `commit_open_shape`.
- **HR-5 transcendental-free** in the tool geometry (selection is view-side but keep the discipline; the stroke
  curve math is already clean).

---

## 6. Current state of the tree (as of this handoff)

Landed locally (NOT pushed), this session, all green (351 tool-painter tests, clippy, LOC/fmt):
- `0f8c675b`,`ebc08dfd` Selection **Offset** (signed-distance grow/shrink + concentric protected/paint rings via
  Apply/Apply&Keep) + every ring line stays visible.
- `14699dfc` Enter = Apply in Selection Offset mode.
- `57b9241b` C&F no stray-point click-through + drag-fill returns to the prior tool.
- `38fea24f` C&F picker + Fill cursor seed from the brush colour (Brush=Fill=picker).
- `8548a1fb` Switching tool/method **bakes** an open shape (Apply), never erases it.
- `e0011a6a` **Converted selection curve point-editable** — THE PARTIAL IMPL THIS HANDOFF REPLACES.

`e0011a6a` is the starting point to unify: keep the routing/rendering plumbing, replace the model + ops with
the shared curve core.

---

## 7. Other OPEN queue items (unrelated to the curve work)

1. **Mask-panel buttons — no hover/press decoration** (`docs/Painter/HANDOFF_selection.md` item 4). Static
   investigation found the paint/populate/dispatch chain correct (paint reads `store.button_state` →
   `flat_button_surface`; ids registered as `InteractiveState::Button`; Move reaches `forward_to_hero →
   handle_pointer → update_hover` on the shared hero store+hit_index). Needs a RUNTIME repro: instrument
   `hit_index.hit(cursor)` under a Modifiers button with the Mask tool active — likely a hit-index z-order
   overlap from the pinned-at-top Mask section, or imperceptible token deltas.
2. **Stroke multi-shape** — multiple simultaneous editable stroke shapes (large architectural round, its own
   build + smoke; `HANDOFF_selection.md` item 5).
3. **Fill should be per-selection-region, not whole-selection** (Enio 2026-07-04). With SEVERAL disjoint
   selection areas, the C&F ColorDrop / Color Fill floods EVERY region; it must fill ONLY the region the
   colour was dragged onto. Cause: `selection_color_fill` (`selection_actions.rs`) and the ColorDrop
   (`fill.rs`) blend/flood over the WHOLE `selection_mask` regardless of the drop point. Fix direction:
   restrict to the connected COMPONENT of the selection mask containing the drop texel — flood-fill the
   selection coverage from the drop (4-connected, `≥128` inside, like `selection_raster`/`selection_trace`)
   to build a one-component mask, then fill against that. Needs the drop coordinate threaded into the fill
   path (the ColorDrop already delivers a canvas Down at the drop; `selection_color_fill` currently takes no
   point — add one). Watch the ring-stack + feathered-edge cases.

---

## 8. Acceptance criteria (definition of done for the curve work)

A converted selection curve is indistinguishable, gesture-for-gesture, from a stroke converted/Free-Hand curve:
- Convert yields a **sparse** anchor count (fit, not raw).
- All **5 handle kinds** work (Free/Aligned/Symmetric/Vector/Auto), with aligned/symmetric mirroring and
  Auto/Vector rebuild-on-edit.
- **Right-click a control point → the handle-kind menu** appears and sets the kind.
- **Click near the curve inserts** an anchor; **Delete** removes the selected; the selected anchor **highlights**.
- Same overlay look as the stroke curve editor (plus the selection's fluorescent accent).
- Raw lasso still shows the transform box; only converted curves are point-editable.
- All new/ported tests green; `./scripts/ship.sh` clean.
