//! The **Mask** brush — a TEMPORARY, layerless mask on the current layer (NOT the layer-system mask).
//!
//! The mask brush keeps its own tool-side **scratch** buffer ([`super::PaintState`]'s `mask_scratch_*`),
//! white = reveal, tied to the active raster layer. It never creates a stack layer. The scratch:
//! - is edited by the sub-brush (Paint = conceal / Erase = reveal / Blur / Smear) — painting swaps the
//!   scratch into `canvas_rgba` so the whole stamp pipeline edits it, then swaps back;
//! - is edited by the whole-canvas **Modifiers** (Expand / Contract / Blur / Sharpen / Invert / Clear);
//! - hides/reveals the target layer LIVE and non-destructively — the preview composites the layer's
//!   pixels MASKED by the scratch ([`Self::mask_scratch_display`]); the colour film shows the coverage;
//! - is TEMPORARY: it clears when you leave the Mask tool or switch layers, and **Apply** bakes it into
//!   the layer's alpha ([`Self::apply_mask_scratch`]). Nothing touches the layer system.

use super::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_painter_brush::MaskCanvasOp;
use std::sync::Arc;

/// Overlay film opacity — how strongly the CONCEALED region reads in the mask colour while editing.
/// High so the masked-out area is clearly "removed" (a solid-ish colour), not a faint tint.
const OVERLAY_STRENGTH: f32 = 0.8;

impl PainterTool {
    /// Set the Mask sub-brush: `0` Paint (conceal) · `1` Erase (reveal) · `2` Blur · `3` Smear. Entering
    /// Blur/Smear densifies Spacing to 5% (a sparse chain leaves gaps), only on the transition in.
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

    /// Set the on-canvas mask overlay tint colour (`0` dark gray + `1..=4` fluorescent). Invalidates the
    /// composite (when a scratch is active) so the new tint shows at once.
    pub fn set_mask_overlay_color(&mut self, v: u8) {
        self.paint.mask_overlay_color = v.min(4);
        if self.mask_scratch_active() {
            self.invalidate_composite();
        }
    }

    /// The active mask overlay colour index (`0..=4`) — mirrored into the panel snapshot.
    #[must_use]
    pub fn mask_overlay_color(&self) -> u8 {
        self.paint.mask_overlay_color
    }

    /// `true` when a transient scratch mask is live on the CURRENT layer (target still active, non-empty).
    #[must_use]
    pub(crate) fn mask_scratch_active(&self) -> bool {
        self.paint.mask_scratch_target.is_some()
            && self.paint.mask_scratch_target == self.layers.active()
            && !self.paint.mask_scratch_rgba.is_empty()
    }

