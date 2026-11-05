//! ⭐⭐ **A fila ② pode morar EMBAIXO** (D10 decisão 2, F5 — 08/10): a opção de layout para tablet.
//!
//! Com as barras reais do vetor no caso mais longo (Node): a faixa passa para baixo da área sem a
//! tapar, e os menus que ela abre — o `⋯`, os pulldowns da área e (F16) o painel do traço e as
//! listas da barra de controle — crescem para CIMA do chip. O CONTROLO: com a fila em cima, os
//! mesmos menus abrem por baixo.

use bumpalo::Bump;
use ph2d_editor_core::ids;
use ph2d_editor_core::screens::hero::paint_hero_screen;
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};

/// A largura do iPad mini — onde o `⋯` da fila do Node existe (F0). ⚠️ A ALTURA é a do mini
/// embaixo, e folgada no controlo de cima: lá um menu mais alto que o espaço por baixo é ENCAIXADO
/// ao ecrã pela lei que já existia (`clamp_menu_to_viewport`), e o controlo mediria essa lei.
fn pinta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    let mut scene = ph2d_vector::VectorScene::new();
    let h = if hero.store.tool_row_bottom() {
        744.0
    } else {
        1400.0
    };
    let vp = Rect::new(0.0, 0.0, 1133.0, h);
    paint_hero_screen(hero, vp, &mut scene, ts);
}

fn com_fila(bottom: bool, mode: DrawMode, ts: &mut ph2d_text::TextSystem) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.store.set_tool_row_bottom(bottom);
    pinta(&mut hero, ts);
    ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
        mode,
        ..VectorStyleSnapshot::default()
    }));
    ph2d_panel_vector::set_current_selection_count(1);
    ph2d_panel_vector::set_current_vertex_count(2);
    // F16: com uma forma com traço, a barra de controle mostra os três chips «▾».
    ph2d_panel_vector::set_stroke_paint_kind(Some(ph2d_panel_vector::StrokePaintKind::Solid));
    let bars = ph2d_panel_vector::bars::tool_bars(&hero.store, None, Vec::new());
    hero.store.set_tool_bars(Some(bars));
    // F16: o vetor já não publica pulldowns de área (o mesmo que `bars_publish::publish`).
    hero.store.set_area_commands(Vec::new(), Vec::new());
    pinta(&mut hero, ts);
    pinta(&mut hero, ts);
    hero
}

fn down(hero: &mut HeroScreen, r: Rect) {
    let arena = Bump::new();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    for kind in [PointerKind::Down, PointerKind::Up] {
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        let evs = ph2d_editor_core::interaction::dispatch_pointer(
            &mut hero.store,
            &hero.hit_index,
            ev,
            &arena,
        )
        .to_vec();
        for e in evs {
            hero.apply_event(e);
        }
    }
}

/// F16: os três chips «▾» da barra de controle do vetor — o painel do traço e as duas listas
/// abrem por baixo deles, e com a fila embaixo têm de subir como os outros menus da fila.
const BAR_LISTS: [NodeId; 3] = [
    ph2d_panel_vector::ids::VECTOR_BAR_STROKE,
    ph2d_panel_vector::ids::VECTOR_BAR_PROFILE,
    ph2d_panel_vector::ids::VECTOR_BAR_KIND,
];

/// As linhas que o chip «▾» `chip` abre (o painel do traço é medido pelo cartão inteiro).
fn bar_list_rows(hero: &HeroScreen, chip: NodeId) -> Vec<Rect> {
    use ph2d_panel_vector::ids as v;
    if chip == v::VECTOR_BAR_STROKE {
        return hero
            .store
            .dropdown_popover()
            .filter(|(id, _)| *id == chip)
            .map(|(_, r)| r)
            .into_iter()
            .collect();
    }
    let rows: Vec<NodeId> = if chip == v::VECTOR_BAR_PROFILE {
        (0..v::MAX_WIDTH_PRESETS)
            .map(v::vector_width_preset_id)
            .collect()
    } else {
        vec![
            ph2d_tool_vector::ids::VECTOR_STROKE_KIND_SOLID,
            ph2d_tool_vector::ids::VECTOR_STROKE_KIND_PATTERN,
            ph2d_tool_vector::ids::VECTOR_STROKE_KIND_BRUSH,
        ]
    };
    rows.iter()
        .filter_map(|&id| hero.hit_index.rect_for(id))
        .collect()
}

