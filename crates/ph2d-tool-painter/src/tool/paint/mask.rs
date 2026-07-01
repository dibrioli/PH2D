//! The **Mask** tool's extras beyond the shared brush path: the mask **sub-brush** (Paint / Erase /
//! Blur / Smear applied to the mask itself), the whole-canvas **mask ops** (Expand / Contract / Blur /
//! Sharpen / Invert / Clear), and the on-canvas **overlay tint** (a quick-mask film so the coverage
//! reads while editing). The panel's collapsible Mask section drives all of it over the Click channel.
//!
//! Because [`PaintMode::Mask`] retargets painting onto the mask (`ensure_mask_edit_target`), the active
//! layer's `canvas_rgba` IS the mask buffer while masking — so the Blur/Smear sub-brushes reuse the
//! existing Blur/Smear routes verbatim, and the canvas ops mutate `canvas_rgba` in place.

use super::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::MaskCanvasOp;
use std::sync::Arc;

/// Overlay film opacity — how strongly the mask coverage tints the composite while editing. Semi-
/// transparent so the underlying image still reads through the quick-mask.
const OVERLAY_STRENGTH: f32 = 0.5;

impl PainterTool {
    /// Set the Mask sub-brush: `0` Paint (reveal) · `1` Erase (conceal) · `2` Blur · `3` Smear. Entering
    /// Blur/Smear densifies Spacing to 5% (like the rail Blur/Smear tools — a sparse chain leaves gaps),
    /// only on the transition in, so a later manual tweak sticks.
    pub fn set_mask_brush(&mut self, v: u8) {
        let v = v.min(3);
        if v != self.paint.mask_brush && (v == 2 || v == 3) {
            self.paint.brush.spacing = 0.05;
        }
        self.paint.mask_brush = v;
    }

    /// The active Mask sub-brush discriminant (`0..=3`) — mirrored into the panel snapshot.
    #[must_use]
    pub fn mask_brush(&self) -> u8 {
        self.paint.mask_brush
    }

    /// Set the on-canvas mask overlay tint colour (`0` neutral gray + `1..=4` fluorescent). Invalidates
    /// the composite (when a mask is active) so the new tint shows at once.
    pub fn set_mask_overlay_color(&mut self, v: u8) {
        self.paint.mask_overlay_color = v.min(4);
        if self.active_is_mask() {
            self.invalidate_composite();
        }
    }

    /// The active mask overlay colour index (`0..=4`) — mirrored into the panel snapshot.
    #[must_use]
    pub fn mask_overlay_color(&self) -> u8 {
        self.paint.mask_overlay_color
    }

    /// Apply a whole-canvas mask op (Expand / Contract / Blur / Sharpen / Invert / Clear) to the active
    /// layer's mask, creating/selecting the mask first (`ensure_mask_edit_target`) so the button "just
    /// works". One structural-undo entry per op; the extent scales with the brush Size. No-op if the
    /// active layer can't hold a mask or the canvas is empty.
    pub fn mask_canvas_op(&mut self, op: u8) {
        let Some(mop) = mask_op_from_u8(op) else {
            return;
        };
        self.ensure_mask_edit_target();
        // After ensure, the mask must be the active edit target (else the layer can't hold one).
        if !self.active_is_mask() {
            return;
        }
        let Some(active) = self.layers.active() else {
            return;
        };
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let before = self.snapshot_model();
        let radius = self.paint.brush.radius_px;
        {
            let buf = Arc::make_mut(&mut self.canvas_rgba);
            ph2d_painter_brush::apply_mask_op(buf, w, h, mop, radius);
        }
        self.bump_layer_pixels(Some(active));
        self.commit_structural_edit(before);
        self.invalidate_composite();
    }

