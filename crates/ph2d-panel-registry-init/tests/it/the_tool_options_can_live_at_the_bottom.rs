//! ⭐⭐ **A fila ② pode morar EMBAIXO** (D10 decisão 2, F5 — 08/10): a opção de layout para tablet.
//!
//! Com as barras reais do vetor no caso mais longo (Node): a faixa passa para baixo da área sem a
//! tapar, e os menus que ela abre — o `⋯` e os pulldowns da área — crescem para CIMA do chip. O
//! CONTROLO: com a fila em cima, os mesmos menus abrem por baixo.

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
    let bars = ph2d_panel_vector::bars::tool_bars(&hero.store, None, Vec::new());
    hero.store.set_tool_bars(Some(bars));
    hero.store
        .set_area_commands(ph2d_panel_vector::bars::module_menus(), Vec::new());
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

/// O chip que abre um menu da fila e a 1.ª linha desse menu, pintada.
fn chip_e_linha(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) -> Vec<(Rect, Rect)> {
    let mut out = Vec::new();
    let chips: Vec<NodeId> = std::iter::once(ids::TOOL_BAR_OVERFLOW)
        .chain((0..ids::MAX_AREA_MENUS).map(ids::area_menu_button))
        .collect();
    for chip in chips {
        let Some(r) = hero.hit_index.rect_for(chip) else {
            continue;
        };
        down(hero, r);
        pinta(hero, ts);
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
            out.push((r, Rect::new(0.0, top, 0.0, bottom - top)));
        }
        hero.store.close_context_menu();
        pinta(hero, ts);
    }
    out
}

/// ⭐⭐ **Embaixo: a faixa por baixo da área, e os menus da fila crescem para cima.**
#[test]
fn at_the_bottom_the_row_sits_below_the_area_and_its_menus_grow_up() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut pares = Vec::new();
    // O Node põe os pulldowns atrás do `⋯` no mini; o Select deixa-os à vista — os dois medem-se.
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
    assert!(
        pares.len() >= 2,
        "o `⋯` e um pulldown — só {} medidos",
        pares.len()
    );
    for (chip, row) in pares {
        assert!(
            row.y + row.h <= chip.y + 0.5,
            "um menu da fila embaixo abriu por baixo do chip ({row:?} sob {chip:?})"
        );
    }
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
    assert!(pares.len() >= 2);
    for (chip, row) in pares {
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
