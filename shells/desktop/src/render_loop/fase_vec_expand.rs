//! **Fase do quadro: O APPLY OFFSET E O POWER STROKE** — materializar o offset vivo e o perfil vivo, ou o caminho numérico do expand; o Power
//! Stroke materializado ACABA o quadro (a fase devolve-o à chamada) (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct VecExpandIntents {
    pub(super) pending_vec_expand: Option<crate::vec_expand::Expand>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    ///
    /// ⛔ **`None` = o QUADRO acaba aqui.** A saída do quadro que o corpo trazia (`return;`) volta à chamada,
    /// que a repete no sítio dela — dentro de um método ela sairia só da fase, e o quadro continuaria.
    pub(super) fn fase_vec_expand(&mut self, intents: VecExpandIntents) -> Option<()> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        let VecExpandIntents { pending_vec_expand } = intents;
        if let Some(cmd) = pending_vec_expand {
            let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
            // **Apply Offset MATERIALIZA o offset vivo** — é o único momento em que os
            // vértices do offset passam a existir no documento (Enio, 2026-07-21). Cada
            // forma é assada com o `VecOffset` DELA, e não com o slider: duas formas podem
            // carregar offsets diferentes, e o botão tem de honrar o que está na TELA.
            // Passa pela MESMA porta do caminho numérico (`expand_selection`), senão
            // haveria uma 2ª maneira de a geometria do offset entrar na cena.
            // **O botão Power Stroke MATERIALIZA o perfil vivo** (ADR-0148) — o espelho
            // exato do Apply Offset logo abaixo. Sem perfil armado na seleção devolve
            // `false`, e o clique segue pelo caminho numérico (que lê os sliders).
            let sel_now: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
            if matches!(cmd, crate::vec_expand::Expand::PowerStroke { .. })
                && crate::profile_live::materialise(
                    vec_scene,
                    sim,
                    &mut self.vec.pen,
                    &self.vec.entities,
                    &xf,
                    &sel_now,
                )
            {
                // A forma nova não tem perfil vivo, e knobs parados num afinamento sobre ela
                // mentiriam sobre o que está na cena — o mesmo argumento do slider de Offset.
                self.vec.profile_mirrored = None;
                crate::profile_live::write_preset_to_store(
                    &mut hero.store,
                    &ph2d_vec_scene::WidthProfile::UNIFORM,
                );
                return None;
            }
            let materialised = matches!(cmd, crate::vec_expand::Expand::Offset { .. }) && {
                let ids: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
                crate::offset_live::materialise(
                    vec_scene,
                    sim,
                    &mut self.vec.pen,
                    &self.vec.entities,
                    &xf,
                    &ids,
                )
            };
            if materialised {
                self.vec.offset_mirrored = None;
                // O slider volta ao zero: a forma nova não tem offset vivo, e um slider
                // parado em +40% sobre ela mentiria sobre o que está na cena.
                hero.store.set_slider_value(
                    ph2d_panel_vector::ids::VECTOR_EXPAND_OFFSET,
                    ph2d_tool_vector::params::offset_frac_to_slider(0.0),
                );
            } else {
                // O caminho NUMÉRICO (sem offset vivo armado): a distância vem do slider —
                // a MESMA fonte que o chip mostra —, fração × escala da seleção atual.
                let d = hero
                    .store
                    .slider(ph2d_panel_vector::ids::VECTOR_EXPAND_OFFSET)
                    .map_or(ph2d_tool_vector::params::OFFSET_DEFAULT_FRAC, |(_, v)| {
                        ph2d_tool_vector::params::slider_to_offset_frac(v)
                    })
                    * crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &xf);
                // ⚠️ O PERFIL também vem dos sliders — a mesma fonte que o chip mostra. O
                // `expand_for_id` devolve o comando com o perfil UNIFORME (ele não tem o
                // store), e é aqui que ele é preenchido; um default cravado lá seria um 2º
                // lugar decidindo o que o artista já arrastou.
                let cmd = match cmd {
                    crate::vec_expand::Expand::PowerStroke { .. } => {
                        crate::vec_expand::Expand::PowerStroke {
                            stops: crate::profile_live::preset_from_store(&hero.store).to_stops(),
                        }
                    }
                    other => other,
                };
                crate::vec_expand::apply_vec_expand(
                    vec_scene,
                    &mut self.vec.pen,
                    &xf,
                    cmd.clone(),
                    d,
                );
                // O botão de Offset (caminho numérico: arrastar sem seleção viva e clicar)
                // recentra o slider — cada aplicação offseta pelo valor mostrado e zera.
                if matches!(cmd, crate::vec_expand::Expand::Offset { .. }) {
                    hero.store.set_slider_value(
                        ph2d_panel_vector::ids::VECTOR_EXPAND_OFFSET,
                        ph2d_tool_vector::params::offset_frac_to_slider(0.0),
                    );
                }
            }
        }
        Some(())
    }
}
