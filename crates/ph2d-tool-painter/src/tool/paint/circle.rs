//! The **Circle** stroke method — an editable ellipse on the canvas (PH2D shape extension;
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

/// In-progress Circle session: the ellipse `(center, u, rx, ry)` plus the current grab + the
/// draw-vs-edit phase. Held in [`PaintState::circle`]; `None` when idle. `u` is the local x-axis
/// unit vector (orientation); the y-axis is its left perpendicular.
pub(super) struct CircleEditor {
    center: [f32; 2],
    u: [f32; 2],
    rx: f32,
    ry: f32,
    grabbed: Option<u8>,
    editing: bool,
    seed: u64,
}

/// A read-only snapshot of the Circle editor for the shell's on-canvas overlay (the outline + the
/// six handles). `None` until the radius drag is released (the handles appear with the ellipse).
pub struct CircleOverlay {
    /// The ellipse outline (image-space px) — drawn as the guide; matches the painted dabs.
    pub perimeter: Vec<[f32; 2]>,
    /// Handle positions (image-space px): `[right, top, left, bottom, rotate, center]`.
    pub handles: [[f32; 2]; 6],
    /// The handle being dragged (index into [`Self::handles`]), drawn highlighted.
    pub grabbed: Option<u8>,
}

impl PainterTool {
    /// Route a canvas pointer event through the Circle editor (the painter calls this instead of the
    /// generic paint path while the method is [`StrokeMethod::Circle`]). Returns `true` when consumed.
    pub(super) fn circle_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.circle_down(ev.pos),
            PointerPhase::Move => self.circle_move(ev.pos),
            PointerPhase::Up => self.circle_up(ev.pos),
            PointerPhase::Hover => false,
        }
    }

    /// Pointer-down: start a session (begin the centre-out radius drag) when idle; otherwise grab the
    /// handle under the cursor.
    fn circle_down(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        let Some(ed) = self.paint.circle.as_mut() else {
            // No session → snapshot for undo + begin the centre-out drag (centre = press point).
            let before = self.snapshot_model();
            self.paint.stroke_undo = Some(before);
            self.paint.drag_preview = None;
            let seed = self.paint.seed;
            self.paint.seed = self.paint.seed.wrapping_add(1);
            self.paint.circle = Some(CircleEditor {
                center: pos,
                u: [1.0, 0.0],
                rx: 0.0,
                ry: 0.0,
                grabbed: None,
                editing: false,
                seed,
            });
            return true;
        };
        if !ed.editing {
            return true; // mid radius-drag — ignore extra Downs
        }
        let handles = circle_handles(ed, tol * ROTATE_GAP_FACTOR);
        ed.grabbed = circle_hit(&handles, pos, tol);
        true
    }

    /// Pointer-move: drag the grabbed handle (edit) or stretch the radius (draw).
    fn circle_move(&mut self, pos: [f32; 2]) -> bool {
        let Some(ed) = self.paint.circle.as_mut() else {
            return false;
        };
        if !ed.editing {
            // Drawing: radius = distance from the centre; a circle (rx == ry), axis-aligned.
            let r = dist(ed.center, pos).max(MIN_AXIS_PX);
            ed.rx = r;
            ed.ry = r;
            self.circle_refill();
            return true;
        }
        let Some(g) = ed.grabbed else {
            return true; // editing but nothing grabbed — ignore stray moves
        };
        apply_handle_drag(ed, g, pos);
        self.circle_refill();
        true
    }

    /// Pointer-up: release a grab (edit), or finalize the radius drag into the editable ellipse.
    fn circle_up(&mut self, pos: [f32; 2]) -> bool {
        let Some(ed) = self.paint.circle.as_mut() else {
            return false;
        };
        if ed.editing {
            ed.grabbed = None;
            return true;
        }
        let r = dist(ed.center, pos).max(MIN_AXIS_PX);
        ed.rx = r;
        ed.ry = r;
        ed.editing = true;
        self.circle_refill();
        true
    }

    /// Commit the ellipse (Enter): keep the painted outline + push one undo entry. Returns `true`
    /// when a committable session was open (the shell consumes Enter only then).
    pub fn circle_commit(&mut self) -> bool {
        let Some(ed) = self.paint.circle.as_ref() else {
            return false;
        };
        if !ed.editing {
            return false; // mid radius-drag — nothing to bake yet
        }
        self.paint.circle = None;
        self.commit_drag_preview(); // drop the restore record → the painted ellipse stays
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
        true
    }

    /// Commit the ellipse (**Apply**) but KEEP the editor + its handles open for re-apply/reshape (the
    /// "Apply & Keep" button); re-baselines the undo + restore onto the now-baked canvas. See
    /// [`PainterTool::curve_commit_keep`].
    pub fn circle_commit_keep(&mut self) -> bool {
        if !self.paint.circle.as_ref().is_some_and(|ed| ed.editing) {
            return false;
        }
        self.commit_drag_preview();
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
        self.paint.stroke_undo = Some(self.snapshot_model());
        self.paint.drag_preview = None;
        true
    }

    /// Cancel the ellipse (Esc): revert the painted preview to the pristine pixels and discard the
    /// session without an undo entry. Returns `true` when a session was open.
    pub fn circle_cancel(&mut self) -> bool {
        if self.paint.circle.is_none() {
            return false;
        }
        self.paint.circle = None;
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        self.paint.stroke_undo = None;
        true
    }

    /// Drop the Circle session without touching pixels — for teardown where the canvas is replaced
    /// or cleared anyway (fresh source / tool deactivate), so a restore would read a stale buffer.
    pub(crate) fn circle_discard(&mut self) {
        self.paint.circle = None;
    }

    /// Snapshot the Circle editor for the shell overlay — `None` until editing (the handles appear
    /// once the radius drag is released).
    #[must_use]
    pub fn circle_overlay(&self) -> Option<CircleOverlay> {
        let ed = self.paint.circle.as_ref()?;
        if !ed.editing {
            return None;
        }
        let mut perimeter = Vec::new();
        ellipse_perimeter(ed.center, ed.u, ed.rx, ed.ry, &mut perimeter);
        Some(CircleOverlay {
            perimeter,
            handles: circle_handles(ed, self.paint.shape_grab_tol_px * ROTATE_GAP_FACTOR),
            grabbed: ed.grabbed,
        })
    }

    // ── internals ────────────────────────────────────────────────────────────────────

    /// Re-fill the painted preview from the editor's current ellipse (a fresh `Stroke` per fill, so
    /// the preview always reflects the live brush spec and is deterministic for identical params).
    pub(super) fn circle_refill(&mut self) {
        let Some(ed) = self.paint.circle.as_ref() else {
            return;
        };
        let (center, u, rx, ry, seed) = (ed.center, ed.u, ed.rx, ed.ry, ed.seed);
        let mut stroke = Stroke::new(self.paint.brush, self.paint.dynamics, seed);
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.fill_ellipse_preview(center, u, rx, ry, &mut dabs);
        self.stamp_drag_preview(&dabs);
        self.paint.dabs = dabs;
    }
}

