//! **O que se pinta POR CIMA da cena na área de desenho** — as réguas, a selecção, os gizmos, a
//! barra da receita, as etiquetas de moldura e a leitura do gesto. Movido VERBATIM do
//! `paint_hero_screen` (MiroClone, 2026-10-05) para um QUADRO activo poder saltá-lo inteiro: os
//! gizmos registam alvos de clique, e por cima de um quadro seriam controlos da cena invisíveis.

use super::*;

/// Ver o cabeçalho do módulo.
pub(super) fn paint_canvas_overlays(
    hero: &mut HeroScreen,
    layout: HeroLayout,
    rulers_on: bool,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) {
    // **As RÉGUAS** (plano 25 §9, a W6.2), por cima da grade e por baixo de tudo o mais: elas
    // são chrome de borda, e a arte passa por baixo delas como passa por baixo do Inspector.
    // O zero é a origem da GRADE — um número, dois consumidores.
    if rulers_on && let Some(view) = hero.grid.view {
        // ⭐ **A ÁREA de desenho, e não o canvas** — é o que faz as réguas deixarem de partilhar
        // coordenada com o trilho e com a barra (D5). ⚠️ A PROJEÇÃO não se mexe: ela deriva de
        // `window_w`/`window_h`, nunca deste rect, então um traço marcado em 100 continua a cair
        // no mesmo pixel — só deixa de o fazer debaixo do chrome (`ruler::in_band` já filtrava os
        // traços que caem fora da faixa).
        let view = crate::grid::GridView {
            canvas: layout.draw_area,
            ..view
        };
        let origin = hero.grid.snap_state.active_origin();
        // A régua imprime na unidade que o artista escolheu — a MESMA porta do
        // Inspector e do painel de Grid Snap (`LengthDisplay`). O `hero.project`
        // é o dono do fato, e ele já está aqui.
        let display = crate::length::LengthDisplay::of(&hero.project);
        crate::ruler::paint_rulers(scene, &view, origin, text_system, hero.theme, display);
    }
    // M14.4c: the legacy mockup selection marquee draws a fixed-size
    // dashed rect at the CANVAS center in screen pixels — it has no
    // world-space coupling and so doesn't follow pan/zoom. Skip it
    // when a `grid_view` is published (live ECS mode) so we don't
    // mislead users into thinking the marquee tracks an entity.
    // Fixture mode keeps the placeholder marquee for the mockup
    // screenshots.
    if hero.grid.view.is_none()
        && let Some(sel) = hero.selection.as_ref()
    {
        paint_selection_overlay(&layout, sel, scene, text_system, hero.theme);
    }
    // M14.7 B: live-mode sprite gizmo. The host publishes a
    // `gizmo_view` carrying the selected sprite's world-space bbox +
    // current camera; the painter projects to screen pixels with the
    // same math the grid uses (so the gizmo and grid stay aligned
    // across pan/zoom).
    if let Some(view) = hero.gizmo.view {
        crate::gizmo::paint_sprite_gizmo(scene, &view, hero.theme, &mut hero.hit_index);
    }
    // Flip W7.5: o gizmo da POSE da chave (modo Edit + quadro instanciado). Pintado
    // como gizmo KEYED — rotate/scale nos ids do espaço `FlipPose`, sem interior
    // (o arrasto de canvas do Edit já move a instância; um interior aqui comeria o
    // clique da seleção de traço) e sem pivot dot (o pivô da pose é o centro da
    // arte, que a caixa já mostra).
    if let Some(v) = hero.gizmo.pose_view {
        crate::gizmo::paint_sprite_gizmo_keyed(
            scene,
            &v,
            hero.theme,
            &mut hero.hit_index,
            &mut hero.gizmo.gizmo_hit_map,
            crate::gizmo::GizmoTarget::FlipPose,
            1.5, // LITERAL-PX-OK: espessura do contorno do gizmo de pose (mesma do primário)
        );
    }
    // Flip §4.A: o gizmo da SELEÇÃO (modo Edit + arte exclusiva + há seleção). Keyed
    // como `FlipSelection`, sem interior (o translate da seleção é o arrasto de canvas
    // do W6.1/W8; um interior comeria o clique de re-seleção). Mutuamente exclusivo
    // com `pose_view`, então nunca pintam juntos.
    if let Some(v) = hero.gizmo.selection_view {
        crate::gizmo::paint_sprite_gizmo_keyed(
            scene,
            &v,
            hero.theme,
            &mut hero.hit_index,
            &mut hero.gizmo.gizmo_hit_map,
            crate::gizmo::GizmoTarget::FlipSelection,
            1.5, // LITERAL-PX-OK: espessura do contorno do gizmo de seleção (mesma do primário)
        );
    }
    // Motion Nodes (fields): o gizmo de canvas de um field espacial (tool Motion ativa
    // + field selecionado no grafo). Keyed como `MotionField`, sem interior — o apply
    // escreve os params do NÓ, não um `Transform`, e o gizmo de sprite (`view`) fica
    // intocado. Nunca coexiste com o de sprite/flip por modalidade da tool.
    if let Some(v) = hero.gizmo.field_view {
        crate::gizmo::paint_sprite_gizmo_keyed(
            scene,
            &v,
            hero.theme,
            &mut hero.hit_index,
            &mut hero.gizmo.gizmo_hit_map,
            crate::gizmo::GizmoTarget::MotionField,
            1.5, // LITERAL-PX-OK: espessura do contorno do gizmo de field (mesma do primário)
        );
    }
    // ⭐⭐⭐ **A BARRA DO MODO DE RECEITA** (o *Edit Prefab*, 2026-09-07) — o nome do que se está a
    // editar, quantas cópias seguem, e a SAÍDA. Ver [`super::prefab_bar`].
    //
    // ⚠️ **Aqui, e não com os painéis:** ela é chrome do CANVAS (ancora na área de desenho e
    // desaparece com o modo), então pertence à camada dos gizmos — acima da cena, abaixo dos
    // painéis flutuantes. ⛔ Sem receita aberta nada é pintado nem registado.
    //
    // ⚠️⚠️ **A âncora é o `last_content`, e não a `draw_area`** (report do Enio, 2026-09-07: *«a
    // barra está em cima da régua»*): a `draw_area` inclui a faixa das réguas, e a barra pousava
    // **sobre** elas. O `last_content` é o que sobra DEPOIS delas — escrito neste mesmo ficheiro,
    // acima, e sob a mesma condição que decide pintá-las, que é o que impede as duas de
    // discordarem. Sem réguas na tela ele É a `draw_area`, e a barra não se mexe.
    if let Some(view) = hero.prefab_edit.as_ref() {
        super::prefab_bar::paint(
            scene,
            hero.last_content,
            view,
            hero.theme,
            text_system,
            &mut hero.hit_index,
        );
    }
    // Onda 2C + z-order fix: the multi-selection extra + global gizmos
    // paint here — at the SAME layer as the primary gizmo, i.e. above the
    // scene but BELOW the floating panels (painted later in this fn). They
    // used to paint in the shell AFTER `paint_hero_screen` returned, which
    // put them visually on top of panels AND registered their hit rects
    // after the panel barriers (so handles were clickable through chrome).
    // Snapshot the `(bits, view)` pairs first so `hero.gizmo` isn't borrowed
    // while `&mut hero.hit_index` + `&mut hero.gizmo.gizmo_hit_map` are held.
    // Each pair carries its own bits, so a handle can never be registered
    // under a different sprite's identity (no zip against `extra_selection`).
    let extras_snapshot: Vec<(u64, crate::gizmo::GizmoView)> = hero.gizmo.extra_views.clone();
    for (bits, v) in extras_snapshot {
        crate::gizmo::paint_sprite_gizmo_keyed(
            scene,
            &v,
            hero.theme,
            &mut hero.hit_index,
            &mut hero.gizmo.gizmo_hit_map,
            crate::gizmo::GizmoTarget::ExtraIndividual(bits),
            1.0,
        );
    }
    if let Some(v) = hero.gizmo.global_view {
        crate::gizmo::paint_sprite_gizmo_keyed(
            scene,
            &v,
            hero.theme,
            &mut hero.hit_index,
            &mut hero.gizmo.gizmo_hit_map,
            crate::gizmo::GizmoTarget::Global,
            2.0, // LITERAL-PX-OK: global gizmo outline stroke width
        );
    }
    // The POINT gizmo — every joint's anchor dots. A joint entity has a
    // `Transform` but no box, so it never publishes a `GizmoView` above; these
    // are the only handles it gets.
    //
    // ⚠️ **Painted LAST among the gizmos, and the order is the feature** (Enio,
    // 2026-07-25: *"devem ter o Z index mais alto que os outros objetos"*).
    // `HitIndex::hit` walks backwards, so the last registration wins: an anchor
    // sitting on a sprite's corner handle is grabbed as the anchor. A joint has
    // no sprite to pick and no box of its own — losing the pixel to whatever it
    // happens to lie on top of is how it becomes unreachable. Panels still win,
    // because they paint after this whole pass.
    if let Some(view) = hero.gizmo.point_view.as_ref() {
        crate::gizmo::paint_point_gizmo(
            scene,
            view,
            hero.theme,
            &mut hero.hit_index,
            &mut hero.gizmo.point_hit_map,
        );
    }
    // **As ETIQUETAS das molduras** — desenho puro, sem hit (a decisão mora no `frame_label`).
    // Depois do gizmo e antes do chrome: elas pertencem ao canvas, e um painel passa por cima
    // delas como passa por cima da arte.
    if !hero.gizmo.frame_labels.is_empty()
        && let Some(view) = hero.grid.view
    {
        let view = crate::grid::GridView {
            canvas: layout.canvas,
            ..view
        };
        crate::frame_label::paint_frame_labels(
            scene,
            &view,
            &hero.gizmo.frame_labels,
            text_system,
            hero.theme,
        );
    }
    // **A FICHA do arrasto** — o número que segue a mão (o estudo da UI viva, C3).
    //
    // Por cima de todo o gizmo e de toda a etiqueta, e por baixo do chrome: ela é a leitura do
    // gesto em curso, então nada do canvas pode tapá-la — e nada dela pode tapar um painel.
    // ⚠️ Desenho puro, sem hit: um alvo aqui roubaria o pen-down a ~18 px do cursor, isto é
    // exactamente onde a mão está a trabalhar. É a mesma decisão (e a mesma razão) da etiqueta de
    // moldura acima.
    if let (Some(text), Some(drag)) = (hero.gizmo.readout.as_deref(), hero.gizmo.drag) {
        let w = crate::readout::chip_width(text_system, text, 0.0);
        let at = crate::readout::at_cursor(
            [drag.cursor_screen.0, drag.cursor_screen.1],
            w,
            layout.canvas,
        );
        crate::readout::paint_chip(text_system, scene, text, at, 0.0, hero.theme);
    }
}
