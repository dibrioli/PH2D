use super::*;

/// **Todo painel que o passeio de z-order alcança**, mesmo que o store ainda não o tenha visto.
///
/// ⚠️ **Estar no registro e estar visível NÃO chega**: se o id não passa por aqui, o painel é
/// registado, visível e **nunca pintado** — nada quebra e nada avisa. Este arquivo pagou esse
/// defeito seis vezes (as notas dentro da lista são o registo de cada uma), e a última foi um smoke
/// reprovado do painel de modelagem 3D.
///
/// ⭐ Desde 2026-08-19 há um gate — `every_registered_panel_is_reachable_by_the_z_order_walk` — que
/// compara esta lista com o REGISTRO. É por isso que ela é uma const com nome em vez de um array
/// anônimo dentro do `for`: uma lista que um teste não consegue ler é uma lista que ninguém confere.
pub const PANEL_Z_ORDER_FALLBACK: &[ph2d_a11y::NodeId] = &[
    ids::HIER_PANEL,
    ids::INSP_PANEL,
    // Geometry-graph smoke panel (ADR-0065): docks over the inspector rect
    // when `PH2D_VECTOR_GRAPH=1`. Its own `paint()` no-ops when hidden, so
    // this is inert in the normal app. After INSP_PANEL → paints on top.
    ids::VGRAPH_PANEL,
    ids::BGR_PANEL,
    ids::PAD_PANEL,
    ids::CEQ_PANEL,
    ids::EQS_PANEL,
    ids::UPS_PANEL,
    ids::PAINTER_SIDEBAR_PANEL,
    ids::VECTOR_INSPECTOR_PANEL,
    // Vector tool Style panel (ADR-0108 docked `ph2d-panel-vector`): docks
    // over the inspector slot while the `vector` tool is active. Its
    // `paint()` no-ops when hidden, so this is inert otherwise.
    ids::VECTOR_PANEL,
    // Flip tool Style panel (ADR-0114 W2 docked `ph2d-panel-flip`): docks
    // over the inspector slot while the `flip` tool is active (bridge-driven
    // visibility). Its `paint()` no-ops when hidden, so this is inert
    // otherwise. WITHOUT this entry the registered+visible panel is never
    // reached by the z-order walk → never painted.
    ids::FLIP_PANEL,
    // Flip frame strip (ADR-0114 W3 docked `ph2d-panel-flip-frames`): a faixa
    // INFERIOR da tool Flip (células + transporte). Mesma disciplina: sem esta
    // entrada o painel registrado e visível nunca é alcançado pelo walk — e
    // nunca é pintado.
    ids::FLIP_STRIP_PANEL,
    // Motion Nodes docked panels (M0.T9): the graph-editor panel fills the
    // `motion_graph` split region, the params panel takes the inspector slot.
    // Both `paint()` no-op when the `motion` tool is inactive (bridge-driven
    // visibility), so they're inert otherwise. WITHOUT these entries a
    // registered+visible panel is never reached by this z-order walk → never
    // painted (the split would be invisible).
    ids::MOTION_GRAPH_PANEL,
    ids::MOTION_PARAMS_PANEL,
    // General timeline (docs/Timeline W2): bottom-docked, visibility toggled
    // by the `timeline` key. WITHOUT this entry the registered+visible panel
    // is never reached by the z-order walk → never painted.
    ids::TIMELINE_PANEL,
    // Physics world panel (ADR-0131 D8 docked `ph2d-panel-physics`): the
    // world/scene-settings category — always available, not tool-gated.
    // Its `paint()` no-ops when hidden. WITHOUT this entry the panel is
    // registered, visible, and NEVER painted — nothing breaks, nothing warns.
    ids::PHYSICS_PANEL,
    // Wet Tuning side panel (doc 22, docked beside the painter panel):
    // visibility mirrored from the tool's Tuning checkbox by the painter
    // bridge; `paint()` no-ops when hidden. WITHOUT this entry the panel
    // is registered, visible, and NEVER painted.
    ids::WET_TUNING_PANEL,
    // Tokens world panel (plano UI/UX W6, docked `ph2d-panel-tokens`): a
    // tabela de cor do design system, mesma categoria do painel de física.
    // `paint()` no-ops when hidden — sem esta entrada ele fica registado,
    // visível, e NUNCA pintado.
    ids::TOKENS_PANEL,
    // O painel AUTORADO (plano UI/UX W8b.2): o painel que o artista desenhou. `paint()`
    // no-opa quando escondido — sem esta entrada ele fica registado, visível, e NUNCA
    // pintado (nada quebra, nada avisa).
    ids::AUTHORED_PANEL,
    // O painel da cena 3D (ADR-0150 W12): mesma categoria dos dois acima.
    // O `paint()` dele sai no primeiro `if` sem cena viva — sem esta entrada
    // ele fica registrado, visível, e NUNCA pintado.
    ids::SCULPT3D_PANEL,
    // O painel de MODELAGEM 3D (ADR-0161 W4) — irmão do de cima, e não ele:
    // `sculpt3d` é escultura, `model3d` é modelagem por campo implícito.
    //
    // ⚠️ **A ausência desta linha foi um smoke reprovado** (Enio, 2026-08-19:
    // *"o painel não abre"*): a crate existia, estava no registro, a
    // visibilidade estava escrita, os 6 gates dela passavam — e este passeio
    // nunca chegava nele. É a **sexta** vez que este arquivo paga o mesmo
    // defeito, e as cinco notas acima já o diziam.
    ids::MODEL3D_PANEL,
    // ⭐ O NAVEGADOR DE ASSETS (plano `docs/Components/07`) — e ele entra AQUI porque as seis
    // notas acima já pagaram esta lição: sem esta linha o painel fica registado, visível, com os
    // gates verdes, e **nunca pintado**.
    ids::ASSET_PANEL,
    ids::INSP_BLENDER_PICKER,
    ids::GAL_PANEL,
    ids::AUDIO_MIXER_PANEL,
    ids::AUDIO_EDITOR_PANEL,
    crate::grid_snap::ids::GS_PANEL,
];

