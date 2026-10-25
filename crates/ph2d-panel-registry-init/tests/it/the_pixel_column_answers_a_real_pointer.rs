//! ⭐⭐⭐ **A coluna do Pixel Sprite ▸ Draw responde a um PONTEIRO real** (report do dono 25/10: as
//! ferramentas à esquerda, como no Painter, e o seletor de modo no topo).
//!
//! Com todos os painéis registados, a coluna que a família publica é pintada pelo ecrã; um Down+Up
//! sobre cada ferramenta tem de virar `Click` e chegar ao barramento como o pedido dela — com o
//! painel PIXEL FECHADO (a coluna não depende dele); cada amostra abre o seletor de cor; e a fila de
//! cima pinta o seletor de modo.

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{AreaMenu, ToolBars, WidgetEvent};
use ph2d_editor_core::pixel_panel::{PixelPanelInfo, PixelRequest, PixelTool};
use ph2d_editor_core::screens::hero::float_bar::tool_bars::placed;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_tools::{column, ids};

fn ptr(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

fn pinta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    let mut scene = ph2d_vector::VectorScene::new();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    paint_hero_screen(hero, vp, &mut scene, ts);
}

fn clica(hero: &mut HeroScreen, r: Rect) -> Vec<WidgetEvent> {
    let arena = Bump::new();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    ev
}

#[test]
fn every_tool_and_swatch_of_the_pixel_column_answers_a_real_pointer() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let info = PixelPanelInfo {
        editing: true,
        tool: PixelTool::Pencil,
        fg: [255, 255, 255, 255],
        bg: [0, 0, 0, 255],
        palette: vec![[1, 2, 3, 255]; 32],
        options: Default::default(),
        ..Default::default()
    };
    let mut hero = HeroScreen::new(NodeId(1));
    // O 1.º quadro arruma os encaixes; a coluna cai na área do 2.º (o mesmo que o teste do vetor).
    pinta(&mut hero, &mut ts);
    hero.panel_visibility
        .insert(ph2d_editor_core::screens::hero::PANEL_PIXEL, false);
    ph2d_panel_pixel_tools::set_current_pixel(info.clone());
    hero.store.publish_mode_menu(Some(AreaMenu {
        label: ph2d_i18n::tr("object_mode.menu").to_owned(),
        face: ph2d_i18n::tr("object_mode.draw").to_owned(),
        faces: Vec::new(),
        rows: Vec::new(),
    }));
    hero.store.set_tool_bars(Some(ToolBars {
        column: column::column(&info),
        hint: column::hint(&info),
        pixel_rail: true,
        ..ToolBars::default()
    }));
    pinta(&mut hero, &mut ts);
    assert!(
        hero.hit_index
            .rect_for(ph2d_editor_core::ids::area_menu_button(0))
            .is_some(),
        "o seletor de modo não foi pintado na fila de cima"
    );
    let bars = hero.store.tool_bars().cloned().expect("publicadas");
    let mut ferramentas = Vec::new();
    for (b, r) in placed(&bars, hero.last_content, false, false) {
        if hero.store.is_picker_swatch(b.id) {
            let _ = clica(&mut hero, r);
            assert_eq!(
                hero.store.picker_target(),
                Some(b.id),
                "a amostra «{}» não abriu a cor",
                b.label
            );
            hero.store.set_picker_target(None);
            continue;
        }
        let ev = clica(&mut hero, r);
        assert!(
            ev.iter()
                .any(|e| matches!(e, WidgetEvent::Click(i) if *i == b.id)),
            "o botão «{}» em {r:?} está pintado e o ponteiro não o clica ({ev:?}); quem responde: {:?}",
            b.label,
            ev.iter().find_map(|e| match e {
                WidgetEvent::Click(i) => Some((*i, hero.hit_index.rect_for(*i))),
                _ => None,
            })
        );
        let _ = hero.bus.drain().count();
        for e in ev {
            hero.apply_event(e);
        }
        let pedidos: Vec<_> = hero
            .bus
            .drain()
            .filter_map(|a| match a {
                EditorAction::Pixel(r) => Some(r),
                _ => None,
            })
            .collect();
        if let Some((t, _)) = ids::TOOLS.iter().find(|(_, id)| *id == b.id) {
            assert_eq!(pedidos, [PixelRequest::Tool(*t)], "«{}»", b.label);
            ferramentas.push(*t);
        } else {
            assert_eq!(b.id, ids::PIXEL_SWAP);
            assert_eq!(pedidos, [PixelRequest::Swap]);
        }
    }
    assert_eq!(
        ferramentas,
        PixelTool::ALL,
        "a coluna perdeu uma ferramenta"
    );
}
