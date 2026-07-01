//! The **Eyedropper** — an on-canvas colour pick. The left-rail Eyedropper arms it (no colour wheel);
//! the next canvas Down samples the pixel the user SEES (the flattened composite, falling back to the
//! working canvas for a trivial stack) into the brush colour, then disarms so painting resumes as Brush.
//!
//! Mirrors the Clone "Set Source" / Symmetry pick modes: it runs before the paintable-target gate in
//! [`super::PainterTool::on_canvas_pointer`] (it samples a pixel, so it works on any active layer).

use super::PainterTool;
use ph2d_editor_core::tool::{CanvasPointer, PointerPhase};

impl PainterTool {
    /// Whether the on-canvas Eyedropper pick is armed (the next Down samples a colour). The shell reads
    /// this to keep the rail Eyedropper button checked, and to snap it back to Brush when it clears.
    #[must_use]
    pub fn eyedropper_armed(&self) -> bool {
        self.paint.eyedropper_armed
    }

    /// Consume a canvas event while the Eyedropper is armed: the Down samples the composited pixel under
    /// the cursor into the brush colour and disarms; Move/Up are swallowed (no paint / no sprite move)
    /// during the armed window. Always returns `true` so the shell never falls through to move the sprite.
    pub(super) fn eyedropper_pointer(&mut self, ev: CanvasPointer) -> bool {
        if ev.phase != PointerPhase::Down {
            return true;
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            self.paint.eyedropper_armed = false;
            return true;
        }
        // Clamp the sample to the canvas without `.clamp` (swap-free): floor, floor-to-0, cap at edge.
        let x = (ev.pos[0].max(0.0) as usize).min(w as usize - 1);
        let y = (ev.pos[1].max(0.0) as usize).min(h as usize - 1);
        let idx = (y * (w as usize) + x) * 4;
        // Sample the flattened composite the user sees; fall back to the working canvas (trivial stack).
        let sampled = self
            .composited
            .as_deref()
            .map(|c| c.as_slice())
            .filter(|c| c.len() >= idx + 3)
            .or_else(|| (self.canvas_rgba.len() >= idx + 3).then(|| self.canvas_rgba.as_slice()));
        if let Some(buf) = sampled {
            self.paint.brush.color = [
                f32::from(buf[idx]) / 255.0,
                f32::from(buf[idx + 1]) / 255.0,
                f32::from(buf[idx + 2]) / 255.0,
            ];
        }
        self.paint.eyedropper_armed = false; // one-shot → back to Brush
        true
    }
}
