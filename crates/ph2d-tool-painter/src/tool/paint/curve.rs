//! The **Arc** stroke method (né Curve) — a simplified on-canvas Bézier-ish path editor (PH2D's
//! divergence from Blender's curve-OBJECT workflow; `docs/Painter/`). A submodule of [`super`] (`paint`)
//! so it shares `PaintState`'s private access + the canvas helpers; split to keep `paint.rs` under the
//! LOC cap.
//!
//! **Draw** (press-drag-release) → 3 control points with a subtle perpendicular bow ([`ARC_BOW`]) — a
//! fresh Arc, not a straight line. **Edit**: drag a point to move, drag its tangent handles to sculpt,
//! click ON the curve to ADD a point (never in empty space); handle kinds ([`curve_handle`]) drive
//! smoothing. **Delete** the selected; **Commit** (Enter) bakes one undo entry, **cancel** (Esc)
//! discards. Keyboard + overlay live in the shell; this module exposes the verbs they drive.
//!
//! The editable state + the pure editing ops live in the shared [`curve_model::CurveModel`] (owned here in
//! `self.paint.curve` AND by the selection Convert-to-Curve editor) — so the two behave identically; this
//! module keeps only the stroke-specific extras (draw phase, transform gizmo, Offset, undo txns, refill).

use super::curve_handle::{self, HandleKind};
use super::curve_model::{CurveGrab, CurveModel};
use super::curve_tangent;
use super::*;

use super::curve_geom;
use super::curve_gizmo;
use super::curve_offset;
use ph2d_painter_brush::{Stroke, lazy_mouse_step};

/// Most control points the Free Hand fit / Simplify aims for — the SPARSE direction.
pub(super) const MAX_CURVE_POINTS: usize = 64;
/// Convert-to-Curve is the DENSE direction (Enio 2026-07-05): extreme precision + many manipulation
/// points at conversion; the artist reduces later with Simplify. Fit tolerance for traced outlines —
/// tight, but above the ~0.5 px marching-squares stair amplitude so mask traces don't zigzag.
pub(super) const CONVERT_FIT_ERROR: f32 = 1.0;
/// Hard cap on a converted curve's anchors (bounds the per-frame re-fill / hit-test / overlay).
pub(super) const MAX_CONVERT_POINTS: usize = 512;
/// Target anchor spacing (image px of arc length) the conversion densifies to — "múltiplos pontos".
pub(super) const CONVERT_ANCHOR_SPACING_PX: f32 = 16.0;
/// The **Arc** method's initial bow: the middle anchor's perpendicular offset as a fraction of the
/// drag chord. Subtle by design — the fresh shape reads as an arc, and the pre-selected mid point
/// bends it further either way.
const ARC_BOW: f32 = 0.15;
/// Free Hand: cubic-fit error tolerance (px) — max deviation of the fitted curve from the captured path.
pub(super) const FREEHAND_FIT_ERROR: f32 = 4.0;
/// Free Hand: minimum squared spacing (px²) between captured path points — keeps the raw capture bounded.
const MIN_CAPTURE_DIST_SQ: f32 = 4.0;
/// Free Hand: hard cap on raw captured points during the drag (before the fit).
const MAX_FREEHAND_CAPTURE: usize = 4096;
/// In-progress Curve session: the shared editable [`CurveModel`] + the stroke-specific grab / phase state.
/// In [`PaintState::curve`].
pub(super) struct CurveEditor {
    /// The shared editing core — control points + parallel `[in, out]` handles + kinds + selection + closed.
    /// EMPTY handles while drawing the initial line; `≥ 2` points + parallel handles once editing.
    pub(super) model: CurveModel,
    /// The active grab this gesture: an anchor or a tangent handle (`Some` between a grab-Down and its Up).
    grab: Option<CurveGrab>,
    /// The **whole-curve transform** drag (bbox gizmo: move / scale / rotate). `Some` Down→Up; beats grabs.
    gizmo: Option<curve_gizmo::GizmoGrab>,
    /// `false` while drawing the initial straight line (Down→Up), `true` once editing the points.
    pub(super) editing: bool,
    /// **Free Hand** mode: the draw phase captures the path + simplifies on release. `false` for plain Curve.
    freehand: bool,
    /// Free Hand only: the lazy-mouse **stabilizer**'s filtered cursor (lags raw by the brush `stabilizer`).
    stabilized: [f32; 2],
    /// The initial line's press point — the curve's first endpoint + the draw-phase pivot.
    anchor: [f32; 2],
    /// Per-session jitter seed, fixed at begin so every re-fill of the same points is identical.
    seed: u64,
    /// `true` once the user inserted a point — gates Simplify for Curve / converted shapes (Free Hand: always).
    pub(super) added_point: bool,
}

