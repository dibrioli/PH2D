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
                self.flip_state.active_layer,
                &l2w,
                camera,
                surface.size(),
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
            // **O gizmo dos deformadores de quadrilátero** — o contorno, os braços e
            // as alças. Lê o retrato publicado no prólogo (a tool e a selecção já
            // foram decididas lá), então aqui não há regra nenhuma, só tinta.
            //
            // ⛔⛔⛔ **ELE JÁ ESTEVE ~2 000 LINHAS ABAIXO, E O GIZMO SUMIU DO ECRÃ.**
            // (report do Enio, 2026-09-08: *«nessa última rodada vc sumiu com o gizmo do
            // Bezier Warp»*.) A mudança tinha sido feita para o pôr por cima do documento
            // vectorial e das formas vivas do Motion, que codificam na MESMA cena depois
            // daqui — mas essa ordem foi **inferida de números de linha e nunca medida**, e
            // a que se pagou foi real.
            //
            // ⚠️ **O mecanismo do desaparecimento NÃO está nomeado**, e as quatro hipóteses
            // óbvias foram descartadas com medição: o sítio novo corre (contagem de chavetas
            // que ignora strings e comentários: só `impl` → `fn` → `if let Some(hero)`), a
            // cena não é reposta nem trocada entre os dois pontos, o `camera` e o `surface`
            // não são sombreados, e o retrato é publicado **uma vez só** (não é um `take`).
            // ⇒ *uma mudança de sítio sem uma medição do que o sítio garante é um palpite*,
            // e quem a repetir começa por instrumentar o quadro, não por mover a linha.
            // ⛔⛔ **OS GIZMOS DE NÓ MUDARAM-SE PARA A `fase_vector_overlays`** (doc 109 §6, report
            // do dono 2026-09-13: *«o collider deve aparecer na frente da shape (z-index maior)»*).
            // Esta fase corre ANTES da que codifica a arte das formas do Motion, e no Vello quem
            // pinta primeiro fica por BAIXO: o gizmo desenhado aqui ficava atrás dos quadrados que
            // ele manipula. ⚠️ A nota de 2026-09-08 do `warp_overlay` dizia *«o gizmo ESTÁ por
            // cima»* e foi medida agora como falsa — a cura de então (o casing) tratou o sintoma.
            // O gate `the_collider_outline_is_painted_after_the_shape_art` prende a ordem.
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
                surface.size(),
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
                surface.size(),
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
                surface.size(),
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
                    self.flip_state.active_layer,
                    &self.playhead,
                ),
                camera,
                surface.size(),
                vector_scene,
            );
        }
    }
}
