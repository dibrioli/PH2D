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

/// O que a entrada da corrida guarda e a saída repõe.
pub(crate) struct FotoDoJogo {
    estado: super::ProjectState,
    marca: super::SelectionMark,
    /// A vista do editor antes do palco (ecrã cheio ou janela); `None` = jogou no editor.
    pub(crate) vista: Option<crate::janela_do_jogo::VistaAntes>,
}

/// O smoke do jogo (ou `PH2D_JOGO_LOG=1`) imprime o resumo do documento no Play e no Stop.
fn log_on() -> bool {
    ph2d_app_game::smoke::armed_level().is_some() || std::env::var_os("PH2D_JOGO_LOG").is_some()
}

impl crate::App {
    /// As bordas da corrida deste quadro. `corre` = a cena está a correr depois do dreno dela.
    pub(crate) fn jogo_quadro(&mut self, corre: bool) {
        match self.undo.jogo.quadro(corre) {
            Passo::Fotografa => self.jogo_entra(),
            Passo::Repoe(foto) => self.jogo_sai(foto),
            Passo::Nada => {}
        }
    }

    fn jogo_entra(&mut self) {
        self.settle_tree_before_capture();
        let Some(estado) = self.capture_project() else {
            return;
        };
        // O que o artista fez e o undo ainda não registou vira um passo ANTES da corrida: a saída
        // repõe a fotografia como baseline, e sem isto essa edição ficava sem Ctrl+Z.
        if self.undo_baseline.as_ref() != Some(&estado) {
            self.empurra(estado.clone());
        }
        if log_on() {
            println!("[jogo] Play: documento {:016x}", estado.digest());
        }
        // As animações com «tocar sozinho» começam com o jogo (só o abrir do projecto as ligava).
        if let Some(g) = self.gfx.as_mut() {
            crate::render_loop::start_autoplay_animations(&mut g.sim);
        }
        let marca = self.field_selection_mark();
        let vista = self.palco_entra();
        self.undo.jogo.guarda(FotoDoJogo {
            estado,
            marca,
            vista,
        });
    }

    fn jogo_sai(&mut self, foto: FotoDoJogo) {
        self.apply_project_with(&foto.estado, Some(&foto.marca));
        if let Some(v) = foto.vista {
            self.palco_sai(v);
        }
        if log_on() {
            let agora = self.capture_project().map(|s| s.digest());
            let antes = foto.estado.digest();
            println!(
                "[jogo] Stop: documento {} · igual ao do Play: {}",
                agora.map_or("?".into(), |d| format!("{d:016x}")),
                agora == Some(antes)
            );
        }
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts
                .push(ph2d_editor_core::toast::Toast::info(ph2d_i18n::tr(
                    "shell.undo_jogo.scene_restored",
                )));
        }
    }

    /// O documento do artista enquanto o jogo corre (o que o save grava), ou `None` fora do Play.
    pub(crate) fn documento_do_jogo(&self) -> Option<super::ProjectState> {
        self.undo.jogo.foto().map(|f| f.estado.clone())
    }

    /// Um Ctrl+Z com o jogo a correr: recusado, e dito. `true` = recusou.
    pub(super) fn jogo_recusa_o_undo(&mut self) -> bool {
        if !self.undo.jogo.a_correr() {
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
