//! **Fase do quadro: A SUPRESSÃO DO GIZMO** — o gizmo da sprite suprimido sob o Deform Transform,
//! os Pairs do Flip, as tools de vetor e o Motion, e os mapas de alças do quadro limpos (OBRA 2 da
//! `line/render-loop`, 2026-09-12). A moldura do modelador 3D que aqui morava saiu com o módulo
//! (ADR-0179).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_gizmo_suppression(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            tools,
            hero_screen,
            sim,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // Onda 2C: clear the gizmo hit_map BEFORE paint_hero_screen
        // runs. `paint_hero_screen` now paints BOTH the primary gizmo
        // AND the multi-selection extras + global gizmo (the latter
        // via `paint_sprite_gizmo_keyed`, which populates `hit_map` —
        // those entries drive the dispatcher's group-transform routing
        // in `on_mouse_input`). Painting them inside `paint_hero_screen`
        // (before the floating panels) keeps gizmos BELOW the panels
        // both visually and in hit-test (z-order fix 2026-05-31).
        // ADR-0076: as tools de vetor e as alças da forma querem o clique: as caixas do gizmo o roubariam.
        // A seleção fica armada; só somem a caixa e as alças.
        // The Deform Transform temperament shows its OWN whole-region gizmo (drawn in the painter
        // overlays). The object-transform gizmo would sit ON TOP (paint_hero_screen draws after those
        // overlays) and fight it, so suppress it while Deform Transform is active — the deform box IS the
        // transform gizmo there (Enio 2026-07-04).
        let painter_deform_transform = tools
            .active_mut()
            .and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_painter::PainterTool>()
            })
            .is_some_and(|p| p.deform_gizmo().is_some());
        // A correção de pares do Flip é o MESMO caso das tools de vetor acima: o overlay de
        // Pairs quer o clique do canvas para re-parear, e a caixa+alças do gizmo do objeto
        // registram hits no `hit_index` que fazem `on_canvas` virar falso — roubando TODO
        // clique de re-par. Enquanto Pairs está aberto, o gizmo do objeto some (a seleção
        // fica armada; só a caixa/alças somem).
        let flip_pairs_active =
            self.flip_state.active && self.flip_state.strip.tween_correct.is_some();
        let jogando = self.playhead.is_playing() || hero.view.palco_do_jogo;
        let forma = ph2d_shape_handles::quadro(sim.world_mut(), hero.gizmo.selection, jogando);
        let (w, loja) = (sim.world_mut(), &mut hero.store); // Regra B: trava e abre a pergunta.
        ph2d_physics_outline::canais_pontos::regra_b(w, &self.timeline, loja);
        let suppress_gizmo = forma
            || painter_deform_transform
            || flip_pairs_active
            || !ph2d_editor_core::screens::hero::mode_drive::object_gizmo_shows(hero)
            || tools
                .active()
                .map(|t| {
                    let id = t.id();
                    id == ph2d_editor_core::ToolId::new("vector_direct")
                            || id == ph2d_editor_core::ToolId::new("vector_pen")
                            || id == ph2d_editor_core::ToolId::new("vector_pencil")
                            || id == ph2d_editor_core::ToolId::new("vector_shape")
                            // Motion Nodes: a tool Motion é dona do canvas (o único gizmo é o
                            // do field, slot próprio `field_view`). Um sprite selecionado por
                            // acaso ao entrar mostraria seu gizmo de sprite projetado na
                            // janela CHEIA (deslocado da cena que renderiza na banda do split)
                            // — some junto com o resto do chrome de sprite.
                            || id == ph2d_editor_core::ToolId::new("motion")
                })
                .unwrap_or(false);
        if suppress_gizmo {
            hero.gizmo.view = None;
            hero.gizmo.extra_views.clear();
            hero.gizmo.global_view = None;
        }
        hero.gizmo.gizmo_hit_map.clear();
        // Its sibling for the anchor dots — same reason (no stale entry
        // from a joint that left the scene), same frame.
        hero.gizmo.point_hit_map.clear();
    }
}
