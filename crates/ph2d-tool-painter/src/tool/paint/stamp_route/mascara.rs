//! A rota do pincel **Mask** (o rascunho transitório da máscara) — filho de `stamp_route` por LOC.

use super::*;

impl PainterTool {
    /// **Mask** route: paint into the Mask brush's TRANSIENT scratch (a tool-side mask, NOT a stack
    /// layer). The scratch is swapped INTO `canvas_rgba` so the whole stamp pipeline edits it, then
    /// swapped back — the layer's pixels are never touched. Paint (0) = conceal (black) · Erase (1) =
    /// reveal (white) · Blur (2) · Smear (3); ramps / Shape / Grain / falloff / Tiling / Symmetry / every
    /// Stroke Method apply as in Paint. The composite hides the layer by the scratch live ([`mask`]).
    pub(in crate::tool::paint) fn stamp_dabs_mask(&mut self, dabs: &[Dab]) {
        if self.paint.mask_scratch_target.is_none() {
            return; // no scratch to paint (ensured at `paint_begin`)
        }
        // A pre-stamp copy of the scratch, so an active selection can revert mask strokes that landed
        // outside it (the scratch swap hides the edit from the canvas-side stamp gate, ADR-0103).
        let scratch_before = self
            .selection_restricts_paint()
            .then(|| (*self.paint.mask_scratch_rgba).clone());
        // Swap the scratch into `canvas_rgba` so the stamp pipeline edits the SCRATCH, then swap back.
        super::super::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut self.paint.mask_scratch_rgba,
            &self.undo.write_state,
        );
        match self.mask_brush() {
            2 => {
                let (w, h) = self.source_size;
                self.stamp_dabs_blur(dabs, w, h);
            }
            3 => {
                let (w, h) = self.source_size;
                self.stamp_dabs_smear(dabs, w, h);
            }
            _ => {
                // The mask is a pure COVERAGE buffer: force every dab to the mask colour with a plain Mix
                // blend. Black = protect (Paint), white = unprotect (Erase). NB the stamp reads each dab's
                // OWN `color` (baked from the user's brush colour at stroke start), so overriding only
                // `brush.color` was a NO-OP — that's why Erase, and any non-black brush colour, painted the
                // wrong coverage. Recolour the dabs AND set the spec colour/blend so every route agrees.
                let color = if self.mask_brush() == 1 {
                    [1.0, 1.0, 1.0] // Erase → unprotect (white)
                } else {
                    [0.0, 0.0, 0.0] // Paint → protect (black)
                };
                let recolored: Vec<Dab> = dabs.iter().map(|d| Dab { color, ..*d }).collect();
                let saved_color = self.paint.brush.color;
                let saved_blend = self.paint.brush.blend;
                self.paint.brush.color = color;
                self.paint.brush.blend = BrushBlend::Mix;
                if self.paint.tiling[0] || self.paint.tiling[1] {
                    let wrapped = super::super::tiling::tiled_dabs(
                        &recolored,
                        self.source_size,
                        self.paint.tiling,
                    );
                    self.stamp_dabs_inner(&wrapped);
                } else {
                    self.stamp_dabs_inner(&recolored);
                }
                self.paint.brush.color = saved_color;
                self.paint.brush.blend = saved_blend;
            }
        }
        // Clip the mask stroke to the selection while the scratch still lives in `canvas_rgba`.
        if let Some(orig) = scratch_before.as_ref() {
            self.clip_canvas_to_selection(orig);
        }
        super::super::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut self.paint.mask_scratch_rgba,
            &self.undo.write_state,
        );
    }
}
