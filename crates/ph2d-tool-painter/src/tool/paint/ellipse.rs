//! The **Ellipse** stroke method — an editable ellipse on the canvas (PH2D shape extension;
//! `docs/Painter/`). A submodule of [`super`] (`paint`) so it shares `PaintState`'s private access
//! and the canvas helpers (save/restore/stamp); split out for the same LOC-cap reason as
//! [`super::curve`] and [`super::brush_settings`].
//!
//! ## Interaction
//!
//! 1. **Draw** — press at the centre and drag out the radius (a circle). On release the editing
//!    handles appear.
//! 2. **Edit** — the session persists. Four **axis handles** (right / top / left / bottom in the
//!    ellipse's local frame) resize each axis symmetrically about the centre (so a circle becomes an
//!    ellipse); a **rotation handle** above the top spins the whole ellipse; the **centre** dot moves
//!    it. The painted outline previews live (restore + re-stamp, no trail).
//! 3. **Commit / cancel** — Enter bakes the outline (one undo step); Esc discards. (No per-point
//!    Delete — the handle set is fixed.)
//!
//! Orientation is carried as a unit VECTOR `u` (never an angle), and the perimeter comes from the
//! transcendental-free [`ellipse_perimeter`] — so the whole editor is bit-deterministic (HR-5).

use super::*;

use ph2d_painter_brush::{Stroke, ellipse_perimeter};

/// Smallest half-axis the editor allows (image px) — keeps a grabbed axis from collapsing to zero.
const MIN_AXIS_PX: f32 = 1.0;
/// The rotation handle sits this many grab-radii beyond the top of the ellipse.
const ROTATE_GAP_FACTOR: f32 = 3.0;

/// Handle indices, shared by the hit-test, the editor's `grabbed` slot, and the overlay's
/// `handles` array: 0 right, 1 top, 2 left, 3 bottom (the four axis handles), 4 rotate, 5 centre.
const H_RIGHT: u8 = 0;
const H_TOP: u8 = 1;
const H_LEFT: u8 = 2;
const H_BOTTOM: u8 = 3;
const H_ROTATE: u8 = 4;
const H_CENTER: u8 = 5;

/// In-progress Ellipse session: the ellipse `(center, u, rx, ry)` plus the current grab + the
/// draw-vs-edit phase. Held in [`PaintState::circle`]; `None` when idle. `u` is the local x-axis
/// unit vector (orientation); the y-axis is its left perpendicular.
pub(super) struct EllipseEditor {
    center: [f32; 2],
    u: [f32; 2],
    rx: f32,
    ry: f32,
    grabbed: Option<u8>,
    editing: bool,
    seed: u64,
}

/// A read-only snapshot of the Ellipse editor for the shell's on-canvas overlay (the outline + the
/// six handles). `None` until the radius drag is released (the handles appear with the ellipse).
pub struct EllipseOverlay {
    /// The ellipse outline (image-space px) — drawn as the guide; matches the painted dabs.
    pub perimeter: Vec<[f32; 2]>,
    /// Handle positions (image-space px): `[right, top, left, bottom, rotate, center]`.
    pub handles: [[f32; 2]; 6],
    /// The handle being dragged (index into [`Self::handles`]), drawn highlighted.
    pub grabbed: Option<u8>,
}

