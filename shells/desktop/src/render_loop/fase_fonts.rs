//! **Fase do quadro: AS FONTES** — a família corrente, o ciclo e a escolha de fonte, os eixos variáveis e importar uma fonte (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct FontsIntents {
    pub(super) pending_vec_text_axis: Option<(usize, f64)>,
    pub(super) pending_vec_font_cycle: Option<i32>,
    pub(super) pending_vec_font_pick: Option<usize>,
    pub(super) pending_vec_font_import: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_fonts(
        &mut self,
        intents: FontsIntents,
        vec_text_sel: Vec<ph2d_vec_scene::VecPathId>,
        editing_session: bool,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        let FontsIntents {
            pending_vec_text_axis,
            pending_vec_font_cycle,
            pending_vec_font_pick,
            pending_vec_font_import,
        } = intents;
        // A família "corrente" para o ciclo `<`/`>` é a do ALVO: o objeto de texto
        // selecionado (sem sessão) ou o default da shell.
        let cur_family = if editing_session {
            self.vec.text.family.clone()
        } else {
            crate::vec_text::selected_text_object(sim, &self.vec.entities, &vec_text_sel)
                .map_or_else(|| self.vec.text.family.clone(), |(_, _, p)| p.family)
        };
        if let Some(dir) = pending_vec_font_cycle {
            let next = crate::vec_font::cycle_family(cur_family.as_deref(), dir);
            if !editing_session {
                crate::vec_text::set_selected_text_font(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &vec_text_sel,
                    next.clone(),
                );
            }
            crate::vec_text::set_text_font(
                &mut self.vec.text_edit,
                &mut self.vec.text.family,
                &mut self.vec.text.extra_axes,
                vec_scene,
                next,
            );
        }
        if let Some(i) = pending_vec_font_pick {
            // Índice na MESMA lista que gerou as previews → família escolhida.
            let family = crate::vec_font::pickable_families()
                .get(i)
                .cloned()
                .flatten();
            if !editing_session {
                crate::vec_text::set_selected_text_font(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &vec_text_sel,
                    family.clone(),
                );
            }
            crate::vec_text::set_text_font(
                &mut self.vec.text_edit,
                &mut self.vec.text.family,
                &mut self.vec.text.extra_axes,
                vec_scene,
                family,
            );
        }
        if let Some((index, value)) = pending_vec_text_axis {
            crate::vec_text::apply_text_axis(
                &mut self.vec.text_edit,
                &mut self.vec.text.extra_axes,
                vec_scene,
                index,
                value,
            );
        }
        if pending_vec_font_import {
            let imported = crate::vec_text::import_text_font(
                &mut self.vec.text_edit,
                &mut self.vec.text.family,
                &mut self.vec.text.extra_axes,
                vec_scene,
            );
            // Sem sessão, a fonte importada vai para o objeto de texto SELECIONADO.
            if imported && !editing_session {
                let fam = self.vec.text.family.clone();
                crate::vec_text::set_selected_text_font(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &vec_text_sel,
                    fam,
                );
            }
            // A fonte importada entra no dropdown: reconstrói as previews agora.
            #[cfg(feature = "panel-vector")]
            if imported {
                ph2d_panel_vector::set_current_text_font_previews(
                    crate::vec_font_preview::build_previews(),
                );
            }
            #[cfg(not(feature = "panel-vector"))]
            let _ = imported;
        }
    }
}
