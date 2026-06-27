//! The **Curve** stroke method — a simplified on-canvas Bézier-ish path editor (PH2D's divergence
//! from Blender's curve-OBJECT workflow; `docs/Painter/`). A submodule of [`super`] (`paint`) so it
//! shares `PaintState`'s private access and the canvas helpers (save/restore/stamp); split out to
//! keep `paint.rs` under the workspace LOC cap.
//!
//! ## Interaction
//!
//! 1. **Draw** — press-drag-release a straight line (like Line). On release, three control points
//!    appear: the two endpoints + the midpoint.
//! 2. **Edit** — the session persists. Drag a control point to move it; click empty / near the curve
//!    to ADD a point there (and immediately drag it); the points auto-smooth with a Catmull-Rom
//!    spline ([`flatten_catmull_rom`]), so moving points sculpts curves of any shape. The painted
//!    stroke previews live along the spline (restore + re-stamp, no trail).
//! 3. **Delete** — select a point (click it) and press Delete (keeps ≥ 2 points).
//! 4. **Commit / cancel** — Enter bakes the stroke (one undo entry); Esc discards it.
//!
//! Keyboard (Enter / Esc / Delete) and the control-point/spine overlay live in the shell (the frozen
//! `CanvasPointer` carries no keys); this module exposes the verbs they drive.

use super::*;

use ph2d_painter_brush::{Stroke, flatten_catmull_rom, lazy_mouse_step};

/// Most control points a Curve session may hold — bounds the per-frame re-fill + hit-test.
const MAX_CURVE_POINTS: usize = 64;
/// Free Hand: minimum squared spacing (px²) between captured path points — keeps the raw capture bounded.
const MIN_CAPTURE_DIST_SQ: f32 = 4.0;
/// Free Hand: hard cap on raw captured points during the drag (before simplification).
const MAX_FREEHAND_CAPTURE: usize = 4096;
/// Free Hand: Douglas–Peucker perpendicular tolerance (px) when simplifying the capture to control points.
const FREEHAND_SIMPLIFY_TOL: f32 = 2.5;
/// In-progress Curve session: the editable control polygon (image-space px) + the current
/// selection / grab + the draw-vs-edit phase. Held in [`PaintState::curve`]; `None` when idle.
pub(super) struct CurveEditor {
    /// Control points along the curve, image-space px. One while drawing the initial line, then
    /// `≥ 2` once editing (the 3-point line, growing as points are added).
    points: Vec<[f32; 2]>,
    /// The selected (highlighted) point — the Delete target. `None` = nothing selected.
    selected: Option<usize>,
    /// The point being dragged this gesture (`Some` between a grab-Down and its Up).
    grabbed: Option<usize>,
    /// `false` while drawing the initial straight line (Down→Up), `true` once editing the points.
    editing: bool,
    /// **Free Hand** mode ([`StrokeMethod::FreeHand`]): the draw phase captures the freehand path into
    /// `points` (instead of a straight anchor→cursor line) and simplifies it to control points on release;
    /// from then on it's an ordinary editable curve. `false` for the plain [`StrokeMethod::Curve`].
    freehand: bool,
    /// Free Hand only: the lazy-mouse **stabilizer**'s filtered cursor position (lags the raw cursor by
    /// the brush `stabilizer` intensity), so the captured path is smoothed live as it's drawn.
    stabilized: [f32; 2],
    /// The initial line's press point — the curve's first endpoint + the draw-phase pivot.
    anchor: [f32; 2],
    /// Per-session jitter seed, fixed at begin so every re-fill of the same points is identical.
    seed: u64,
}

/// A read-only snapshot of the Curve editor for the shell's on-canvas overlay (control dots + the
/// auto-smoothed spine). `None` until the line is released (the chrome appears with the 3 points).
pub struct CurveOverlay {
    /// Control points (image-space px) — drawn as draggable dots.
    pub points: Vec<[f32; 2]>,
    /// The selected point index (drawn highlighted), if any.
    pub selected: Option<usize>,
    /// The flattened Catmull-Rom spine (image-space px) — drawn as the curve guide; matches the
    /// painted dabs exactly (same [`flatten_catmull_rom`]).
    pub spine: Vec<[f32; 2]>,
}

