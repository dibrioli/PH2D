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
    grabbed: Option<usize>,
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

    /// Pointer-down: start a session (drop the first point) when idle; while drawing, close (click on the
    /// first point) / finish (click on the last point) / else drop a new point; while editing, grab a point.
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
                seed,
            });
            self.line_refill();
            self.commit_shape_txn(); // record: no shape → first point
            return true;
        }
        let drawing = !self.paint.line.as_ref().expect("line present").editing;
        if drawing {
            // Decide the action with a short borrow, then mutate + refill outside it.
            let (close, finish) = {
                let ed = self.paint.line.as_ref().expect("line present");
                let n = ed.points.len();
                let on_first = n >= 3 && dist(ed.points[0], pos) <= tol;
                let on_last = n >= 2 && dist(ed.points[n - 1], pos) <= tol;
                (on_first, on_last && !on_first)
            };
            // Each click — add a point, or finish/close — is ONE undo step (point-by-point undo/redo).
            self.begin_shape_txn();
            {
                let ed = self.paint.line.as_mut().expect("line present");
                if close {
                    ed.closed = true;
                    ed.editing = true;
                    ed.draft = None;
                } else if finish {
                    ed.editing = true;
                    ed.draft = None;
                } else {
                    ed.points.push(pos);
                }
            }
            self.line_refill();
            self.commit_shape_txn();
            return true;
        }
        // Editing: bracket a reshape gesture; grab the point under the cursor.
        self.begin_shape_txn();
        let ed = self.paint.line.as_mut().expect("line present");
        ed.grabbed = nearest_point(&ed.points, pos, tol);
        true
    }

    /// Pointer-move: rubber-band the next point (draw) or drag the grabbed point (edit).
    fn line_move(&mut self, pos: [f32; 2]) -> bool {
        let editing = match self.paint.line.as_ref() {
            Some(ed) => ed.editing,
            None => return false,
        };
        if !editing {
            self.paint.line.as_mut().expect("line present").draft = Some(pos);
            self.line_refill();
            return true;
        }
        let g = self.paint.line.as_ref().and_then(|ed| ed.grabbed);
        if let Some(g) = g {
            if let Some(ed) = self.paint.line.as_mut()
                && g < ed.points.len()
            {
                ed.points[g] = pos;
            }
            self.line_refill();
        }
        true
    }

    /// Pointer-up: release an edit grab (record the reshape). Drawing adds points on Down, so Up is inert.
    fn line_up(&mut self) -> bool {
        let released = match self.paint.line.as_mut() {
            Some(ed) if ed.editing => ed.grabbed.take().is_some(),
            Some(_) => return true,
            None => return false,
        };
        if released {
            self.commit_shape_txn();
        }
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
            seed: s.seed,
        }
    }
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

/// Euclidean distance between two points.
fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    dist2(a, b).sqrt()
}
