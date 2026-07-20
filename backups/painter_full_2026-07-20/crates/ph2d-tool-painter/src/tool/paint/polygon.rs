//! The **Polygon** stroke method — an editable regular polygon on the canvas (PH2D shape extension;
//! `docs/Painter/`). A submodule of [`super`] (`paint`); mirrors [`super::circle`] with one extra
//! handle: the side count (3..=12). Split out for the same private-access + LOC-cap reasons.
//!
//! ## Interaction
//!
//! 1. **Draw** — press at the centre and drag out the radius. On release the editing handles appear.
//! 2. **Edit** — four **axis handles** (right/top/left/bottom) resize width/height symmetrically; a
//!    **rotation handle** above the top spins it; a **sides handle** to the right changes the side
//!    count (its distance from the centre encodes 3..12); the **centre** dot moves it. The painted
//!    outline previews live (restore + re-stamp, no trail).
//! 3. **Commit / cancel** — Enter bakes the outline (one undo step); Esc discards.
//!
//! Orientation is a unit VECTOR (never an angle) and the perimeter comes from the transcendental-free
//! [`polygon_perimeter`] — so the whole editor is bit-deterministic (HR-5).

use super::*;

use ph2d_painter_brush::{POLY_MAX_SIDES, POLY_MIN_SIDES, Stroke, polygon_perimeter};

/// Smallest half-axis the editor allows (image px).
const MIN_AXIS_PX: f32 = 1.0;
/// The rotation handle sits this many grab-radii beyond the top of the polygon.
const ROTATE_GAP_FACTOR: f32 = 3.0;
/// The sides handle's base offset beyond the right edge, in grab-radii (its position for 3 sides).
const SIDES_GAP_FACTOR: f32 = 3.0;
/// How far (in grab-radii) the sides handle moves further out per extra side.
const SIDES_STEP_FACTOR: f32 = 1.5;
/// Side count a fresh polygon starts with.
const DEFAULT_SIDES: u32 = 5;

/// Handle indices — shared by the hit-test, `grabbed`, and the overlay's `handles`: 0 right, 1 top,
/// 2 left, 3 bottom (axis), 4 rotate, 5 sides, 6 centre.
const H_RIGHT: u8 = 0;
const H_TOP: u8 = 1;
const H_LEFT: u8 = 2;
const H_BOTTOM: u8 = 3;
const H_ROTATE: u8 = 4;
const H_SIDES: u8 = 5;
const H_CENTER: u8 = 6;

/// In-progress Polygon session: the inscribing ellipse `(center, u, rx, ry)`, the side count, the
/// current grab + the draw-vs-edit phase. Held in [`PaintState::polygon`]; `None` when idle.
pub(super) struct PolygonEditor {
    center: [f32; 2],
    u: [f32; 2],
    rx: f32,
    ry: f32,
    sides: u32,
    grabbed: Option<u8>,
    editing: bool,
    seed: u64,
}

/// A read-only snapshot of the Polygon editor for the shell overlay (the outline + the seven
/// handles). `None` until the radius drag is released.
pub struct PolygonOverlay {
    /// The polygon outline (image-space px) — drawn as the guide; matches the painted dabs.
    pub perimeter: Vec<[f32; 2]>,
    /// Handle positions: `[right, top, left, bottom, rotate, sides, center]`.
    pub handles: [[f32; 2]; 7],
    /// The handle being dragged (index into [`Self::handles`]), drawn highlighted.
    pub grabbed: Option<u8>,
    /// The current side count (3..=12) — the shell can label the sides handle.
    pub sides: u32,
}

