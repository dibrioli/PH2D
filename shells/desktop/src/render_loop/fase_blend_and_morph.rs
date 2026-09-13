//! **Fase do quadro: O BLEND E O MORPH** — criar o blend e o morph, arrastar o `t` do morph, e expandir e soltar o blend (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct BlendAndMorphIntents {
    pub(super) pending_create_blend: bool,
    pub(super) pending_expand_blend: bool,
    pub(super) pending_release_blend: bool,
    pub(super) pending_create_morph: bool,
    pub(super) pending_morph_t: Option<f32>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_blend_and_morph(&mut self, intents: BlendAndMorphIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            tools,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let BlendAndMorphIntents {
            pending_create_blend,
            pending_expand_blend,
            pending_release_blend,
            pending_create_morph,
            pending_morph_t,
        } = intents;
        // Apply a Boolean button press (drained above) to the document before
        // the bridge/render so the result selects + renders this frame
        // (mirror of the U/I/D hotkeys' `vec_boolean`).
        // ADR-0128: o botão "Blend" cria o Blend Object VIVO sobre as formas fechadas
        // selecionadas (2..=5, em z). `create` empurra o spine e devolve o componente; o
        // `sync`/`upkeep`/`recook` do frame dão vida a ele. Seleciona o OBJETO (o spine) para
        // o slider Steps passar a mirar nele.
        if pending_create_blend {
            let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
            // Os passos vêm do slider do painel — a fonte da verdade é o widget, não uma
            // cópia no shell (uma cópia driftaria do que o artista está VENDO).
            let steps = hero
                .store
                .slider(ph2d_panel_vector::ids::VECTOR_BLEND_STEPS)
                .map_or(ph2d_tool_vector::params::BLEND_STEPS_DEFAULT, |(_, v)| {
                    ph2d_tool_vector::params::blend_steps_from_track(f64::from(v))
                });
            // A ORDEM da cadeia: no modo Pick Shapes, a de CLIQUE (a lista escolhida a dedo);
            // fora dele, a de z da seleção (ADR-0128 C2b). O Pick é o "escolher a ordem" do Enio.
            let picking = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::PickBlend;
            let sources = if picking && self.vec.blend_picks.len() >= 2 {
                self.vec.blend_picks.clone()
            } else {
                crate::blend_live::selected_closed_in_z(vec_scene, &self.vec.pen)
            };
            if let Some((spine, blend)) = crate::blend_live::create(vec_scene, &xf, &sources, steps)
            {
                self.vec.pen.select_many(&[spine]);
                self.vec.blend_pending = Some((spine, blend));
                self.vec.blend_picks.clear();
                // Feito o blend, volta ao Select — o objeto novo está selecionado e o gizmo
                // manda (o modo Pick já cumpriu o papel de juntar a lista). Inline do
                // `vec_set_draw_mode` (que re-borrowaria o `gfx` já destructurado): a tool é a
                // dona do modo, `vec_draw_config` é o espelho lido no mesmo frame.
                crate::render_loop::vector_bridge::set_mode(
                    tools,
                    ph2d_tool_vector::DrawMode::Select,
                );
                self.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Select;
                eprintln!(
                    "[ph2d-vec] blend: objeto vivo sobre {} formas, {steps} passos/elo",
                    sources.len()
                );
            } else {
                eprintln!("[ph2d-vec] blend: selecione de 2 a 5 formas FECHADAS");
            }
        }
        // **MORPH** — o irmão animável do blend: UMA forma entre DUAS, com o `t` keyável.
        // Mesma mecânica do `create` acima (`push` do path + componente; o
        // `sync`/`upkeep`/`recook` do frame lhe dão vida), e a mesma escolha de fontes: no
        // Pick Shapes a ordem de CLIQUE, fora dele a ordem de z.
        if pending_create_morph {
            let picking = self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::PickBlend;
            let sources = if picking && self.vec.blend_picks.len() >= 2 {
                self.vec.blend_picks.clone()
            } else {
                crate::blend_live::selected_closed_in_z(vec_scene, &self.vec.pen)
            };
            // DUAS, e exatamente duas: o morph é um `t` sobre UM par. Uma cadeia de 3+ formas
            // é o Blend — e recusar aqui em voz alta é melhor do que morfar as duas primeiras
            // e deixar o artista a descobrir sozinho quais foram escolhidas.
            if let [a, b] = sources[..] {
                let (id, morph) = crate::morph_live::create(vec_scene, a, b);
                self.vec.pen.select_many(&[id]);
                self.vec.morph_pending = Some((id, morph));
                self.vec.blend_picks.clear();
                crate::render_loop::vector_bridge::set_mode(
                    tools,
                    ph2d_tool_vector::DrawMode::Select,
                );
                self.vec.draw_config.mode = ph2d_tool_vector::DrawMode::Select;
                eprintln!("[ph2d-vec] morph: objeto vivo entre 2 formas (t animável)");
            } else {
                eprintln!(
                    "[ph2d-vec] morph: selecione exatamente 2 formas FECHADAS (tem {})",
                    sources.len()
                );
            }
        }
        // Arrastar o slider `t` move o morph SELECIONADO pelo caminho, ao vivo.
        if let Some(t) = pending_morph_t {
            for id in self.vec.pen.selected_paths() {
                let Some(&bits) = self.vec.entities.get(id) else {
                    continue;
                };
                let e = ph2d_ecs::Entity::from_bits(bits);
                if let Some(mut m) = sim.world_mut().get_mut::<ph2d_ecs::VecMorph>(e) {
                    m.t = t;
                }
            }
        }
        // ADR-0128 Fase D: **Expand** — materializa os passos VIRTUAIS em formas REAIS e
        // descarta o objeto vivo. A sequência de z que ele pede espera em `vec_restack`: as
        // entidades dos passos só nascem no `sync`, e quem manda no z é a ÁRVORE (ADR-0110).
        if pending_expand_blend {
            let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
            let runs = crate::blend_live::expand(
                sim,
                vec_scene,
                &self.vec.entities,
                &xf,
                &mut self.vec.pen,
            );
            if runs.is_empty() {
                eprintln!("[ph2d-vec] blend: selecione um blend (a linha, ou uma forma dele)");
            } else {
                let n: usize = runs.iter().map(Vec::len).sum();
                eprintln!("[ph2d-vec] blend: expandido em {n} forma(s)");
                self.vec.restack.extend(runs);
            }
        }
        // ADR-0128 Fase D: **Release** — desfaz o blend (os passos somem, as fontes ficam).
        if pending_release_blend
            && crate::blend_live::release(sim, vec_scene, &self.vec.entities, &mut self.vec.pen)
        {
            eprintln!("[ph2d-vec] blend: solto (as formas-fonte ficam)");
        }
    }
}