/// A read-only snapshot of the Curve editor for the shell's overlay (control dots + auto-smoothed spine).
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
    /// Route a canvas pointer through the Curve editor (vs the generic paint path). `true` when consumed.
    pub(super) fn curve_pointer(&mut self, ev: CanvasPointer) -> bool {
        match ev.phase {
            PointerPhase::Down => self.curve_down(ev.pos),
            PointerPhase::Move => self.curve_move(ev.pos),
            PointerPhase::Up => self.curve_up(ev.pos),
            PointerPhase::Hover => false,
        }
    }

    /// Pointer-down: start a session when idle; else grab the point under the cursor, or insert one on a miss.
    fn curve_down(&mut self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        self.flush_shape_txn(); // close any coalesced Offset drag as its own undo entry first
        self.bake_curve_offset(); // an edit gesture locks in any live Offset (so hit-test = displayed dots)
        if self.paint.curve.is_none() {
            self.paint.stroke_undo = Some(self.capture_shape_model()); // creation txn (`before` = parked set, no active)
            self.begin_shape_session_base(); // full recompose; keep baseline + GLOBAL Offset when shapes are parked
            let seed = self.paint.seed;
            self.paint.seed = self.paint.seed.wrapping_add(1);
            self.paint.curve = Some(CurveEditor {
                model: CurveModel::raw_lasso(vec![pos], false), // authored / Free Hand curves are open
                grab: None,
                gizmo: None,
                editing: false,
                freehand: matches!(self.paint.brush.stroke_method, StrokeMethod::FreeHand),
                stabilized: pos,
                anchor: pos,
                seed,
                added_point: false,
            });
            return true;
        }
        if !self.paint.curve.as_ref().is_some_and(|ed| ed.editing) {
            return true; // mid initial-line drag — ignore extra Downs
        }
        self.begin_shape_txn(); // bracket this edit gesture; curve_up drops it if nothing changed
        let ed = self.paint.curve.as_mut().expect("curve present");
        // Order (mirrors the selection editor): anchor / tangent (via the shared model), then the stroke's
        // transform gizmo, then insert-on-curve, then nearest-select at the point cap.
        let need_refill = if let Some(g) = ed.model.hit_point(pos, tol) {
            ed.grab = Some(g);
            false
        } else if let Some(g) = curve_gizmo::grab(&ed.model.points, &ed.model.handles, pos, tol) {
            ed.gizmo = Some(g); // a transform-gizmo handle (box corner / edge / centre / rotate ring)
            false
        } else if ed.model.points.len() < MAX_CONVERT_POINTS && {
            // Insert ONLY when the click is ON the spine (within the grab radius `tol`) — a click in
            // empty space must never create a point (Enio 2026-07-04). The multi-shape router already
            // band-gates its routing; this re-check keeps every other entry path honest.
            let mut spine = Vec::new();
            curve_geom::flatten_spine(
                &ed.model.points,
                &ed.model.handles,
                ed.model.closed,
                &mut spine,
            );
            stroke_multi::point_near_polyline(pos, &spine, tol)
        } {
            // Subdivide the curve at the NEAREST point on it (incl. the closing seam for a converted
            // Ellipse / Polygon) via a de Casteljau split — the new anchor lands on the curve and the shape
            // is unchanged.
            let i = ed.model.insert(pos);
            ed.model.selected = Some(i);
            ed.grab = Some(CurveGrab::Anchor(i));
            ed.added_point = true; // unlocks the Simplify button for Curve / converted shapes
            true
        } else {
            ed.model.select_nearest(pos);
            ed.grab = None;
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
                let last = *ed.model.points.last().unwrap_or(&ed.anchor);
                if ed.model.points.len() < MAX_FREEHAND_CAPTURE
                    && curve_geom::dist2(last, p) >= MIN_CAPTURE_DIST_SQ
                {
                    ed.model.points.push(p);
                }
                let pts = ed.model.points.clone();
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
            ed.model.points = p;
            ed.model.handles = h;
            let closed = ed.model.closed;
            curve_handle::rebuild(
                &ed.model.points,
                &ed.model.kinds,
                &mut ed.model.handles,
                closed,
            );
            self.curve_refill();
            return true;
        }
        // Anchor / tangent drag → the shared model (Auto/Vector→Aligned, mirror, carry manual handles, rebuild).
        match ed.grab {
            Some(g) => ed.model.drag(g, pos),
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
            ed.grab = None;
            ed.gizmo = None;
            self.commit_shape_txn(); // record this edit gesture iff it changed the curve / Offset
            return true;
        }
        if ed.freehand {
            // Free Hand released → append the final point, FIT the path to cubic Béziers (Schneider) keeping
            // the anchors + tangent handles so flattening reproduces the stroke faithfully.
            let last = *ed.model.points.last().unwrap_or(&ed.anchor);
            if curve_geom::dist2(last, pos) >= MIN_CAPTURE_DIST_SQ {
                ed.model.points.push(pos);
            }
            if ed.model.points.len() >= 3 {
                let fit = ph2d_painter_brush::fit_curve(&ed.model.points, FREEHAND_FIT_ERROR);
                if fit.anchors.len() <= MAX_CURVE_POINTS {
                    ed.model.points = fit.anchors;
                    ed.model.handles = fit.handles; // Bézier mode — faithful tangents
                } else {
                    // Busy scribble over the cap → decimate to anchors, drop handles (rebuild re-derives).
                    ed.model.points = curve_geom::cap_curve_points(fit.anchors, MAX_CURVE_POINTS);
                    ed.model.handles = Vec::new();
                }
            }
            if ed.model.points.len() < 2 {
                ed.model.points = vec![ed.anchor, pos]; // a tap/short flick → a degenerate 2-point curve
                ed.model.handles = Vec::new();
            }
            // The fitted tangents are the artist's smooth handles → Aligned (manual, kept across edits).
            ed.model.kinds = vec![HandleKind::Aligned; ed.model.points.len()];
            ed.model.selected = None;
        } else {
            // Arc released → start / bowed midpoint / end (mid pre-selected to bend further). The middle
            // anchor is offset perpendicular to the chord by a SUBTLE fraction so the new shape reads as
            // an ARC, not a straight line (Enio 2026-07-04). `[dy, -dx]` carries the chord's length, so
            // the bow scales with the drag (and bows UP for a left→right drag); the Auto kinds bend the
            // spine smoothly through it. Transcendental-free (HR-5).
            let a = ed.anchor;
            let (dx, dy) = (pos[0] - a[0], pos[1] - a[1]);
            let mid = [
                (a[0] + pos[0]) * 0.5 + dy * ARC_BOW,
                (a[1] + pos[1]) * 0.5 - dx * ARC_BOW,
            ];
            ed.model.points = vec![a, mid, pos];
            ed.model.kinds = vec![HandleKind::Auto; 3];
            ed.model.selected = Some(1);
        }
        curve_handle::rebuild(
            &ed.model.points,
            &ed.model.kinds,
            &mut ed.model.handles,
            ed.model.closed,
        );
        ed.grab = None;
        ed.editing = true;
        self.curve_refill();
        self.commit_shape_txn(); // the initial line is the curve's CREATION → one undo entry
        true
    }

    /// `true` when a Down at `pos` would EDIT the active curve rather than start a NEW shape: it grabs an
    /// anchor / tangent / transform-gizmo handle, or lands within the insert band of the curve's spine (a
    /// click there subdivides the curve). A click clearly away from the curve is "empty space" → the
    /// multi-shape router parks this curve and begins a fresh one. Mid initial-drag always counts as active.
    pub(super) fn curve_hit_active(&self, pos: [f32; 2]) -> bool {
        let tol = self.paint.shape_grab_tol_px;
        let Some(ed) = self.paint.curve.as_ref() else {
            return false;
        };
        if !ed.editing {
            return true; // still drawing the initial line / freehand path
        }
        let hits_anchor = curve_geom::curve_hit(&ed.model.points, pos, tol).is_some();
        let hits_tangent = ed.model.selected.is_some_and(|s| {
            curve_tangent::tangent_hit(
                &ed.model.points,
                &ed.model.handles,
                s,
                pos,
                tol,
                ed.model.closed,
            )
            .is_some()
        });
        if hits_anchor
            || hits_tangent
            || curve_gizmo::grab(&ed.model.points, &ed.model.handles, pos, tol).is_some()
        {
            return true;
        }
        // ON the spine (within the grab radius — the same `tol` as a point) → a click subdivides the
        // curve (an edit). Anything farther is empty space: no point is ever created off the curve
        // (Enio 2026-07-04); the multi-shape router parks this curve and begins a fresh one.
        let mut spine = Vec::new();
        curve_geom::flatten_spine(
            &ed.model.points,
            &ed.model.handles,
            ed.model.closed,
            &mut spine,
        );
        stroke_multi::point_near_polyline(pos, &spine, tol)
    }

    /// Delete the selected control point (Delete key). Keeps `≥ 2` points; `true` when one was removed.
    pub fn curve_delete_selected(&mut self) -> bool {
        let deletable = self.paint.curve.as_ref().is_some_and(|ed| {
            ed.editing
                && ed
                    .model
                    .selected
                    .is_some_and(|s| ed.model.points.len() > 2 && s < ed.model.points.len())
        });
        if !deletable {
            return false;
        }
        self.begin_shape_txn(); // a delete is one undo step
        let ed = self.paint.curve.as_mut().expect("curve present");
        let removed = ed.model.delete_selected();
        debug_assert!(removed, "deletable checked above");
        ed.grab = None;
        self.curve_refill();
        self.commit_shape_txn();
        true
    }

    /// Convert an open **Ellipse / Polygon** into an editable Bézier **Curve** (the panel's **Edit** / E
    /// button): take its faithful anchors, drop the shape editor, switch the method to Curve. `false` if none.
    pub(crate) fn convert_open_shape_to_curve(&mut self) -> bool {
        self.flush_shape_txn(); // close any coalesced Offset drag first
        // Add/Remove shapes interacting → convert the WHOLE boolean RESULT into one editable curve (Enio
        // 2026-07-04), not just the active shape.
        if self.has_boolean_shapes() {
            return self.convert_boolean_result_to_curve();
        }
        let before = self.capture_shape_model(); // the open Ellipse / Polygon (for undo of the conversion)
        // Bake any live Offset into the shape's radii FIRST (resets the slider) so the conversion reads the
        // displayed shape, not a doubly-offset one. At most one editor is open; each bake no-ops otherwise.
        self.bake_ellipse_offset();
        self.bake_polygon_offset();
        // Ellipse arcs are smooth (Aligned); polygon corners sharp (Free). Both keep explicit (manual) handles.
        let Some((points, handles, kind)) = self
            .ellipse_to_curve()
            .map(|(p, h)| (p, h, HandleKind::Aligned))
            .or_else(|| {
                self.polygon_to_curve()
                    .map(|(p, h)| (p, h, HandleKind::Free))
            })
        else {
            return false; // nothing was editing → no conversion, nothing to record
        };
        // Densify to the convert spacing — the shape is reproduced EXACTLY (de Casteljau splits), it
        // just gains many manipulation anchors (Enio 2026-07-05); Simplify is the way back down.
        let (points, handles) = curve_geom::densify_closed_curve(
            &points,
            &handles,
            CONVERT_ANCHOR_SPACING_PX,
            MAX_CONVERT_POINTS,
        );
        let anchor = points[0];
        let seed = self.paint.seed;
        self.paint.seed = self.paint.seed.wrapping_add(1);
        self.paint.ellipse = None;
        self.paint.polygon = None;
        self.paint.brush.stroke_method = StrokeMethod::Arc; // route future pointers to the curve editor
        self.paint.curve = Some(CurveEditor {
            model: CurveModel::from_curve(points, handles, /*closed=*/ true, kind),
            grab: None,
            gizmo: None,
            editing: true,
            freehand: true, // keep the explicit handles (drag translates them), like the Free Hand fit
            stabilized: anchor,
            anchor,
            seed,
            added_point: false,
        });
        self.curve_refill();
        let after = self.capture_shape_model(); // the new Curve overlay
        self.undo.record_structural(before, after); // Edit (Ellipse/Polygon → Curve) is one undo step
        true
    }

    /// Cancel the curve (Esc): revert the painted preview to pristine + discard the session. `true` if open.
    pub fn curve_cancel(&mut self) -> bool {
        if self.paint.curve.is_none() {
            return false;
        }
        self.paint.curve = None;
        if let Some(prev) = self.paint.drag_preview.take() {
            self.restore_region(&prev.rect, &prev.pixels);
        }
        self.paint.stroke_undo = None;
        self.paint.shape_offset_base_px = 0.0; // a cancelled curve must NOT carry its Offset to the next one
        self.paint.shape_offset_norm = 0.5;
        true
    }

    /// Drop the Curve session without touching pixels — for teardown where the canvas is replaced
    /// or cleared anyway (fresh source / tool deactivate), so a restore would read a stale buffer.
    pub(crate) fn curve_discard(&mut self) {
        self.paint.curve = None;
        self.paint.drag_preview = None;
        self.paint.stroke_undo = None;
        self.paint.shape_offset_base_px = 0.0;
        self.paint.shape_offset_norm = 0.5;
    }

    /// Snapshot the Curve editor for the shell overlay — `None` until editing (the chrome appears
    /// once the 3 control points exist).
    #[must_use]
    pub fn curve_overlay(&self) -> Option<CurveOverlay> {
        let ed = self.paint.curve.as_ref()?;
        if !ed.editing {
            return None;
        }
        let d = self.shape_offset_px();
        // Offset via the CAD reconstruction (inserts as many anchors + handles as the true parallel curve
        // needs): the displayed offset is a faithful NEW curve and the user SEES the multiple points — no
        // auto-simplification (Enio 2026-06-29). `origin` remaps the selection onto the dense anchors.
        let (points, handles, origin) = curve_offset::offset_curve_refined(
            &ed.model.points,
            &ed.model.handles,
            d,
            ed.model.closed,
        );
        let osel = ed
            .model
            .selected
            .and_then(|s| origin.iter().position(|o| *o == Some(s)));
        let mut spine = Vec::new();
        curve_geom::flatten_spine(&points, &handles, ed.model.closed, &mut spine);
        // Trim only the DRAWING (the painted spine + guide), NOT the control points — so the curve keeps its
        // fidelity and the anchors may cross; the crossed loop just isn't painted (Enio 2026-06-28).
        if self.paint.offset_trim {
            spine = trim_offset_spine(&spine, ed.model.closed);
        }
        // Which tangent handle (if any) is being dragged — for the overlay accent colour.
        let grabbed_handle = match ed.grab {
            Some(CurveGrab::Tangent(i, is_out)) => Some((i, is_out)),
            _ => None,
        };
        let tangents = osel.and_then(|s| {
            curve_tangent::build_tangents(
                &points,
                &handles,
                s,
                grabbed_handle,
                self.paint.shape_grab_tol_px,
                ed.model.closed,
            )
        });
        let selected_kind = ed.model.selected_kind_wire();
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

    /// Set the selected point's **handle kind** from the right-click menu's wire `u8` (`0=Free 1=Aligned
    /// 2=Vector 3=Auto`); re-derives the geometry (a sharp point → manual seeds smooth tangents) + re-fills.
    pub fn set_curve_handle_kind(&mut self, wire: u8) -> bool {
        if HandleKind::from_wire(wire).is_none() {
            return false;
        }
        let ok = self.paint.curve.as_ref().is_some_and(|ed| {
            ed.editing && ed.model.selected.is_some_and(|s| s < ed.model.points.len())
        });
        if !ok {
            return false;
        }
        self.begin_shape_txn(); // a handle-kind change is one undo step
        let ed = self.paint.curve.as_mut().expect("curve present");
        let changed = ed.model.set_kind(wire);
        debug_assert!(changed, "guard checked above");
        self.curve_refill();
        self.commit_shape_txn();
        changed
    }

    /// Re-fill the painted preview from the editor's current control points + handles (edit phase).
    pub(super) fn curve_refill(&mut self) {
        let Some(ed) = self.paint.curve.as_ref() else {
            return;
        };
        if !ed.editing {
            return;
        }
        let pts = ed.model.points.clone();
        let handles = ed.model.handles.clone();
        let closed = ed.model.closed;
        let seed = ed.seed;
        self.curve_fill(&pts, &handles, closed, seed);
    }

    /// Build a fresh [`Stroke`], flatten the (offset) curve, fill spaced dabs along the spine + route through
    /// restore/re-stamp (no trail). Fresh-per-fill ⇒ deterministic for equal input.
    fn curve_fill(&mut self, pts: &[[f32; 2]], handles: &[[[f32; 2]; 2]], closed: bool, seed: u64) {
        // Offset (densify) the control geometry → flatten to the dab spine (matches the overlay guide).
        let (pts, handles, _) =
            curve_offset::offset_curve_refined(pts, handles, self.shape_offset_px(), closed);
        let mut spine = Vec::new();
        curve_geom::flatten_spine(&pts, &handles, closed, &mut spine);
        // Trim only the painted spine (matches the guide) — the control points keep crossing, full fidelity.
        if self.paint.offset_trim {
            spine = trim_offset_spine(&spine, closed);
        }
        let mut stroke = Stroke::new(self.paint.brush, self.paint.dynamics, seed);
        let mut dabs = std::mem::take(&mut self.paint.dabs);
        stroke.fill_polyline_preview(&spine, &mut dabs);
        self.restamp_shapes_preview(&dabs); // active shape + every parked shape onto one baseline
        self.paint.dabs = dabs;
    }

    /// **Materialise** the effective offset into the control geometry via the DENSIFIED reconstruction (the
    /// editable curve GAINS the inserted anchors — keeps the displayed points; Enio 2026-06-29). Pre-edit only.
    pub(super) fn bake_curve_offset(&mut self) {
        let off = self.shape_offset_px();
        if off != 0.0
            && let Some(ed) = self.paint.curve.as_mut().filter(|ed| ed.editing)
        {
            let (p, h, k, sel) = curve_offset::offset_curve_refined_kinds(
                &ed.model.points,
                &ed.model.handles,
                &ed.model.kinds,
                ed.model.selected,
                off,
                ed.model.closed,
            );
            (
                ed.model.points,
                ed.model.handles,
                ed.model.kinds,
                ed.model.selected,
            ) = (p, h, k, sel);
            self.paint.shape_offset_base_px = 0.0;
            self.paint.shape_offset_norm = 0.5;
        }
    }
}

