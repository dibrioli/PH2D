//! The **Line** stroke method — an editable on-canvas POLYLINE (PH2D shape extension; `docs/Painter/`).
//! Unlike Curve/Ellipse/Polygon (Bézier handles + alças), a Line is a chain of plain corner points with
//! NO handles. A submodule of [`super`] (`paint`), sharing `PaintState`'s private access + the canvas
//! save/restore/stamp helpers, like [`super::ellipse`].
//!
//! ## Interaction (point creation + editing + the whole-line transform gizmo; per-corner Fillet/Chamfer
//! are layered on next)
//!
//! Every gesture is ONE pointer press. There is no rubber-band between clicks: the shell only feeds Move
//! events while the button is held, so a point is *placed on Down* and the *same held drag* nudges it.
//!
//! 1. **Create** — press on EMPTY space drops a corner point AND grabs it, so dragging (before release)
//!    adjusts it live; Shift snaps the new segment's direction to 15°. A plain tap just places it. Points
//!    are only ever created in empty space — never on top of an existing point.
//! 2. **Move** — press ON an existing point and drag it past a small slop. A press that does NOT drag is a
//!    TAP: on the FIRST point (≥3) it CLOSES the loop, on the LAST it ENDS creation, on an interior one it
//!    just selects.
//! 3. **Edit** — the session persists after creation ends: corner points stay draggable, and a whole-line
//!    transform gizmo (bbox move / scale / rotate, identical to the Curve gizmo) frames the polyline.
//! 4. **Commit / cancel** — Enter/Apply bakes the polyline (one undo step); Apply & Keep bakes + keeps the
//!    editor open; Esc discards. Same flow as the other shapes.
//!
//! Transcendental-free (HR-5): only straight-segment fills + squared-distance hit-tests + the 15° snap's
//! precomputed unit-direction table.

use super::curve_gizmo::{self, TransformGizmo};
use super::line_corner::{self, CornerMod, LineCornerGizmo};
use super::line_offset;
use super::*;
use ph2d_painter_brush::Stroke;

/// A press is a TAP (not a drag) until the cursor leaves the grab point by this fraction of the grab
/// tolerance — the slop that separates a *tap* (close / end / select) from a *drag* (move the point).
const DRAG_SLOP_FRAC: f32 = 0.4;

/// In-progress Line session: the polyline `points` + draw-vs-edit phase. Held in [`PaintState::line`];
/// `None` when idle.
pub(super) struct LineEditor {
    points: Vec<[f32; 2]>,
    closed: bool,
    editing: bool,
    /// The point being dragged (grabbed on Down over an existing point, or the point just created), in
    /// either phase; `None` idle.
    grabbed: Option<usize>,
    /// The grabbed point was CREATED on this same Down (an empty-space press) — its commit is a new-point
    /// undo step regardless of whether it was dragged (vs an existing-point grab, where a tap resolves).
    grab_created: bool,
    /// Whether the current grab moved past the slop — distinguishes a DRAG (reshape) from a TAP (which,
    /// while drawing, closes on the first point / ends on the last / selects an interior one).
    grab_moved: bool,
    /// The pointer position at grab-down — the origin the slop is measured from.
    grab_origin: [f32; 2],
    /// The **whole-line transform** drag (bbox gizmo: move / scale / rotate), live only in the editing
    /// phase, Down→Up. Beats a point grab is N/A — a point grab wins the hit-test first; this is for a press
    /// on a gizmo handle in empty space. Reuses the Curve editor's gizmo (identical appearance + maths).
    gizmo: Option<curve_gizmo::GizmoGrab>,
    /// Per-corner Fillet / Chamfer, parallel to `points` (index-aligned). All `None` while drawing; edited
    /// in the editing phase via the corner gizmos. Persisted through undo.
    corner_mods: Vec<CornerMod>,
    /// The corner-gizmo handle being dragged: `(corner_index, is_fillet)`. `Some` Down→Up in the editing
    /// phase; `is_fillet` picks the circle (Fillet) vs the square (Chamfer) handle.
    corner_grab: Option<(usize, bool)>,
    /// The highlighted point (last clicked/dragged), shown selected in the overlay; transient (not undone).
    selected: Option<usize>,
    seed: u64,
}