    /// Ensure a white (fully-revealed) scratch mask exists for the active RASTER layer, re-targeting (and
    /// discarding any stale scratch) if the active layer changed. No-op if the active layer isn't a raster
    /// or the canvas is empty. Called before a Mask stroke / a Modifier op.
    pub(super) fn ensure_mask_scratch(&mut self) {
        let Some(active) = self.layers.active() else {
            return;
        };
        if !matches!(
            self.layers.get(active).map(|l| &l.kind),
            Some(crate::layers::LayerKind::Raster(_))
        ) {
            return;
        }
        if self.mask_scratch_active() {
            return;
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        self.paint.mask_scratch_rgba = Arc::new(vec![255u8; (w as usize) * (h as usize) * 4]);
        self.paint.mask_scratch_target = Some(active);
        self.invalidate_composite();
    }

    /// Discard the transient scratch (the target layer returns to unmasked). Named `reset_` so the
    /// mode-reconcile gate recognises it; called when leaving the Mask tool.
    pub(super) fn reset_mask_scratch(&mut self) {
        if self.paint.mask_scratch_target.take().is_some() {
            self.paint.mask_scratch_rgba = Arc::new(Vec::new());
            self.invalidate_composite();
        }
    }

    /// **Apply** — bake the transient scratch into the current layer's alpha (destructive: `α *= mask`),
    /// then clear the scratch. The Mask section's Apply button; one undo step. No-op without a live scratch.
    pub fn apply_mask_scratch(&mut self) {
        if !self.mask_scratch_active() {
            return;
        }
        let Some(target) = self.layers.active() else {
            return;
        };
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.paint.mask_scratch_rgba.len() < n * 4 || self.canvas_rgba.len() < n * 4 {
            return;
        }
        let before = self.snapshot_model();
        let scratch = self.paint.mask_scratch_rgba.clone();
        {
            let buf = Arc::make_mut(&mut self.canvas_rgba);
            for i in 0..n {
                let v = crate::compositor::mask_value(&scratch, i);
                let a = f32::from(buf[i * 4 + 3]);
                buf[i * 4 + 3] = (a * v).round().clamp(0.0, 255.0) as u8;
            }
        }
        self.paint.mask_scratch_target = None;
        self.paint.mask_scratch_rgba = Arc::new(Vec::new());
        self.bump_layer_pixels(Some(target));
        self.commit_structural_edit(before);
        self.invalidate_composite();
    }

    /// The active layer's pixels MASKED by the transient scratch (`α *= scratch`) for the LIVE preview,
    /// or `None` when no scratch is active. The composite paths pass this as the active layer's pixels so
    /// the scratch hides/reveals the layer without altering it (Apply bakes it for real).
    #[must_use]
    pub(crate) fn mask_scratch_display(&self) -> Option<Vec<u8>> {
        if !self.mask_scratch_active() {
            return None;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if self.canvas_rgba.len() < n * 4 || self.paint.mask_scratch_rgba.len() < n * 4 {
            return None;
        }
        let mut out = self.canvas_rgba.as_ref().clone();
        let scratch = &self.paint.mask_scratch_rgba;
        for i in 0..n {
            let v = crate::compositor::mask_value(scratch, i);
            let a = f32::from(out[i * 4 + 3]);
            out[i * 4 + 3] = (a * v).round().clamp(0.0, 255.0) as u8;
        }
        Some(out)
    }

    /// Apply a whole-canvas Modifier (Expand / Contract / Blur / Sharpen / Invert / Clear) to the scratch
    /// mask (creating it first). No-op if the active layer can't hold a scratch or the canvas is empty.
    pub fn mask_canvas_op(&mut self, op: u8) {
        let Some(mop) = mask_op_from_u8(op) else {
            return;
        };
        self.ensure_mask_scratch();
        if !self.mask_scratch_active() {
            return;
        }
        let (w, h) = self.source_size;
        if w == 0 || h == 0 {
            return;
        }
        let radius = self.paint.brush.radius_px;
        {
            let buf = Arc::make_mut(&mut self.paint.mask_scratch_rgba);
            ph2d_painter_brush::apply_mask_op(buf, w, h, mop, radius);
        }
        self.invalidate_composite();
    }

    /// Blend the mask overlay film over the composited RGBA `buf`: where the scratch CONCEALS, tint by the
    /// selected colour at [`OVERLAY_STRENGTH`] (straight-alpha src-over, raising alpha). No-op without a
    /// live scratch. Called at the composite production points AFTER the full compose.
    pub(crate) fn apply_mask_overlay(&self, buf: &mut [u8]) {
        if !self.mask_scratch_active() {
            return;
        }
        let (w, h) = self.source_size;
        let n = (w as usize) * (h as usize);
        if buf.len() < n * 4 || self.paint.mask_scratch_rgba.len() < n * 4 {
            return;
        }
        let film = mask_overlay_rgb(self.paint.mask_overlay_color);
        let scratch = &self.paint.mask_scratch_rgba;
        for i in 0..n {
            let cov = crate::compositor::mask_value(scratch, i);
            let sa = (1.0 - cov) * OVERLAY_STRENGTH;
            if sa <= 0.0 {
                continue;
            }
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

    /// Route the Mask-section panel Clicks: sub-brush segments, canvas-op buttons, overlay-colour
    /// swatches, and Apply. Returns `true` iff consumed. Called from `route_brush_dab_event`.
    pub(crate) fn route_mask_event(&mut self, event: &PanelEvent) -> bool {
        use ph2d_editor_core::ids as core_ids;
        let PanelEvent::Click(id) = event else {
            return false;
        };
        if *id == core_ids::PAINTER_MASK_APPLY {
            self.apply_mask_scratch();
            return true;
        }
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

/// The overlay tint palette (straight sRGB `0..=255`): index `0` a DARK gray (default), `1..=4`
/// fluorescent highlighter hues (yellow / pink / green / orange). Out-of-range falls back to dark gray.
fn mask_overlay_rgb(idx: u8) -> [u8; 3] {
    match idx {
        1 => [220, 255, 0],  // fluorescent yellow
        2 => [255, 42, 160], // fluorescent pink
        3 => [80, 255, 60],  // fluorescent green
        4 => [255, 120, 0],  // fluorescent orange
        _ => [51, 51, 51],   // dark gray (default)
    }
}