/// O fundo do `⋯` aberto: o último `RAIL_BACKDROP` do quadro (a faixa regista o dela antes).
fn moldura_do_transbordo(hero: &HeroScreen) -> Option<Rect> {
    hero.hit_index
        .iter_registrations()
        .filter(|(id, _)| *id == ids::RAIL_BACKDROP)
        .map(|(_, r)| r)
        .last()
}

/// O chip que abre um menu da fila e a 1.ª linha desse menu, pintada.
fn chip_e_linha(
    hero: &mut HeroScreen,
    ts: &mut ph2d_text::TextSystem,
) -> Vec<(NodeId, Rect, Rect)> {
    let mut out = Vec::new();
    let chips: Vec<NodeId> = std::iter::once(ids::TOOL_BAR_OVERFLOW)
        .chain((0..ids::MAX_AREA_MENUS).map(ids::area_menu_button))
        .chain(BAR_LISTS)
        .collect();
    for chip in chips {
        let Some(r) = hero.hit_index.rect_for(chip) else {
            continue;
        };
        down(hero, r);
        pinta(hero, ts);
        if BAR_LISTS.contains(&chip) {
            let painted = bar_list_rows(hero, chip);
            if let (Some(top), Some(bottom)) = (
                painted.iter().map(|q| q.y).reduce(f32::min),
                painted.iter().map(|q| q.y + q.h).reduce(f32::max),
            ) {
                out.push((chip, r, Rect::new(0.0, top, 0.0, bottom - top)));
            }
            for id in BAR_LISTS {
                if let Some(ph2d_editor_core::interaction::InteractiveState::Dropdown {
                    open,
                    ..
                }) = hero.store.get_mut(id)
                {
                    *open = false;
                }
            }
            pinta(hero, ts);
            continue;
        }
        let rows: Vec<NodeId> = if chip == ids::TOOL_BAR_OVERFLOW {
            hero.store
                .tool_overflow()
                .iter()
                .filter_map(|e| e.node_id())
                .collect()
        } else {
            (0..ids::MAX_AREA_MENUS)
                .find(|&s| ids::area_menu_button(s) == chip)
                .and_then(|s| u8::try_from(s).ok())
                .map(|s| {
                    hero.store
                        .area_menu_rows(s)
                        .iter()
                        .filter_map(|e| e.node_id())
                        .collect()
                })
                .unwrap_or_default()
        };
        // ⚠️ O menu INTEIRO (todas as linhas pintadas): a 1.ª linha sozinha deixava passar um menu
        // que começasse no chip e descesse por cima dele (a mutação M23 sobrevivia).
        let painted: Vec<Rect> = rows
            .iter()
            .filter_map(|&id| hero.hit_index.rect_for(id))
            .collect();
        if let (Some(top), Some(bottom)) = (
            painted.iter().map(|q| q.y).reduce(f32::min),
            painted.iter().map(|q| q.y + q.h).reduce(f32::max),
        ) {
            // ⚠️ O `⋯` mede a MOLDURA: o menu antigo tinha a coluna mais curta que o fundo, que tapava o chip.
            let (top, bottom) =
                match moldura_do_transbordo(hero).filter(|_| chip == ids::TOOL_BAR_OVERFLOW) {
                    Some(m) => (top.min(m.y), bottom.max(m.y + m.h)),
                    None => (top, bottom),
                };
            out.push((chip, r, Rect::new(0.0, top, 0.0, bottom - top)));
        }
        hero.store.close_context_menu();
        pinta(hero, ts);
    }
    out
}

/// O CONTROLO dos dois testes: os três chips «▾» da barra de controle foram abertos e medidos — sem
/// isto a lei passaria sobre o `⋯` sozinho (F16 tirou os pulldowns de área do vetor).
fn mediu_os_tres_da_barra(pares: &[(NodeId, Rect, Rect)]) {
    for id in BAR_LISTS {
        assert!(
            pares.iter().any(|(c, _, _)| *c == id),
            "{id:?}: o chip da barra não abriu nada medível ({} medidos)",
            pares.len()
        );
    }
}