/// A read-only snapshot of the Line editor for the shell's on-canvas overlay: the polyline vertices,
/// whether it is closed / in the editing phase, and the selected corner.
pub struct LineOverlay {
    /// Committed corner points (image-space px).
    pub points: Vec<[f32; 2]>,
    /// `true` once the loop is closed (last point clicked on the first).
    pub closed: bool,
    /// `true` in the editing phase (point-creation ended) — the shell then draws draggable corner dots.
    pub editing: bool,
    /// The highlighted (selected) corner index — the shell draws it emphasised. `None` = no selection.
    pub selected: Option<usize>,
    /// The whole-line **transform gizmo** (bbox move / scale / rotate) once creation has ended — `None`
    /// while still drawing, or when the line is too small to frame. Drawn like the Curve gizmo (identical).
    pub transform_gizmo: Option<TransformGizmo>,
    /// Per-corner **Fillet (circle) / Chamfer (square)** gizmos, one per real corner — the shell draws the
    /// two handles + accents the active mod. Empty while drawing (editing-phase chrome only).
    pub corner_gizmos: Vec<LineCornerGizmo>,
}

impl PainterTool {
    /// Route a canvas pointer through the Line editor (the painter calls this instead of the generic paint
    /// path while the method is [`StrokeMethod::Line`]). Returns `true` when consumed.
    pub(super) fn line_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.line_down(ev.pos),
            PointerPhase::Move => self.line_move(ev.pos),
            PointerPhase::Up => self.line_up(),
            PointerPhase::Hover => false,
        }
    }

    /// Forward the live Shift state before each Line pointer event: Shift enables the **15° direction
    /// snap** while dragging a new point (its incoming segment is guided to a 15°-graduated ray from the
    /// previous point). Out-of-band, like [`PainterTool::set_line_constrain`].
    pub fn set_line_snap(&mut self, on: bool) {
        self.paint.line_snap = on;
    }

    /// Apply the 15° snap to a grabbed point at index `g`: when armed (Shift) and a previous point anchors
    /// the segment, quantise the direction from `points[g-1]` to the nearest 15°; otherwise `pos` unchanged.
    fn line_snapped_grab(&self, g: usize, pos: [f32; 2]) -> [f32; 2] {
        if !self.paint.line_snap || g == 0 {
            return pos;
        }
        match self.paint.line.as_ref().and_then(|ed| ed.points.get(g - 1)) {
            Some(&anchor) => snap_to_15(anchor, pos),
            None => pos,
        }
    }

    /// Pointer-down: on an EXISTING point → grab it (drag to move, or tap-close/end/select on Up); on EMPTY
    /// space → create a new point AND grab it, so the same held drag nudges it. Starts the session when
    /// idle. Points are only ever CREATED in empty space, never on top of another point.
    fn line_down(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        if self.paint.line.is_none() {
            // Start a new polyline — the first point is grabbed so the same press can drag it into place.
            self.begin_shape_txn(); // before = no shape
            self.paint.drag_preview = None;
            self.reseed_preview_base();
            self.paint.shape_offset_base_px = 0.0;
            self.paint.shape_offset_norm = 0.5;
            let seed = self.paint.seed;
            self.paint.seed = self.paint.seed.wrapping_add(1);
            self.paint.line = Some(LineEditor {
                points: vec![pos],
                closed: false,
                editing: false,
                grabbed: Some(0),
                grab_created: true,
                grab_moved: false,
                grab_origin: pos,
                gizmo: None,
                corner_mods: vec![CornerMod::None],
                corner_grab: None,
                selected: Some(0),
                seed,
            });
            self.line_refill();
            return true; // committed on Up (one undo step for the new point)
        }
        let (hit, editing) = {
            let ed = self.paint.line.as_ref().expect("line present");
            (nearest_point(&ed.points, pos, tol), ed.editing)
        };
        if let Some(i) = hit {
            // On an existing point → grab it (drag to move; a tap resolves on Up). Bracket the potential
            // reshape; select the point. NEVER adds — creation is empty-space only.
            self.begin_shape_txn();
            let ed = self.paint.line.as_mut().expect("line present");
            ed.grabbed = Some(i);
            ed.grab_created = false;
            ed.grab_moved = false;
            ed.grab_origin = pos;
            ed.selected = Some(i);
            return true;
        }
        if editing {
            // Editing phase, off every point. Priority: a per-corner Fillet/Chamfer handle (most specific)
            // → the whole-line transform-gizmo handle → nothing (points are done — none are created here).
            let corner = self
                .paint
                .line
                .as_ref()
                .and_then(|ed| line_corner::hit_handle(&ed.points, ed.closed, pos, tol));
            if let Some((i, is_fillet)) = corner {
                self.begin_shape_txn();
                if let Some(ed) = self.paint.line.as_mut() {
                    ed.corner_grab = Some((i, is_fillet));
                }
                return true;
            }
            let grab = self
                .paint
                .line
                .as_ref()
                .and_then(|ed| curve_gizmo::grab(&ed.points, &[], pos, tol));
            if let Some(g) = grab {
                self.begin_shape_txn();
                if let Some(ed) = self.paint.line.as_mut() {
                    ed.gizmo = Some(g);
                }
            }
            return true;
        }
        // Empty space while drawing → create a new point AND grab it (drag adjusts it). Shift snaps the new
        // segment to 15° at creation AND during the drag; `grab_origin` stays the RAW cursor for the slop.
        let placed = match (
            self.paint.line_snap,
            self.paint.line.as_ref().and_then(|ed| ed.points.last()),
        ) {
            (true, Some(&anchor)) => snap_to_15(anchor, pos),
            _ => pos,
        };
        self.begin_shape_txn();
        {
            let ed = self.paint.line.as_mut().expect("line present");
            ed.points.push(placed);
            ed.corner_mods.push(CornerMod::None); // stays index-aligned with points
            let last = ed.points.len() - 1;
            ed.grabbed = Some(last);
            ed.grab_created = true;
            ed.grab_moved = false;
            ed.grab_origin = pos;
            ed.selected = Some(last);
        }
        self.line_refill();
        true // committed on Up
    }

    /// Pointer-move: drag the grabbed point. A freshly-created point follows the cursor from the first
    /// pixel (so you can set the angle live); an existing point only starts moving past the slop, so a
    /// jittery tap doesn't nudge it. Shift snaps the new segment to 15°.
    fn line_move(&mut self, pos: [f32; 2]) -> bool {
        // A per-corner Fillet/Chamfer drag takes priority: the drag distance sets that corner's amount.
        if let Some((i, is_fillet)) = self.paint.line.as_ref().and_then(|ed| ed.corner_grab) {
            let tol = self.paint.shape_grab_tol_px;
            let amt = {
                let ed = self.paint.line.as_ref().expect("line present");
                line_corner::amount_from_cursor(&ed.points, ed.closed, i, pos, tol)
            };
            if let Some(ed) = self.paint.line.as_mut()
                && i < ed.corner_mods.len()
            {
                ed.corner_mods[i] = if amt < line_corner::MIN_AMOUNT {
                    CornerMod::None
                } else if is_fillet {
                    CornerMod::Fillet(amt)
                } else {
                    CornerMod::Chamfer(amt)
                };
            }
            self.line_refill();
            return true;
        }
        // Whole-line transform gizmo takes priority (editing phase): re-map the grab snapshot every move.
        if self
            .paint
            .line
            .as_ref()
            .is_some_and(|ed| ed.gizmo.is_some())
        {
            if let Some(ed) = self.paint.line.as_mut() {
                let (p, _handles) =
                    curve_gizmo::apply(ed.gizmo.as_ref().expect("gizmo present"), pos);
                ed.points = p;
            }
            self.line_refill();
            return true;
        }
        let (grabbed, created, origin) = match self.paint.line.as_ref() {
            Some(ed) => (ed.grabbed, ed.grab_created, ed.grab_origin),
            None => return false,
        };
        let Some(g) = grabbed else {
            return true; // no grab (shouldn't happen: Down always grabs) — swallow the move
        };
        let slop = self.paint.shape_grab_tol_px * DRAG_SLOP_FRAC;
        let moved_far = dist2(pos, origin) > slop * slop;
        if !created && !moved_far {
            return true; // an existing-point press that hasn't left the slop yet stays a TAP
        }
        let target = self.line_snapped_grab(g, pos);
        if let Some(ed) = self.paint.line.as_mut()
            && g < ed.points.len()
        {
            ed.points[g] = target;
            if moved_far {
                ed.grab_moved = true;
            }
        }
        self.line_refill();
        true
    }

    /// Pointer-up: resolve the grab. A freshly-created point commits (one undo step) whether or not it was
    /// dragged. An existing-point DRAG records the reshape. An existing-point TAP while drawing: on the
    /// first point (≥3) → CLOSE; on the last → END creation; on an interior point → just select (no undo
    /// step). In the editing phase a tap is a plain select.
    fn line_up(&mut self) -> bool {
        // Release a per-corner Fillet/Chamfer drag → record it (self-drops if the amount is unchanged).
        if self
            .paint
            .line
            .as_ref()
            .is_some_and(|ed| ed.corner_grab.is_some())
        {
            if let Some(ed) = self.paint.line.as_mut() {
                ed.corner_grab = None;
            }
            self.commit_shape_txn();
            return true;
        }
        // Release a whole-line transform gizmo drag → record it (self-drops if the geometry is unchanged).
        if self
            .paint
            .line
            .as_ref()
            .is_some_and(|ed| ed.gizmo.is_some())
        {
            if let Some(ed) = self.paint.line.as_mut() {
                ed.gizmo = None;
            }
            self.commit_shape_txn();
            return true;
        }
        let (grabbed, created, moved, editing, n) = match self.paint.line.as_ref() {
            Some(ed) => (
                ed.grabbed,
                ed.grab_created,
                ed.grab_moved,
                ed.editing,
                ed.points.len(),
            ),
            None => return false,
        };
        let Some(i) = grabbed else {
            return true; // nothing grabbed — nothing to resolve
        };
        if created {
            // A new point was placed (and maybe dragged) → one undo step.
            if let Some(ed) = self.paint.line.as_mut() {
                ed.grabbed = None;
                ed.grab_created = false;
                ed.grab_moved = false;
            }
            self.commit_shape_txn();
            return true;
        }
        if moved {
            // Dragging an existing point reshaped the polyline → one undo step.
            if let Some(ed) = self.paint.line.as_mut() {
                ed.grabbed = None;
                ed.grab_moved = false;
            }
            self.commit_shape_txn();
            return true;
        }
        // A tap on an existing point (no drag).
        if !editing && i == 0 && n >= 3 {
            if let Some(ed) = self.paint.line.as_mut() {
                ed.closed = true;
                ed.editing = true;
                ed.grabbed = None;
            }
            self.line_refill();
            self.commit_shape_txn(); // close → one undo step
            return true;
        }
        if !editing && i == n - 1 && n >= 2 {
            if let Some(ed) = self.paint.line.as_mut() {
                ed.editing = true;
                ed.grabbed = None;
            }
            self.line_refill();
            self.commit_shape_txn(); // end creation → one undo step
            return true;
        }
        // Interior / editing tap → selection only (no geometry change): drop the speculative txn.
        if let Some(ed) = self.paint.line.as_mut() {
            ed.grabbed = None;
            ed.grab_moved = false;
        }
        self.paint.stroke_undo = None;
        true
    }

    /// Snapshot the Line editor for the shell overlay — always `Some` while a session is live.
    #[must_use]
    pub fn line_overlay(&self) -> Option<LineOverlay> {
        let ed = self.paint.line.as_ref()?;
        // The whole-line transform gizmo is editing-phase chrome (the drawing phase is still placing
        // points). `grabbed` / `rotating` come from the live drag so the shell can highlight + re-style.
        let transform_gizmo = if ed.editing {
            let (grabbed, rotating) = match &ed.gizmo {
                Some(g) => (Some(g.handle), curve_gizmo::is_rotate(g)),
                None => (None, false),
            };
            curve_gizmo::overlay(&ed.points, self.paint.shape_grab_tol_px, grabbed, rotating)
        } else {
            None
        };
        // Per-corner Fillet/Chamfer gizmos, editing-phase chrome only (drawing is still placing points).
        let corner_gizmos = if ed.editing {
            line_corner::gizmos(
                &ed.points,
                ed.closed,
                &ed.corner_mods,
                self.paint.shape_grab_tol_px,
            )
        } else {
            Vec::new()
        };
        Some(LineOverlay {
            points: ed.points.clone(),
            closed: ed.closed,
            editing: ed.editing,
            selected: ed.selected,
            transform_gizmo,
            corner_gizmos,
        })
    }

    // ── internals ────────────────────────────────────────────────────────────────────

    /// Re-fill the painted preview from the editor's polyline. A fresh `Stroke` per fill (deterministic for
    /// identical params). `< 2` path points paints nothing (a lone first point) — the on-canvas overlay
    /// still shows its dot.
    pub(super) fn line_refill(&mut self) {
        let off = self.shape_offset_px();
        let trim = self.paint.offset_trim;
        let (mut path, closed, seed) = {
            let Some(ed) = self.paint.line.as_ref() else {
                return;
            };
            // Expand each corner's Fillet/Chamfer into the rendered path (closes the loop when `closed`).
            let path = line_corner::expand(
                &ed.points,
                ed.closed,
                &ed.corner_mods,
                self.paint.shape_grab_tol_px,
            );
            (path, ed.closed, ed.seed)
        };
        // Perpendicular Offset (parallel polyline), then optionally Trim its self-intersections — same
        // Offset-card affordance + Trim checkbox the other shapes use (real-time via `refill_open_shape`).
        path = line_offset::offset_polyline(&path, closed, off);
        if trim && off != 0.0 {
            path = if closed {
                super::curve_trim::trim_self_intersections_closed(&path)
            } else {
                super::curve_trim::trim_self_intersections(&path)
            };
        }
        if path.len() < 2 {
            return;
        }
        let mut stroke = Stroke::new(self.paint.brush, self.paint.dynamics, seed);
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.fill_polyline_preview(&path, &mut dabs);
        self.stamp_drag_preview(&dabs);
        self.paint.dabs = dabs;
    }
}

