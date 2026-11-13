//! Selection **on-canvas overlay** + **panel event routing** (ADR-0103). The overlay is a canvas-sized
//! straight-RGBA image the shell blits image→screen: diagonal hatching over the deselected area; the
//! marching ants are screen strokes over `selection_contorno` (B20.11). The router maps every Selection-panel event
//! (mode / op segments, Feather / Threshold / Offset / Overlay sliders, Edit + action buttons) to the tool.
//! Split from `selection` for the LOC cap.

use super::PainterTool;

#[cfg(test)]
#[path = "fios_do_so_tests.rs"]
mod fios_do_so_tests;

impl PainterTool {
    /// Build the on-canvas selection **overlay** image: semi-transparent diagonal **hatching** over the
    /// DESELECTED area, drifting with `phase / 2` (the ants step). The ants themselves are screen strokes
    /// ([`PainterTool::selection_contornos`]). `None` when no selection is live.
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
        const HATCH_MAX_ALPHA: f32 = 110.0; // full-strength hatch alpha (scaled by the opacity setting)
        let hatch_shift = (phase / 2) as usize; // hatch drifts at half the ant speed
        let opacity = self.paint.selection_overlay_opacity.clamp(0.0, 1.0);
        let mut out = vec![0u8; wu * hu * 4];
        // B20.11: only the hatch lives in the image. The ants and the cyan Offset contours are screen-space
        // strokes the shell draws from `selection_contornos` (1 image px vanished with zoom < 1).
        // Pure per-pixel work → disjoint row bands across the cores; bit-identical.
        let overlay_rows = |band: &mut [u8], y0: usize| {
            let rows = band.len() / (wu * 4);
            for ry in 0..rows {
                let y = y0 + ry;
                for x in 0..wu {
                    let idx = y * wu + x;
                    let cov = f32::from(mask[idx]) / 255.0; // 1 = fully selected, 0 = fully outside
                    let o = (ry * wu + x) * 4;
                    if ((x + y + hatch_shift) / STRIPE).is_multiple_of(2) {
                        // Diagonal hatch whose opacity FADES with the selection coverage — a realistic
                        // gradient across a feathered edge (full outside → clear inside).
                        let a = (HATCH_MAX_ALPHA * opacity * (1.0 - cov)).round() as u8;
                        if a > 0 {
                            band[o..o + 4].copy_from_slice(&[30u8, 30, 30, a]);
                        }
                    }
                }
            }
        };
        let threads = crate::compositor::parallel_threads(wu * hu, hu);
        crate::compositor::em_faixas_de_linhas(&mut out, wu * 4, threads, overlay_rows);
        Some((out, w, h))
    }

    /// Route a Selection-panel event to the tool — mode / boolean-op segments (Click), the Feather /
    /// Threshold / Offset / Overlay sliders (SetValue), the Edit toggle, Convert-to-Curve, and the action
    /// buttons (Invert / Clear / Select layer contents / Color Fill / Copy / Paste). `true` when handled.
    pub(crate) fn route_selection_event(
        &mut self,
        event: &ph2d_editor_core::tool::PanelEvent,
    ) -> bool {
        use ph2d_editor_core::tool::PanelEvent;
        match event {
            PanelEvent::Click(id) if crate::ids::PAINTER_SEL_MODE_IDS.contains(id) => {
                let idx = crate::ids::PAINTER_SEL_MODE_IDS
                    .iter()
                    .position(|x| x == id)
                    .unwrap_or(0) as u8;
                self.set_selection_mode(idx);
                true
            }
            PanelEvent::Click(id) if crate::ids::PAINTER_SEL_OP_IDS.contains(id) => {
                let idx = crate::ids::PAINTER_SEL_OP_IDS
                    .iter()
                    .position(|x| x == id)
                    .unwrap_or(0) as u8;
                self.set_selection_bool_op(idx);
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_EDIT => {
                self.toggle_selection_edit();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_CONVERT => {
                self.selection_convert_to_curve();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_SIMPLIFY => {
                self.selection_simplify_curve();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_MERGE => {
                self.selection_merge_curves();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_OFFSET_APPLY => {
                self.selection_offset_apply();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_OFFSET_APPLY_KEEP => {
                self.selection_offset_apply_keep();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_INVERT => {
                self.invert_selection();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_CLEAR => {
                self.clear_selection();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_LAYER_CONTENTS => {
                self.selection_from_layer_contents();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_FILL => {
                self.selection_color_fill();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_ALL => {
                self.selection_select_all();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_CUT => {
                self.selection_cut();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_COPY => {
                self.selection_copy();
                true
            }
            PanelEvent::Click(id) if *id == crate::ids::PAINTER_SEL_PASTE => {
                self.selection_paste();
                true
            }
            PanelEvent::SetValue(id, v) if *id == crate::ids::PAINTER_SEL_FEATHER_SLIDER => {
                self.set_selection_feather(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == crate::ids::PAINTER_SEL_THRESHOLD_SLIDER => {
                self.set_selection_threshold(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == crate::ids::PAINTER_SEL_STABILIZE_SLIDER => {
                self.set_selection_stabilizer(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == crate::ids::PAINTER_SEL_OPACITY_SLIDER => {
                self.set_selection_overlay_opacity(*v as f32);
                true
            }
            PanelEvent::SetValue(id, v) if *id == crate::ids::PAINTER_SEL_OFFSET_SLIDER => {
                self.set_selection_offset(*v as f32);
                true
            }
            _ => false,
        }
    }
}
