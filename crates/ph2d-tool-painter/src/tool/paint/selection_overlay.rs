//! Selection **on-canvas overlay** + **panel event routing** (ADR-0103). The overlay is a canvas-sized
//! straight-RGBA image the shell blits image→screen: diagonal hatching over the deselected area + animated
//! marching ants on the boundary (Procreate's visual language). The router maps every Selection-panel event
//! (mode / op segments, Feather / Threshold / Offset / Overlay sliders, Edit + action buttons) to the tool.
//! Split from `selection` for the LOC cap.

use super::PainterTool;

impl PainterTool {
    /// Build the on-canvas selection **overlay**: semi-transparent diagonal **hatching** over the DESELECTED
    /// area + animated **marching ants** on the boundary. `phase` (a shell frame counter) animates the ants
    /// (fast) and the hatch drift (slow). `None` when no selection is live.
    #[must_use]
    pub fn selection_overlay_rgba(&self, phase: u32) -> Option<(Vec<u8>, u32, u32)> {
        if !self.paint.selection_active {
            return None;
        }
        let (w, h) = self.source_size;
        let (wu, hu) = (w as usize, h as usize);
        let mask = &self.paint.selection_mask;
        if wu == 0 || hu == 0 || mask.len() != wu * hu {
            return None;
        }
        const STRIPE: usize = 7; // hatch stripe width (image px)
        const DASH: usize = 8; // marching-ants dash period (image px)
        const HATCH_MAX_ALPHA: f32 = 110.0; // full-strength hatch alpha (scaled by the opacity setting)
        let hatch_shift = (phase / 2) as usize; // hatch drifts at half the ant speed
        let ant = phase as usize;
        let opacity = self.paint.selection_overlay_opacity.clamp(0.0, 1.0);
        let mut out = vec![0u8; wu * hu * 4];
        for y in 0..hu {
            for x in 0..wu {
                let idx = y * wu + x;
                let cov = f32::from(mask[idx]) / 255.0; // 1 = fully selected, 0 = fully outside
                let inside = mask[idx] >= 128;
                // Boundary = the 128 (half-coverage) contour flips across the right or down neighbour.
                let edge = (x + 1 < wu && (mask[idx + 1] >= 128) != inside)
                    || (y + 1 < hu && (mask[idx + wu] >= 128) != inside);
                let o = idx * 4;
                if edge {
                    // Alternating white / black dashes marching along x+y with the phase.
                    let c = if (x + y + ant) % DASH < DASH / 2 {
                        [255u8, 255, 255, 255]
                    } else {
                        [0u8, 0, 0, 255]
                    };
                    out[o..o + 4].copy_from_slice(&c);
                } else if ((x + y + hatch_shift) / STRIPE).is_multiple_of(2) {
                    // Diagonal hatch whose opacity FADES with the selection coverage — a realistic gradient
                    // across a feathered edge (full outside, half at the 50% contour, clear inside).
                    let a = (HATCH_MAX_ALPHA * opacity * (1.0 - cov)).round() as u8;
                    if a > 0 {
                        out[o..o + 4].copy_from_slice(&[30u8, 30, 30, a]);
                    }
                }
            }
        }
        Some((out, w, h))
    }

    /// Route a Selection-panel event to the tool — mode / boolean-op segments (Click), the Feather /
    /// Threshold / Offset / Overlay sliders (SetValue), the Edit toggle, Convert-to-Curve, and the action
    /// buttons (Invert / Clear / Select layer contents / Color Fill / Copy / Paste). `true` when handled.
    pub(crate) fn route_selection_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use ph2d_editor_core::ids as core_ids;
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::Click(id) if core_ids::PAINTER_SEL_MODE_IDS.contains(id) => {
                let idx = core_ids::PAINTER_SEL_MODE_IDS
                    .iter()
                    .position(|x| x == id)
                    .unwrap_or(0) as u8;
                self.set_selection_mode(idx);
                true
            }
            PanelEvent::Click(id) if core_ids::PAINTER_SEL_OP_IDS.contains(id) => {
                let idx = core_ids::PAINTER_SEL_OP_IDS
                    .iter()
                    .position(|x| x == id)
                    .unwrap_or(0) as u8;
                self.set_selection_bool_op(idx);
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_EDIT => {
                self.toggle_selection_edit();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_CONVERT => {
                self.selection_convert_to_curve();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_SIMPLIFY => {
                self.selection_simplify_curve();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_INVERT => {
                self.invert_selection();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_CLEAR => {
                self.clear_selection();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_LAYER_CONTENTS => {
                self.selection_from_layer_contents();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_FILL => {
                self.selection_color_fill();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_COPY => {
                self.selection_copy();
                true
            }
            PanelEvent::Click(id) if *id == core_ids::PAINTER_SEL_PASTE => {
                self.selection_paste();
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SEL_FEATHER_SLIDER => {
                self.set_selection_feather(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SEL_THRESHOLD_SLIDER => {
                self.set_selection_threshold(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SEL_STABILIZE_SLIDER => {
                self.set_selection_stabilizer(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == core_ids::PAINTER_SEL_OPACITY_SLIDER => {
                self.set_selection_overlay_opacity(*v as f32);
                true
            }
            _ => false,
        }
    }
}