impl PainterTool {
    /// Route a canvas pointer event through the Polygon editor (the painter calls this while the
    /// method is [`StrokeMethod::Polygon`]). Returns `true` when consumed.
    pub(super) fn polygon_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.polygon_down(ev.pos),
            PointerPhase::Move => self.polygon_move(ev.pos),
            PointerPhase::Up => self.polygon_up(ev.pos),
            PointerPhase::Hover => false,
        }
    }

    /// Pointer-down: start a session (begin the centre-out drag) when idle; otherwise grab a handle.
    fn polygon_down(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        self.flush_shape_txn(); // close any coalesced Offset drag as its own undo entry first
        self.bake_polygon_offset(); // an edit gesture locks in any live Offset (hit-test = displayed handles)
        if self.paint.polygon.is_none() {
            // No session → open the creation txn (`before` = no shape) + begin the centre-out radius drag.
            self.paint.stroke_undo = Some(self.capture_shape_model()); // creation txn (`before` = parked set, no active)
            self.begin_shape_session_base(); // full recompose; keep baseline + GLOBAL Offset when shapes are parked
            let seed = self.paint.seed;
            self.paint.seed = self.paint.seed.wrapping_add(1);
            self.paint.polygon = Some(PolygonEditor {
                center: pos,
                u: [1.0, 0.0],
                rx: 0.0,
                ry: 0.0,
                sides: DEFAULT_SIDES,
                grabbed: None,
                editing: false,
                seed,
            });
            return true;
        }
        if !self.paint.polygon.as_ref().is_some_and(|ed| ed.editing) {
            return true;
        }
        self.begin_shape_txn(); // bracket this reshape gesture; polygon_up drops it if nothing changed
        let ed = self.paint.polygon.as_mut().expect("polygon present");
        let handles = polygon_handles(ed.center, ed.u, ed.rx, ed.ry, ed.sides, tol);
        ed.grabbed = polygon_hit(&handles, pos, tol);
        true
    }

    /// `true` when a Down at `pos` would EDIT the active polygon (grab a handle) — the multi-shape router
    /// must not treat it as empty space. Mid-draw always counts as active. Same hit-test as `polygon_down`.
    pub(super) fn polygon_hit_active(&self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        let Some(ed) = self.paint.polygon.as_ref() else {
            return false;
        };
        if !ed.editing {
            return true;
        }
        let handles = polygon_handles(ed.center, ed.u, ed.rx, ed.ry, ed.sides, tol);
        polygon_hit(&handles, pos, tol).is_some()
    }

    /// Pointer-move: drag the grabbed handle (edit) or stretch the radius (draw).
    fn polygon_move(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        let Some(ed) = self.paint.polygon.as_mut() else {
            return false;
        };
        if !ed.editing {
            let r = dist(ed.center, pos).max(MIN_AXIS_PX);
            ed.rx = r;
            ed.ry = r;
            self.polygon_refill();
            return true;
        }
        let Some(g) = ed.grabbed else {
            return true;
        };
        apply_handle_drag(ed, g, pos, tol);
        self.polygon_refill();
        true
    }

    /// Pointer-up: release a grab (edit), or finalize the radius drag into the editable polygon.
    fn polygon_up(&mut self, pos: [f32; 2]) -> bool {
        let Some(ed) = self.paint.polygon.as_mut() else {
            return false;
        };
        if ed.editing {
            ed.grabbed = None;
            self.commit_shape_txn(); // record this reshape gesture iff it changed the polygon
            return true;
        }
        let r = dist(ed.center, pos).max(MIN_AXIS_PX);
        ed.rx = r;
        ed.ry = r;
        ed.editing = true;
        self.polygon_refill();
        self.commit_shape_txn(); // the radius drag is the polygon's CREATION → one undo entry
        true
    }

    /// Commit the polygon (Enter): keep the painted outline + push one undo entry. Returns `true`
    /// when a committable session was open.
    pub fn polygon_commit(&mut self) -> bool {
        if !self.paint.polygon.as_ref().is_some_and(|ed| ed.editing) {
            return false;
        }
        self.flush_shape_txn(); // close any coalesced Offset drag as its own entry first
        let before = self.capture_shape_model(); // the open polygon (for undo of the Apply)
        self.paint.polygon = None;
        self.commit_drag_preview(); // drop the restore record → the painted polygon stays baked
        self.paint.shape_offset_base_px = 0.0; // the offset baked into the painted dabs → reset the Offset
        self.paint.shape_offset_norm = 0.5;
        let after = self.capture_shape_model();
        self.undo.record_structural(before, after); // Apply (bake + close) is one undo entry
        true
    }

    /// Commit the polygon (**Apply**) but KEEP the editor + handles open for re-apply/reshape (the
    /// "Apply & Keep" button). The bake is ONE undo entry whose `after` keeps the editor open over the baked
    /// pixels, interleaving with the surrounding shape edits. See [`PainterTool::curve_commit_keep`].
    pub fn polygon_commit_keep(&mut self) -> bool {
        if !self.paint.polygon.as_ref().is_some_and(|ed| ed.editing) {
            return false;
        }
        self.flush_shape_txn();
        let before = self.capture_shape_model(); // polygon + live preview, pre-bake
        self.accumulate_offset(); // fold the slider into the accumulator + re-centre it (radii untouched)
        self.commit_drag_preview(); // the painted polygon becomes permanent (no live preview left)
        let after = self.capture_shape_model(); // polygon kept open (same radii), pixels baked, no preview
        self.undo.record_structural(before, after);
        self.paint.drag_preview = None;
        true
    }

    /// Cancel the polygon (Esc): revert the painted preview and discard the session.
    pub fn polygon_cancel(&mut self) -> bool {
        if self.paint.polygon.is_none() {
            return false;
        }
        self.paint.polygon = None;
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        self.paint.stroke_undo = None;
        self.paint.shape_offset_base_px = 0.0; // a cancelled shape must NOT carry its Offset to the next one
        self.paint.shape_offset_norm = 0.5;
        true
    }

    /// Drop the Polygon session without touching pixels (teardown where the canvas is replaced).
    pub(crate) fn polygon_discard(&mut self) {
        self.paint.polygon = None;
    }

    /// Convert the editing polygon to an editable **closed** curve (anchors + `[in, out]` handles): the N
    /// corner vertices with ZERO-length (sharp-corner) handles so the converted curve is the exact polygon to
    /// start; the editor closes the loop itself (no duplicate seam vertex), then the artist can pull handles
    /// to round any edge. `None` unless editing. Drives the panel's **Edit** (E) button via
    /// [`PainterTool::convert_open_shape_to_curve`].
    #[allow(clippy::type_complexity)]
    pub(super) fn polygon_to_curve(&self) -> Option<(Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>)> {
        let ed = self.paint.polygon.as_ref()?;
        if !ed.editing {
            return None;
        }
        let mut points = Vec::new();
        polygon_perimeter(ed.center, ed.u, ed.rx, ed.ry, ed.sides, &mut points);
        if points.is_empty() {
            return None;
        }
        let handles = points.iter().map(|&p| [p, p]).collect(); // zero handles = sharp corners
        Some((points, handles))
    }

    /// Snapshot the Polygon editor for the shell overlay — `None` until editing.
    #[must_use]
    pub fn polygon_overlay(&self) -> Option<PolygonOverlay> {
        let ed = self.paint.polygon.as_ref()?;
        if !ed.editing {
            return None;
        }
        // Display the OFFSET (grown/shrunk) polygon, so the outline + handles + paint move together.
        let off = self.shape_offset_px();
        let (erx, ery) = ((ed.rx + off).max(0.5), (ed.ry + off).max(0.5));
        let mut perimeter = Vec::new();
        polygon_perimeter(ed.center, ed.u, erx, ery, ed.sides, &mut perimeter);
        Some(PolygonOverlay {
            perimeter,
            handles: polygon_handles(
                ed.center,
                ed.u,
                erx,
                ery,
                ed.sides,
                self.paint.shape_grab_tol_px,
            ),
            grabbed: ed.grabbed,
            sides: ed.sides,
        })
    }

    // ── internals ────────────────────────────────────────────────────────────────────

    /// **Materialise** the EFFECTIVE offset into the polygon radii + zero the offset — see
    /// [`Self::bake_curve_offset`]. Called before an edit gesture; Apply & Keep accumulates instead.
    pub(super) fn bake_polygon_offset(&mut self) {
        let off = self.shape_offset_px();
        if off != 0.0
            && let Some(ed) = self.paint.polygon.as_mut()
            && ed.editing
        {
            ed.rx = (ed.rx + off).max(0.5);
            ed.ry = (ed.ry + off).max(0.5);
            self.paint.shape_offset_base_px = 0.0;
            self.paint.shape_offset_norm = 0.5;
        }
    }

    /// Re-fill the painted preview from the editor's current polygon (a fresh `Stroke` per fill).
    pub(super) fn polygon_refill(&mut self) {
        let Some(ed) = self.paint.polygon.as_ref() else {
            return;
        };
        let (center, u, rx, ry, sides, seed) = (ed.center, ed.u, ed.rx, ed.ry, ed.sides, ed.seed);
        let off = self.shape_offset_px();
        let (erx, ery) = ((rx + off).max(0.5), (ry + off).max(0.5));
        let mut stroke = Stroke::new(self.paint.brush, self.paint.dynamics, seed);
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.fill_polygon_preview(center, u, erx, ery, sides, &mut dabs);
        self.restamp_shapes_preview(&dabs); // active shape + every parked shape onto one baseline
        self.paint.dabs = dabs;
    }
}