/// Apply a handle drag to the ellipse: the axis handles resize their axis (projection onto it,
/// symmetric about the centre), the rotation handle re-orients (orientation = direction to the
/// cursor), the centre handle moves it. Transcendental-free (dot / `unit_or`).
fn apply_handle_drag(ed: &mut CircleEditor, handle: u8, pos: [f32; 2]) {
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
fn circle_handles(ed: &CircleEditor, gap: f32) -> [[f32; 2]; 6] {
    let c = ed.center;
    let u = ed.u;
    let v = [-u[1], u[0]];
    [
        [c[0] + u[0] * ed.rx, c[1] + u[1] * ed.rx], // right
        [c[0] + v[0] * ed.ry, c[1] + v[1] * ed.ry], // top
        [c[0] - u[0] * ed.rx, c[1] - u[1] * ed.rx], // left
        [c[0] - v[0] * ed.ry, c[1] - v[1] * ed.ry], // bottom
        [c[0] + v[0] * (ed.ry + gap), c[1] + v[1] * (ed.ry + gap)], // rotate
        c,                                          // center
    ]
}

/// Index of the handle within `tol` px of `pos`. Axis + rotate handles (0..=4) are tested first so a
/// nearby axis handle wins over the centre on a small ellipse; the centre (5) is the fallback.
fn circle_hit(handles: &[[f32; 2]; 6], pos: [f32; 2], tol: f32) -> Option<u8> {
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
