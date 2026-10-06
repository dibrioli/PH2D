use super::*;
use crate::ids;
#[path = "na_escala.rs"]
mod na_escala;
pub use na_escala::paint_hero_screen_na_escala;

/// **Todo painel que o passeio de z-order alcança**, mesmo que o store ainda não o tenha visto.
///
/// ⚠️ **Estar no registro e estar visível NÃO chega**: se o id não passa por aqui, o painel é
/// registado, visível e **nunca pintado** — nada quebra e nada avisa. Este arquivo pagou esse
/// defeito várias vezes (as notas dentro da lista são o registo de cada uma).
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
    // ⭐⭐⭐ O painel do ESQUELETO (`ph2d-panel-skeleton`, 2026-09-09): encaixa na coluna da
    // direita, com a visibilidade conduzida pela shell (*a cena tem ossos, ou a ferramenta que os
    // cria está na mão*). O `paint()` dele sai cedo quando escondido, então esta entrada é inerte
    // no resto do tempo — e SEM ela o painel registado e visível nunca é alcançado pelo passeio, e
    // nunca é pintado.
    ids::SKELETON_PANEL,
    // ⭐⭐⭐ O painel das TAGS (`ph2d-panel-tags`, TOP-20 #9): encaixa na coluna da direita e é
    // categoria MUNDO, como o de física. O `paint()` dele sai cedo quando escondido, então esta
    // entrada é inerte no resto do tempo — e SEM ela o painel registado e visível nunca é
    // alcançado pelo passeio, e nunca é pintado.
    ids::TAGS_PANEL,
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
    // ⭐ O NAVEGADOR DE ASSETS (plano `docs/Components/07`) — e ele entra AQUI porque as
    // notas acima já pagaram esta lição: sem esta linha o painel fica registado, visível, com os
    // gates verdes, e **nunca pintado**.
    ids::ASSET_PANEL,
    ids::INSP_BLENDER_PICKER,
    ids::GAL_PANEL,
    // A bancada de widgets (`ph2d-panel-widget-lab`). ⚠️ **Sétima vez que este arquivo paga o
    // mesmo defeito** — o painel nasceu registado, visível ao clique do menu, e o passeio de
    // z-order nunca chegava a ele: nada quebra, nada avisa, e o menu abre um painel invisível.
    // Desta vez o `every_registered_panel_is_reachable_by_the_z_order_walk` apanhou-o na primeira
    // corrida, que é exactamente o que as notas acima pediam que acontecesse.
    ids::LAB_PANEL,
    ids::AUDIO_MIXER_PANEL,
    ids::AUDIO_EDITOR_PANEL,
    ids::GS_PANEL,
];

/// Top-level hero paint orchestrator. Clears + re-populates the
/// hit-index, then walks each region painter in z-order
/// (canvas → selection overlay → chrome → HUD).
/// A aparência pedida pelo ambiente, lida **uma vez** por processo.
///
/// ⚠️ `OnceLock` de propósito: `std::env::var` num caminho por-quadro é uma leitura do SO em cada
/// quadro, e o valor não pode mudar a meio de uma sessão sem ninguém saber porquê.
// ⚠️ `pub`, e re-exportada por `hero.rs` (`pub use paint::*`): a shell resolve o tema de
//    arranque (`PH2D_THEME`) pela MESMA leitura da aparência que o `HeroScreen::new` usa — duas
//    leituras do ambiente seriam duas respostas à mesma pergunta.
pub fn ui_look_from_env() -> ph2d_tokens::UiLook {
    static LOOK: std::sync::OnceLock<ph2d_tokens::UiLook> = std::sync::OnceLock::new();
    *LOOK.get_or_init(|| {
        ph2d_tokens::UiLook::from_env_value(std::env::var("PH2D_UI_NEW").ok().as_deref())
    })
}