    /// Blend the mask overlay film over the composited RGBA `buf` (straight sRGB8): where the active mask
    /// CONCEALS, tint by the selected colour at [`OVERLAY_STRENGTH`] — a quick-mask film so the painted
    /// (hidden) regions glow in the marker colour, while a fully-revealed mask shows nothing (no flood).
    /// No-op unless a mask is the active target. Called at both composite points AFTER the full compose.
    pub(crate) fn apply_mask_overlay(&self, buf: &mut [u8]) {
        if !self.active_is_mask() {
            return;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if buf.len() < n * 4 || self.canvas_rgba.len() < n * 4 {
            return;
        }
        let inverted = self
            .layers
            .active()
            .and_then(|id| self.layers.get(id))
            .and_then(|l| match &l.kind {
                crate::layers::LayerKind::Mask(m) => Some(m.inverted),
                _ => None,
            })
            .unwrap_or(false);
        let film = mask_overlay_rgb(self.paint.mask_overlay_color);
        let mask = &self.canvas_rgba;
        for i in 0..n {
            let v = crate::compositor::mask_value(mask, i);
            let cov = if inverted { 1.0 - v } else { v };
            // Tint the CONCEALED area (1 − coverage): a fresh white/revealed mask (cov = 1) shows nothing,
            // painted-black regions (cov = 0) glow — the quick-mask film, not a full-canvas flood.
            let sa = (1.0 - cov) * OVERLAY_STRENGTH;
            if sa <= 0.0 {
                continue;
            }
            // Straight-alpha src-over of the film onto the composite pixel — RAISES alpha too, so the film
            // is visible even where the mask made the layer fully transparent (concealed → composite α 0).
            let b = i * 4;
            let da = f32::from(buf[b + 3]) / 255.0;
            let oa = sa + da * (1.0 - sa);
            if oa <= 0.0 {
                continue;
            }
            for c in 0..3 {
                let s = f32::from(film[c]);
                let d = f32::from(buf[b + c]);
                buf[b + c] = ((s * sa + d * da * (1.0 - sa)) / oa)
                    .round()
                    .clamp(0.0, 255.0) as u8;
            }
            buf[b + 3] = (oa * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }

    /// Route the Mask-section panel Clicks: the sub-brush segments, the canvas-op buttons, and the
    /// overlay-colour swatches. Returns `true` iff consumed. Called from `route_brush_dab_event`.
    pub(crate) fn route_mask_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        let PanelEvent::Click(id) = event else {
            return false;
        };
        if let Some(i) = core_ids::PAINTER_MASK_BRUSH.iter().position(|x| x == id) {
            self.set_mask_brush(i as u8);
            return true;
        }
        if let Some(i) = core_ids::PAINTER_MASK_OP.iter().position(|x| x == id) {
            self.mask_canvas_op(i as u8);
            return true;
        }
        if let Some(i) = core_ids::PAINTER_MASK_COLOR.iter().position(|x| x == id) {
            self.set_mask_overlay_color(i as u8);
            return true;
        }
        false
    }
}

/// Map a wire discriminant to the engine op (`None` for an out-of-range index).
fn mask_op_from_u8(op: u8) -> Option<MaskCanvasOp> {
    Some(match op {
        0 => MaskCanvasOp::Expand,
        1 => MaskCanvasOp::Contract,
        2 => MaskCanvasOp::Blur,
        3 => MaskCanvasOp::Sharpen,
        4 => MaskCanvasOp::Invert,
        5 => MaskCanvasOp::Clear,
        _ => return None,
    })
}

/// The overlay tint palette (straight sRGB `0..=255`): index `0` a neutral gray, `1..=4` fluorescent
/// highlighter hues (yellow / pink / green / orange). Out-of-range falls back to gray.
fn mask_overlay_rgb(idx: u8) -> [u8; 3] {
    match idx {
        1 => [220, 255, 0],   // fluorescent yellow
        2 => [255, 42, 160],  // fluorescent pink
        3 => [80, 255, 60],   // fluorescent green
        4 => [255, 120, 0],   // fluorescent orange
        _ => [128, 128, 128], // neutral gray (default)
    }
}
