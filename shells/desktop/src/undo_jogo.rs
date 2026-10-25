//! ⭐ **O Play sobre uma CÓPIA** (GameEngineFuncional R0a, `docs/GameEngineFuncional/04_R0_N0_plano.md`):
//! a entrada da corrida fotografa o projecto pela MESMA porta do undo e do save, a saída repõe-no pela
//! porta do Ctrl+Z. Quando, decide [`ph2d_app_game::Sessao`]; a fase que pergunta é a `fase_jogo`,
//! antes dos motores do passo fixo.

use ph2d_app_game::Passo;

impl super::ProjectState {
    /// A impressão digital do documento: os bytes do save numa soma curta (chaves fixas, logo a
    /// mesma em toda corrida). É o que o gate e o log do Play comparam.
    #[must_use]
    pub(crate) fn digest(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::hash::DefaultHasher::new();
        postcard::to_allocvec(self).unwrap_or_default().hash(&mut h);
        h.finish()
    }
}

impl crate::App {
    /// As bordas da corrida deste quadro. `corre` = a cena está a correr depois do dreno dela.
    pub(crate) fn jogo_quadro(&mut self, corre: bool) {
        match self.jogo.quadro(corre) {
            Passo::Fotografa => self.jogo_entra(),
            Passo::Repoe((foto, mark)) => {
                self.apply_project_with(&foto, Some(&mark));
                if let Some(gfx) = self.gfx.as_mut() {
                    gfx.toasts
                        .push(ph2d_editor_core::toast::Toast::info(ph2d_i18n::tr(
                            "shell.undo_jogo.scene_restored",
                        )));
                }
            }
            Passo::Nada => {}
        }
    }

    fn jogo_entra(&mut self) {
        self.settle_tree_before_capture();
        let Some(foto) = self.capture_project() else {
            return;
        };
        // O que o artista fez e o undo ainda não registou vira um passo ANTES da corrida: a saída
        // repõe a fotografia como baseline, e sem isto essa edição ficava sem Ctrl+Z.
        if self.undo_baseline.as_ref() != Some(&foto) {
            self.empurra(foto.clone());
        }
        let mark = self.field_selection_mark();
        self.jogo.guarda((foto, mark));
    }

    /// Um Ctrl+Z com o jogo a correr: recusado, e dito. `true` = recusou.
    pub(super) fn jogo_recusa_o_undo(&mut self) -> bool {
        if !self.jogo.a_correr() {
            return false;
        }
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts
                .push(ph2d_editor_core::toast::Toast::info(ph2d_i18n::tr(
                    "shell.undo_jogo.stop_to_undo",
                )));
        }
        true
    }
}

#[cfg(test)]
#[path = "undo_jogo_tests.rs"]
mod tests;
