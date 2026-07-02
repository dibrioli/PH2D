//! The **Line** stroke method — an editable on-canvas POLYLINE (PH2D shape extension; `docs/Painter/`).
//! Unlike Curve/Ellipse/Polygon (Bézier handles + alças), a Line is a chain of plain corner points with
//! NO handles. A submodule of [`super`] (`paint`), sharing `PaintState`'s private access + the canvas
//! save/restore/stamp helpers, like [`super::ellipse`].
//!
//! ## Interaction (this stage: point creation + editing; Fillet/Chamfer + 15° snap + the finish
//! transform gizmo are layered on next)
//!
//! 1. **Draw** — each click drops a corner point; every segment continues from the previous point. A
//!    rubber-band segment previews the next point live. End point-creation by Esc / right-click / clicking
//!    the LAST point; CLOSE by clicking the FIRST point.
//! 2. **Edit** — the session persists: corner points are draggable.
//! 3. **Commit / cancel** — Enter/Apply bakes the polyline (one undo step); Apply & Keep bakes + keeps the
//!    editor open; Esc discards. Same flow as the other shapes.
//!
//! Transcendental-free (HR-5): only straight-segment fills + squared-distance hit-tests.

use super::*;
use ph2d_painter_brush::Stroke;

/// In-progress Line session: the polyline `points` + draw-vs-edit phase. Held in [`PaintState::line`];
/// `None` when idle. `draft` is the live rubber-band cursor while drawing (not a committed point).
pub(super) struct LineEditor {
    points: Vec<[f32; 2]>,
    draft: Option<[f32; 2]>,
    closed: bool,
    editing: bool,
    /// The point being dragged (grabbed on Down over an existing point), in either phase; `None` idle.
    grabbed: Option<usize>,
    /// Whether the current grab actually moved — distinguishes a DRAG (reshape) from a TAP (which, while
    /// drawing, closes on the first point / ends on the last / selects an interior one).
    grab_moved: bool,
    /// The highlighted point (last clicked/dragged), shown selected in the overlay; transient (not undone).
    selected: Option<usize>,
    seed: u64,
}

/// A read-only snapshot of the Line editor for the shell's on-canvas overlay: the polyline vertices, the
/// live draft point (while drawing), and whether it is closed / in the editing phase.
pub struct LineOverlay {
    /// Committed corner points (image-space px).
    pub points: Vec<[f32; 2]>,
    /// The live rubber-band cursor while drawing (`None` once editing / before the first move).
    pub draft: Option<[f32; 2]>,
    /// `true` once the loop is closed (last point clicked on the first).
    pub closed: bool,
    /// `true` in the editing phase (point-creation ended) — the shell then draws draggable corner dots.
    pub editing: bool,
    /// The highlighted (selected) corner index — the shell draws it emphasised. `None` = no selection.
    pub selected: Option<usize>,
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
    /// snap** while drawing (each new segment is guided to a 15°-graduated ray from the previous point).
    /// Out-of-band, like [`PainterTool::set_line_constrain`] / [`PainterTool::set_uniform_scale`].
    pub fn set_line_snap(&mut self, on: bool) {
        self.paint.line_snap = on;
    }

    /// Apply the 15° snap to `pos` when it is armed (Shift) and a previous point anchors the segment —
    /// otherwise `pos` unchanged. The anchor is the last committed corner.
    fn line_snapped(&self, pos: [f32; 2]) -> [f32; 2] {
        if !self.paint.line_snap {
            return pos;
        }
        match self.paint.line.as_ref().and_then(|ed| ed.points.last()) {
            Some(&anchor) => snap_to_15(anchor, pos),
            None => pos,
        }
    }