/// Top-level hero paint orchestrator. Clears + re-populates the
/// hit-index, then walks each region painter in z-order
/// (canvas → selection overlay → chrome → HUD).
pub fn paint_hero_screen(
    hero: &mut HeroScreen,
    viewport: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) {
    // Publish the user-picked radius scale to the thread-local read
    // by `paint::fill_rounded_rect` / `stroke_rounded_rect`. Set
    // every frame so it stays in sync with the topbar's radius menu.
    crate::paint::set_radius_scale(hero.store.radius_scale());
    // Same pattern for the text-rendering strategy — read by
    // `paint_text*` via the `paint::text_rendering()` thread-local.
    crate::paint::set_text_rendering(hero.text_rendering);
    // Stash the viewport so chrome event handlers in `chrome/` can
    // make smart layout decisions (cascade submenu side-flip etc.).
    hero.last_viewport = viewport;

    // Rail width follows the user's Themes-menu rail-button-size
    // preset (Small / Medium / Large; default Small). Switching size
    // shifts Inspector/Hierarchy x-positions accordingly.
    let rail_w = hero.store.rail_button_size().rail_width_px();
    // Motion Nodes M0.T4: `center_split` is `None` for every non-Motion tool, so
    // this is identical to the legacy layout there; the Motion bridge sets a split
    // while its tool is active.
    // **Quais colunas laterais estão ocupadas** — a área de desenho (e com ela as réguas) cresce
    // para dentro de uma coluna fechada. É o mesmo padrão do `dock_timeline_into_motion` logo
    // abaixo: o layout é uma função pura do que lhe dizem, e ESTE é o sítio que sabe.
    // **Quais colunas laterais estão ocupadas** — perguntado aos rects que os painéis
    // PUBLICARAM no quadro anterior, nunca a uma lista de nomes: são 20 crates a publicar, e a
    // lista de cinco que aqui esteve estava errada exactamente no modo que importava.
    // A sonda serve só para saber ONDE ficam as duas colunas — a geometria delas não depende
    // dos flags —, e o `side_columns` devolve-as ordenadas por `x`, que é o que torna o
    // `mirrored` inofensivo aqui.
    let probe = HeroLayout::for_viewport_docked(
        viewport,
        hero.view.ui_mirrored,
        rail_w,
        hero.view.center_split,
        crate::screens::layout::DockSides::BOTH,
    );
    let published: Vec<_> = hero.store.panel_rects().collect();
    let (left_col, right_col) = probe.side_columns();
    let docks = crate::screens::layout::DockSides::from_published(left_col, right_col, published);
    let mut layout = HeroLayout::for_viewport_docked(
        viewport,
        hero.view.ui_mirrored,
        rail_w,
        hero.view.center_split,
        docks,
    );
    // **The timeline docks INTO the Motion workspace** (W4.T4). Only when both are on screen:
    // otherwise the graph keeps its full band and the timeline keeps its own dock. The condition
    // is read from the panel visibility the bridges already publish — the layout stays a pure
    // function of what it is told, and this is the one place that tells it.
    //
    // Before this, `motion_graph` ran down to the chrome and `timeline` was the bottom strip, so
    // the two occupied the SAME pixels and the timeline (drawn later) painted over the graph.
    if hero.is_panel_visible(super::PANEL_MOTION_GRAPH)
        && hero.is_panel_visible(super::PANEL_TIMELINE)
    {
        layout.dock_timeline_into_motion();
    }
    // ⛔ **As faixas do FUNDO também comem a área de desenho** (auditoria de 2026-08-30): o
    // `timeline` nasce exactamente no `area_x0` e ocupa 240 px no fundo da banda, então a régua
    // da esquerda corria por baixo dele. Depois do `dock_timeline_into_motion`, de propósito —
    // ele MOVE o rect do timeline, e reservar antes reservaria o sítio errado.
    if hero.is_panel_visible(super::PANEL_TIMELINE) {
        layout.reserve_bottom_strip(layout.timeline);
    }
    if hero.is_panel_visible("flip_frames") {
        layout.reserve_bottom_strip(layout.flip_strip);
    }
    // ⭐⭐ **AS COLUNAS LATERAIS SÃO ANCORADAS** (Enio, 2026-08-30, com foto: *«só fica legal
    // depois de fixar os painéis nas laterais»*). O rect que o [`HeroLayout`] calculou **é** o
    // rect que elas ocupam — não há offset de arrasto entre os dois.
    //
    // ⛔ Aqui estava o bloco que lia `blender_picker_offset` + `panel_resize_delta` do Inspector
    // e da Hierarchy, clampava-os e **escrevia o resultado por cima** de `layout.inspector` /
    // `layout.hierarchy`. Ele governava **dezasseis** painéis sem que nenhum soubesse: as quatro
    // linhas de espelho abaixo levam o rect ao `bgremoval`/`padding`/`painter_sidebar`/
    // `painter_layers`, e outras doze crates lêem `ctx.layout.inspector` directamente. ⇒
    // *arrastar o Inspector arrastava os dezasseis.*
    //
    // ⚠️ **As alças saíram EM PAR com o braço** — o registo no `HitIndex` (nos dois painéis) e o
    // `InteractiveState::BlenderHit` do `pre_populate`. Deixar uma ponta viva daria a forma
    // exacta do controlo morto sob o dedo que este repo varre a cada wave: uma alça pintada e
    // registada cujo arrasto não move nada.
    //
    // ⚠️ **A flutuação DECLARADA (D1) não foi tocada:** o Grid Snap, a galeria de widgets, o
    // `authored`, o `wet-tuning` e o Timeline têm rect **próprio**, com clamp nas crates deles —
    // continuam a arrastar-se, de propósito.
    layout.bgremoval = layout.inspector;
    layout.padding = layout.inspector;
    layout.painter_sidebar = layout.inspector;
    layout.painter_layers = layout.inspector;
    hero.hit_index.clear_for_frame();

    // M14.5: in live mode (`grid_view` published) the compositor pass
    // shows `game_rt` underneath wherever vello_rt has α=0, so we
    // **skip** the opaque canvas Bg1 fill. Chrome panels (BgElev,
    // panels, topbar) paint their own backdrops — verified in the
    // M14.5 audit. Fixture mode keeps the canvas tint so mockup
    // screenshots stay theme-correct.
    if hero.grid.view.is_none() {
        paint_canvas_bg(&layout, scene, hero.theme);
    }
    // M14.4b: world-space grid overlay. Painted between the canvas
    // background and the selection marquee so the marquee remains
    // legible over the grid. Skipped when toggle is off or host
    // hasn't published a camera view. We substitute the layout's
    // computed canvas rect into the view so the host doesn't have
    // to mirror layout math — it only owns camera + window dims.
    //
    // Layer-order toggle (2026-05-15): the compositor currently
    // composes `game_rt_ldr` UNDER `vello_intermediate` in a single
    // pass — chrome (including the grid) always lands on top of
    // sprites. Real "behind" rendering needs a second Vello
    // intermediate + a 3-layer compositor shader (TODO follow-up).
    // For now we approximate by halving the grid's effective opacity
    // when `grid_in_front == false`, which reads as "the grid is
    // farther / underneath" without changing the compositing path.
    if hero.view.grid_visible
        && let Some(view) = hero.grid.view
    {
        let view = crate::grid::GridView {
            canvas: layout.canvas,
            ..view
        };
        let mut state_for_paint = hero.grid.snap_state.clone();
        if !state_for_paint.grid_in_front {
            state_for_paint.opacity *= 0.4; // LITERAL-PX-OK: grid behind-canvas dim ratio (visual effect)
        }
        crate::grid_snap::render::paint(scene, &view, &state_for_paint, hero.theme);
    }
    // O rect que ESTE paint resolveu para as RÉGUAS, para quem trata ponteiro (o gesto da guia)
    // ler o mesmo retângulo — o irmão do `last_viewport`, e pelo mesmo motivo.
    //
    // ⚠️⚠️ **É a `draw_area`, não o `canvas`** (2026-08-30). O gesto da guia é geométrico e corre
    // ANTES do hit-test de chrome (`input_dispatch.rs`, com um `return` quando acerta), e a régua
    // não está no `HitIndex` — enquanto isto foi a viewport inteira, um press nos 6 px de cima de
    // qualquer botão da barra ou nos 3 px da esquerda de um chip do trilho **nascia uma guia em
    // vez de carregar no botão**. Pintar e agarrar leem a MESMA fonte, que é o que impede a
    // metade visível e a metade do dedo de divergirem.
    hero.last_canvas = layout.draw_area;
    // **As RÉGUAS** (plano 25 §9, a W6.2), por cima da grade e por baixo de tudo o mais: elas
    // são chrome de borda, e a arte passa por baixo delas como passa por baixo do Inspector.
    // O zero é a origem da GRADE — um número, dois consumidores.
    if hero.rulers_live()
        && let Some(view) = hero.grid.view
    {
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
    paint_top_bar(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        hero.image_edit.mode_on,
        &hero.motion,
    );
    // Publish Inspector + Hierarchy panel rects so wheel-event
    // dispatch can route to them. Both are static (no drag offset).
    // When a panel is hidden via its left-rail toggle we DROP the
    // published rect so dispatch's "inside panel" tests don't match
    // a stale geometry.
    if hero.is_panel_visible("inspector") {
        hero.store.set_panel_rect(ids::INSP_PANEL, layout.inspector);
    } else {
        hero.store.clear_panel_rect(ids::INSP_PANEL);
    }
    if hero.is_panel_visible("hierarchy") {
        hero.store.set_panel_rect(ids::HIER_PANEL, layout.hierarchy);
    } else {
        hero.store.clear_panel_rect(ids::HIER_PANEL);
    }
    // Mirror the global picker's current value into the target
    // widget's `widget_colors` slot before either panel paints so
    // color circles inside the Inspector see this frame's value.
    if let Some(target) = hero.store.picker_target()
        && let Some((value, _, _, _)) = hero.store.blender_picker(ids::INSP_BLENDER_PICKER)
    {
        hero.store.set_widget_color(target, value.rgba);
        // Mirror Grid-Settings swatch edits back into the grid_snap
        // state so the canvas overlay re-paints with the new color.
        if target == crate::grid_snap::ids::GS_COLOR_PICKER {
            hero.grid.snap_state.color_rgba = value.rgba;
        }
    }
    // ADR-0029 Phase C.2: Hierarchy migrated to a typed Panel — selection
    // label is read via `host.selection()` inside the panel's `paint`;
    // live entries and rename-target live in panel-owned thread-local /
    // typed `HierarchyState` respectively. No host-side publish needed.
    //
    // Publish the picker's outer rect so dispatch's "is the click
    // inside the picker?" test can reason about its bounds.
    if hero.store.picker_target().is_some()
        && let Some(picker_rect) = color_picker_demo::current_picker_rect(&layout, &hero.store)
    {
        hero.store
            .set_panel_rect(ids::INSP_BLENDER_PICKER, picker_rect);
    } else {
        hero.store.clear_panel_rect(ids::INSP_BLENDER_PICKER);
    }

    // Wave 5 stage D — paint each panel via the PanelRegistry in
    // z-order. Bottom-first, so the panel most recently clicked /
    // dragged / opened sits on top. Panels that haven't been touched
    // yet inherit a default order at the bottom (fallback list below
    // also covers floating panels that have their own panel rects:
    // GAL_PANEL + GS_PANEL).
    //
    // INSP_BLENDER_PICKER is intentionally NOT in the panel
    // registry — it's painted out-of-band AFTER every floating panel
    // (see `paint_blender_picker_demo` below) so it sits on top of
    // every other panel regardless of z order.
    //
    // Each manifest's `paint_fn` owns its full per-frame logic:
    // visibility check + lazy default rect + drag/resize clamp +
    // chrome publish + actual paint + content_h publish + scroll
    // clamp + stale-rect cleanup on hide. Adding a new panel needs
    // zero edits to this iteration — drop `PANEL_MANIFEST` in the
    // panel module + 1 line in `panel_registry::PANEL_REGISTRY`.
    let mut z_order: Vec<ph2d_a11y::NodeId> = hero.store.panel_z_order().to_vec();
    for &fallback in PANEL_Z_ORDER_FALLBACK {
        if !z_order.contains(&fallback) {
            z_order.push(fallback);
        }
    }
    // ADR-0029 Phase D: legacy fn-pointer dispatch deleted. Every
    // in-tree panel lives in `crate::panel::PANEL_REGISTRY` as a
    // typed `Panel<State>`. The z-order walk resolves each id to its
    // typed entry; ids that don't match (e.g. `INSP_BLENDER_PICKER`,
    // painted out-of-band below) are silently skipped.
    crate::panel::with_registry_opt(|reg| {
        for panel_id in z_order {
            if let Some(idx) = reg.find_by_panel_node_id(panel_id) {
                // Hit barrier: register the panel rect BEFORE the
                // widgets inside `panel.paint()` so the gizmo's hit
                // rects (registered earlier this frame) don't bleed
                // through the panel surface. `HitIndex::hit()` walks
                // back-to-front, so internal panel widgets registered
                // by `paint()` below still outrank this barrier — only
                // empty panel area falls back to it. Enio 2026-05-25:
                // "alças do gizmo da sprite podem ser acessadas
                // através dos painéis. Isso não pode acontecer."
                if let Some(panel_rect) = hero.store.panel_rect(panel_id) {
                    hero.hit_index.register(panel_id, panel_rect);
                }
                let mut typed_ctx = crate::panel::PaintCtx {
                    host: hero,
                    layout: &layout,
                    viewport,
                    scene,
                    text_system,
                };
                reg.panels_mut()[idx].paint(&mut typed_ctx);
            }
        }
    });
    // hero/scene/text_system unborrowed for the
    // rest of paint_hero_screen (bottom HUD, picker overlay, tooltip,
    // context menu, drop overlay).
    //
    // Left rail painted AFTER the docked panels so its buttons — and the
    // Painter Shapes flyout, which extends over the Inspector/Hierarchy area —
    // sit ABOVE them, both visually and for hit-testing (HitIndex walks
    // back-to-front, so the rail chips registered here win any overlapping
    // click). Still below the bottom HUD / color picker / context menu, which
    // paint after this (unchanged). Painter mode = Image-Tools on AND the
    // active tool is the Painter (mirrored shell-side into `active_tool_id`),
    // which swaps the transform block for the paint tools.
    let painter_active = hero.rail_shows_painter_tools();
    paint_left_rail(
        &layout,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        painter_active,
        &hero.motion,
    );
    if hero.view.stats_visible {
        paint_bottom_hud(&layout, scene, text_system, hero.theme, hero.stats);
    }
    // W2.T2.3: the Painter color swatch lives INSIDE the Painter sidebar
    // panel (`ph2d-panel-painter-sidebar`), painted there alongside
    // Size/Opacity and registering hit `ids::PAINTER_COLOR_THUMB`. The
    // open-picker dispatch (pointer.rs) + the bridge read-back are keyed
    // on that hit id and are placement-agnostic, so nothing here paints
    // the swatch — the docked panel owns it (the earlier floating
    // top-right swatch was the wrong home and was removed).
    // BlenderColorPicker — painted AFTER every floating panel
    // (Inspector, Hierarchy, Widget Gallery, Grid Settings) so it
    // never sits visually behind one of them. The painter is a no-op
    // when `picker_target` is None.
    if hero.store.picker_target().is_some() {
        color_picker_demo::paint_blender_picker_demo(
            &layout,
            scene,
            text_system,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
        );
    }
    // Tooltip overlay on top of all chrome (Phase 3 polish).
    topbar::paint_hover_tooltip(
        scene,
        text_system,
        hero.theme,
        &hero.hit_index,
        &hero.store,
        layout.viewport,
    );
    // Context menu overlay — last so the floating menu sits above
    // every panel, including the floating BlenderColorPicker.
    context_menu_overlay::paint_context_menu_overlay(
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        &hero.project,
        &hero.motion,
        viewport,
    );
    // Fill (Bucket) "Fill adjust" modal — a floating, draggable card at the ColorDrop release point
    // (no-op when closed). Painted after the context menu so its hit rects sit above the canvas.
    chrome::paint_fill_adjust_modal(
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        &hero.tether,
        viewport,
    );
    // A JANELA DO INPUT MAP (plano 30 §0.2) — flutuante sobre o canvas, à la Godot. No-op quando
    // fechada. Mesma camada de diálogo flutuante que o Fill modal, e pintada DEPOIS do menu de
    // contexto pelo mesmo motivo: os hit rects dela ficam acima do canvas.
    chrome::paint_input_map_window(
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        &hero.input_map,
        viewport,
    );
    // Onion settings modal (ADR-0142 W3b) — a floating, draggable card opened from the timeline's
    // Onion-settings button (no-op when closed). Same floating-dialog layer as the Fill modal.
    chrome::paint_onion_modal(
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        viewport,
    );
    // Command palette (Motion's "Add Node") — a full-screen dimmed modal painted over the whole app
    // (no-op when closed). Above the floating dialogs so it dominates; its full-viewport scrim registers
    // FIRST so the card + item pills (registered after) win the back-to-front hit walk.
    chrome::paint_command_palette(
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        viewport,
        &hero.motion,
    );
    // ⭐ **O PIE MENU** (estudo de UI viva, E4) — acima da paleta porque ele é o gesto EM CURSO: o
    // artista está com a tecla em baixo, e nada pode ficar por cima do que a mão está a fazer.
    //
    // ⚠️ Ele **não regista hit-rect nenhum**, e a ausência é o desenho: quem escolhe é a DIRECÇÃO,
    // não um clique num rectângulo. Registar caixas daria um segundo caminho para a escolha — o que
    // fica sob o dedo — e os dois divergiriam na borda de cada sector.
    if let Some(radial) = hero.store.radial() {
        crate::widget::paint_radial_menu(radial, scene, text_system, hero.theme);
    }
    // M14.4e: file-drop overlay sits above EVERY layer (chrome,
    // tooltips, context menus) so the user always sees the "Drop to
    // import" hint while the OS drag is active.
    if let Some((paths, cursor)) = hero.dragging_files.as_ref() {
        paint_drop_overlay(&layout, paths, *cursor, scene, text_system, hero.theme);
    }
    // ⭐⭐ **O que vai na mão** (plano `docs/Components/07`, B4) — o PRIMEIRO fantasma deste editor
    // a seguir o cursor. Por cima de tudo, inclusive do aviso de largar ficheiro: os dois nunca
    // coexistem (um é arrasto interno, o outro é do sistema operativo), e a ordem torna isso
    // observável se algum dia coexistirem.
    super::asset_drag_ghost::paint_asset_drag_ghost(
        hero.store.asset_drag(),
        scene,
        text_system,
        hero.theme,
    );
}