impl PolygonEditor {
    /// Capture the polygon for the unified undo timeline; the transient `grabbed` is not stored.
    pub(super) fn to_state(&self) -> crate::undo::PolygonState {
        crate::undo::PolygonState {
            center: self.center,
            u: self.u,
            rx: self.rx,
            ry: self.ry,
            sides: self.sides,
            editing: self.editing,
            seed: self.seed,
        }
    }

    /// Rebuild the editor from a restored snapshot (grab idle) — the inverse of [`Self::to_state`].
    pub(super) fn from_state(s: crate::undo::PolygonState) -> Self {
        PolygonEditor {
            center: s.center,
            u: s.u,
            rx: s.rx,
            ry: s.ry,
            sides: s.sides,
            grabbed: None,
            editing: s.editing,
            seed: s.seed,
        }
    }
}

/// Apply a handle drag: axis handles resize their axis, the rotation handle re-orients, the sides
/// handle changes the side count (its projected distance along `u` maps to 3..12), the centre moves.
/// Transcendental-free. `tol` is the grab radius (image px) the sides mapping is scaled by.
fn apply_handle_drag(ed: &mut PolygonEditor, handle: u8, pos: [f32; 2], tol: f32) {
    let v = [-ed.u[1], ed.u[0]];
    let rel = [pos[0] - ed.center[0], pos[1] - ed.center[1]];
    match handle {
        H_RIGHT | H_LEFT => ed.rx = dot(rel, ed.u).abs().max(MIN_AXIS_PX),
        H_TOP | H_BOTTOM => ed.ry = dot(rel, v).abs().max(MIN_AXIS_PX),
        H_ROTATE => {
            let nv = unit_or(rel, v);
            ed.u = [nv[1], -nv[0]];
        }
        H_SIDES => {
            // The handle lives beyond the right edge at rx + base + (sides-3)·step; invert that to
            // read the side count from where it was dragged.
            let proj = dot(rel, ed.u);
            let base = ed.rx + tol * SIDES_GAP_FACTOR;
            let step = (tol * SIDES_STEP_FACTOR).max(1.0);
            let raw = POLY_MIN_SIDES as f32 + ((proj - base) / step).round();
            ed.sides = (raw as i32).clamp(POLY_MIN_SIDES as i32, POLY_MAX_SIDES as i32) as u32;
        }
        H_CENTER => ed.center = pos,
        _ => {}
    }
}

