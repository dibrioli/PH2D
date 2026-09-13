//! **Fase do quadro: OS COMANDOS E OS KNOBS DO CONTORNO** — os comandos e os knobs do contorno. ⚠️ O bloco da pilha de filtros, a seguir, fica no
//! ORQUESTRADOR: ele já só chama as suas fases, e a cola delas é do quadro (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct ContourVerbsIntents {
    pub(super) pending_contour: Option<crate::contour_live::ContourCmd>,
    pub(super) pending_contour_steps: Option<f64>,
    pub(super) pending_contour_d: Option<f64>,
    pub(super) pending_contour_accel: Option<f64>,
    pub(super) pending_contour_join: Option<u8>,
    pub(super) pending_contour_side: Option<u8>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_contour_verbs(&mut self, intents: ContourVerbsIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let ContourVerbsIntents {
            pending_contour,
            pending_contour_steps,
            pending_contour_d,
            pending_contour_accel,
            pending_contour_join,
            pending_contour_side,
        } = intents;
        // Contour (pesquisa `20_*` #9): os comandos e os knobs, todos pela porta única
        // `contour_live`. O `recook` do frame seguinte redesenha os anéis.
        //
        // ⚠️ **Add/Remove/Expand agem sobre a SELEÇÃO inteira** e os knobs também: uma seleção
        // de duas formas ganha dois contours, e mexer no slider afina os dois. É o mesmo
        // desenho do Offset vivo — e o oposto do Join da física, que precisa de fan-out
        // BLOQUEADO porque criaria um objeto por par. Aqui cada forma tem o seu, e um efeito
        // por forma é exatamente o que o artista pediu ao selecionar duas.
        {
            let sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            match pending_contour {
                Some(crate::contour_live::ContourCmd::Add) => {
                    let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                    let scale = crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &xf);
                    let n = crate::contour_live::arm(sim, &self.vec.entities, &sel, scale);
                    eprintln!("[ph2d-vec] contour: armado em {n} forma(s)");
                }
                Some(crate::contour_live::ContourCmd::Remove) => {
                    let n = crate::contour_live::remove(sim, &self.vec.entities, &sel);
                    eprintln!("[ph2d-vec] contour: removido de {n} forma(s)");
                }
                Some(crate::contour_live::ContourCmd::Expand) => {
                    let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                    let runs =
                        self.contour_live
                            .expand(sim, vec_scene, &self.vec.entities, &xf, &sel);
                    if runs.is_empty() {
                        eprintln!(
                            "[ph2d-vec] contour: nada a expandir (selecione uma forma com contour)"
                        );
                    } else {
                        let n: usize = runs.iter().map(|r| r.len().saturating_sub(1)).sum();
                        eprintln!("[ph2d-vec] contour: expandido em {n} anel(is)");
                        self.vec.restack.extend(runs);
                    }
                }
                None => {}
            }
            if let Some(v) = pending_contour_steps {
                let steps = v.max(1.0) as u16;
                crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.steps = steps);
            }
            if let Some(frac) = pending_contour_d {
                // FRAÇÃO → MUNDO na fronteira, com a MESMA escala do `arm` e do Offset.
                let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
                let d = frac * crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &xf);
                crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.d = d);
            }
            if let Some(v) = pending_contour_accel {
                #[allow(clippy::cast_possible_truncation)]
                let accel = v as f32;
                crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.accel = accel);
            }
            if let Some(code) = pending_contour_join {
                crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.join = code);
            }
            if let Some(code) = pending_contour_side {
                crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.side = code);
            }
            // A COR-ALVO vem do picker OKLCH partilhado, lido de volta como o Stroke e o Fill
            // fazem no `vector_bridge` — mas AQUI, porque o alvo da escrita é um componente
            // ECS e este é o bloco que tem `sim` e o mapa em mãos. A swatch é marcada como
            // picker-swatch no `paint.rs` do painel; o Down abre o picker por dispatch
            // genérico, e o que chega cá é só a escolha.
            if hero.store.picker_target() == Some(ph2d_editor_core::ids::VECTOR_CONTOUR_TO)
                && let Some((value, _, _, _)) = hero
                    .store
                    .blender_picker(ph2d_editor_core::ids::INSP_BLENDER_PICKER)
            {
                let to = value.rgba;
                crate::contour_live::edit(sim, &self.vec.entities, &sel, |c| c.to = to);
            }
        }
    }
}