impl LineEditor {
    /// Enter the editing phase (point-creation ended): stop any live grab. `pub(super)` so the commit
    /// paths in [`super::line_commit`] can drive it without reaching the private fields.
    pub(super) fn set_editing(&mut self) {
        self.editing = true;
        self.grabbed = None;
    }

    /// `true` once point-creation has ended (the editing phase) — accessor for [`super::line_commit`].
    pub(super) fn is_editing(&self) -> bool {
        self.editing
    }

    /// Number of committed corner points — accessor for [`super::line_commit`].
    pub(super) fn point_count(&self) -> usize {
        self.points.len()
    }

    /// Capture the polyline for the unified undo timeline (transient grab/selection reset on restore).
    pub(super) fn to_state(&self) -> crate::undo::LineState {
        crate::undo::LineState {
            points: self.points.clone(),
            closed: self.closed,
            editing: self.editing,
            corner_mods: self.corner_mods.iter().map(|m| m.to_wire()).collect(),
            seed: self.seed,
        }
    }

    /// Rebuild the editor from a restored snapshot (grab idle, no selection) — inverse of [`Self::to_state`].
    pub(super) fn from_state(s: crate::undo::LineState) -> Self {
        LineEditor {
            points: s.points,
            closed: s.closed,
            editing: s.editing,
            grabbed: None,
            grab_created: false,
            grab_moved: false,
            grab_origin: [0.0, 0.0],
            gizmo: None,
            corner_mods: s
                .corner_mods
                .into_iter()
                .map(CornerMod::from_wire)
                .collect(),
            corner_grab: None,
            selected: None,
            seed: s.seed,
        }
    }
}

