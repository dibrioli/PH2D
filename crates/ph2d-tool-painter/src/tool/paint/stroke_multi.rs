//! **Stroke MULTI-SHAPE** — multiple simultaneously-editable stroke shapes on one canvas (Enio 2026-07-04),
//! mirroring the Selection subsystem: the shapes are the parametric source of truth and the canvas PIXELS
//! are a DERIVED cache, re-stamped from ALL shapes onto one pristine baseline every recompose (the invariant
//! that makes overlapping vector edits clean: nothing is baked until the final Apply).
//!
//! To keep the ~100 `self.paint.{curve,ellipse,polygon,line}` sites untouched, exactly ONE shape is the
//! **active** live editor (the existing single slots) and the rest are **parked** here as plain geometry
//! ([`StrokeShape`]) + their Operation ([`StrokeOp`]). Switching active serializes the live editor into the
//! parked set and deserializes the target back out (cheap — geometry only). Undo clones the whole set.

use super::*;

/// The boolean Operation a stroke shape participates in. Mirrors the Selection ops, except the former
/// "New" (replace) is replaced by **Overlay** — no boolean at all, the shape simply paints independently.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum StrokeOp {
    /// No boolean: the shape paints on its own; offset acts on it individually.
    #[default]
    Overlay,
    /// Boolean UNION with the overlapping Add/Remove shapes; offset acts on the combined region.
    Add,
    /// Boolean SUBTRACT from the overlapping Add/Remove shapes; offset acts on the combined region.
    Remove,
}

impl StrokeOp {
    /// Next op in the +/−/o cycle (the gizmo's centre square quick-click walks this): Overlay→Add→Remove→…
    pub(crate) fn cycle(self) -> Self {
        match self {
            StrokeOp::Overlay => StrokeOp::Add,
            StrokeOp::Add => StrokeOp::Remove,
            StrokeOp::Remove => StrokeOp::Overlay,
        }
    }

    /// The single-char glyph drawn in the gizmo's type square: `+` Add · `-` Remove · `o` Overlay.
    pub(crate) fn glyph(self) -> &'static str {
        match self {
            StrokeOp::Overlay => "o",
            StrokeOp::Add => "+",
            StrokeOp::Remove => "-",
        }
    }

    /// Wire `u8` for undo/panel round-trips (`0`=Overlay `1`=Add `2`=Remove).
    pub(crate) fn to_wire(self) -> u8 {
        match self {
            StrokeOp::Overlay => 0,
            StrokeOp::Add => 1,
            StrokeOp::Remove => 2,
        }
    }

    /// Inverse of [`Self::to_wire`] (any unknown value → Overlay, the neutral default).
    pub(crate) fn from_wire(w: u8) -> Self {
        match w {
            1 => StrokeOp::Add,
            2 => StrokeOp::Remove,
            _ => StrokeOp::Overlay,
        }
    }
}

/// A PARKED (inactive but still-editable) stroke shape: its frozen geometry + Operation. The live editor
/// lives in `PaintState::{curve,ellipse,polygon,line}`; everything else is a `StrokeShape` in the list.
#[derive(Clone, Debug)]
pub(crate) struct StrokeShape {
    pub state: crate::undo::ShapeEditState,
    pub op: StrokeOp,
}

/// A multi-shape gizmo **op badge** for the shell to draw: the centre square shows the `+`/`−`/`o` glyph of
/// the shape's Operation, and (for a PARKED shape) a faint AABB frame marks it as still-selectable. Image px.
#[derive(Clone, Debug)]
pub struct StrokeOpBadge {
    /// Centre of the shape (where the type-square + glyph sit).
    pub center: [f32; 2],
    /// AABB `[minx,miny,maxx,maxy]` of the shape (drawn as a faint frame for parked shapes).
    pub bbox: [f32; 4],
    /// The Operation glyph: `"+"` Add · `"-"` Remove · `"o"` Overlay.
    pub glyph: &'static str,
    /// `true` for the live editor's shape (its full gizmo is drawn separately — only the glyph is added);
    /// `false` for a parked shape (draw the frame + glyph so it reads as an editable shape).
    pub active: bool,
}

