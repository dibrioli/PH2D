//! **Fase do quadro: A PALETA DE COMPONENTES** — abrir quando o `+` do Inspector a pede, reconstruir no
//! *Show all* e anexar o componente escolhido (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Corre DEPOIS do dreno do pivot do joint: esteve no meio do par `let joint_pivot_commit` → `if let`,
//! e um gate reprovou.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_component_palette(&mut self, add_component_for: Option<u64>) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            toasts,
            hero_screen,
            component_registry,
            component_palette_target,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // ⚠️ **E fica DEPOIS do dreno do pivot do joint, pela razão que os dois blocos abaixo
        // já têm escrita:** ele esteve NO MEIO do par `let joint_pivot_commit = …` →
        // `if let Some(…) = joint_pivot_commit`, e o gate
        // `the_position_commit_reseats_the_anchor_through_the_door` reprovou — ele lê os 3000
        // bytes a seguir à captura à procura da porta, e 27 linhas alheias empurraram-na para
        // fora da janela. *A janela é a forma de o gate exigir que a captura e o dreno de uma
        // intenção fiquem à vista um do outro; a cura é tirar o intruso, nunca alargá-la.*
        // ⭐ **O `+` do Inspector, as DUAS pontas** (ADR-0166 / F3) — abrir a paleta para quem
        // pediu, e anexar o que ela escolheu. Irmã por assunto (`component_attach`), como a
        // biblioteca do Motion é irmã do `motion_bridge`.
        ph2d_app_components::component_attach::open_palette_if_asked(
            hero,
            sim,
            component_registry,
            add_component_for,
            component_palette_target,
        );
        // ⭐ **A caixa *Show all*** — o widget vira o estado dele e avisa; quem reconstrói o
        // modelo é quem abriu a paleta (só ele sabe o que «mostrar tudo» quer dizer).
        ph2d_app_components::component_attach::refresh_palette_on_toggle(
            hero,
            sim,
            component_registry,
            *component_palette_target,
        );
        // ⚠️ O pick chega **noutro quadro** (a paleta fica aberta), e por isso o alvo vive no
        // `AppGfx` em vez de num local deste laço.
        let picked =
            ph2d_app_components::component_attach::route_pick(hero, component_palette_target);
        ph2d_app_components::component_attach::attach_picked(
            picked.as_ref(),
            sim,
            component_registry,
            // ⭐ A COMPOSIÇÃO entrega as sementes da família dona (auditoria A1): a família de
            // componentes não conhece a física.
            ph2d_app_physics::physics_seed::COMPONENT_SEEDS,
            toasts,
        );
    }
}