/// Trim the offset spine's self-intersections for painting. A CLOSED curve keeps the larger region and
/// drops the smaller "ear" loops an over-offset folds onto a concave side — so it never paints an unwanted
/// crossed area; an OPEN curve drops the looped excess. The control points are never touched (drawing-only).
pub(super) fn trim_offset_spine(spine: &[[f32; 2]], closed: bool) -> Vec<[f32; 2]> {
    if closed {
        super::curve_trim::trim_self_intersections_closed(spine)
    } else {
        super::curve_trim::trim_self_intersections(spine)
    }
}

impl CurveEditor {
    /// Capture the editable state for the unified undo timeline (handle kinds → wire `u8`); the transient
    /// grab / gizmo fields are not stored (a gesture is always closed at snapshot time).
    pub(super) fn to_state(&self) -> crate::undo::CurveState {
        crate::undo::CurveState {
            points: self.model.points.clone(),
            handles: self.model.handles.clone(),
            kinds: self.model.kinds.iter().map(|k| k.wire()).collect(),
            selected: self.model.selected,
            added_point: self.added_point,
            closed: self.model.closed,
            editing: self.editing,
            freehand: self.freehand,
            seed: self.seed,
            anchor: self.anchor,
            stabilized: self.stabilized,
        }
    }

    /// Rebuild the editor from a restored snapshot — the inverse of [`Self::to_state`]; grabs/gizmo idle.
    pub(super) fn from_state(s: crate::undo::CurveState) -> Self {
        CurveEditor {
            model: CurveModel {
                points: s.points,
                handles: s.handles,
                kinds: s
                    .kinds
                    .iter()
                    .map(|&w| HandleKind::from_wire(w).unwrap_or(HandleKind::Free))
                    .collect(),
                selected: s.selected,
                closed: s.closed,
            },
            grab: None,
            gizmo: None,
            editing: s.editing,
            freehand: s.freehand,
            stabilized: s.stabilized,
            anchor: s.anchor,
            seed: s.seed,
            added_point: s.added_point,
        }
    }
}