impl PainterTool {
    /// `true` when at least one shape is parked (i.e. more than one shape is currently on canvas).
    pub(crate) fn has_parked_shapes(&self) -> bool {
        !self.paint.parked_shapes.is_empty()
    }

    /// Set the multi-shape **Operation** mode a NEW shape is created with (panel OPERATION card): `0`=Overlay
    /// `1`=Add `2`=Remove. Also retags the ACTIVE shape's op so the change is visible immediately on its
    /// gizmo (matches the Selection Operation feel) + recomposes so any boolean result updates live.
    pub fn set_stroke_op_mode(&mut self, wire: u8) {
        let op = StrokeOp::from_wire(wire);
        self.paint.stroke_op_mode = op;
        if self.is_editing_shape() {
            self.paint.active_op = op;
            self.refill_open_shape(); // the combined boolean result may change → recompose the pixels
        }
    }

    /// The current panel Operation mode as a wire `u8` (for the snapshot / tests).
    pub fn stroke_op_mode(&self) -> u8 {
        self.paint.stroke_op_mode.to_wire()
    }

    /// Serialize the ACTIVE editor (Curve/Ellipse/Polygon/Line) into the parked set with the active op,
    /// then clear the live slots — WITHOUT baking pixels (the shape keeps painting via `recompose`). Bakes
    /// any live Offset into the geometry first so the parked copy is self-contained. No-op when idle.
    pub(crate) fn park_active_shape(&mut self) {
        // Do NOT bake the Offset and do NOT reset the slider: the one Offset slider is GLOBAL — every shape
        // (active + parked) is offset by the live value in `parked_shape_dabs`, so parking keeps the raw
        // geometry (the base) and leaves the slider where it is (Enio 2026-07-04: offset acts on all at once).
        let Some(state) = self.capture_shape() else {
            return;
        };
        self.paint.parked_shapes.push(StrokeShape {
            state: *state,
            op: self.paint.active_op,
        });
        self.paint.curve = None;
        self.paint.ellipse = None;
        self.paint.polygon = None;
        self.paint.line = None;
        // Re-stamp the parked shapes onto the pristine canvas so they stay visible with no active editor.
        self.restamp_shapes_preview(&[]);
    }

    /// Pull parked shape `i` back into the live editor (its op becomes the active op), parking whatever is
    /// currently active first. `true` when `i` was in range. The stroke method follows the restored shape.
    pub(crate) fn activate_parked_shape(&mut self, i: usize) -> bool {
        if i >= self.paint.parked_shapes.len() {
            return false;
        }
        // Park the current active shape first (if any), so switching never drops it.
        let had_active = self.capture_shape().is_some();
        if had_active {
            self.park_active_shape();
            // `park_active_shape` pushed the previously-active shape at the end; `i` still points at the
            // originally-targeted entry (it was before the push).
        }
        let StrokeShape { state, op } = self.paint.parked_shapes.remove(i);
        self.paint.active_op = op;
        // Rebuild the live editor from the geometry + point the tool at its method.
        self.install_shape_editor(state);
        // Re-stamp active + remaining parked from the pristine baseline.
        self.refill_open_shape();
        true
    }

