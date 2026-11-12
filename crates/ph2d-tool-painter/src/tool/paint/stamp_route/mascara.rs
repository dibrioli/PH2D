//! A rota do pincel **Mask** (o rascunho transitório da máscara) — filho de `stamp_route` por LOC.

use super::*;

#[cfg(test)]
thread_local! {
    /// O pior lote: texels copiados + recortados pela seleção (B40.A5).
    pub(super) static TEXELS_DO_LOTE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// Força a rota antiga (cópia e recorte da tela inteira), o oráculo do gate.
    pub(super) static ROTA_ANTIGA: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn conta_o_lote(texels: usize) {
    TEXELS_DO_LOTE.with(|c| c.set(c.get().max(texels)));
}

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
        // Paint/Erase write only under their dabs: copy and clip just the dabs' box (B40.A5). Blur/Smear
        // keep the whole-scratch copy.
        let restringe = self.selection_restricts_paint();
        #[cfg(test)]
        let antiga = ROTA_ANTIGA.with(std::cell::Cell::get);
        #[cfg(not(test))]
        let antiga = false;
        let por_caixa = restringe && !antiga && self.mask_brush() <= 1;
        let scratch_before =
            (restringe && !por_caixa).then(|| (*self.paint.mask_scratch_rgba).clone());
        let mut recorte = None;
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
                let lista = if self.paint.tiling[0] || self.paint.tiling[1] {
                    super::super::tiling::tiled_dabs(
                        &recolored,
                        self.source_size,
                        self.paint.tiling,
                    )
                } else {
                    recolored
                };
                if por_caixa {
                    let (w, h) = self.source_size;
                    let reach = self.stroke_spec().tip_reach();
                    recorte = super::super::region::dabs_bounds(&lista, w, h, reach).map(|c| {
                        (
                            c,
                            super::super::selection::copia_a_caixa(&self.canvas_rgba, w, c),
                        )
                    });
                }
                self.stamp_dabs_inner(&lista);
                self.paint.brush.color = saved_color;
                self.paint.brush.blend = saved_blend;
            }
        }
        // Clip the mask stroke to the selection while the scratch still lives in `canvas_rgba`.
        if let Some(orig) = scratch_before.as_ref() {
            #[cfg(test)]
            conta_o_lote(orig.len() / 4 * 2);
            self.clip_canvas_to_selection(orig);
        }
        if let Some((caixa, orig)) = recorte.as_ref() {
            #[cfg(test)]
            conta_o_lote(orig.len() / 4 * 2);
            self.clip_canvas_to_selection_na_caixa(orig, *caixa);
        }
        super::super::plane_fork::swap_canvas_plane(
            &mut self.canvas_rgba,
            &mut self.paint.mask_scratch_rgba,
            &self.undo.write_state,
        );
    }
}
