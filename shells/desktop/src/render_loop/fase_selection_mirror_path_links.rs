//! **Fase do quadro: O TEXTO, O PADRÃO E O CONTORNO NO PAINEL** — o texto no caminho, o padrão no caminho e o contorno da selecção, publicados no painel
//! vectorial, e o espelho dos knobs do contorno (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_selection_mirror_path_links(
        &mut self,
    ) -> Option<Vec<ph2d_vec_scene::VecPathId>> {
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
        // Text on Path (plano 22): as duas perguntas que só a shell sabe responder —
        // *"esta seleção permite prender?"* (um texto + um caminho) e *"o texto em foco
        // já cavalga alguma coisa, e com que valores?"*. A primeira usa a MESMA porta
        // que o clique honra (`link_candidate`), senão o botão apareceria e recusaria.
        let sel = self.vec.pen.selected_paths().to_vec();
        ph2d_panel_vector::set_current_textpath_can_link(
            crate::vec_text_ride::link_candidate(sim, &self.vec.entities, &sel).is_some(),
        );
        let ride = crate::vec_text_ride::current(sim, &self.vec.entities, &sel);
        ph2d_panel_vector::set_current_textpath(
            ride.is_some(),
            ride.map_or(0.0, |r| f64::from(r.start_offset)),
            ride.is_some_and(|r| r.flip),
        );
        // Pattern on Path (plano 23): as MESMAS duas perguntas — *"esta seleção permite
        // prender?"* (dois caminhos) e *"o motivo em foco já cavalga algo, com que
        // valores?"*. A 1ª usa a MESMA porta que o clique honra (`link_candidate`), a 2ª
        // acha o motivo LINKADO na seleção (não o primário — depois de prender ele pode ser
        // o guia).
        ph2d_panel_vector::set_current_patternpath_can_link(
            crate::pattern_live::link_candidate(vec_scene, &sel).is_some(),
        );
        let pat = crate::pattern_live::current(sim, &self.vec.entities, &sel);
        // O Picker (Enio 2026-07-23): a porta EXPLÍCITA, oferecida com UM caminho selecionado
        // que ainda não é um motivo vinculado — a fonte à espera do clique do guia.
        // `pat.is_none()` exclui o motivo já preso (que mostra os controles, não a porta).
        ph2d_panel_vector::set_current_patternpath_can_pick(sel.len() == 1 && pat.is_none());
        ph2d_panel_vector::set_current_patternpath(
            pat.is_some(),
            pat.map_or(0.0, |p| f64::from(p.start_offset)),
            pat.map_or(1.0, |p| f64::from(p.end_offset)),
            pat.map_or(1.0, |p| f64::from(p.spacing)),
            pat.map_or(0.0, |p| f64::from(p.offset)),
            pat.is_some_and(|p| p.flip),
            f64::from(crate::pattern_live::current_rotation(
                sim,
                &self.vec.entities,
                &sel,
            )),
        );
        // Contour (pesquisa `20_*` #9): as MESMAS duas perguntas do Pattern — *"esta
        // seleção permite criar?"* e *"o que está armado, com que valores?"*. `can_add`
        // exige forma selecionada e nenhum contour nela: a seção mostra o botão OU os
        // controles, nunca os dois, e é isso que impede a swatch de existir sem alvo.
        let cont = crate::contour_live::current(sim, &self.vec.entities, &sel);
        ph2d_panel_vector::set_current_contour_can_add(!sel.is_empty() && cont.is_none());
        // O `d` do componente é MUNDO; o painel fala FRAÇÃO. A conversão usa a MESMA
        // `offset_scale` do arm e do drain — três leituras da mesma régua, uma função.
        // ⚠️ **Só com contour armado**, e é medida de custo, não de estilo: o
        // `vec_transform::build` percorre TODO caminho da cena e sobe a cadeia de pais
        // de cada um, alocando um mapa. Calculá-lo aqui incondicionalmente poria essa
        // varredura em todo frame com o painel aberto, para publicar um número que só
        // é lido quando há efeito — e sem contour o `d_frac` publicado é `0.0` de
        // qualquer maneira, sem passar pela escala.
        let cont_scale = cont.map_or(0.0, |_| {
            let xf = ph2d_vec_entities::transform::build(sim, &self.vec.entities);
            crate::vec_expand::offset_scale(vec_scene, &self.vec.pen, &xf)
        });
        ph2d_panel_vector::set_current_contour(
            cont.is_some(),
            cont.map_or(4.0, |(_, c)| f64::from(c.steps)),
            cont.map_or(0.0, |(_, c)| {
                if cont_scale > 0.0 {
                    c.d / cont_scale
                } else {
                    0.0
                }
            }),
            cont.map_or(1.0, |(_, c)| f64::from(c.accel)),
            cont.map_or(1, |(_, c)| c.join),
            cont.map_or(0, |(_, c)| c.side),
            cont.map_or([255, 255, 255, 255], |(_, c)| c.to),
        );
        // A BORDA: quando a forma espelhada muda (trocou a seleção, ou o Add acabou de
        // armar), os controles são reescritos no store a partir do componente. Sem isto o
        // `paint` — que lê o store primeiro, para não brigar com o arrasto — mostraria os
        // números da forma anterior sobre a forma nova.
        let cont_mirror = cont.map(|(id, _)| id);
        if cont_mirror != self.vec.contour_mirrored {
            self.vec.contour_mirrored = cont_mirror;
            if let Some((_, c)) = cont {
                let frac = if cont_scale > 0.0 {
                    c.d / cont_scale
                } else {
                    0.0
                };
                let steps = f64::from(c.steps);
                let accel = f64::from(c.accel);
                for (slider, chip, track, value) in [
                    (
                        ph2d_panel_vector::ids::VECTOR_CONTOUR_STEPS,
                        ph2d_panel_vector::ids::VECTOR_CONTOUR_STEPS_NUM,
                        ph2d_panel_vector::contour_steps_to_track(steps),
                        steps,
                    ),
                    (
                        ph2d_panel_vector::ids::VECTOR_CONTOUR_OFFSET,
                        ph2d_panel_vector::ids::VECTOR_CONTOUR_OFFSET_NUM,
                        ph2d_panel_vector::contour_d_to_track(frac),
                        frac * 100.0, // LITERAL-PX-OK: fração -> percentual do readout
                    ),
                    (
                        ph2d_panel_vector::ids::VECTOR_CONTOUR_ACCEL,
                        ph2d_panel_vector::ids::VECTOR_CONTOUR_ACCEL_NUM,
                        ph2d_panel_vector::contour_accel_to_track(accel),
                        accel,
                    ),
                ] {
                    hero.store.set_slider_value(slider, track);
                    hero.store.set_number_value(chip, value);
                }
            }
        }
        Some(sel)
    }
}
