//! **Fase do quadro: OS EFEITOS, O SPINE, OS PASSOS E A BOOLEANA** — a pilha de efeitos do caminho, o *Reset Spine*, os passos do blend e a booleana (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct PathEffectsSpineBoolIntents {
    pub(super) pending_vec_bool: Option<ph2d_vec_boolean::PathfinderOp>,
    pub(super) pending_reset_spine: bool,
    pub(super) pending_blend_steps: Option<u32>,
    pub(super) pending_fx_add: Option<usize>,
    pub(super) pending_fx_button: Option<(usize, crate::fx_bridge_dispatch::FxRowAction)>,
    pub(super) pending_fx_param: Option<(usize, usize, f64)>,
    pub(super) pending_fx_apply: bool,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_path_effects_spine_bool(&mut self, intents: PathEffectsSpineBoolIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        let PathEffectsSpineBoolIntents {
            pending_vec_bool,
            pending_reset_spine,
            pending_blend_steps,
            pending_fx_add,
            pending_fx_button,
            pending_fx_param,
            pending_fx_apply,
        } = intents;
        // ADR-0132: a pilha de efeitos do caminho selecionado. Os dois passam pela MESMA
        // `sole_path`, entao o que a secao PINTA e o que o clique ESCREVE nao podem divergir.
        if pending_fx_add.is_some()
            || pending_fx_button.is_some()
            || pending_fx_param.is_some()
            || pending_fx_apply
        {
            let sel = self.vec.pen.selected_paths().to_vec();
            if let Some(pid) = crate::fx_bridge::sole_path(&sel) {
                crate::fx_bridge_dispatch::apply(
                    vec_scene,
                    pid,
                    pending_fx_add,
                    pending_fx_button,
                    pending_fx_param,
                    pending_fx_apply,
                );
            }
        }
        // ADR-0128 C2b: Reset Spine — volta o(s) blend(s) selecionado(s) ao spine automático.
        if pending_reset_spine
            && crate::blend_live::reset_spine(
                sim,
                &self.vec.entities,
                &self.vec.pen,
                &mut self.vec.blend_spines,
            )
        {
            eprintln!("[ph2d-vec] blend: spine resetado ao automático");
        }
        // Arrastar o slider Steps retuna o blend SELECIONADO ao vivo (o recook lê
        // `VecBlend.steps`). Sem blend selecionado, é o valor de criação do próximo Blend.
        if let Some(steps) = pending_blend_steps {
            crate::blend_live::set_selected_steps(sim, &self.vec.entities, &self.vec.pen, steps);
        }
        if let Some(op) = pending_vec_bool {
            // **Um clique, três destinos** (`bool_gesture`): re-mirar um grupo booleano que a
            // seleção já habita · criar um, com o modo `Live` ligado · ou o caminho
            // destrutivo de sempre. ⚠️ A ordem é a lei: sem o primeiro, clicar "Intersect"
            // sobre um grupo vivo com o modo desligado CONSUMIRIA os operandos, e o artista
            // perderia a arte no gesto que ele fez para trocar a operação.
            let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            let live_mode = ph2d_panel_vector::state::bool_live_on();
            let has_group =
                crate::bool_gesture::group_of_selection(sim, &self.vec.entities, &sel).is_some();
            if has_group || live_mode {
                crate::bool_gesture::arm(
                    sim,
                    vec_scene,
                    &self.vec.entities,
                    &sel,
                    crate::bool_live::code_of_op(op),
                );
            } else {
                let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                crate::input_dispatch::apply_vec_boolean(vec_scene, &mut self.vec.pen, &xf, op);
            }
        }
    }
}