impl PainterTool {
    /// Route a canvas pointer event through the Curve editor (the painter calls this instead of the
    /// generic paint path while the active method is [`StrokeMethod::Curve`]). Returns `true` when
    /// the event was consumed (so the shell keeps the gesture open).
    pub(super) fn curve_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.curve_down(ev.pos),
            PointerPhase::Move => self.curve_move(ev.pos),
            PointerPhase::Up => self.curve_up(ev.pos),
            PointerPhase::Hover => false,
        }
    }

    /// Pointer-down: start a session (begin drawing the line) when idle; otherwise grab the control
    /// point under the cursor, or — on a miss — insert a new point there and grab it.
    fn curve_down(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        let Some(ed) = self.paint.curve.as_mut() else {
            // No session → snapshot for undo + begin drawing the initial straight line.
            let before = self.snapshot_model();
            self.paint.stroke_undo = Some(before);
            self.paint.drag_preview = None;
            let seed = self.paint.seed;
            self.paint.seed = self.paint.seed.wrapping_add(1);
            self.paint.curve = Some(CurveEditor {
                points: vec![pos],
                selected: None,
                grabbed: None,
                editing: false,
                freehand: matches!(self.paint.brush.stroke_method, StrokeMethod::FreeHand),
                stabilized: pos,
                anchor: pos,
                seed,
            });
            return true;
        };
        if !ed.editing {
            return true; // mid initial-line drag — ignore extra Downs
        }
        let need_refill = if let Some(i) = curve_hit(&ed.points, pos, tol) {
            ed.selected = Some(i);
            ed.grabbed = Some(i);
            false
        } else if ed.points.len() < MAX_CURVE_POINTS {
            let i = curve_insert_index(&ed.points, pos);
            ed.points.insert(i, pos);
            ed.selected = Some(i);
            ed.grabbed = Some(i);
            true
        } else {
            ed.selected = curve_nearest(&ed.points, pos);
            ed.grabbed = None;
            false
        };
        if need_refill {
            self.curve_refill();
        }
        true
    }

    /// Pointer-move: drag the grabbed point (edit) or stretch the initial line preview (draw).
    fn curve_move(&mut self, pos: [f32; 2]) -> bool {
        let stabilizer = self.paint.brush.stabilizer;
        let Some(ed) = self.paint.curve.as_mut() else {
            return false;
        };
        if !ed.editing {
            if ed.freehand {
                // Free Hand draw: lazy-mouse stabilize the cursor (the "how regular" knob), then accumulate
                // the filtered point (min-spacing gated) and preview the stroke along the captured path.
                ed.stabilized = lazy_mouse_step(ed.stabilized, pos, stabilizer);
                let p = ed.stabilized;
                let last = *ed.points.last().unwrap_or(&ed.anchor);
                if ed.points.len() < MAX_FREEHAND_CAPTURE && dist2(last, p) >= MIN_CAPTURE_DIST_SQ {
                    ed.points.push(p);
                }
                let pts = ed.points.clone();
                let seed = ed.seed;
                self.curve_fill(&pts, seed);
                return true;
            }
            // Curve draw: preview the straight line anchor→cursor as a 2-point curve (no trail).
            let pts = [ed.anchor, pos];
            let seed = ed.seed;
            self.curve_fill(&pts, seed);
            return true;
        }
        match ed.grabbed {
            Some(g) => ed.points[g] = pos,
            None => return true, // editing but nothing grabbed — ignore stray moves
        }
        self.curve_refill();
        true
    }

    /// Pointer-up: release a grab (edit), or finalize the initial line into a 3-point editable curve.
    fn curve_up(&mut self, pos: [f32; 2]) -> bool {
        let Some(ed) = self.paint.curve.as_mut() else {
            return false;
        };
        if ed.editing {
            ed.grabbed = None;
            return true;
        }
        if ed.freehand {
            // Free Hand released → append the final point, simplify the captured path into editable
            // control points (Douglas–Peucker, capped), and enter edit mode. Nothing pre-selected.
            let last = *ed.points.last().unwrap_or(&ed.anchor);
            if dist2(last, pos) >= MIN_CAPTURE_DIST_SQ {
                ed.points.push(pos);
            }
            if ed.points.len() >= 3 {
                ed.points = simplify_path(&ed.points, FREEHAND_SIMPLIFY_TOL);
            }
            if ed.points.len() < 2 {
                ed.points = vec![ed.anchor, pos]; // a tap/short flick → a degenerate 2-point curve
            }
            ed.selected = None;
        } else {
            // Curve line released → materialize start / midpoint / end. The midpoint is pre-selected so
            // the artist can bend the line immediately (drag it / Delete it).
            let a = ed.anchor;
            let mid = [(a[0] + pos[0]) * 0.5, (a[1] + pos[1]) * 0.5];
            ed.points = vec![a, mid, pos];
            ed.selected = Some(1);
        }
        ed.grabbed = None;
        ed.editing = true;
        self.curve_refill();
        true
    }

    /// Delete the selected control point (Delete key). Keeps `≥ 2` points so a curve always remains;
    /// returns `true` when a point was removed (the shell consumes the key only then).
    pub fn curve_delete_selected(&mut self) -> bool {
        let Some(ed) = self.paint.curve.as_mut() else {
            return false;
        };
        if !ed.editing {
            return false;
        }
        let Some(sel) = ed.selected else {
            return false;
        };
        if ed.points.len() <= 2 || sel >= ed.points.len() {
            return false;
        }
        ed.points.remove(sel);
        ed.selected = Some(sel.min(ed.points.len() - 1));
        ed.grabbed = None;
        self.curve_refill();
        true
    }

    /// Commit the curve (Enter): keep the painted dabs + push one undo entry for the whole session.
    /// Returns `true` when a committable session was open (the shell consumes Enter only then).
    pub fn curve_commit(&mut self) -> bool {
        let Some(ed) = self.paint.curve.as_ref() else {
            return false;
        };
        if !ed.editing {
            return false; // mid initial-drag — nothing to bake yet
        }
        self.paint.curve = None;
        self.commit_drag_preview(); // drop the restore record → the painted curve stays
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
        true
    }

    /// Commit the curve (Enter / **Apply**) but KEEP the editor + its control points open, so the same
    /// curve can be re-applied or reshaped (the "Apply & Keep" button). Bakes the current painted preview
    /// (one undo entry) and RE-BASELINES: the now-baked canvas becomes the base a further edit restores
    /// onto + the next apply is a fresh undo entry. Returns `true` when a committable session was open.
    pub fn curve_commit_keep(&mut self) -> bool {
        if !self.paint.curve.as_ref().is_some_and(|ed| ed.editing) {
            return false;
        }
        self.commit_drag_preview(); // drop the restore record → the painted curve stays baked
        if let Some(before) = self.paint.stroke_undo.take() {
            self.commit_structural_edit(before);
        }
        // Re-baseline onto the baked canvas; KEEP the editor (the curve persists as a re-applicable shape).
        self.paint.stroke_undo = Some(self.snapshot_model());
        self.paint.drag_preview = None;
        true
    }

    /// Commit whichever on-canvas shape editor is open but KEEP it editable (the **Apply & Keep** button)
    /// — the keep-mode aggregator paired with [`PainterTool::commit_open_shape`]. At most one is open.
    pub fn commit_open_shape_keep(&mut self) -> bool {
        self.curve_commit_keep() || self.circle_commit_keep() || self.polygon_commit_keep()
    }

    /// Cancel the curve (Esc): revert the painted preview to the pristine pixels and discard the
    /// session WITHOUT an undo entry (nothing was committed). Returns `true` when a session was open.
    pub fn curve_cancel(&mut self) -> bool {
        if self.paint.curve.is_none() {
            return false;
        }
        self.paint.curve = None;
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        self.paint.stroke_undo = None;
        true
    }

    /// Drop the Curve session without touching pixels — for teardown where the canvas is replaced
    /// or cleared anyway (fresh source / tool deactivate), so a restore would read a stale buffer.
    pub(crate) fn curve_discard(&mut self) {
        self.paint.curve = None;
        self.paint.drag_preview = None;
        self.paint.stroke_undo = None;
    }

    /// Snapshot the Curve editor for the shell overlay — `None` until editing (the chrome appears
    /// once the 3 control points exist).
    #[must_use]
    pub fn curve_overlay(&self) -> Option<CurveOverlay> {
        let ed = self.paint.curve.as_ref()?;
        if !ed.editing {
            return None;
        }
        let mut spine = Vec::new();
        flatten_catmull_rom(&ed.points, &mut spine);
        Some(CurveOverlay {
            points: ed.points.clone(),
            selected: ed.selected,
            spine,
        })
    }

    // ── internals ────────────────────────────────────────────────────────────────────

    /// Re-fill the painted preview from the editor's current control points (edit phase).
    fn curve_refill(&mut self) {
        let Some(ed) = self.paint.curve.as_ref() else {
            return;
        };
        if !ed.editing {
            return;
        }
        let pts = ed.points.clone();
        let seed = ed.seed;
        self.curve_fill(&pts, seed);
    }

    /// Build a fresh [`Stroke`] from the live brush spec + `seed`, fill the curve through `pts`, and
    /// route the dabs through the restore + re-stamp preview (so the reshaping curve leaves no trail
    /// and `commit` keeps the last fill). Fresh-per-fill ⇒ the preview always reflects the current
    /// brush (size / spacing / dash) and is deterministic for identical points.
    fn curve_fill(&mut self, pts: &[[f32; 2]], seed: u64) {
        let mut stroke = Stroke::new(self.paint.brush, self.paint.dynamics, seed);
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.fill_curve_preview(pts, &mut dabs);
        self.stamp_drag_preview(&dabs);
        self.paint.dabs = dabs;
    }
}

