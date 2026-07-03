//! Stroke-**method** control: set the method (cancelling any stale shape session), remember the last
//! NON-shape method, and restore it. Drives both the Brush panel's Method dropdown and the tool rail's
//! Shapes flyout / Brush button over one frozen channel. A submodule of `paint`, split from
//! `brush_settings` for the workspace LOC cap.

use crate::tool::PainterTool;
use ph2d_painter_brush::StrokeMethod;

impl PainterTool {
    /// Set the stroke method from a wire discriminant (out-of-range → Space). Leaving a shape method
    /// (Curve/Ellipse/Polygon/Line) with an OPEN session **BAKES** it (Apply) — a drawn shape is ALWAYS
    /// applied on a method/tool switch, never erased (Enio 2026-07-03). Re-selecting the SAME method keeps
    /// its session editable. A NON-shape method is remembered as the resting method the rail's Brush button
    /// restores.
    pub fn set_brush_stroke_method(&mut self, m: u8) {
        let method = StrokeMethod::from_u8(m);
        // Switching to a DIFFERENT method commits whichever shape editor is open (no-op when none is);
        // re-selecting the current method leaves its session untouched so the artist keeps editing it.
        if method != self.paint.brush.stroke_method {
            self.commit_open_shape();
        }
        self.paint.brush.stroke_method = method;
        if !method.is_shape() {
            self.paint.last_non_shape_method = method;
        }
    }

    /// Restore the stroke method to the last NON-shape method the user chose — the tool rail's **Brush**
    /// button (and every other non-Shapes tool) calls this when leaving a shape, so the resting method is
    /// whatever freehand/dab method the artist last used (never a Line/Curve/Ellipse/Polygon/FreeHand).
    pub fn restore_non_shape_stroke_method(&mut self) {
        self.set_brush_stroke_method(self.paint.last_non_shape_method.to_u8());
    }

    /// Apply a stroke-method command from the frozen `PAINTER_BRUSH_STROKE_METHOD` channel: the sentinel
    /// `"brush"` (rail Brush button / any non-Shapes tool) restores the last non-shape method; otherwise
    /// the value is a wire discriminant (the Method dropdown or a rail shape pick). Unparsable → ignored.
    pub fn apply_stroke_method_command(&mut self, value: &str) {
        if value == "brush" {
            self.restore_non_shape_stroke_method();
        } else if let Ok(m) = value.parse::<u8>() {
            self.set_brush_stroke_method(m);
        }
    }
}