/// `1/√2` = cos/sin of 45° — the named constant (clippy `approx_constant` rejects the 0.7071 literal).
const D45: f32 = std::f32::consts::FRAC_1_SQRT_2;

/// The 24 unit directions at 15° increments (`cos`, `sin` of `k·15°`, `k = 0..24`) — compile-time
/// constants so the snap is transcendental-free at runtime (HR-5). Used by [`snap_to_15`].
#[rustfmt::skip]
const DIRS_15: [[f32; 2]; 24] = [
    [1.0, 0.0],            [0.965_925_8, 0.258_819_04],   [0.866_025_4, 0.5],
    [D45, D45],            [0.5, 0.866_025_4],            [0.258_819_04, 0.965_925_8],
    [0.0, 1.0],           [-0.258_819_04, 0.965_925_8],  [-0.5, 0.866_025_4],
    [-D45, D45],          [-0.866_025_4, 0.5],           [-0.965_925_8, 0.258_819_04],
    [-1.0, 0.0],          [-0.965_925_8, -0.258_819_04], [-0.866_025_4, -0.5],
    [-D45, -D45],         [-0.5, -0.866_025_4],          [-0.258_819_04, -0.965_925_8],
    [0.0, -1.0],           [0.258_819_04, -0.965_925_8],  [0.5, -0.866_025_4],
    [D45, -D45],           [0.866_025_4, -0.5],           [0.965_925_8, -0.258_819_04],
];