/// Squared distance between two points (the Free Hand capture's min-spacing gate).
fn dist2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

/// Simplify a captured freehand path into control points via **Douglas–Peucker** (perpendicular-distance,
/// transcendental-free via [`dist2_point_seg`] — HR-5), keeping the endpoints. If the result still exceeds
/// [`MAX_CURVE_POINTS`] it is uniformly decimated to fit (the true endpoints preserved).
fn simplify_path(pts: &[[f32; 2]], tol: f32) -> Vec<[f32; 2]> {
    let n = pts.len();
    if n <= 2 {
        return pts.to_vec();
    }
    let tol2 = tol * tol;
    let mut keep = vec![false; n];
    keep[0] = true;
    keep[n - 1] = true;
    let mut stack = vec![(0usize, n - 1)];
    while let Some((a, b)) = stack.pop() {
        if b <= a + 1 {
            continue;
        }
        let (mut maxd, mut split) = (tol2, None);
        for (i, p) in pts.iter().enumerate().take(b).skip(a + 1) {
            let d = dist2_point_seg(*p, pts[a], pts[b]);
            if d > maxd {
                maxd = d;
                split = Some(i);
            }
        }
        if let Some(i) = split {
            keep[i] = true;
            stack.push((a, i));
            stack.push((i, b));
        }
    }
    let out: Vec<[f32; 2]> = (0..n).filter(|&i| keep[i]).map(|i| pts[i]).collect();
    if out.len() <= MAX_CURVE_POINTS {
        return out;
    }
    // Too many even after DP → uniformly decimate the interior, keeping the true endpoints.
    let last = out.len() - 1;
    let step = out.len() as f32 / MAX_CURVE_POINTS as f32;
    let mut capped: Vec<[f32; 2]> = (0..MAX_CURVE_POINTS)
        .map(|i| out[((i as f32 * step) as usize).min(last)])
        .collect();
    capped[MAX_CURVE_POINTS - 1] = out[last];
    capped
}

