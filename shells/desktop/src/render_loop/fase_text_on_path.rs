//! **Fase do quadro: O TEXTO NO CAMINHO** — o deslocamento, o comando e o picker do texto no caminho (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct TextOnPathIntents {
    pub(super) pending_textpath: Option<crate::vec_text_ride::TextPathCmd>,
    pub(super) pending_textpath_offset: Option<f64>,
    pub(super) pending_text_pick: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_text_on_path(&mut self, intents: TextOnPathIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        let TextOnPathIntents {
            pending_textpath,
            pending_textpath_offset,
            pending_text_pick,
        } = intents;
        // ADR-0129: **Envelope** — envolve a seleção (1..N formas) num container com a gaiola em
        // repouso. Síncrono (as formas já existem; o container não tem path), então age já.
        // Plano 22: prender / soltar / trocar o lado. Um comando só por frame (é um
        // clique), e todos passam pelas portas do `vec_text_ride` — que re-cozinham pela
        // porta de sempre, para não haver uma segunda resposta a "como um texto vira
        // geometria".
        if let Some(v) = pending_textpath_offset {
            let sel = self.vec.pen.selected_paths().to_vec();
            crate::vec_text_ride::edit(sim, vec_scene, &self.vec.entities, &sel, |l| {
                l.start_offset = v as f32;
            });
        }
        if let Some(cmd) = pending_textpath {
            let sel = self.vec.pen.selected_paths().to_vec();
            let done = match cmd {
                crate::vec_text_ride::TextPathCmd::Link => {
                    crate::vec_text_ride::link(sim, vec_scene, &self.vec.entities, &sel)
                }
                crate::vec_text_ride::TextPathCmd::Detach => {
                    crate::vec_text_ride::detach(sim, vec_scene, &self.vec.entities, &sel)
                }
                crate::vec_text_ride::TextPathCmd::Flip(v) => {
                    crate::vec_text_ride::edit(sim, vec_scene, &self.vec.entities, &sel, |l| {
                        l.flip = v;
                    })
                }
            };
            if !done {
                eprintln!(
                    "[ph2d-vec] text on path: selecione o TEXTO e um caminho (ou um texto \
                         ja' preso, para soltar)"
                );
            }
        }
        // Picker do texto (Enio 2026-07-23): o botão só ARMOU; aqui capturamos a FONTE — o texto
        // em foco — para o clique seguinte no canvas escolher o guia. Capturamos o id agora porque
        // esse clique pode mudar a seleção (ele ESCOLHE o guia, não deve virar a fonte).
        if pending_text_pick {
            let sel = self.vec.pen.selected_paths().to_vec();
            if let Some((text, _, _)) =
                crate::vec_text_object::selected_text_object(sim, &self.vec.entities, &sel)
            {
                self.vec.path_pick = Some(crate::vec_pick::PathPick::TextObject(text));
                eprintln!(
                    "[ph2d-vec] text on path: pick armado -- clique no CAMINHO-guia (vazio = \
                         desiste)"
                );
            }
        }
    }
}