/// ⭐⭐ **Embaixo: a faixa por baixo da área, e os menus da fila crescem para cima.**
#[test]
fn at_the_bottom_the_row_sits_below_the_area_and_its_menus_grow_up() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut pares = Vec::new();
    // O Node põe a cauda atrás do `⋯` no mini; o Select mostra a barra de controle — os dois medem-se.
    for mode in [DrawMode::Node, DrawMode::Select] {
        let mut hero = com_fila(true, mode, &mut ts);
        let l = hero.last_layout.expect("layout");
        assert!(
            l.tool_bar.y + 0.5 >= l.draw_area.y + l.draw_area.h,
            "a fila não está por baixo da área: {:?} × {:?}",
            l.tool_bar,
            l.draw_area
        );
        pares.extend(chip_e_linha(&mut hero, &mut ts));
    }
    mediu_os_tres_da_barra(&pares);
    assert!(
        pares.iter().any(|(c, _, _)| *c == ids::TOOL_BAR_OVERFLOW),
        "o `⋯` não abriu nada medível: a lei mediria só a barra de controle"
    );
    for (_, chip, row) in pares {
        assert!(
            row.y + row.h <= chip.y + 0.5,
            "um menu da fila embaixo abriu por baixo do chip ({row:?} sob {chip:?})"
        );
    }
}

/// O pulldown de um módulo (o vetor já não publica nenhum, F16) passa pelo outro pintor do menu.
#[test]
fn an_area_pulldown_also_opens_upwards_at_the_bottom() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = com_fila(true, DrawMode::Node, &mut ts);
    let linha = |n: u64| ph2d_editor_core::widget::ToolRailEntry::Glyph {
        id: NodeId(9_000 + n),
        label: format!("Row {n}"),
        path: ph2d_vector::BezPath::new(),
        active: false,
        sub: String::new(),
        row_w: 0.0,
    };
    hero.store.set_area_commands(
        vec![ph2d_editor_core::interaction::AreaMenu {
            label: "View".to_string(),
            face: "View".to_string(),
            faces: Vec::new(),
            rows: (0..4).map(linha).collect(),
        }],
        Vec::new(),
    );
    pinta(&mut hero, &mut ts);
    pinta(&mut hero, &mut ts);
    // No mini o chip dele pode ir para trás do `⋯`, e a âncora é então o `⋯` (`tool_bar_overflow`).
    let botao = ids::area_menu_button(0);
    let chip = hero
        .hit_index
        .rect_for(botao)
        .or_else(|| hero.hit_index.rect_for(ids::TOOL_BAR_OVERFLOW))
        .expect("nem o pulldown nem o `⋯` estão na fila");
    hero.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(botao));
    pinta(&mut hero, &mut ts);
    let fundo = (0..4)
        .filter_map(|n| hero.hit_index.rect_for(NodeId(9_000 + n)))
        .map(|r| r.y + r.h)
        .reduce(f32::max)
        .expect("o pulldown não pintou as linhas");
    assert!(
        fundo <= chip.y + 0.5,
        "o pulldown desceu sobre o chip: linhas até {fundo} sob {chip:?}"
    );
}

/// A paleta abre o `⋯` sem Down e sem o `tool_bar_overflow` saber por onde veio: a mesma lei.
#[test]
fn the_palette_door_also_opens_the_overflow_upwards_at_the_bottom() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = com_fila(true, DrawMode::Node, &mut ts);
    let chip = hero
        .hit_index
        .rect_for(ids::TOOL_BAR_OVERFLOW)
        .expect("o Node põe a cauda atrás do `⋯` no mini");
    hero.apply_event(ph2d_editor_core::interaction::WidgetEvent::Click(
        ids::TOOL_BAR_OVERFLOW,
    ));
    pinta(&mut hero, &mut ts);
    let m = moldura_do_transbordo(&hero).expect("o menu pintou o fundo");
    assert!(
        m.y + m.h <= chip.y + 0.5,
        "o `⋯` pela paleta tapou o chip: {m:?} sobre {chip:?}"
    );
}