    /// Pointer-down: start a session (first point) when idle; over an EXISTING point → grab it (to move,
    /// or tap-close/end/select on Up); on EMPTY space while drawing → drop a new point. Points are only
    /// CREATED in empty space, never on top of another point.
    fn line_down(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        if self.paint.line.is_none() {
            // Start a new polyline — the FIRST point is its own undo step (point-by-point undo/redo).
            self.begin_shape_txn(); // before = no shape
            self.paint.drag_preview = None;
            self.reseed_preview_base();
            self.paint.shape_offset_base_px = 0.0;
            self.paint.shape_offset_norm = 0.5;
            let seed = self.paint.seed;
            self.paint.seed = self.paint.seed.wrapping_add(1);
            self.paint.line = Some(LineEditor {
                points: vec![pos],
                draft: None,
                closed: false,
                editing: false,
                grabbed: None,
                grab_moved: false,
                selected: Some(0),
                seed,
            });
            self.line_refill();
            self.commit_shape_txn(); // record: no shape → first point
            return true;
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
            ed.grab_moved = false;
            ed.selected = Some(i);
            return true;
        }
        if editing {
            return true; // editing phase: empty space adds nothing (points are done)
        }
        // Empty space while drawing → drop a new point (one undo step). 15° snap (Shift) applies.
        let add_pos = self.line_snapped(pos);
        self.begin_shape_txn();
        {
            let ed = self.paint.line.as_mut().expect("line present");
            ed.points.push(add_pos);
            ed.selected = Some(ed.points.len() - 1);
            ed.draft = None;
        }
        self.line_refill();
        self.commit_shape_txn();
        true
    }

    /// Pointer-move: drag the grabbed point (either phase); otherwise, while drawing, rubber-band the next
    /// point (the draft).
    fn line_move(&mut self, pos: [f32; 2]) -> bool {
        let (editing, grabbed) = match self.paint.line.as_ref() {
            Some(ed) => (ed.editing, ed.grabbed),
            None => return false,
        };
        if let Some(g) = grabbed {
            if let Some(ed) = self.paint.line.as_mut()
                && g < ed.points.len()
            {
                ed.points[g] = pos; // dragging a point is un-snapped (15° snap is for NEW segments)
                ed.grab_moved = true;
            }
            self.line_refill();
            return true;
        }
        if !editing {
            let snapped = self.line_snapped(pos);
            self.paint.line.as_mut().expect("line present").draft = Some(snapped);
            self.line_refill();
        }
        true
    }