/// Snap `cursor` onto the 15°-graduated ray from `anchor`: keep the distance, quantise the direction to
/// the nearest of the 24 [`DIRS_15`] by maximum dot product (no `atan2` — HR-5). A ~zero move is unchanged.
fn snap_to_15(anchor: [f32; 2], cursor: [f32; 2]) -> [f32; 2] {
    let v = [cursor[0] - anchor[0], cursor[1] - anchor[1]];
    let len = (v[0] * v[0] + v[1] * v[1]).sqrt();
    if len < 1e-4 {
        return cursor;
    }
    let mut best = DIRS_15[0];
    let mut best_dot = f32::MIN;
    for d in DIRS_15 {
        let dot = v[0] * d[0] + v[1] * d[1];
        if dot > best_dot {
            best_dot = dot;
            best = d;
        }
    }
    [anchor[0] + best[0] * len, anchor[1] + best[1] * len]
}

/// Index of the polyline point within `tol` px of `pos` (nearest wins), or `None`.
fn nearest_point(points: &[[f32; 2]], pos: [f32; 2], tol: f32) -> Option<usize> {
    let tol2 = tol * tol;
    let mut best = None;
    let mut bestd = tol2;
    for (i, p) in points.iter().enumerate() {
        let d = dist2(*p, pos);
        if d <= bestd {
            bestd = d;
            best = Some(i);
        }
    }
    best
}

/// Squared distance between two points (no `sqrt` — comparisons only).
fn dist2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}
