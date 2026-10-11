//! **Fase do quadro: O OVERLAY DE EDIÇÃO** — os nós e as alças da forma em edição (OBRA 2 da
//! `line/render-loop`, 2026-09-12). ⚠️ As imagens presas ao esqueleto desenhavam-se aqui, pelo Vello, até
//! 2026-09-13; hoje são sprites do quadro com malha (plano `docs/Skeleton/03`, W2).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_edit_overlay(
        &mut self,
        vec_view: ph2d_vec_scene::VecViewState,
        vec_xf: ph2d_vec_scene::VecXforms,
        cam_affine: ph2d_vector::Affine,
        overlay: ph2d_app_vec::overlay::VecOverlayPlan,
    ) -> Option<(ph2d_vec_scene::VecXforms, ph2d_vector::Affine)> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            vector_scene,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        if overlay.edit {
            // ⚠️ A gaiola do Envelope SUBSTITUI a edição de nós. Quando a seleção é um envelope,
            // a forma sob a gaiola é DERIVADA (os nós dela são a SAÍDA do warp, não estado
            // editável), então NÃO se desenham as alças de nó dela — elas confundem (não se
            // pode arrastá-las) e expõem o handle longo que o refit deixa numa quina CÔNCAVA
            // (o "traço à deriva" que o Enio reportou numa estrela sob envelope, 2026-07-24).
            // Você edita a GAIOLA (desenhada abaixo). É o idioma do Illustrator/Affinity: com um
            // envelope ativo, os nós do objeto somem.
            let envelope_selected = hero
                .gizmo
                .selection
                .is_some_and(|bits| ph2d_app_vec::envelope_gesture::is_envelope(sim, bits));
            if !envelope_selected
                && !ph2d_app_vec::shape_handles_live::hides_the_nodes(&self.vec, sim)
            {
                ph2d_vec_render::draw_overlays(
                    vec_scene,
                    &vec_view,
                    ph2d_app_vec::edit_marks::node_overlay_selected(&self.vec),
                    self.vec.pen.selected_paths(),
                    self.vec.pen.selected_verts(),
                    &vec_xf,
                    cam_affine,
                    vector_scene,
                );
                // ⭐⭐⭐ O anel do nó SOLDADO (plano 39) e os nós do Edit Shape (F7e).
                ph2d_app_vec::edit_marks::draw(
                    &self.vec,
                    vec_scene,
                    &vec_view,
                    &vec_xf,
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
            }
            // NOTA: as alças de raio de quina (Live Corners) não são mais desenhadas — o
            // arredondar/chanfrar quina virou o par de ferramentas Fillet / Chamfer (o gesto de
            // clicar-e-arrastar). A exclusão de forma VIVA (`crate::corner_handles::has_derived_verts`)
            // segue viva: é o que o press dessas ferramentas consulta no `input_dispatch`.
            // A gaiola do **Envelope** (ADR-0129): os 4 cantos que o Node arrasta para
            // deformar as formas que a gaiola contém. Alça PRÓPRIA (não o gizmo de sprite,
            // §3.3), desenhada só quando a seleção do gizmo é de fato um envelope — o flag de
            // modo (`envelope_cage`) diz o MODO, o `view` devolve `None` quando a entidade
            // selecionada não carrega um `VecEnvelope`. O alvo é o CONTAINER (Fatia 3): a
            // regra seleciona-só-o-container põe os bits dele em `hero.gizmo.selection`.
            if overlay.envelope_cage
                && let Some(cage) = ph2d_app_vec::envelope_gesture::view(
                    sim,
                    hero.gizmo.selection,
                    self.vec.envelope_drag,
                )
            {
                ph2d_vec_render::draw_envelope_cage(&cage, cam_affine, hero.theme, vector_scene);
            }
            crate::vec_overlay_diag::dump(vec_scene, sim, hero.gizmo.selection);
            // Os PINOS (Fatia E) — o mesmo gate de modo, outro overlay: no gesto Pins o `view`
            // devolve `None` (nao ha gaiola) e sao os pinos que se desenham. Nunca os dois.
            if overlay.envelope_cage
                && let Some(bits) = hero.gizmo.selection
            {
                let pins = ph2d_app_vec::envelope_gesture::pins_world(sim, bits);
                ph2d_vec_render::draw_envelope_pins(
                    &pins,
                    self.vec
                        .envelope_drag
                        .filter(|(d, _)| *d == bits)
                        .map(|(_, i)| i),
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
            }
            // Gradient handles (multi-point dots, or linear/radial endpoints)
            // when the selected path has a gradient fill. A geometria do gradiente
            // é LOCAL como a do path, então sobe pelo afim dele.
            if let Some(sel) = self.vec.pen.selected() {
                ph2d_vec_render::draw_gradient_handles(
                    vec_scene,
                    Some(sel),
                    self.vec.grad_selected,
                    ph2d_vec_render::path_to_screen(&vec_xf, sel, cam_affine),
                    vector_scene,
                );
            }
            // O gesto de REGIÃO em curso, em px de tela — retângulo ou LAÇO, conforme a
            // forma que o press congelou.
            if let Some(m) = self.vec.marquee.as_ref() {
                match m.shape {
                    ph2d_tool_vector::params::MarqueeShape::Box => {
                        ph2d_vec_render::draw_marquee(
                            [f64::from(m.start.0), f64::from(m.start.1)],
                            [f64::from(m.cur.0), f64::from(m.cur.1)],
                            vector_scene,
                        );
                    }
                    ph2d_tool_vector::params::MarqueeShape::Lasso => {
                        let pts: Vec<(f64, f64)> = m
                            .closed_path()
                            .into_iter()
                            .map(|(x, y)| (f64::from(x), f64::from(y)))
                            .collect();
                        ph2d_vec_render::draw_lasso(&pts, vector_scene);
                    }
                }
            }
        }
        // ⚠️ **As imagens presas ao esqueleto já NÃO se desenham aqui** (plano `docs/Skeleton/03`,
        // W2, 2026-09-13): elas são sprites do quadro, desenhadas como malha no passe de sprites — a
        // malha é posta na `fase_sim_extract`.
        Some((vec_xf, cam_affine))
    }
}
