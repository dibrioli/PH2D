//! The **Curve** stroke method — a simplified on-canvas Bézier-ish path editor (PH2D's divergence from
//! Blender's curve-OBJECT workflow; `docs/Painter/`). A submodule of [`super`] (`paint`) so it shares
//! `PaintState`'s private access + the canvas helpers; split out to keep `paint.rs` under the LOC cap.
//!
//! **Draw** a straight line (press-drag-release) → 3 control points. **Edit**: drag a point to move, drag
//! its tangent handles to sculpt, click near the curve to ADD a point; handle kinds ([`curve_handle`]) drive
//! smoothing. **Delete** the selected; **Commit** (Enter) bakes one undo entry, **cancel** (Esc) discards.
//! Keyboard + overlay live in the shell; this module exposes the verbs they drive.

use super::curve_handle::{self, HandleKind};
use super::curve_tangent;
use super::*;

use super::curve_geom;
use super::curve_gizmo;
use super::curve_offset;
use ph2d_painter_brush::{Stroke, lazy_mouse_step};

/// Most control points a Curve session may hold — bounds the per-frame re-fill + hit-test.
const MAX_CURVE_POINTS: usize = 64;
/// Free Hand: cubic-fit error tolerance (px) — the max the fitted curve may deviate from the captured
/// path. Larger ⇒ fewer, cleaner control points. Sits a touch above the stabilizer's residual jitter so
/// a steady stroke collapses to a handful of anchors rather than tracing every wobble.
const FREEHAND_FIT_ERROR: f32 = 4.0;
/// Free Hand: minimum squared spacing (px²) between captured path points — keeps the raw capture bounded.
const MIN_CAPTURE_DIST_SQ: f32 = 4.0;
/// Free Hand: hard cap on raw captured points during the drag (before the fit).
const MAX_FREEHAND_CAPTURE: usize = 4096;
/// In-progress Curve session: the editable control polygon (image-space px) + the current
/// selection / grab + the draw-vs-edit phase. Held in [`PaintState::curve`]; `None` when idle.
pub(super) struct CurveEditor {
    /// Control points (image-space px): one while drawing the initial line, then `≥ 2` once editing.
    pub(super) points: Vec<[f32; 2]>,
    /// Per-anchor **Bézier handles** `[in, out]` (absolute px), parallel to `points` once editing; the
    /// `kinds` entry drives updates. EMPTY transiently (draw-phase preview) ⇒ Catmull-Rom in `flatten_spine`.
    handles: Vec<[[f32; 2]; 2]>,
    /// Per-anchor **handle kind** (Free / Aligned / Vector / Auto), parallel to `points` once editing. Derived
    /// (Auto / Vector) are recomputed by [`curve_handle::rebuild`] each edit; manual (Aligned / Free) keep the
    /// artist's tangents. Empty during the draw phase.
    kinds: Vec<HandleKind>,
    /// The selected (highlighted) point — the Delete target. `None` = nothing selected.
    pub(super) selected: Option<usize>,
    /// The point being dragged this gesture (`Some` between a grab-Down and its Up).
    grabbed: Option<usize>,
    /// The **tangent handle** dragged this gesture: `(anchor, is_out)` picking the `[in, out]` slot. `Some`
    /// between its grab-Down and Up; takes precedence over `grabbed`.
    grabbed_handle: Option<(usize, bool)>,
    /// The active **whole-curve transform** drag (the bbox gizmo: move / scale / rotate). `Some` between its
    /// grab-Down and Up; takes precedence over anchor / tangent grabs.
    gizmo: Option<curve_gizmo::GizmoGrab>,
    /// `false` while drawing the initial straight line (Down→Up), `true` once editing the points.
    pub(super) editing: bool,
    /// `true` when the curve is a **closed loop** (converted Circle / Polygon): the spine / fill / offset wrap
    /// last → first, no duplicate seam anchor.
    pub(super) closed: bool,
    /// **Free Hand** mode ([`StrokeMethod::FreeHand`]): the draw phase captures the path into `points` +
    /// simplifies to control points on release, then an ordinary editable curve. `false` for plain Curve.
    freehand: bool,
    /// Free Hand only: the lazy-mouse **stabilizer**'s filtered cursor (lags raw by the brush `stabilizer`).
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
    /// The flattened spine (px) — the curve guide; matches the painted dabs exactly (same `flatten_spine`).
    pub spine: Vec<[f32; 2]>,
    /// The selected anchor's draggable Bézier **tangent handles**, or `None` when none selected / collapsed.
    pub tangents: Option<TangentHandles>,
    /// The selected anchor's **handle kind** wire `u8` (`0=Free 1=Aligned 2=Vector 3=Auto`), or `None` —
    /// lets the shell mark the active menu item.
    pub selected_kind: Option<u8>,
    /// The whole-curve **transform gizmo** (bbox move / scale / rotate), or `None` when too small to frame.
    pub transform_gizmo: Option<TransformGizmo>,
}

