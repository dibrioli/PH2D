//! **Fase do quadro: O REALCE DA SELECÇÃO** — a selecção que se VÊ: o realce e o marquee do Flip, o gizmo de warp, as âncoras, os objectos
//! vazios, o tween e o vão (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_selection_highlight(
        &mut self,
        flip_active: bool,
        flip_style: Option<ph2d_tool_flip::FlipStyleSnapshot>,
        viewport: EditorRect,
    ) {
        if self.no_palco() {
            self.tarjas_do_palco();
            return;
        }
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            surface,
            sim,
            camera,
            theme,
            vector_scene,
            flip,
            text_system,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };
        // O realce da seleção (W6): uma seleção que não se VÊ não existe. Overlay
        // (chrome), nunca render de traço — ver o cabeçalho do módulo.
        {
            let l2w = flip
                .objects()
                .first()
                .map(|o| o.id)
                .and_then(|oid| self.flip_state.entities.get(&oid).copied())
                .map(ph2d_ecs::Entity::from_bits)
                .filter(|e| sim.world().get_entity(*e).is_ok())
                .map_or(ph2d_vec_scene::Xform::IDENTITY, |e| {
                    ph2d_flip_entities::transform::object_xform(sim, e)
                });
            // W8/§4.C: o realce fala a linguagem do DOMÍNIO — halo de traço (Stroke),
            // dots (Point), ou halo do PEDAÇO + preview de hover (Segment).
            let overlay_domain = match flip_style.map(|s| s.edit_domain) {
                Some(ph2d_tool_flip::EditDomain::Point) => {
                    ph2d_app_flip::selection_overlay::OverlayDomain::Point
                }
                Some(ph2d_tool_flip::EditDomain::Segment) => {
                    ph2d_app_flip::selection_overlay::OverlayDomain::Segment
                }
                _ => ph2d_app_flip::selection_overlay::OverlayDomain::Stroke,
            };
            let hover = self
                .flip_state
                .segment_hover
                .as_ref()
                .map(|(si, pts)| (*si, pts.as_slice()));
            ph2d_app_flip::selection_overlay::draw_flip_selection(
                flip_active,
                matches!(
                    flip_style.map(|s| s.mode),
                    Some(ph2d_tool_flip::FlipMode::Edit)
                ),
                overlay_domain,
                hover,
                flip,
                &self.playhead,
                self.flip_state.target,
                &l2w,
                camera,
                crate::scene_mapping::janela(hero.view.center_split, surface.size()),
                vector_scene,
            );
            // A caixa do marquee (W6.1) — em px de tela, como o realce.
            ph2d_app_flip::selection_overlay::draw_flip_marquee(
                self.flip_state.edit_gesture,
                vector_scene,
            );

            // **§12 Sockets / Named Anchors** (spec Sprite 07 §7.6) — os marcadores só
            // aparecem com a seção EXPANDIDA, senão todo sprite com âncoras ficaria coberto
            // de cruzes. Sem eles a §12 é um formulário que não mexe em nada na tela.
            // **O gizmo dos deformadores de quadrilátero** — lê o retrato publicado no prólogo; aqui
            // só tinta. ⛔ Já esteve ~2 000 linhas abaixo e SUMIU do ecrã (report 2026-09-08, causa
            // nunca nomeada: mover sem medir o que o sítio garante é palpite — instrumente o quadro).
            // ⛔⛔ Os gizmos de NÓ vivem na `fase_vector_overlays` (doc 109 §6): esta fase corre ANTES
            // da arte das formas do Motion, e no Vello quem pinta primeiro fica por BAIXO. O gate
            // `the_collider_outline_is_painted_after_the_shape_art` prende a ordem.
            // ⭐ Os Pixel Sprites (ADR-0196) — antes das marcas, para elas ficarem por cima.
            let window = crate::scene_mapping::janela(hero.view.center_split, surface.size());
            let cena = (&mut *vector_scene, &mut *paint_ctx.text);
            self.pixel.draw_of(sim.world(), camera, window, hero, cena);
            anchor_overlay::draw_anchor_marks(
                !hero
                    .store
                    .is_collapsed(ph2d_editor_core::ids::INSP_LIVE_ANCHOR_SECTION),
                sim.world(),
                hero.gizmo.selection,
                // ⚠️ A linha ABERTA vem do PAINEL — é o canal que o gizmo estreou. Ela é o
                // que decide quem ganha alças; sem ela o canvas não sabe a quem obedecer.
                ph2d_panel_inspector::open_anchor_row(),
                hero.project.pixels_per_meter,
                camera,
                crate::scene_mapping::janela(hero.view.center_split, surface.size()),
                vector_scene,
                // Reborrow por `paint_ctx`, como o rótulo do overlay de física — o
                // `text_system` já está emprestado desde o começo do frame.
                paint_ctx.text,
            );

            // ⭐ **O ANEL do objeto vazio** (Enio, 2026-08-26) — um objeto sem geometria não
            // emite pixel nenhum, e sem marca o artista não sabe onde ele está. A pergunta
            // *«está vazio?»* é a MESMA que dimensiona a caixa do gizmo (`group_gizmo_view`).
            empty_object_overlay::draw_empty_object_marks(
                sim,
                hero.project.pixels_per_meter,
                hero.theme,
                camera,
                crate::scene_mapping::janela(hero.view.center_split, surface.size()),
                vector_scene,
            );
            // ⭐ **A CÂMERA no canvas** (plano GEF 17): o ícone, a moldura do jogo, o rótulo, os cantos e os limites.
            ph2d_app_game::moldura_tinta::desenhar(
                &self.undo.camera_no_canvas.molduras,
                ph2d_app_game::moldura_tinta::Pintura {
                    mostrar: hero.project.camera_frame,
                    unidade: hero.project.display_unit,
                    pixels_per_meter: hero.project.pixels_per_meter,
                    icone_px: hero.project.camera_icon_px,
                    em_edicao: self.undo.camera_no_canvas.em_edicao,
                    canvas: [viewport.x, viewport.y, viewport.w, viewport.h],
                    ponteiro: self.last_pointer,
                },
                hero.theme,
                camera,
                crate::scene_mapping::janela(hero.view.center_split, surface.size()),
                vector_scene,
                paint_ctx.text,
            );
            // ⭐ **O GIZMO DA LUZ** (Luz 2D): a lâmpada no tamanho real, os raios e o alcance.
            ph2d_app_light2d::gizmo::desenhar(
                sim.world(),
                hero.gizmo.selection,
                hero.project.pixels_per_meter,
                hero.theme,
                camera,
                crate::scene_mapping::janela(hero.view.center_split, surface.size()),
                vector_scene,
            );

            // Tween v2 — a correção de pares: os dois desenhos-chave sobrepostos + as
            // linhas de par (pela confiança) + órfãos, no MESMO `l2w` do objeto. Só
            // desenha com a sessão Pairs aberta.
            ph2d_app_flip::tween_overlay::draw(
                flip_active && self.flip_state.strip.tween_correct.is_some(),
                self.flip_state.strip.tween_correct.as_ref(),
                &l2w,
                camera,
                crate::scene_mapping::janela(hero.view.center_split, surface.size()),
                vector_scene,
            );

            // Gap Closure (doc 06 §8): os helpers ao vivo — cada vão que o alcance
            // atual fecha, desenhado onde o clique vai fechá-lo. Os segmentos vêm do
            // worker (`flip_gap_live`, coords de ARTE); a pergunta do modo é a MESMA
            // porta do tick, e a projeção é a MESMA cadeia do render (l2w ∘ pose).
            ph2d_app_flip::gap_overlay::draw(
                ph2d_app_flip::gap_live::wants_gap_helpers(flip_active, flip_style),
                &self.flip_state.gap.segments,
                &l2w,
                // A MESMA pose que a autoria dobra (`flip_transform::active_pose`) —
                // função livre porque aqui `self.gfx` está destruturado.
                ph2d_flip_entities::transform::active_pose(
                    flip,
                    self.flip_state.target,
                    &self.playhead,
                ),
                camera,
                crate::scene_mapping::janela(hero.view.center_split, surface.size()),
                vector_scene,
            );
        }
    }
}