/// **O CONTROLO: em cima, os mesmos menus abrem por baixo do chip** — e a fila fica por cima da área.
#[test]
fn at_the_top_the_same_menus_open_below() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut pares = Vec::new();
    for mode in [DrawMode::Node, DrawMode::Select] {
        let mut hero = com_fila(false, mode, &mut ts);
        let l = hero.last_layout.expect("layout");
        assert!(l.tool_bar.y + l.tool_bar.h <= l.draw_area.y + 0.5);
        pares.extend(chip_e_linha(&mut hero, &mut ts));
    }
    mediu_os_tres_da_barra(&pares);
    for (_, chip, row) in pares {
        assert!(
            row.y >= chip.y + chip.h - 0.5,
            "em cima, o menu abriu por cima do chip"
        );
    }
}

/// ⭐ **O flyout de grupo do Painter (Formas) sobe com a fila embaixo** — ele é posto à mão por baixo
/// do chip (sem o encaixe ao ecrã que os menus têm), então sem a lei dele cairia fora da área.
/// O CONTROLO: com a fila em cima, desce.
#[test]
fn the_painter_flyout_climbs_when_the_row_is_at_the_bottom() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let _ = ph2d_panel_registry_init::register_all_panels();
    for bottom in [true, false] {
        let mut hero = HeroScreen::new(NodeId(1));
        hero.image_edit.mode_on = true;
        hero.image_edit.active_tool_id = Some("painter");
        hero.store.set_tool_row_bottom(bottom);
        hero.store.set_painter_shapes_flyout_open(true);
        let mut scene = ph2d_vector::VectorScene::new();
        let vp = Rect::new(0.0, 0.0, 1930.0, 1040.0);
        paint_hero_screen(&mut hero, vp, &mut scene, &mut ts);
        let mut scene = ph2d_vector::VectorScene::new();
        paint_hero_screen(&mut hero, vp, &mut scene, &mut ts);
        let bar = hero.last_layout.expect("layout").tool_bar;
        let chip = hero
            .hit_index
            .rect_for(ids::PAINTER_RAIL_SHAPE_FREEHAND)
            .expect("o flyout das formas não foi pintado");
        if bottom {
            assert!(
                chip.y + chip.h <= bar.y + 0.5,
                "embaixo, o flyout desceu: {chip:?} × {bar:?}"
            );
        } else {
            assert!(
                chip.y >= bar.y + bar.h - 0.5,
                "em cima, o flyout subiu: {chip:?} × {bar:?}"
            );
        }
    }
}

/// ⭐ Com a fila embaixo os painéis continuam no alto e a pílula não a cobre. Visto em janela real (F8):
/// «a lista começa abaixo da barra» atirava a Hierarchy e o Inspector para fora da janela, e a pílula
/// ficava em cima da fila. *Mutação: `tool_bar` no lugar de `fila_que_cobre_o_topo` em `cards_lista`, ou
/// sem o `bottom_hud` de `move_tool_bar_to_bottom` ⇒ vermelho.*
#[test]
fn with_the_row_at_the_bottom_the_panels_stay_up_and_the_pill_clears_it() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert("inspector", true);
    pinta(&mut hero, &mut ts);
    let em_cima = hero.store.panel_rect(ids::INSP_PANEL).expect("Inspector");
    hero.store.set_tool_row_bottom(true);
    for _ in 0..3 {
        pinta(&mut hero, &mut ts);
    }
    let l = hero.last_layout.expect("layout");
    assert!(l.tool_bar.y > l.draw_area.y, "a fila desceu");
    let lista = hero.store.panel_rect(ids::INSP_PANEL).expect("Inspector");
    assert!(
        (lista.y - em_cima.y).abs() <= l.tool_bar.h + 1.0 && lista.y < l.tool_bar.y,
        "o Inspector saiu do alto: {} (antes {}), a fila em {}",
        lista.y,
        em_cima.y,
        l.tool_bar.y
    );
    assert!(
        l.bottom_hud.y + l.bottom_hud.h <= l.tool_bar.y,
        "a pílula cobre a fila: {:?} × {:?}",
        l.bottom_hud,
        l.tool_bar
    );
    hero.store.set_tool_row_bottom(false);
}
