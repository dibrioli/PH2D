//! **Fase do quadro: AS PONTES DOS PAINÉIS DE MUNDO** — hoje, o painel de tokens (OBRA 2 da
//! `line/render-loop`, 2026-09-13). As pontes do modelador 3D e da escultura que aqui viviam saíram
//! com o módulo (ADR-0179).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_world_panel_bridges(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            toasts,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // O painel de TOKENS (plano UI/UX W6): um painel de MUNDO, cuja visibilidade é do artista.
        // ⚠️ Ele tem de correr DEPOIS do dispatch de eventos (o intent de Reset é enfileirado ali) e
        // ANTES do paint (senão o frame pintaria a cor de antes do clique e o picker piscaria de
        // volta).
        if tokens_bridge::dispatch(hero, toasts) {
            self.title_dirty = true;
        }
    }
}