    /// Rebuild the live editor slot + stroke method from a captured [`crate::undo::ShapeEditState`]. Shared
    /// by [`Self::activate_parked_shape`] and the undo restore path.
    pub(crate) fn install_shape_editor(&mut self, state: crate::undo::ShapeEditState) {
        use crate::undo::ShapeEditState as S;
        self.paint.curve = None;
        self.paint.ellipse = None;
        self.paint.polygon = None;
        self.paint.line = None;
        match state {
            S::Curve(s) => {
                self.paint.curve = Some(curve::CurveEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Curve;
            }
            S::Ellipse(s) => {
                self.paint.ellipse = Some(ellipse::EllipseEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Ellipse;
            }
            S::Polygon(s) => {
                self.paint.polygon = Some(polygon::PolygonEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Polygon;
            }
            S::Line(s) => {
                self.paint.line = Some(line::LineEditor::from_state(s));
                self.paint.brush.stroke_method = StrokeMethod::Line;
            }
        }
    }

    /// Stamp the ACTIVE shape's `own` dabs PLUS every parked shape's dabs as one drag-preview batch — the
    /// single seam the four `*_refill` paths funnel through. With no parked shapes `all == own`, so the
    /// behaviour is byte-identical to the former single-shape `stamp_drag_preview(own)`.
    pub(super) fn restamp_shapes_preview(&mut self, own: &[Dab]) {
        if self.paint.parked_shapes.is_empty() {
            self.stamp_drag_preview(own);
            return;
        }
        let mut all = own.to_vec();
        self.append_parked_dabs(&mut all);
        self.stamp_drag_preview(&all);
    }

    /// Append the dabs of every parked shape (their frozen geometry) to `out`.
    fn append_parked_dabs(&mut self, out: &mut Vec<Dab>) {
        let states: Vec<crate::undo::ShapeEditState> = self
            .paint
            .parked_shapes
            .iter()
            .map(|s| s.state.clone())
            .collect();
        let mut scratch = Vec::new();
        for st in &states {
            scratch.clear();
            self.parked_shape_dabs(st, &mut scratch);
            out.extend_from_slice(&scratch);
        }
    }

    /// Fill `out` with the dabs for one parked shape, applying the LIVE Offset slider so the one global
    /// Offset acts on EVERY shape simultaneously and in real time (Enio 2026-07-04), not just the active one.
    /// Mirrors each `*_refill`'s geometry→(offset)→spine→`fill_*_preview` core, minus the stamp.
    fn parked_shape_dabs(&self, st: &crate::undo::ShapeEditState, out: &mut Vec<Dab>) {
        use crate::undo::ShapeEditState as S;
        let off = self.shape_offset_px();
        let trim = self.paint.offset_trim;
        let mut stroke;
        match st {
            S::Curve(c) => {
                let (pts, handles, _) =
                    curve_offset::offset_curve_refined(&c.points, &c.handles, off, c.closed);
                let mut spine = Vec::new();
                curve_geom::flatten_spine(&pts, &handles, c.closed, &mut spine);
                if trim {
                    spine = curve::trim_offset_spine(&spine, c.closed);
                }
                stroke = Stroke::new(self.paint.brush, self.paint.dynamics, c.seed);
                stroke.fill_polyline_preview(&spine, out);
            }
            S::Ellipse(e) => {
                let (erx, ery) = self.ellipse_offset_radii(e.rx, e.ry);
                stroke = Stroke::new(self.paint.brush, self.paint.dynamics, e.seed);
                stroke.fill_ellipse_preview(e.center, e.u, erx, ery, out);
            }
            S::Polygon(p) => {
                let (erx, ery) = ((p.rx + off).max(0.5), (p.ry + off).max(0.5));
                stroke = Stroke::new(self.paint.brush, self.paint.dynamics, p.seed);
                stroke.fill_polygon_preview(p.center, p.u, erx, ery, p.sides, out);
            }
            S::Line(l) => {
                let mods: Vec<line_corner::CornerMod> = l
                    .corner_mods
                    .iter()
                    .copied()
                    .map(line_corner::CornerMod::from_wire)
                    .collect();
                let mut path =
                    line_corner::expand(&l.points, l.closed, &mods, self.paint.shape_grab_tol_px);
                path = line_offset::offset_polyline(&path, l.closed, off);
                if trim && off != 0.0 {
                    path = if l.closed {
                        curve_trim::trim_self_intersections_closed(&path)
                    } else {
                        curve_trim::trim_self_intersections(&path)
                    };
                }
                if path.len() >= 2 {
                    stroke = Stroke::new(self.paint.brush, self.paint.dynamics, l.seed);
                    stroke.fill_polyline_preview(&path, out);
                }
            }
        }
    }

    /// Capture the parked shapes for a [`crate::undo::ModelSnapshot`] (geometry + wire op).
    pub(crate) fn parked_shapes_snapshot(&self) -> Vec<crate::undo::ParkedShapeState> {
        self.paint
            .parked_shapes
            .iter()
            .map(|s| crate::undo::ParkedShapeState {
                state: s.state.clone(),
                op: s.op.to_wire(),
            })
            .collect()
    }

    /// Reinstate the parked shapes from a restored snapshot (inverse of [`Self::parked_shapes_snapshot`]).
    pub(crate) fn restore_parked_shapes(&mut self, parked: Vec<crate::undo::ParkedShapeState>) {
        self.paint.parked_shapes = parked
            .into_iter()
            .map(|p| StrokeShape {
                state: p.state,
                op: StrokeOp::from_wire(p.op),
            })
            .collect();
    }

    /// Drop every parked shape (Apply / cancel / teardown clear the whole multi-shape set).
    pub(crate) fn clear_parked_shapes(&mut self) {
        self.paint.parked_shapes.clear();
    }

    /// Route a canvas pointer through the shape editors WITH multi-shape switching: on a Down that lands on
    /// a PARKED shape re-activate it (parking the current one first); on a Down in EMPTY space with a
    /// complete active shape, park it so the following `*_down` begins a fresh shape. Then dispatch to the
    /// active editor by method (the classic single-shape path). Line point-placement is never interrupted.
    pub(super) fn route_shape_pointer_multi(&mut self, ev: CanvasPointer) -> bool {
        let slop = self.paint.shape_grab_tol_px;
        match ev.phase {
            PointerPhase::Down => {
                self.maybe_switch_or_new_shape(ev.pos);
                // Arm an op-cycle tap iff the Down lands on the active shape's centre square.
                self.paint.op_tap = self
                    .on_active_centre_square(ev.pos, slop * 2.0)
                    .then_some(ev.pos);
            }
            PointerPhase::Move => {
                // A drag past the slop is a MOVE, not an op tap — disarm.
                if let Some(p) = self.paint.op_tap
                    && dist2(ev.pos, p) > slop * slop
                {
                    self.paint.op_tap = None;
                }
            }
            _ => {}
        }
        let consumed = match self.paint.brush.stroke_method {
            StrokeMethod::Curve | StrokeMethod::FreeHand => self.curve_pointer(ev),
            StrokeMethod::Ellipse => self.ellipse_pointer(ev),
            StrokeMethod::Polygon => self.polygon_pointer(ev),
            StrokeMethod::Line => self.line_pointer(ev),
            _ => false,
        };
        // Pen-up on an un-dragged centre-square tap → cycle the active shape's Operation (+ → − → o).
        if ev.phase == PointerPhase::Up
            && let Some(p) = self.paint.op_tap.take()
            && dist2(ev.pos, p) <= slop * slop
        {
            self.cycle_active_op();
        }
        consumed
    }

    /// `true` when `pos` is within `tol` of the ACTIVE shape's centre (its gizmo type-square). Used both to
    /// keep a centre click from being read as "empty space" and to arm the op-cycle tap.
    fn on_active_centre_square(&self, pos: [f32; 2], tol: f32) -> bool {
        let Some(state) = self.capture_shape() else {
            return false;
        };
        let Some(bb) = self.shape_state_bbox(&state) else {
            return false;
        };
        let c = [(bb[0] + bb[2]) * 0.5, (bb[1] + bb[3]) * 0.5];
        dist2(pos, c) <= tol * tol
    }

    /// Cycle the ACTIVE shape's Operation (+ → − → o), keep the panel mode in sync, and recompose (a boolean
    /// result may change). The `+`/`−`/`o` glyph in its gizmo updates live.
    pub(crate) fn cycle_active_op(&mut self) {
        if !self.is_editing_shape() {
            return;
        }
        self.paint.active_op = self.paint.active_op.cycle();
        self.paint.stroke_op_mode = self.paint.active_op;
        self.refill_open_shape();
    }

    /// The `+`/`−`/`o` glyph for the ACTIVE shape's Operation — the shell draws it INSIDE the shape's gizmo
    /// centre-move square (enlarged), so there is no separate badge square (Enio 2026-07-04). `None` when no
    /// shape is being edited.
    pub fn active_op_glyph(&self) -> Option<&'static str> {
        self.is_editing_shape()
            .then(|| self.paint.active_op.glyph())
    }

    /// The op badges the shell draws for the PARKED shapes only — each a faint AABB frame + its `+`/`−`/`o`
    /// glyph so it reads as a still-editable shape. The ACTIVE shape's glyph is drawn INSIDE its gizmo
    /// centre square instead (`active_op_glyph`), so there is never a second/duplicate square (Enio
    /// 2026-07-04). Empty in the classic single-shape / no-parked case. Image px.
    pub fn stroke_op_badges(&self) -> Vec<StrokeOpBadge> {
        let parked: Vec<(crate::undo::ShapeEditState, StrokeOp)> = self
            .paint
            .parked_shapes
            .iter()
            .map(|s| (s.state.clone(), s.op))
            .collect();
        let mut out = Vec::new();
        for (st, op) in &parked {
            if let Some(bb) = self.shape_state_bbox(st) {
                out.push(StrokeOpBadge {
                    center: [(bb[0] + bb[2]) * 0.5, (bb[1] + bb[3]) * 0.5],
                    bbox: bb,
                    glyph: op.glyph(),
                    active: false,
                });
            }
        }
        out
    }

    /// The Down-time multi-shape decision (see [`Self::route_shape_pointer_multi`]): switch to a parked
    /// shape under the cursor, or park the active shape when the click is in empty space.
    fn maybe_switch_or_new_shape(&mut self, pos: [f32; 2]) {
        // Never interrupt a Line that is still placing its corner points (multi-Down authoring).
        if self.paint.line.as_ref().is_some_and(|l| !l.is_editing()) {
            return;
        }
        let tol = self.paint.shape_grab_tol_px.max(6.0);
        // The active shape's centre square (its op type-square) is an edit target (tap = cycle op, drag =
        // move) — never treat it as empty space.
        if self.on_active_centre_square(pos, tol * 2.0) {
            return;
        }
        // A click that would EDIT the ACTIVE shape (grab a handle / point / gizmo, or land on its outline)
        // is left to the per-type editor — never park it. This precise per-editor test (not a coarse bbox)
        // is why grabbing a rotate ring that sits OUTSIDE the outline still counts as editing.
        if self.active_shape_hit(pos) {
            return;
        }
        // A click on a PARKED shape re-activates it (its handle is then grabbed by the following `*_down`).
        if let Some(i) = self.hit_parked_shape_bbox(pos, tol) {
            self.activate_parked_shape(i);
            return;
        }
        // Empty space with a complete editable active shape → park it; the following `*_down` starts fresh.
        if self.is_editing_shape() {
            self.park_active_shape();
        }
    }

    /// `true` when a Down at `pos` targets the ACTIVE editor (whichever slot is live) for editing — the
    /// per-type predicate; each returns `false` when its slot is empty, so exactly the live one answers.
    fn active_shape_hit(&self, pos: [f32; 2]) -> bool {
        self.curve_hit_active(pos)
            || self.ellipse_hit_active(pos)
            || self.polygon_hit_active(pos)
            || self.line_hit_active(pos)
    }

    /// Index of the topmost (last-drawn) parked shape whose padded AABB contains `pos`, or `None`.
    fn hit_parked_shape_bbox(&self, pos: [f32; 2], tol: f32) -> Option<usize> {
        let states: Vec<crate::undo::ShapeEditState> = self
            .paint
            .parked_shapes
            .iter()
            .map(|s| s.state.clone())
            .collect();
        states
            .iter()
            .enumerate()
            .rev()
            .find(|(_, st)| {
                self.shape_state_bbox(st)
                    .is_some_and(|bb| bbox_contains(bb, pos, tol))
            })
            .map(|(i, _)| i)
    }

    /// AABB over a shape's stamped dab centres (reuses [`Self::parked_shape_dabs`]).
    fn shape_state_bbox(&self, st: &crate::undo::ShapeEditState) -> Option<[f32; 4]> {
        let mut dabs = Vec::new();
        self.parked_shape_dabs(st, &mut dabs);
        let mut bb = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
        for d in &dabs {
            bb[0] = bb[0].min(d.center[0]);
            bb[1] = bb[1].min(d.center[1]);
            bb[2] = bb[2].max(d.center[0]);
            bb[3] = bb[3].max(d.center[1]);
        }
        (bb[0] <= bb[2]).then_some(bb)
    }

    /// Prepare the pixel baseline for a NEW shape session's idle-create branch. Always forces a full
    /// recompose (drops the composite cache), but drops the drag-preview RESTORE record ONLY when no shape
    /// is parked. With parked shapes present the baseline must persist — dropping it would leave the parked
    /// stamp on canvas as a false "pristine" base — so the new shape's first fill peels + re-stamps the
    /// whole set from the real pristine baseline instead.
    pub(super) fn begin_shape_session_base(&mut self) {
        // A freshly-created shape adopts the current panel Operation mode.
        self.paint.active_op = self.paint.stroke_op_mode;
        if !self.has_parked_shapes() {
            // Truly fresh session: drop the baseline + re-centre the Offset. With shapes already parked the
            // baseline persists AND the Offset slider stays put — it is GLOBAL across the whole set, so a new
            // shape must not reset it (else the parked shapes would jump back to zero offset).
            self.paint.drag_preview = None;
            self.paint.shape_offset_base_px = 0.0;
            self.paint.shape_offset_norm = 0.5;
        }
        self.reseed_preview_base();
    }
}

/// `true` when `pos` lies inside `[minx,miny,maxx,maxy]` padded by `tol` on every side.
fn bbox_contains(bb: [f32; 4], pos: [f32; 2], tol: f32) -> bool {
    pos[0] >= bb[0] - tol && pos[0] <= bb[2] + tol && pos[1] >= bb[1] - tol && pos[1] <= bb[3] + tol
}

/// Minimum distance (px), regardless of grab tolerance, within which a click on a Curve counts as an
/// insert-on-curve EDIT rather than a "start a new shape". Keeps the new-shape gesture from firing on a
/// click that visually reads as "on the curve".
pub(super) const NEW_SHAPE_INSERT_BAND_PX: f32 = 20.0;

/// `true` when `pos` is within `band` px of the polyline `spine` (min distance to any segment).
pub(super) fn point_near_polyline(pos: [f32; 2], spine: &[[f32; 2]], band: f32) -> bool {
    min_dist2_to_polyline(pos, spine).is_some_and(|d2| d2 <= band * band)
}

/// The MIN squared distance from `pos` to the polyline `spine` (any segment), or `None` when empty. Used to
/// pick which of several curves a click is nearest to (multi-curve point insertion).
pub(super) fn min_dist2_to_polyline(pos: [f32; 2], spine: &[[f32; 2]]) -> Option<f32> {
    if spine.is_empty() {
        return None;
    }
    if spine.len() == 1 {
        return Some(dist2(pos, spine[0]));
    }
    spine
        .windows(2)
        .map(|w| dist2_point_segment(pos, w[0], w[1]))
        .fold(None, |acc, d| Some(acc.map_or(d, |a: f32| a.min(d))))
}

fn dist2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dx * dx + dy * dy
}

/// Squared distance from `p` to segment `a`–`b` (transcendental-free; HR-5).
fn dist2_point_segment(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let (abx, aby) = (b[0] - a[0], b[1] - a[1]);
    let len2 = abx * abx + aby * aby;
    if len2 <= f32::EPSILON {
        return dist2(p, a);
    }
    let t = (((p[0] - a[0]) * abx + (p[1] - a[1]) * aby) / len2).clamp(0.0, 1.0);
    let proj = [a[0] + t * abx, a[1] + t * aby];
    dist2(p, proj)
}
