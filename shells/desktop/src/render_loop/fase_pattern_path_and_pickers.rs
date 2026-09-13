//! **Fase do quadro: O PADRÃO NO CAMINHO, O PINCEL E OS PICKERS** — os sliders e o comando do padrão no caminho, a lei do pincel e os pickers do pincel, do
//! padrão de textura e do motivo (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct PatternPathAndPickersIntents {
    pub(super) pending_patternpath: Option<crate::pattern_live::PatternPathCmd>,
    pub(super) pending_pp_spacing: Option<f64>,
    pub(super) pending_pp_start: Option<f64>,
    pub(super) pending_pp_end: Option<f64>,
    pub(super) pending_pp_slide: Option<f64>,
    pub(super) pending_pp_offset: Option<f64>,
    pub(super) pending_pp_rotation: Option<f64>,
    pub(super) pending_pp_pick: bool,
    pub(super) pending_texpat_pick: Option<ph2d_vec_render::PatternSlot>,
    pub(super) pending_brush_pick: bool,
    pub(super) pending_brush: Option<crate::vec_stroke_paint::BrushCmd>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_pattern_path_and_pickers(&mut self, intents: PatternPathAndPickersIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        let PatternPathAndPickersIntents {
            pending_patternpath,
            pending_pp_spacing,
            pending_pp_start,
            pending_pp_end,
            pending_pp_slide,
            pending_pp_offset,
            pending_pp_rotation,
            pending_pp_pick,
            pending_texpat_pick,
            pending_brush_pick,
            pending_brush,
        } = intents;
        // Pattern on Path (plano 23): os sliders afinam o vínculo do MOTIVO — que é o caminho
        // LINKADO da seleção (`linked_motif`), não o primário: depois de prender, o primário
        // pode ser o GUIA. O comando prende/solta/vira. Tudo pela porta única `pattern_live`, e
        // o `recook` do frame seguinte redesenha as cópias.
        let pp_motif = crate::pattern_live::linked_motif(
            sim,
            &self.vec.entities,
            self.vec.pen.selected_paths(),
        );
        if let Some(v) = pending_pp_spacing
            && let Some(motif) = pp_motif
        {
            crate::pattern_live::edit(sim, &self.vec.entities, motif, |l| l.spacing = v as f32);
        }
        if let Some(v) = pending_pp_start
            && let Some(motif) = pp_motif
        {
            crate::pattern_live::edit(sim, &self.vec.entities, motif, |l| {
                l.start_offset = v as f32;
            });
        }
        if let Some(v) = pending_pp_end
            && let Some(motif) = pp_motif
        {
            crate::pattern_live::edit(sim, &self.vec.entities, motif, |l| {
                l.end_offset = v as f32;
            });
        }
        if let Some(v) = pending_pp_offset
            && let Some(motif) = pp_motif
        {
            crate::pattern_live::edit(sim, &self.vec.entities, motif, |l| l.offset = v as f32);
        }
        // A rotação tem porta PRÓPRIA (`set_rotation`) e não o `edit`: ela vive num componente
        // separado, para não bumpar o `PROJECT_SCHEMA` -- e essa porta destaca no neutro.
        if let Some(v) = pending_pp_rotation
            && let Some(motif) = pp_motif
        {
            crate::pattern_live::set_rotation(sim, &self.vec.entities, motif, v as f32);
        }
        // Slide re-centra o trecho `[Start, End]` PRESERVANDO o comprimento: move as duas
        // âncoras juntas (o pedido do Enio). O centro é clampado para a janela caber em [0,1].
        if let Some(v) = pending_pp_slide
            && let Some(motif) = pp_motif
        {
            crate::pattern_live::edit(sim, &self.vec.entities, motif, |l| {
                let half = (f64::from(l.end_offset) - f64::from(l.start_offset)) * 0.5;
                let c = v.clamp(half, 1.0 - half);
                l.start_offset = (c - half) as f32;
                l.end_offset = (c + half) as f32;
            });
        }
        if let Some(cmd) = pending_patternpath {
            let sel = self.vec.pen.selected_paths().to_vec();
            let done = match cmd {
                // O guia é o caminho de MAIOR extensão dos dois (independe da ordem de clique)
                // — a correção do "escolhendo a si mesmo" (Enio).
                crate::pattern_live::PatternPathCmd::Link => crate::pattern_live::link_candidate(
                    vec_scene, &sel,
                )
                .is_some_and(|(motif, guide)| {
                    crate::pattern_live::link(sim, &self.vec.entities, motif, guide)
                }),
                crate::pattern_live::PatternPathCmd::Detach => pp_motif
                    .is_some_and(|m| crate::pattern_live::detach(sim, &self.vec.entities, m)),
                crate::pattern_live::PatternPathCmd::Flip(v) => pp_motif.is_some_and(|m| {
                    crate::pattern_live::edit(sim, &self.vec.entities, m, |l| l.flip = v)
                }),
            };
            if !done {
                eprintln!(
                    "[ph2d-vec] pattern on path: selecione o MOTIVO e um caminho (ou um motivo \
                         ja' preso, para soltar/afinar)"
                );
            }
        }
        // Picker do motivo (Enio 2026-07-23): o botão só ARMOU; a FONTE é o motivo selecionado (a
        // `can_pick` já garantiu um só, ainda solto). O clique seguinte no canvas escolhe o guia.
        // ⭐⭐⭐ O PINCEL (plano 36, W4): a lei primeiro, o arm depois — a mesma ordem do padrão.
        if let Some(cmd) = pending_brush {
            crate::vec_stroke_paint::apply(vec_scene, &self.vec.pen, cmd);
        }
        if pending_brush_pick && let Some(host) = self.vec.pen.selected() {
            self.vec.path_pick = Some(crate::vec_pick::PathPick::BrushArt(host));
            eprintln!(
                "[ph2d-vec] brush: pick armado -- clique na FORMA ou no GRUPO que vai ser a \
                     arte do contorno (vazio = desiste)"
            );
        }
        if let Some(slot) = pending_texpat_pick
            && let Some(host) = self.vec.pen.selected()
        {
            self.vec.path_pick = Some(crate::vec_pick::PathPick::TexturePatternArt(host, slot));
            eprintln!(
                "[ph2d-vec] texture pattern: pick armado -- clique na FORMA ou no GRUPO que \
                     vai ser a arte (vazio = desiste)"
            );
        }
        if pending_pp_pick && let Some(motif) = self.vec.pen.selected() {
            self.vec.path_pick = Some(crate::vec_pick::PathPick::PatternMotif(motif));
            eprintln!(
                "[ph2d-vec] pattern on path: pick armado -- clique no CAMINHO-guia (vazio = \
                     desiste)"
            );
        }
    }
}
