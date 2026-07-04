//! The [`CanvasPaintTool`] pointer entry point (`on_canvas_pointer`) — routes each canvas sample to the
//! armed pick modes, the persistent shape editors, the Stencil handles, then the generic stroke
//! lifecycle (`paint_begin`/`_extend`/`_end`, private in `paint.rs` — visible to this child module).
//! Split from `paint.rs` for the workspace file-LOC cap.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_painter_brush::StrokeMethod;

impl CanvasPaintTool for PainterTool {
    fn on_canvas_pointer(&mut self, ev: CanvasPointer) -> bool {
        if ev.phase == PointerPhase::Hover {
            return false; // hover is cursor/preview only
        }
        // While a Symmetry pick mode is armed, the canvas sets the mirror line / radial centre instead
        // of painting (works on any layer, so it precedes the paintable-target gate).
        if self.symmetry_pick_active() {
            return self.symmetry_pick_pointer(ev);
        }
        // Clone "Set Source" pick mode: the next canvas Down samples the source anchor (consumes the
        // click, no paint), like the Symmetry picks. Works on any layer (it records a coordinate).
        if self.clone_sample_armed() {
            return self.clone_sample_pointer(ev);
        }
        // Eyedropper pick: the next Down samples the composited pixel colour into the brush (then → Brush).
        if self.eyedropper_armed() {
            return self.eyedropper_pointer(ev);
        }
        // Selection mode builds a canvas-wide selection mask (Procreate-style), NOT a layer paint — so it
        // precedes the paintable-target gate. Routes to the mode engine (Automatic / Freehand / Rectangle /
        // Ellipse + Add/Remove operators); the mask joins the single undo queue on pen-up.
        if matches!(self.paint.paint_mode, super::PaintMode::Selection) {
            // "Show Selection Gizmos" mode: EVERY shape's isolated gizmo is live (ADR-0103 Am.2 v2). Route
            // pointers to the isolated gizmo system (never the stroke editors), which recomposites the mask.
            if self.paint.selection_edit_mode {
                return self.selection_gizmo_pointer(ev);
            }
            return self.selection_pointer(ev);
        }
        if !self.paint_target_ready() {
            // Active layer isn't paintable (mask/group/adjustment) or no canvas:
            // finalize any half-open stroke (records its undo) before bailing. Drop any open
            // shape session too (its restore would read a stale buffer once the layer changed).
            self.discard_open_shape();
            self.close_stroke();
            return false;
        }
        // Fill (Bucket) — the ColorDrop gesture (flood fill on drop + live threshold adjust). It is not a
        // stroke and ignores the stroke method, so it precedes the shape-editor routing below.
        if matches!(self.paint.paint_mode, super::PaintMode::Fill) {
            return self.fill_pointer(ev);
        }
        // Deform (Liquify) — the brush-driven reshape kernel (Push/Twist/Pinch/…). Its own dab lifecycle
        // (inverse-warp gather + structural undo), not a stroke of the brush engine, so it routes before the
        // shape editors + generic stroke path.
        if matches!(self.paint.paint_mode, super::PaintMode::Deform) {
            return self.warp_pointer(ev);
        }
        // Curve and Ellipse are persistent on-canvas shape editors (draw → edit → commit), not a
        // single press→release stroke — route every canvas event through them instead of the generic
        // path.
        // Free Hand shares the Curve editor (its draw phase captures a freehand path, then it's an ordinary
        // editable curve). All four route through `route_shape_pointer_multi`, which adds MULTI-SHAPE
        // switching (Down on a parked shape re-activates it; Down in empty space parks the active one and
        // starts a fresh shape) before dispatching to the active editor.
        if matches!(
            self.paint.brush.stroke_method,
            StrokeMethod::Curve
                | StrokeMethod::FreeHand
                | StrokeMethod::Ellipse
                | StrokeMethod::Polygon
                | StrokeMethod::Line
        ) {
            return self.route_shape_pointer_multi(ev);
        }
        // Stencil texture: grabbing an overlay handle (corner = resize, centre = move) edits the
        // rect and consumes the event; a Down away from every handle (or any move without a grab)
        // falls through to normal painting — the handles disambiguate, so no modifier is needed.
        if self.stencil_edit_active()
            && (ev.phase == PointerPhase::Down || self.paint.stencil_grab.is_some())
            && self.stencil_pointer(ev)
        {
            return true;
        }
        match ev.phase {
            PointerPhase::Down => {
                self.paint_begin(ev);
                true
            }
            PointerPhase::Move => self.paint_extend(ev),
            PointerPhase::Up => {
                self.paint_end(ev);
                true
            }
            PointerPhase::Hover => false,
        }
    }
}
