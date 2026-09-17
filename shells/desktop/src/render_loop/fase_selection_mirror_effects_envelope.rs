//! **Fase do quadro: OS EFEITOS E O ENVELOPE NO PAINEL** — a pilha de efeitos do caminho seleccionado e o modo e os presets da gaiola, publicados no
//! painel vectorial (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_selection_mirror_effects_envelope(&mut self, env_container: Option<u64>) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        // ADR-0132: o Trim do caminho selecionado. A MESMA `sole_path` do dispatch --
        // o painel nao pode oferecer controles para um caminho que o clique nao alcanca.
        let fx_target = crate::fx_bridge::sole_path(self.vec.pen.selected_paths());
        ph2d_panel_vector::set_current_effects(
            fx_target.is_some(),
            ph2d_vec_scene::effect::PathEffect::KINDS,
            fx_target.map_or_else(Vec::new, |pid| crate::fx_bridge::stack_view(vec_scene, pid)),
        );
        // Qual chip de gesto acende. O painel pergunta ao MESMO container que o
        // dispatch vai escrever, senao a tela mostraria um gesto e o clique mudaria outro.
        ph2d_panel_vector::set_current_envelope_mode(env_container.map_or(0, |b| {
            match ph2d_app_vec::envelope_gesture::kind_of(sim, b) {
                ph2d_ecs::EnvelopeKind::Perspective => 0,
                ph2d_ecs::EnvelopeKind::Mesh => 1,
                ph2d_ecs::EnvelopeKind::Pins => 2,
            }
        }));
        // Os presets de gaiola: o painel se auto-popula desta lista, entao acrescentar um
        // preset e' uma linha em `EnvelopeWarp::ALL` e ZERO mudanca de painel.
        let labels: Vec<&'static str> = ph2d_ecs::EnvelopeWarp::ALL
            .iter()
            .map(|w| w.label())
            .collect();
        let (active, bend) = env_container
            .and_then(|b| ph2d_app_vec::envelope_gesture::warp_of(sim, b))
            .map_or((None, 0.0), |(w, bend)| {
                (
                    w.and_then(|w| ph2d_ecs::EnvelopeWarp::ALL.iter().position(|c| *c == w)),
                    bend,
                )
            });
        ph2d_panel_vector::set_current_envelope_presets(&labels, active, bend);
    }
}