/// Handle positions for `ed`: `[right, top, left, bottom, rotate, sides, center]`. `tol` is the grab
/// radius (sets the rotate / sides handle offsets, which stay a constant on-screen size).
fn polygon_handles(
    c: [f32; 2],
    u: [f32; 2],
    rx: f32,
    ry: f32,
    sides: u32,
    tol: f32,
) -> [[f32; 2]; 7] {
    let v = [-u[1], u[0]];
    let rot_gap = tol * ROTATE_GAP_FACTOR;
    let sides_off =
        rx + tol * SIDES_GAP_FACTOR + (sides - POLY_MIN_SIDES) as f32 * tol * SIDES_STEP_FACTOR;
    [
        [c[0] + u[0] * rx, c[1] + u[1] * rx], // right
        [c[0] + v[0] * ry, c[1] + v[1] * ry], // top
        [c[0] - u[0] * rx, c[1] - u[1] * rx], // left
        [c[0] - v[0] * ry, c[1] - v[1] * ry], // bottom
        [c[0] + v[0] * (ry + rot_gap), c[1] + v[1] * (ry + rot_gap)], // rotate
        [c[0] + u[0] * sides_off, c[1] + u[1] * sides_off], // sides
        c,                                    // center
    ]
}

/// Index of the handle within `tol` px of `pos`. Axis + rotate + sides (0..=5) are tested first so a
/// nearby edge handle wins over the centre on a small polygon; the centre (6) is the fallback.
fn polygon_hit(handles: &[[f32; 2]; 7], pos: [f32; 2], tol: f32) -> Option<u8> {
    let tol2 = tol * tol;
    let mut best = None;
    let mut bestd = tol2;
    for i in H_RIGHT..=H_SIDES {
        let d = dist2(handles[i as usize], pos);
        if d <= bestd {
            bestd = d;
            best = Some(i);
        }
    }
    if best.is_some() {
        return best;
    }
    if dist2(handles[H_CENTER as usize], pos) <= tol2 {
        return Some(H_CENTER);
    }
    None
}

/// Dot product of two 2-vectors.
fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// Squared distance between two points.
fn dist2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

/// Euclidean distance between two points.
fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    dist2(a, b).sqrt()
}

/// Normalize `p`, falling back to `fallback` for a (near) zero vector. Only `sqrt` (deterministic).
fn unit_or(p: [f32; 2], fallback: [f32; 2]) -> [f32; 2] {
    let len = (p[0] * p[0] + p[1] * p[1]).sqrt();
    if len > 1e-6 {
        [p[0] / len, p[1] / len]
    } else {
        fallback
    }
}