impl PainterTool {
    /// Route a canvas pointer through the Curve editor (called instead of the generic paint path while the
    /// method is [`StrokeMethod::Curve`]). `true` when consumed (the shell keeps the gesture open).
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
        self.bake_curve_offset(); // an edit gesture locks in any live Offset (so hit-test = displayed dots)
        let Some(ed) = self.paint.curve.as_mut() else {
            // No session → snapshot for undo + begin drawing the initial straight line.
            let before = self.snapshot_model();
            self.paint.stroke_undo = Some(before);
            self.paint.drag_preview = None;
            let seed = self.paint.seed;
            self.paint.seed = self.paint.seed.wrapping_add(1);
            self.paint.curve = Some(CurveEditor {
                points: vec![pos],
                handles: Vec::new(),
                kinds: Vec::new(),
                selected: None,
                grabbed: None,
                grabbed_handle: None,
                gizmo: None,
                editing: false,
                closed: false, // authored / Free Hand curves are open
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
        let tangent = ed.selected.and_then(|s| {
            curve_tangent::tangent_hit(&ed.points, &ed.handles, s, pos, tol, ed.closed)
        });
        let need_refill = if let Some(i) = curve_geom::curve_hit(&ed.points, pos, tol) {
            ed.selected = Some(i);
            ed.grabbed = Some(i);
            false
        } else if let Some(h) = tangent {
            ed.grabbed_handle = Some(h); // a tangent handle off the selected anchor — grab it, not the curve
            false
        } else if let Some(g) = curve_gizmo::grab(&ed.points, &ed.handles, pos, tol) {
            ed.gizmo = Some(g); // a transform-gizmo handle (box corner / edge / centre / rotate ring)
            false
        } else if ed.points.len() < MAX_CURVE_POINTS {
            // Subdivide the curve at the NEAREST point on it (incl. the closing seam for a converted
            // Circle / Polygon) via a de Casteljau split — the new anchor lands on the curve and the shape
            // is unchanged. Parallel handles ⇒ update the segment's neighbour handles + splice the new one;
            // a degenerate (handle-less) draw-phase curve just takes the projected point.
            let ins = curve_geom::curve_insert(&ed.points, &ed.handles, ed.closed, pos);
            let parallel = ed.handles.len() == ed.points.len();
            if parallel {
                ed.handles[ins.prev_idx][1] = ins.prev_out; // pre-insert indices are still valid
                ed.handles[ins.next_idx][0] = ins.next_in;
            }
            ed.points.insert(ins.index, ins.anchor);
            // A split point is smooth (its handles are collinear); rebuild keeps it (Aligned is manual).
            ed.kinds.insert(ins.index, HandleKind::Aligned);
            if parallel {
                ed.handles.insert(ins.index, ins.handles);
            }
            curve_handle::rebuild(&ed.points, &ed.kinds, &mut ed.handles, ed.closed);
            ed.selected = Some(ins.index);
            ed.grabbed = Some(ins.index);
            true
        } else {
            ed.selected = curve_geom::curve_nearest(&ed.points, pos);
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
                // Free Hand draw: lazy-mouse stabilize, accumulate the filtered point (min-spacing gated),
                // preview along the captured path.
                ed.stabilized = lazy_mouse_step(ed.stabilized, pos, stabilizer);
                let p = ed.stabilized;
                let last = *ed.points.last().unwrap_or(&ed.anchor);
                if ed.points.len() < MAX_FREEHAND_CAPTURE
                    && curve_geom::dist2(last, p) >= MIN_CAPTURE_DIST_SQ
                {
                    ed.points.push(p);
                }
                let pts = ed.points.clone();
                let seed = ed.seed;
                self.curve_fill(&pts, &[], false, seed); // raw dense capture → Catmull-Rom preview
                return true;
            }
            // Curve draw: preview the straight line anchor→cursor as a 2-point curve (no trail).
            let pts = [ed.anchor, pos];
            let seed = ed.seed;
            self.curve_fill(&pts, &[], false, seed);
            return true;
        }
        if let Some(grab) = &ed.gizmo {
            // Whole-curve transform: re-map the grab snapshot, then rebuild derived (Auto / Vector) handles
            // for the new geometry — manual handles already rode the affine transform.
            let (p, h) = curve_gizmo::apply(grab, pos);
            ed.points = p;
            ed.handles = h;
            let closed = ed.closed;
            curve_handle::rebuild(&ed.points, &ed.kinds, &mut ed.handles, closed);
            self.curve_refill();
            return true;
        }
        if let Some((i, is_out)) = ed.grabbed_handle {
            // Dragging a tangent: a derived (Auto/Vector) point becomes Aligned; Aligned/Symmetric mirror
            // the opposite (collinear, keep-length / equal-length); Free moves only the dragged side.
            if i < ed.handles.len() && i < ed.points.len() && i < ed.kinds.len() {
                if matches!(ed.kinds[i], HandleKind::Auto | HandleKind::Vector) {
                    ed.kinds[i] = HandleKind::Aligned;
                }
                let anchor = ed.points[i];
                ed.handles[i][usize::from(is_out)] = pos;
                if let Some(eq) = ed.kinds[i].mirror_mode() {
                    curve_tangent::mirror_tangent(&mut ed.handles[i], anchor, is_out, eq);
                }
            }
        } else {
            match ed.grabbed {
                Some(g) => {
                    let old = ed.points[g];
                    ed.points[g] = pos;
                    // Carry the moved anchor's MANUAL handles with it (rigid); rebuild re-derives the rest.
                    if ed.kinds.get(g).is_some_and(|k| k.is_manual())
                        && let Some(h) = ed.handles.get_mut(g)
                    {
                        let d = [pos[0] - old[0], pos[1] - old[1]];
                        *h = [
                            [h[0][0] + d[0], h[0][1] + d[1]],
                            [h[1][0] + d[0], h[1][1] + d[1]],
                        ];
                    }
                    curve_handle::rebuild(&ed.points, &ed.kinds, &mut ed.handles, ed.closed);
                }
                None => return true, // editing but nothing grabbed — ignore stray moves
            }
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
            ed.grabbed_handle = None;
            ed.gizmo = None;
            return true;
        }
        if ed.freehand {
            // Free Hand released → append the final point, FIT the path to cubic Béziers (Schneider) keeping
            // the anchors + tangent handles so flattening reproduces the stroke faithfully.
            let last = *ed.points.last().unwrap_or(&ed.anchor);
            if curve_geom::dist2(last, pos) >= MIN_CAPTURE_DIST_SQ {
                ed.points.push(pos);
            }
            if ed.points.len() >= 3 {
                let fit = ph2d_painter_brush::fit_curve(&ed.points, FREEHAND_FIT_ERROR);
                if fit.anchors.len() <= MAX_CURVE_POINTS {
                    ed.points = fit.anchors;
                    ed.handles = fit.handles; // Bézier mode — faithful tangents
                } else {
                    // Busy scribble over the cap → decimate to anchors, drop handles (rebuild re-derives).
                    ed.points = curve_geom::cap_curve_points(fit.anchors, MAX_CURVE_POINTS);
                    ed.handles = Vec::new();
                }
            }
            if ed.points.len() < 2 {
                ed.points = vec![ed.anchor, pos]; // a tap/short flick → a degenerate 2-point curve
                ed.handles = Vec::new();
            }
            // The fitted tangents are the artist's smooth handles → Aligned (manual, kept across edits).
            ed.kinds = vec![HandleKind::Aligned; ed.points.len()];
            ed.selected = None;
        } else {
            // Curve line released → start / midpoint / end (mid pre-selected to bend); Auto kind until the menu.
            let a = ed.anchor;
            let mid = [(a[0] + pos[0]) * 0.5, (a[1] + pos[1]) * 0.5];
            ed.points = vec![a, mid, pos];
            ed.kinds = vec![HandleKind::Auto; 3];
            ed.selected = Some(1);
        }
        curve_handle::rebuild(&ed.points, &ed.kinds, &mut ed.handles, ed.closed);
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
        if sel < ed.handles.len() {
            ed.handles.remove(sel); // keep the parallel handles + kinds invariants
        }
        if sel < ed.kinds.len() {
            ed.kinds.remove(sel);
        }
        curve_handle::rebuild(&ed.points, &ed.kinds, &mut ed.handles, ed.closed); // re-derive Auto/Vector neighbours
        ed.selected = Some(sel.min(ed.points.len() - 1));
        ed.grabbed = None;
        self.curve_refill();
        true
    }

    /// **Simplify** the editable curve (the Simplify button): re-fit it to a clean minimal control polygon
    /// via the Free Hand fit, collapsing a dense offset reconstruction (or any busy curve) back to a faithful
    /// handful, then re-fill the preview. `true` when a curve editor was open and the fit applied.
    pub fn curve_simplify(&mut self) -> bool {
        let Some(ed) = self.paint.curve.as_ref().filter(|e| e.editing) else {
            return false;
        };
        let Some((p, h)) = curve_geom::simplify_curve(
            &ed.points,
            &ed.handles,
            ed.closed,
            FREEHAND_FIT_ERROR,
            MAX_CURVE_POINTS,
        ) else {
            return false;
        };
        let ed = self.paint.curve.as_mut().expect("curve present");
        ed.kinds = vec![HandleKind::Aligned; p.len()];
        (ed.points, ed.handles, ed.selected) = (p, h, None);
        curve_handle::rebuild(&ed.points, &ed.kinds, &mut ed.handles, ed.closed);
        self.curve_refill();
        true
    }

    /// Convert an open **Circle / Polygon** into an editable Bézier **Curve** (the panel's **Edit** / E
    /// button): take the shape's faithful anchors + handles, drop the shape editor, switch the method to Curve
    /// so pointers route here. Undo + drag-preview carry over (bakes/undoes as one). `false` if none open.
    pub(crate) fn convert_open_shape_to_curve(&mut self) -> bool {
        // Bake any live Offset into the shape's radii FIRST (resets the slider) so the conversion reads the
        // displayed shape, not a doubly-offset one. At most one editor is open; each bake no-ops otherwise.
        self.bake_circle_offset();
        self.bake_polygon_offset();
        // Circle arcs are smooth (Aligned); polygon corners sharp (Free). Both keep explicit (manual) handles.
        let Some((points, handles, kind)) = self
            .circle_to_curve()
            .map(|(p, h)| (p, h, HandleKind::Aligned))
            .or_else(|| {
                self.polygon_to_curve()
                    .map(|(p, h)| (p, h, HandleKind::Free))
            })
        else {
            return false;
        };
        let anchor = points[0];
        let kinds = vec![kind; points.len()];
        let seed = self.paint.seed;
        self.paint.seed = self.paint.seed.wrapping_add(1);
        self.paint.circle = None;
        self.paint.polygon = None;
        self.paint.brush.stroke_method = StrokeMethod::Curve; // route future pointers to the curve editor
        self.paint.curve = Some(CurveEditor {
            points,
            handles,
            kinds,
            selected: None,
            grabbed: None,
            grabbed_handle: None,
            gizmo: None,
            editing: true,
            closed: true,   // a converted Circle / Polygon is a closed loop
            freehand: true, // keep the explicit handles (drag translates them), like the Free Hand fit
            stabilized: anchor,
            anchor,
            seed,
        });
        self.curve_refill();
        true
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
        // Offset the CONTROL geometry (handles recalculated, not deformed), so the dots + tangents + spine
        // guide all move WITH the paint and stay consistent on the offset curve (Enio 2026-06-28).
        let (points, handles, origin) = curve_offset::offset_curve_refined(
            &ed.points,
            &ed.handles,
            self.shape_offset_px(),
            ed.closed,
        );
        let mut spine = Vec::new();
        curve_geom::flatten_spine(&points, &handles, ed.closed, &mut spine);
        if self.paint.offset_trim {
            spine = curve_offset::trim_self_intersections(&spine);
        }
        // The reconstruction inserts anchors, so remap the selected index onto its densified position.
        let osel = ed
            .selected
            .and_then(|s| origin.iter().position(|o| *o == Some(s)));
        let tangents = osel.and_then(|s| {
            curve_tangent::build_tangents(
                &points,
                &handles,
                s,
                ed.grabbed_handle,
                self.paint.shape_grab_tol_px,
                ed.closed,
            )
        });
        let selected_kind = ed.selected.and_then(|s| ed.kinds.get(s)).map(|k| k.wire());
        let (grabbed, rotating) = match &ed.gizmo {
            Some(g) => (Some(g.handle), curve_gizmo::is_rotate(g)),
            None => (None, false),
        };
        let transform_gizmo =
            curve_gizmo::overlay(&points, self.paint.shape_grab_tol_px, grabbed, rotating);
        Some(CurveOverlay {
            points,
            selected: osel,
            spine,
            tangents,
            selected_kind,
            transform_gizmo,
        })
    }

    /// Set the selected point's **handle kind** from the right-click menu's wire `u8` (`0 = Free` /
    /// `1 = Aligned` / `2 = Vector` / `3 = Auto`); re-derives the geometry (a sharp point switched to a
    /// manual kind is seeded with smooth tangents to drag) and re-fills. `true` if a point accepted it.
    pub fn set_curve_handle_kind(&mut self, wire: u8) -> bool {
        let Some(kind) = HandleKind::from_wire(wire) else {
            return false;
        };
        let Some(ed) = self.paint.curve.as_mut() else {
            return false;
        };
        if !ed.editing {
            return false;
        }
        let Some(sel) = ed.selected.filter(|&s| s < ed.points.len()) else {
            return false;
        };
        if ed.kinds.len() != ed.points.len() {
            ed.kinds = vec![HandleKind::Auto; ed.points.len()];
        }
        ed.kinds[sel] = kind;
        // Switching a sharp (collapsed) point to a manual kind: seed a visible tangent to drag. Closed-aware,
        // so a converted Polygon / Circle seam anchor gets BOTH arms (not the one-armed open-curve seed).
        if kind.is_manual() && curve_handle::is_collapsed(ed.handles.get(sel), ed.points[sel]) {
            let auto = curve_handle::auto_handles(&ed.points, ed.closed);
            if let (Some(h), Some(a)) = (ed.handles.get_mut(sel), auto.get(sel)) {
                *h = *a;
            }
        }
        if let Some(true) = kind.mirror_mode() {
            curve_tangent::mirror_tangent(&mut ed.handles[sel], ed.points[sel], true, true); // enforce Symmetric
        }
        curve_handle::rebuild(&ed.points, &ed.kinds, &mut ed.handles, ed.closed);
        self.curve_refill();
        true
    }

    /// Re-fill the painted preview from the editor's current control points + handles (edit phase).
    pub(super) fn curve_refill(&mut self) {
        let Some(ed) = self.paint.curve.as_ref() else {
            return;
        };
        if !ed.editing {
            return;
        }
        let pts = ed.points.clone();
        let handles = ed.handles.clone();
        let closed = ed.closed;
        let seed = ed.seed;
        self.curve_fill(&pts, &handles, closed, seed);
    }

    /// Build a fresh [`Stroke`] from the live brush + `seed`, flatten the curve (Bézier when `handles`
    /// present, else Catmull-Rom), fill spaced dabs along the spine, and route them through restore +
    /// re-stamp (no trail; `commit` keeps the last fill). Fresh-per-fill ⇒ deterministic for equal input.
    fn curve_fill(&mut self, pts: &[[f32; 2]], handles: &[[[f32; 2]; 2]], closed: bool, seed: u64) {
        // Offset the CONTROL geometry (handles recalculated), so the painted spine matches the overlay guide
        // and carries no deformation; flatten the offset curve to the dab spine.
        let (pts, handles, _) =
            curve_offset::offset_curve_refined(pts, handles, self.shape_offset_px(), closed);
        let mut spine = Vec::new();
        curve_geom::flatten_spine(&pts, &handles, closed, &mut spine);
        if self.paint.offset_trim {
            spine = curve_offset::trim_self_intersections(&spine);
        }
        let mut stroke = Stroke::new(self.paint.brush, self.paint.dynamics, seed);
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.fill_polyline_preview(&spine, &mut dabs);
        self.stamp_drag_preview(&dabs);
        self.paint.dabs = dabs;
    }

    /// **Bake** the live Offset into the curve's control geometry (anchors moved, handles recalculated, the
    /// reconstruction's extra anchors folded in) and reset the slider. A no-op without an offset / open
    /// editor. Called before any edit gesture (hit-test = displayed dots) and on Apply & Keep.
    pub(super) fn bake_curve_offset(&mut self) {
        let off = self.shape_offset_px();
        if off != 0.0
            && let Some(ed) = self.paint.curve.as_mut().filter(|ed| ed.editing)
        {
            let (p, h, origin) =
                curve_offset::offset_curve_refined(&ed.points, &ed.handles, off, ed.closed);
            // Carry kinds (original anchors keep theirs; inserted points = Free) + remap the selection.
            let kinds: Vec<HandleKind> = origin
                .iter()
                .map(|o| {
                    o.and_then(|i| ed.kinds.get(i).copied())
                        .unwrap_or(HandleKind::Free)
                })
                .collect();
            ed.selected = ed
                .selected
                .and_then(|s| origin.iter().position(|o| *o == Some(s)));
            ed.points = p;
            ed.handles = h;
            ed.kinds = kinds;
            self.paint.shape_offset_norm = 0.5;
        }
    }
}
