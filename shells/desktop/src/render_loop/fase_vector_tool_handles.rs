//! **Fase do quadro: AS ALÇAS DA FERRAMENTA VECTORIAL** — o overlay do modo, as alças do texto e do padrão em
//! caminho, da largura, o cursor de texto e as alças de ponto do falloff e da curva (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_vector_tool_handles(
        &mut self,
        vector_active: bool,
        cam_affine: ph2d_vector::Affine,
        overlay: ph2d_app_vec::overlay::VecOverlayPlan,
    ) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            sim,
            tools,
            vector_scene,
            vec_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // **As alças de ponta do conector** — FORA do `overlay.edit`, e isso é a coisa toda:
        // elas vivem no modo **Select**, que é exatamente onde `overlay.edit` é FALSO (lá
        // quem fala é o gizmo, ADR-0112). Pô-las dentro do guard as tornaria invisíveis no
        // único modo em que existem.
        //
        // O conector não publica gizmo (`vec_gizmo_view::view` o pula), então não há
        // disputa: a caixa de transformação não cobre estas bolinhas.
        if vector_active && self.vec.draw_config.mode == ph2d_tool_vector::DrawMode::Select {
            let handles = crate::connector_handles::view(
                sim,
                vec_scene,
                &self.vec.entities,
                self.vec.pen.selected_paths(),
            );
            ph2d_vec_render::draw_connector_handles(&handles, cam_affine, vector_scene);
            // Os pontos de passagem — QUADRADOS, por cima das bolinhas: quando um waypoint
            // é arrastado até uma ponta, é ele que está sob o dedo.
            let ways = crate::connector_handles::waypoint_view(
                sim,
                &self.vec.entities,
                self.vec.pen.selected_paths(),
            );
            ph2d_vec_render::draw_connector_waypoints(&ways, cam_affine, vector_scene);
        }
        // **A alça do TEXTO EM CAMINHO** (W5) — FORA do `overlay.edit`, pela MESMA razão das
        // alças do conector logo acima: ela é do modo **Select**, onde `overlay.edit` é FALSO
        // (dentro daquele guard ela nunca desenharia — foi o bug do 1º smoke). O
        // `handle::world` devolve `None` sem um texto vinculado na seleção, e o
        // `overlay.textpath_handle` já confina ao Select. Geometria em MUNDO (o guia traz a
        // pose) ⇒ sobe pelo afim da CÂMERA. Desenhada DEPOIS do gizmo para ficar por cima
        // dele — ela é a ficha que o artista agarra, não um ponto atrás da caixa.
        if overlay.textpath_handle
            && let Some(at) = crate::vec_text_ride::handle::world(
                sim,
                vec_scene,
                &self.vec.entities,
                self.vec.pen.selected_paths(),
            )
        {
            ph2d_vec_render::draw_text_handle(
                at,
                self.vec.textpath_handle_drag,
                cam_affine,
                hero.theme,
                vector_scene,
            );
        }
        // **As DUAS alças do PATTERN ON PATH** (W4) — Start e End do trecho, no MESMO lugar da
        // alça do texto (Select, depois do gizmo). Reusa a ficha (`draw_text_handle`): as duas
        // fichas são iguais, a POSIÇÃO diz qual é Start e qual é End. `handle::world` devolve
        // `None` sem um pattern vinculado no primário.
        if overlay.patternpath_handles
            && let Some((start_pt, end_pt)) = crate::pattern_live::handle::world(
                sim,
                vec_scene,
                &self.vec.entities,
                self.vec.pen.selected_paths(),
            )
        {
            use crate::pattern_live::PatternHandle;
            let dragging = self.vec.patternpath_handle;
            ph2d_vec_render::draw_text_handle(
                start_pt,
                dragging == Some(PatternHandle::Start),
                cam_affine,
                hero.theme,
                vector_scene,
            );
            ph2d_vec_render::draw_text_handle(
                end_pt,
                dragging == Some(PatternHandle::End),
                cam_affine,
                hero.theme,
                vector_scene,
            );
        }
        // **As alças de LARGURA** (plano 25 §5) — uma por parada do perfil da forma primária.
        // A ficha fica SOBRE a curva e uma haste sai dela até a borda da fita, que é o que a
        // parada mede: a ficha na borda atravessava a linha vizinha em multiplicador alto, e
        // era isso que fazia uma alça nascer na linha de ao lado (report do Enio 2026-07-30 —
        // ver o doc de `width_handles::handles`). Reusa a MESMA ficha do texto e do pattern.
        // `handles` devolve vazio sem traço (nada a editar).
        if overlay.width_handles
            && let Some(pid) = self.vec.pen.selected()
        {
            let grabbed = self.vec.width_grab.map(|g| g.stop);
            for (k, h) in crate::width_handles::handles(sim, vec_scene, &self.vec.entities, pid)
                .into_iter()
                .enumerate()
            {
                ph2d_vec_render::draw_width_handle(
                    h.at,
                    h.tip,
                    grabbed == Some(k),
                    cam_affine,
                    hero.theme,
                    vector_scene,
                );
            }
        }
        // Cursor de texto (modo Text): na ponta da última linha em edição. Lê só o
        // campo `vec_text_edit` (fn livre), pra não colidir com o borrow de gfx.
        if let Some((a, b)) = crate::vec_text::caret_of(self.vec.text_edit.as_ref()) {
            ph2d_vec_render::draw_text_caret(a, b, cam_affine, vector_scene);
        }
        // Drain the Painter Falloff right-click handle menu choice (chrome
        // parked the HandleType wire u8 in `pending_falloff_point_handle`) →
        // apply it to the selected control point.
        if let Some(handle) = hero.pending_falloff_point_handle.take()
            && let Some(id) = ph2d_panel_painter_layers::selected_falloff_point()
            && let Some(painter) = tools.active_mut().and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_painter::PainterTool>()
            })
        {
            painter.set_brush_falloff_point_handle(id, handle);
        }
        // Drain the on-canvas Curve / Free Hand right-click handle-kind choice (chrome parked the wire
        // u8 in `pending_curve_point_handle`) → apply it to the selected control point.
        if let Some(kind) = hero.pending_curve_point_handle.take()
            && let Some(painter) = tools.active_mut().and_then(|t| {
                t.as_any_mut()
                    .downcast_mut::<ph2d_tool_painter::PainterTool>()
            })
        {
            // Either curve owner: the stroke Shape curve, else the selection Convert-to-Curve editor.
            if !painter.set_curve_handle_kind(kind) {
                painter.set_selection_curve_handle_kind(kind);
            }
        }
    }
}