impl PainterTool {
    /// Route a canvas pointer event through the Ellipse editor (the painter calls this instead of the
    /// generic paint path while the method is [`StrokeMethod::Ellipse`]). Returns `true` when consumed.
    pub(super) fn ellipse_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.ellipse_down(ev.pos),
            PointerPhase::Move => self.ellipse_move(ev.pos),
            PointerPhase::Up => self.ellipse_up(ev.pos),
            PointerPhase::Hover => false,
        }
    }

    /// Pointer-down: start a session (begin the centre-out radius drag) when idle; otherwise grab the
    /// handle under the cursor.
    fn ellipse_down(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        self.flush_shape_txn(); // close any coalesced Offset drag as its own undo entry first
        self.bake_ellipse_offset(); // an edit gesture locks in any live Offset (hit-test = displayed handles)
        if self.paint.ellipse.is_none() {
            // No session → open the creation txn (`before` = no shape) + begin the centre-out radius drag.
            self.paint.stroke_undo = Some(self.capture_shape_model()); // creation txn (`before` = parked set, no active)
            self.begin_shape_session_base(); // full recompose; keep baseline + GLOBAL Offset when shapes are parked
            let seed = self.paint.seed;
            self.paint.seed = self.paint.seed.wrapping_add(1);
            self.paint.ellipse = Some(EllipseEditor {
                center: pos,
                u: [1.0, 0.0],
                rx: 0.0,
                ry: 0.0,
                grabbed: None,
                editing: false,
                seed,
            });
            return true;
        }
        if !self.paint.ellipse.as_ref().is_some_and(|ed| ed.editing) {
            return true; // mid radius-drag — ignore extra Downs
        }
        self.begin_shape_txn(); // bracket this reshape gesture; ellipse_up drops it if nothing changed
        let ed = self.paint.ellipse.as_mut().expect("circle present");
        let handles = ellipse_handles(ed.center, ed.u, ed.rx, ed.ry, tol * ROTATE_GAP_FACTOR);
        ed.grabbed = ellipse_hit(&handles, pos, tol);
        true
    }

    /// `true` when a Down at `pos` would EDIT the active ellipse (grab one of its handles), so the
    /// multi-shape router must NOT treat it as empty space. Mid-draw (no session settled) always counts as
    /// active. Uses the SAME hit-test as [`Self::ellipse_down`], so it includes the rotate ring outside the
    /// outline.
    pub(super) fn ellipse_hit_active(&self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        let Some(ed) = self.paint.ellipse.as_ref() else {
            return false;
        };
        if !ed.editing {
            return true;
        }
        let handles = ellipse_handles(ed.center, ed.u, ed.rx, ed.ry, tol * ROTATE_GAP_FACTOR);
        ellipse_hit(&handles, pos, tol).is_some()
    }

    /// Pointer-move: drag the grabbed handle (edit) or stretch the radius (draw).
    fn ellipse_move(&mut self, pos: [f32; 2]) -> bool {
        let Some(ed) = self.paint.ellipse.as_mut() else {
            return false;
        };
        if !ed.editing {
            // Drawing: radius = distance from the centre; a circle (rx == ry), axis-aligned.
            let r = dist(ed.center, pos).max(MIN_AXIS_PX);
            ed.rx = r;
            ed.ry = r;
            self.ellipse_refill();
            return true;
        }
        let Some(g) = ed.grabbed else {
            return true; // editing but nothing grabbed — ignore stray moves
        };
        apply_handle_drag(ed, g, pos);
        self.ellipse_refill();
        true
    }

    /// Pointer-up: release a grab (edit), or finalize the radius drag into the editable ellipse.
    fn ellipse_up(&mut self, pos: [f32; 2]) -> bool {
        let Some(ed) = self.paint.ellipse.as_mut() else {
            return false;
        };
        if ed.editing {
            ed.grabbed = None;
            self.commit_shape_txn(); // record this reshape gesture iff it changed the ellipse
            return true;
        }
        let r = dist(ed.center, pos).max(MIN_AXIS_PX);
        ed.rx = r;
        ed.ry = r;
        ed.editing = true;
        self.ellipse_refill();
        self.commit_shape_txn(); // the radius drag is the ellipse's CREATION → one undo entry
        true
    }

    /// Commit the ellipse (Enter): keep the painted outline + push one undo entry. Returns `true`
    /// when a committable session was open (the shell consumes Enter only then).
    pub fn ellipse_commit(&mut self) -> bool {
        if !self.paint.ellipse.as_ref().is_some_and(|ed| ed.editing) {
            return false; // none open / mid radius-drag — nothing to bake yet
        }
        self.flush_shape_txn(); // close any coalesced Offset drag as its own entry first
        let before = self.capture_shape_model(); // the open ellipse (for undo of the Apply)
        self.paint.ellipse = None;
        self.commit_drag_preview(); // drop the restore record → the painted ellipse stays baked
        self.paint.shape_offset_base_px = 0.0; // the offset baked into the painted dabs → reset the Offset
        self.paint.shape_offset_norm = 0.5;
        let after = self.capture_shape_model(); // shape gone, pixels baked
        self.undo.record_structural(before, after); // Apply (bake + close) is one undo entry
        true
    }

    /// Commit the ellipse (**Apply**) but KEEP the editor + its handles open for re-apply/reshape (the
    /// "Apply & Keep" button). The bake is ONE undo entry whose `after` keeps the editor open over the baked
    /// pixels — so it interleaves with the surrounding shape edits on the unified timeline. See
    /// [`PainterTool::curve_commit_keep`].
    pub fn ellipse_commit_keep(&mut self) -> bool {
        if !self.paint.ellipse.as_ref().is_some_and(|ed| ed.editing) {
            return false;
        }
        self.flush_shape_txn();
        let before = self.capture_shape_model(); // ellipse + live preview, pre-bake
        self.accumulate_offset(); // fold the slider into the accumulator + re-centre it (radii untouched)
        self.commit_drag_preview(); // the painted ellipse becomes permanent (no live preview left)
        let after = self.capture_shape_model(); // ellipse kept open (same radii), pixels baked, no preview
        self.undo.record_structural(before, after);
        self.paint.drag_preview = None;
        true
    }

    /// Cancel the ellipse (Esc): revert the painted preview to the pristine pixels and discard the
    /// session without an undo entry. Returns `true` when a session was open.
    pub fn ellipse_cancel(&mut self) -> bool {
        if self.paint.ellipse.is_none() {
            return false;
        }
        self.paint.ellipse = None;
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        self.paint.stroke_undo = None;
        self.paint.shape_offset_base_px = 0.0; // a cancelled shape must NOT carry its Offset to the next one
        self.paint.shape_offset_norm = 0.5;
        true
    }

    /// Drop the Ellipse session without touching pixels — for teardown where the canvas is replaced
    /// or cleared anyway (fresh source / tool deactivate), so a restore would read a stale buffer.
    pub(crate) fn ellipse_discard(&mut self) {
        self.paint.ellipse = None;
    }

    /// Convert the editing ellipse to an editable **closed** Bézier curve (anchors + `[in, out]` handles):
    /// the four cardinal points with the canonical `k = 0.5523` tangent handles so the curve reproduces the
    /// ellipse exactly; the editor closes the loop (bottom→right) itself, so NO duplicate seam anchor. The
    /// artist can then drag the points/handles. `None` unless editing. Drives the panel's **Edit** (E) button
    /// via [`PainterTool::convert_open_shape_to_curve`].
    #[allow(clippy::type_complexity)]
    pub(super) fn ellipse_to_curve(&self) -> Option<(Vec<[f32; 2]>, Vec<[[f32; 2]; 2]>)> {
        let ed = self.paint.ellipse.as_ref()?;
        if !ed.editing {
            return None;
        }
        const K: f32 = 0.552_285; // 4-arc cubic-Bézier circle constant
        let (c, u) = (ed.center, ed.u);
        let v = [-u[1], u[0]];
        let pt = |su: f32, sv: f32| {
            [
                c[0] + su * ed.rx * u[0] + sv * ed.ry * v[0],
                c[1] + su * ed.rx * u[1] + sv * ed.ry * v[1],
            ]
        };
        let (right, top, left, bottom) = (pt(1.0, 0.0), pt(0.0, 1.0), pt(-1.0, 0.0), pt(0.0, -1.0));
        let hv = [v[0] * K * ed.ry, v[1] * K * ed.ry]; // tangent handle at right/left (along ±v)
        let hu = [u[0] * K * ed.rx, u[1] * K * ed.rx]; // tangent handle at top/bottom (along ±u)
        let sub = |a: [f32; 2], b: [f32; 2]| [a[0] - b[0], a[1] - b[1]];
        let add = |a: [f32; 2], b: [f32; 2]| [a[0] + b[0], a[1] + b[1]];
        // CCW right→top→left→bottom (the closing bottom→right arc is the editor's closed-loop segment).
        let points = vec![right, top, left, bottom];
        let handles = vec![
            [sub(right, hv), add(right, hv)],
            [add(top, hu), sub(top, hu)],
            [add(left, hv), sub(left, hv)],
            [sub(bottom, hu), add(bottom, hu)],
        ];
        Some((points, handles))
    }

    /// Snapshot the Ellipse editor for the shell overlay — `None` until editing (the handles appear
    /// once the radius drag is released).
    #[must_use]
    pub fn ellipse_overlay(&self) -> Option<EllipseOverlay> {
        let ed = self.paint.ellipse.as_ref()?;
        if !ed.editing {
            return None;
        }
        // Display the OFFSET (grown/shrunk) radii, so the outline + handles + paint move together.
        let (erx, ery) = self.ellipse_offset_radii(ed.rx, ed.ry);
        let mut perimeter = Vec::new();
        ellipse_perimeter(ed.center, ed.u, erx, ery, &mut perimeter);
        let gap = self.paint.shape_grab_tol_px * ROTATE_GAP_FACTOR;
        Some(EllipseOverlay {
            perimeter,
            handles: ellipse_handles(ed.center, ed.u, erx, ery, gap),
            grabbed: ed.grabbed,
        })
    }

    // ── internals ────────────────────────────────────────────────────────────────────

    /// The Offset slider applied to the ellipse radii: `(rx, ry) + offset_px`, clamped to a paintable
    /// minimum. The single source for the fill, the overlay, and the bake.
    pub(super) fn ellipse_offset_radii(&self, rx: f32, ry: f32) -> (f32, f32) {
        let off = self.shape_offset_px();
        ((rx + off).max(0.5), (ry + off).max(0.5))
    }

    /// **Materialise** the EFFECTIVE offset into the ellipse radii + zero the offset — see
    /// [`Self::bake_curve_offset`]. Called before an edit gesture; Apply & Keep accumulates instead.
    pub(super) fn bake_ellipse_offset(&mut self) {
        let off = self.shape_offset_px();
        if off != 0.0
            && let Some(ed) = self.paint.ellipse.as_mut()
            && ed.editing
        {
            ed.rx = (ed.rx + off).max(0.5);
            ed.ry = (ed.ry + off).max(0.5);
            self.paint.shape_offset_base_px = 0.0;
            self.paint.shape_offset_norm = 0.5;
        }
    }

    /// Re-fill the painted preview from the editor's current ellipse (a fresh `Stroke` per fill, so
    /// the preview always reflects the live brush spec and is deterministic for identical params).
    pub(super) fn ellipse_refill(&mut self) {
        let Some(ed) = self.paint.ellipse.as_ref() else {
            return;
        };
        let (center, u, rx, ry, seed) = (ed.center, ed.u, ed.rx, ed.ry, ed.seed);
        let (erx, ery) = self.ellipse_offset_radii(rx, ry);
        let mut stroke = Stroke::new(self.paint.brush, self.paint.dynamics, seed);
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.fill_ellipse_preview(center, u, erx, ery, &mut dabs);
        self.restamp_shapes_preview(&dabs); // active shape + every parked shape onto one baseline
        self.paint.dabs = dabs;
    }
}