pub fn paint_hero_screen(
    hero: &mut HeroScreen,
    viewport: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) {
    // ⭐ **O quadro começa aqui para o BALÃO** — ⚠️ ANTES de qualquer pintor, senão o que ele
    // recolheu já foi apagado. Ver [`crate::text_elide::balao::novo_quadro`].
    crate::text_elide::balao::novo_quadro();
    // ⭐ **E para a ROLAGEM** (D9): as alturas e os donos de barra que este quadro publicar são os
    // únicos que sobrevivem a ele — ver [`crate::interaction::WidgetStore::end_scroll_frame`].
    hero.store.begin_scroll_frame();
    // Publish the user-picked radius scale to the thread-local read
    // by `paint::fill_rounded_rect` / `stroke_rounded_rect`. Set
    // every frame so it stays in sync with the topbar's radius menu.
    crate::paint::set_radius_scale(hero.store.radius_scale());
    // Same pattern for the text-rendering strategy — read by
    // `paint_text*` via the `paint::text_rendering()` thread-local.
    crate::paint::set_text_rendering(hero.text_rendering);
    // E o estilo do texto (fonte · peso · tamanho) — lido pela mesma porta que mede e pinta.
    ph2d_text::set_active_text_style(hero.text_style);
    // E a escala da interface inteira — a marca do menu lê-a (`crate::ui_scale`).
    crate::ui_scale::publish(hero.ui_scale);
    // ⭐⭐⭐ **A APARÊNCIA do app, uma vez por quadro** (Enio, 2026-09-03: *«por enquanto permanece
    // a antiga»*). ⚠️ Lida do ambiente **uma só vez** — `PH2D_UI_NEW=1` liga o redesenho, tudo o
    // resto é a UI de sempre. ⛔ Não é uma preferência gravada: um redesenho a meio não deve poder
    // ficar ligado sem que se saiba porquê.
    crate::paint::set_ui_look(ui_look_from_env());
    // Stash the viewport so chrome event handlers in `chrome/` can
    // make smart layout decisions (cascade submenu side-flip etc.).
    hero.last_viewport = viewport;
    // A régua não tem quem lhe mantenha o `ButtonState`; sem isto a linha dela no menu *Ver*
    // nunca aparece marcada. Ver `menu_bar::publish_toggle_state`.
    super::menu_bar::publish_toggle_state(hero);
    // ⭐⭐ **A ordem z passa a ser «os painéis visíveis, o último a aparecer no topo»** — e tem de
    // ser reconciliada ANTES do layout, porque é dela que sai qual aba está à frente em cada
    // encaixe (`slot_tabs`).
    super::slot_tabs::reconcile_z(hero);
    // ⭐⭐ **E uma aba largada muda o painel de encaixe** (decisão D4). ANTES do layout, porque é a
    // posição nova que este quadro tem de desenhar — e a largada foi julgada contra o layout do
    // quadro anterior, que é o que o artista estava a ver quando largou.
    super::slot_tabs::resolve_tab_drop(hero);

    let mut layout = super::frame_layout::frame_layout(hero, viewport, text_system);
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
    // ⭐⭐⭐ **O CHÃO — sempre**, e é o que faz cada área ler-se como cartão. ⚠️ Fora do `if` de
    //    propósito: o pintor acima só corre em modo FIXTURA, e foi essa a razão de a wave 31 não
    //    ter mudado nada no ecrã do dono.
    super::canvas::paint_window_ground(&layout, scene, hero.theme);
    // M14.4b: world-space grid overlay, painted between the canvas background and the selection
    // marquee so the marquee remains legible over it. ⭐ **À FRENTE ou ATRÁS dos objectos** —
    // a decisão vive em [`super::grid_layer`] (report do dono de 2026-09-24: o `Behind` era um
    // `opacity × 0,4` e não punha a grade atrás de nada).
    super::grid_layer::paint_in_chrome(hero, &layout, scene);
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
    // ⭐⭐⭐ **E o que sobra DEPOIS das réguas** — ver `HeroScreen::last_content`.
    //
    // ⚠️ **A condição é HOISTADA e usada duas vezes**, e tem de ser: se este rect fosse derivado de
    // `rulers_live()` sozinho, um quadro sem `grid.view` publicaria um recuo de `20 px` contra uma
    // régua que não chegou a ser pintada. *Um rect que promete um recuo que a tela não tem é a
    // mesma doença de duas metades a divergir, com o sinal trocado.*
    let rulers_on =
        hero.rulers_live() && hero.grid.view.is_some() && hero.documents.active().is_none();
    hero.last_content = crate::ruler::content(layout.draw_area, rulers_on);
    // E o layout INTEIRO, para o gesto de largura das colunas ler os mesmos rects (ver o
    // doc do campo).
    hero.last_layout = Some(layout);
    // ⭐⭐ **Um QUADRO activo é dono da área de desenho** (MiroClone): pinta-se por cima da cena e
    //    nada do que é da cena (réguas, gizmos, selecção) chega a ser pintado ou clicável.
    if hero.documents.active().is_some() {
        let r = super::board_view::area(&layout);
        let area = super::board_view::area_of(r);
        let theme = hero.theme;
        if let Some((board, live)) = hero.documents.active_parts() {
            ph2d_board_render::paint(
                board,
                area,
                scene,
                theme,
                text_system,
                &mut live.render_cache,
            );
            if let Some(ed) = live.editor.as_mut() {
                let overlay = ed.overlay(&board.doc, text_system);
                ph2d_board_render::paint_overlay(board, area, scene, theme, &overlay, ed.metrics());
            }
        }
        super::board_bar::paint(hero, r, scene, text_system);
        // Pintar e agarrar leem o MESMO rect (o `board_view` lê o `last_canvas`).
        hero.last_canvas = r;
        hero.last_content = r;
    } else {
        super::paint_canvas_overlays::paint_canvas_overlays(
            hero,
            layout,
            rulers_on,
            scene,
            text_system,
        );
    }
    // ⛔ **O CHROME LEGADO** — os clusters de botões da barra e o trilho lateral. Fora por
    // omissão desde 2026-08-30 (Enio: *«pode tirar também os botões do topo para começarmos a
    // trabalhar a barra superior»*), e **`F9` devolve-os**: nenhum atalho de teclado alcança as
    // pílulas de módulo, então apagá-los deixaria o app sem forma de abrir um módulo.
    if hero.view.legacy_chrome {
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
    } else {
        super::menu_bar::paint_menu_bar(
            &layout,
            scene,
            text_system,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
            &hero.motion,
            &hero.documents,
        );
    }
    // ⛔⛔⛔ **A SEGUNDA PORTA DO RECT DO INSPECTOR E DA HIERARQUIA SAIU DAQUI (2026-09-08).**
    //
    // > *«se arrastar um para a área da Hierarquia e colapsar a hierarquia (puxando para
    // > esquerda) as abas do painel esquerdo ficam travadas»* — Enio, no smoke da wave 34b.
    //
    // Este bloco publicava os dois rects **de fora**, com `layout.inspector` / `layout.hierarchy`
    // — isto é, *a coluna da direita* e *a coluna da esquerda* **por nome**. Os outros 20 painéis
    // publicam o próprio `ctx.slot`, que é o encaixe RESOLVIDO (`slot_of` honra o encaixe que o
    // artista arrumou por cima do declarado).
    //
    // ⇒ arrastar a aba de um destes dois para a outra coluna movia a **aba** e deixava o **corpo**
    // onde sempre esteve. E como o [`crate::screens::dock_sides::DockSides::from_published`]
    // responde *«esta coluna está ocupada?»* cruzando os rects PUBLICADOS com o rect da coluna, a
    // coluna de destino lia-se **vazia**: a fila de abas ficava a flutuar sobre a área de desenho,
    // com a alça de reabertura armada por baixo dela — abas que se vêem, se clicam, e não trazem
    // corpo nenhum.
    //
    // ⚠️ **A metade que este bloco fazia bem — largar o rect ao ficar invisível — foi PARA DENTRO
    // dos dois painéis**, que é onde as outras 20 crates a fazem. Ela continua a correr todo
    // quadro porque os dois estão no [`PANEL_Z_ORDER_FALLBACK`]: o `panel_walk` caminha-os mesmo
    // invisíveis, e o `paint` deles no-opa depois de limpar.
    //
    // ⛔ O censo que o prova é `every_docked_panel_paints_where_its_tab_says`
    // (`ph2d-panel-registry-init/tests/`): ele MOVE cada painel para cada encaixe que ele permite
    // e compara o rect publicado com o do encaixe. Antes desta cura acusava estes dois, e só
    // estes dois, em 4 células.
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

    panel_walk::walk(hero, &layout, viewport, scene, text_system);
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
    // ⭐⭐ **A ALÇA de uma coluna fechada** — antes do rail, porque ela vive na margem e nada se
    //    sobrepõe a ela. Não pinta nada com as colunas abertas.
    super::dock_reopen::paint_dock_reopen(&layout, scene, hero.theme);
    let painter_active = hero.rail_shows_painter_tools();
    if hero.view.legacy_chrome {
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
    } else if hero.documents.active().is_none() {
        // ⚠️ Só com a aba `Scene`: num QUADRO estes chips (mover, girar, desfazer da cena) seriam
        //    controlos da cena por cima de outro documento.
        // ⭐ **O que não coube fica publicado** para o corpo do `⋯` o desenhar — ANTES do pintor,
        // porque o pintor toma `&hero.store` emprestado.
        super::tool_bar::publish_overflow(
            &mut hero.store,
            text_system,
            &layout,
            painter_active,
            hero.image_edit.mode_on,
        );
        // ⭐ **A FILA** — os mesmos chips, deitados, na região que a área lhes reservou.
        super::tool_bar::paint_tool_bar(
            &layout,
            scene,
            text_system,
            hero.theme,
            &mut hero.hit_index,
            &hero.store,
            painter_active,
            hero.image_edit.mode_on,
            &hero.motion,
        );
    }
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
    // ⭐ As duas bolhas, por cima de todo o chrome — a dica AUTORADA de um widget, ou o balão de
    // uma palavra cortada quando ela se cala. A precedência mora com os dois pintores.
    topbar::paint_hover_overlays(
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
    // A pergunta antes de apagar um quadro: um diálogo do overlay que precisa do NOME do quadro.
    super::document_tabs_menu::paint_confirm_delete(
        &hero.documents,
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
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
    //
    // ⭐ Desde 2026-09-29 o corpo dela passa pela PORTA da rolagem: o que o pintor devolve é a
    // metade que precisa do store mutável (alturas, clamp, dono da barra) e o rect do cartão, que se
    // publica como rect de painel — é ele que dá à janela a roda, o arrasto no corpo e a inércia
    // pelo mesmo caminho de todo painel, sem o caso especial que a shell tinha. Fechada, o rect sai.
    match chrome::paint_input_map_window(
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        &hero.input_map,
        viewport,
    ) {
        Some((card, pending)) => {
            pending.publish(&mut hero.store);
            hero.store.set_panel_rect(ids::INPUT_MAP_SURFACE, card);
        }
        None => hero.store.clear_panel_rect(ids::INPUT_MAP_SURFACE),
    }
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
    if let Some(pending) = chrome::paint_command_palette(
        scene,
        text_system,
        hero.theme,
        &mut hero.hit_index,
        &hero.store,
        viewport,
        &hero.motion,
    ) {
        pending.publish(&mut hero.store);
    }
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
    hero.store.end_scroll_frame();
}
