//! **Fase do quadro: A RECEITA ABERTA E AS VISTAS DO GIZMO** — a receita que vem ao artista (o palco do prefab), as
//! vistas do gizmo sob o Flip e o Motion, e o gizmo de warp (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_gizmo_views_and_prefab(
        &mut self,
        window_size: ph2d_host::WindowSize,
    ) -> Option<bool> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            sim,
            camera,
            tools,
            flip,
            hero_screen,
            motion,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // ⭐⭐⭐ **A RECEITA VEM AO ARTISTA** (Enio, 2026-09-07) — servido AQUI porque é a linha
        // acima que publica a caixa dela, e é dessa caixa que o deslocamento sai. ⚠️ O gizmo
        // deste quadro já foi projectado com a pose ANTIGA, então ele desenha um quadro
        // atrasado; o desenho do mundo (que é encodado mais abaixo) já usa a nova. *Um quadro
        // de 16 ms, contra a alternativa de reconstruir a vista inteira só para o esconder.*
        ph2d_app_components::prefab_stage::run(
            &mut self.prefab_stage_pending,
            &mut self.prefab_stage,
            hero,
            ph2d_editor_core::zones::Rect::new(
                0.0,
                0.0,
                window_size.width as f32,
                window_size.height as f32,
            ),
            window_size,
            camera,
            sim,
            &mut self.preview_drive,
        );
        // Flip W7.5/§4.A: os gizmos do modo Edit — só na tool Flip em modo Edit. Os
        // dois campos próprios no `GizmoStateGroup` (append-only) são MUTUAMENTE
        // EXCLUSIVOS por `is_instanced`: a `pose_view` só publica quando o quadro
        // visível é uma INSTÂNCIA (rotate/escala da pose), a `selection_view` só
        // quando é arte EXCLUSIVA com seleção (rotate/escala assado na geometria).
        // O painter os desenha keyed (`FlipPose`/`FlipSelection`), sem interior — a
        // seleção de traço do Edit continua dona do canvas.
        let flip_edit_mode = tools
            .active()
            .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("flip"))
            && matches!(
                self.flip_state.style.map(|s| s.mode),
                Some(ph2d_tool_flip::FlipMode::Edit)
            );
        hero.gizmo.pose_view = flip_edit_mode
            .then(|| {
                ph2d_app_flip::pose_gizmo::pose_view(
                    sim,
                    flip,
                    &self.flip_state.entities,
                    ph2d_app_flip::pose_gizmo::PoseViewInputs {
                        playhead: &self.playhead,
                        active_layer: self.flip_state.active_layer,
                        last_pointer: self.last_pointer,
                    },
                    camera,
                    window_size,
                )
            })
            .flatten();
        hero.gizmo.selection_view = flip_edit_mode
            .then(|| {
                ph2d_app_flip::selection_gizmo::selection_view(
                    sim,
                    flip,
                    &self.flip_state.entities,
                    ph2d_app_flip::selection_gizmo::SelectionViewInputs {
                        playhead: &self.playhead,
                        active_layer: self.flip_state.active_layer,
                        last_pointer: self.last_pointer,
                    },
                    camera,
                    window_size,
                )
            })
            .flatten();
        // Motion Nodes: o gizmo de canvas de um FIELD ESPACIAL (`field.box`, …) — só com
        // a tool Motion ativa + um field espacial selecionado no grafo. Slot próprio
        // (`field_view`), desenhado keyed (`MotionField`) ⇒ o gizmo de sprite (`view`)
        // fica INTOCADO e os dois nunca coexistem por modalidade da tool. `tools` é o
        // local (não `self.motion_tool_active()`, que re-emprestaria `self.gfx`), espelho
        // do `flip_edit_mode` acima.
        let motion_tool_active = tools
            .active()
            .is_some_and(|t| t.id() == ph2d_editor_core::ToolId::new("motion"));
        // As dims da CENA (o sub-retângulo do split, `CenterSplit::scene_viewport`) — o
        // gizmo é pintado e arrastado com ELAS, casando com o `set_viewport` do render
        // (present.rs). É o fix do drift crônico: sob o split a cena renderiza na banda
        // e o chrome projetava a janela cheia. Fora do split = janela cheia (no-op).
        let (scene_w, scene_h) =
            ph2d_app_motion::field_gizmo::scene_window_wh(hero.view.center_split, window_size);
        hero.gizmo.field_view = motion_tool_active
            .then(|| {
                ph2d_app_motion::field_gizmo::field_view(
                    motion,
                    camera,
                    scene_w,
                    scene_h,
                    self.last_pointer,
                )
            })
            .flatten();
        // **O gizmo dos DEFORMADORES DE QUADRILÁTERO** (Corner Pin + Bezier Warp) —
        // publicado no mesmo sítio e pela mesma modalidade do field: só com a tool
        // Motion activa. ⚠️ Publicar de novo SUBSTITUI, então largar a selecção limpa
        // as alças em vez de as deixar a pairar.
        ph2d_app_motion::warp_gizmo::publish(ph2d_app_motion::warp_gizmo::resolve(
            motion,
            motion_tool_active,
        ));
        // **O gizmo do COLISOR da forma** (doc 109 §5) — o mesmo sítio, a mesma modalidade.
        ph2d_app_motion::collider_gizmo::publish(ph2d_app_motion::collider_gizmo::resolve_at(
            motion,
            motion_tool_active,
            camera,
            hero.view.center_split,
            window_size,
            self.last_pointer,
        ));
        Some(motion_tool_active)
    }
}