    /// Pointer-up: resolve a grab. A DRAG (moved) records the reshape. A TAP (no move) while drawing:
    /// on the first point (≥3) → CLOSE; on the last → END creation; on an interior point → just select
    /// (no undo step). In the editing phase a tap is a plain select.
    fn line_up(&mut self) -> bool {
        let (grabbed, moved, editing, n) = match self.paint.line.as_ref() {
            Some(ed) => (ed.grabbed, ed.grab_moved, ed.editing, ed.points.len()),
            None => return false,
        };
        let Some(i) = grabbed else {
            return true; // no grab (a point was just added on Down) — nothing to resolve
        };
        if moved {
            if let Some(ed) = self.paint.line.as_mut() {
                ed.grabbed = None;
                ed.grab_moved = false;
            }
            self.commit_shape_txn(); // the drag reshaped the polyline → one undo step
            return true;
        }
        // A tap (no drag).
        if !editing && i == 0 && n >= 3 {
            if let Some(ed) = self.paint.line.as_mut() {
                ed.closed = true;
                ed.editing = true;
                ed.draft = None;
                ed.grabbed = None;
            }
            self.line_refill();
            self.commit_shape_txn(); // close → one undo step
            return true;
        }
        if !editing && i == n - 1 && n >= 2 {
            if let Some(ed) = self.paint.line.as_mut() {
                ed.editing = true;
                ed.draft = None;
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

    /// Commit the polyline (**Enter / Apply**): drop the rubber-band draft, bake the painted line, close
    /// the session, one undo step. Works whether still drawing (≥2 points) or editing. No-op otherwise.
    pub fn line_commit(&mut self) -> bool {
        if !self.line_paintable() {
            return false;
        }
        self.line_finish_draft();
        self.flush_shape_txn();
        let before = self.capture_shape_model();
        self.paint.line = None;
        self.commit_drag_preview();
        self.paint.shape_offset_base_px = 0.0;
        self.paint.shape_offset_norm = 0.5;
        let after = self.capture_shape_model();
        self.undo.record_structural(before, after);
        true
    }

    /// Commit the polyline (**Apply & Keep**) but keep the editor open for further reshape — one undo step
    /// whose `after` keeps the editor over the baked pixels (mirrors [`PainterTool::ellipse_commit_keep`]).
    pub fn line_commit_keep(&mut self) -> bool {
        if !self.line_paintable() {
            return false;
        }
        self.line_finish_draft();
        self.flush_shape_txn();
        let before = self.capture_shape_model();
        self.commit_drag_preview();
        let after = self.capture_shape_model();
        self.undo.record_structural(before, after);
        self.paint.drag_preview = None;
        true
    }

    /// Cancel the polyline (**Esc**): revert the painted preview + discard the session, no undo entry.
    pub fn line_cancel(&mut self) -> bool {
        if self.paint.line.is_none() {
            return false;
        }
        self.paint.line = None;
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        self.paint.stroke_undo = None;
        self.paint.shape_offset_base_px = 0.0;
        self.paint.shape_offset_norm = 0.5;
        true
    }

    /// Drop the Line session without touching pixels — teardown where the canvas is replaced/cleared
    /// (fresh source / tool deactivate), so a restore would read a stale buffer.
    pub(crate) fn line_discard(&mut self) {
        self.paint.line = None;
    }

    /// Snapshot the Line editor for the shell overlay — always `Some` while a session is live.
    #[must_use]
    pub fn line_overlay(&self) -> Option<LineOverlay> {
        let ed = self.paint.line.as_ref()?;
        Some(LineOverlay {
            points: ed.points.clone(),
            draft: if ed.editing { None } else { ed.draft },
            closed: ed.closed,
            editing: ed.editing,
            selected: ed.selected,
        })
    }

    // ── internals ────────────────────────────────────────────────────────────────────

    /// Whether the editor holds a paintable line (≥ 2 committed points), so Enter/Apply act.
    fn line_paintable(&self) -> bool {
        self.paint
            .line
            .as_ref()
            .is_some_and(|ed| ed.points.len() >= 2)
    }

    /// Enter/Apply on a still-drawing line ends point-creation: drop the rubber-band draft, enter the
    /// editing phase, and re-fill so the baked preview is the committed points only (no draft segment).
    fn line_finish_draft(&mut self) {
        if let Some(ed) = self.paint.line.as_mut()
            && !ed.editing
        {
            ed.editing = true;
            ed.draft = None;
        }
        self.line_refill();
    }

    /// Re-fill the painted preview from the editor's polyline (committed points + the draft while drawing).
    /// A fresh `Stroke` per fill (deterministic for identical params). `< 2` path points paints nothing
    /// (a lone first point) — the on-canvas overlay still shows its dot.
    pub(super) fn line_refill(&mut self) {
        let (mut path, closed, seed) = {
            let Some(ed) = self.paint.line.as_ref() else {
                return;
            };
            let mut path = ed.points.clone();
            if !ed.editing
                && let Some(d) = ed.draft
            {
                path.push(d);
            }
            (path, ed.closed, ed.seed)
        };
        // Close the loop by re-appending the first point — a plain extra segment for the polyline fill.
        if closed && path.len() >= 3 {
            path.push(path[0]);
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
    /// Capture the polyline for the unified undo timeline (transient `grabbed`/`draft` reset on restore).
    pub(super) fn to_state(&self) -> crate::undo::LineState {
        crate::undo::LineState {
            points: self.points.clone(),
            closed: self.closed,
            editing: self.editing,
            seed: self.seed,
        }
    }

    /// Rebuild the editor from a restored snapshot (grab idle, no draft) — inverse of [`Self::to_state`].
    pub(super) fn from_state(s: crate::undo::LineState) -> Self {
        LineEditor {
            points: s.points,
            draft: None,
            closed: s.closed,
            editing: s.editing,
            grabbed: None,
            grab_moved: false,
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