/// Index of the control point within `tol` px of `pos` (closest wins), or `None` on a miss.
fn curve_hit(pts: &[[f32; 2]], pos: [f32; 2], tol: f32) -> Option<usize> {
    let mut best = None;
    let mut bestd = tol * tol;
    for (i, p) in pts.iter().enumerate() {
        let dx = p[0] - pos[0];
        let dy = p[1] - pos[1];
        let d = dx * dx + dy * dy;
        if d <= bestd {
            bestd = d;
            best = Some(i);
        }
    }
    best
}

/// Index of the nearest control point (no tolerance) — the select fallback at the point cap.
fn curve_nearest(pts: &[[f32; 2]], pos: [f32; 2]) -> Option<usize> {
    let mut best = None;
    let mut bestd = f32::INFINITY;
    for (i, p) in pts.iter().enumerate() {
        let dx = p[0] - pos[0];
        let dy = p[1] - pos[1];
        let d = dx * dx + dy * dy;
        if d < bestd {
            bestd = d;
            best = Some(i);
        }
    }
    best
}

/// Insert position for a new point at `pos`: just after the segment whose body it lies closest to,
/// so a click on/near the curve subdivides it there (and a click in open space drops into the
/// nearest run, bending the curve toward it).
fn curve_insert_index(pts: &[[f32; 2]], pos: [f32; 2]) -> usize {
    if pts.len() < 2 {
        return pts.len();
    }
    let mut best = 0usize;
    let mut bestd = f32::INFINITY;
    for i in 0..pts.len() - 1 {
        let d = dist2_point_seg(pos, pts[i], pts[i + 1]);
        if d < bestd {
            bestd = d;
            best = i;
        }
    }
    best + 1
}

/// Squared distance from `p` to the segment `a→b` — transcendental-free (projection + clamp), so the
/// editor geometry stays bit-deterministic across platforms (HR-5).
fn dist2_point_seg(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let abx = b[0] - a[0];
    let aby = b[1] - a[1];
    let apx = p[0] - a[0];
    let apy = p[1] - a[1];
    let denom = abx * abx + aby * aby;
    let t = if denom > 0.0 {
        ((apx * abx + apy * aby) / denom).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let cx = a[0] + abx * t;
    let cy = a[1] + aby * t;
    let dx = p[0] - cx;
    let dy = p[1] - cy;
    dx * dx + dy * dy
}