impl EllipseEditor {
    /// Capture the ellipse for the unified undo timeline; the transient `grabbed` is not stored.
    pub(super) fn to_state(&self) -> crate::undo::EllipseState {
        crate::undo::EllipseState {
            center: self.center,
            u: self.u,
            rx: self.rx,
            ry: self.ry,
            editing: self.editing,
            seed: self.seed,
        }
    }

    /// Rebuild the editor from a restored snapshot (grab idle) — the inverse of [`Self::to_state`].
    pub(super) fn from_state(s: crate::undo::EllipseState) -> Self {
        EllipseEditor {
            center: s.center,
            u: s.u,
            rx: s.rx,
            ry: s.ry,
            grabbed: None,
            editing: s.editing,
            seed: s.seed,
        }
    }
}

/// Apply a handle drag to the ellipse: the axis handles resize their axis (projection onto it,
/// symmetric about the centre), the rotation handle re-orients (orientation = direction to the
/// cursor), the centre handle moves it. Transcendental-free (dot / `unit_or`).
fn apply_handle_drag(ed: &mut EllipseEditor, handle: u8, pos: [f32; 2]) {
    let v = [-ed.u[1], ed.u[0]]; // local y-axis (left perpendicular of u)
    let rel = [pos[0] - ed.center[0], pos[1] - ed.center[1]];
    match handle {
        H_RIGHT | H_LEFT => ed.rx = dot(rel, ed.u).abs().max(MIN_AXIS_PX),
        H_TOP | H_BOTTOM => ed.ry = dot(rel, v).abs().max(MIN_AXIS_PX),
        H_ROTATE => {
            // The rotation handle points along +y (local up); the new up direction is the cursor
            // direction, and the x-axis is its right perpendicular. `u = perp(v)` keeps the frame
            // orthonormal. Lengths (rx/ry) are unchanged — pure rotation.
            let nv = unit_or(rel, v);
            ed.u = [nv[1], -nv[0]];
        }
        H_CENTER => ed.center = pos,
        _ => {}
    }
}

