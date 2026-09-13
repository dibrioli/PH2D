//! **Fase do quadro: OS RESTOS DO DRENO** — as acções que o dreno do barramento deixou para a remoção de fundo e para o Painter (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct DrainLeftoversIntents {
    pub(super) bgremoval_leftover: Vec<ph2d_editor_core::action_bus::EditorAction>,
    pub(super) painter_leftover: Vec<ph2d_editor_core::action_bus::EditorAction>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_drain_leftovers(&mut self, intents: DrainLeftoversIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { hero_screen, .. } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let DrainLeftoversIntents {
            bgremoval_leftover,
            painter_leftover,
        } = intents;
        for a in bgremoval_leftover {
            hero.bus.push(a);
        }
        for a in painter_leftover {
            hero.bus.push(a);
        }
    }
}