/// Handle positions for the ellipse `ed`, `[right, top, left, bottom, rotate, center]`. `gap` is how
/// far (image px) the rotation handle sits beyond the top.
fn ellipse_handles(c: [f32; 2], u: [f32; 2], rx: f32, ry: f32, gap: f32) -> [[f32; 2]; 6] {
    let v = [-u[1], u[0]];
    [
        [c[0] + u[0] * rx, c[1] + u[1] * rx],                 // right
        [c[0] + v[0] * ry, c[1] + v[1] * ry],                 // top
        [c[0] - u[0] * rx, c[1] - u[1] * rx],                 // left
        [c[0] - v[0] * ry, c[1] - v[1] * ry],                 // bottom
        [c[0] + v[0] * (ry + gap), c[1] + v[1] * (ry + gap)], // rotate
        c,                                                    // center
    ]
}

/// Index of the handle within `tol` px of `pos`. Axis + rotate handles (0..=4) are tested first so a
/// nearby axis handle wins over the centre on a small ellipse; the centre (5) is the fallback.
fn ellipse_hit(handles: &[[f32; 2]; 6], pos: [f32; 2], tol: f32) -> Option<u8> {
    let tol2 = tol * tol;
    let mut best = None;
    let mut bestd = tol2;
    for i in H_RIGHT..=H_ROTATE {
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

/// Squared distance between two points (no `sqrt` — comparisons only).
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
