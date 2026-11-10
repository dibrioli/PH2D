//! ⭐ **Layer Properties ▸ Normal Map responde a um PONTEIRO real** (P14 §6): no painel PIXEL, a
//! caixa `Normal Map` das propriedades da camada activa e o ícone da linha (só pintado na camada
//! Normal Map) chegam ao barramento como `PixelRequest::LayerNormal` dessa camada.

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::pixel_layer_row::PixelLayerWidget;
use ph2d_editor_core::pixel_panel::{PixelLayerRow, PixelPanelInfo, PixelRequest};
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};

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
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    // Passo fixo: a mola do cartão que abre assenta em quadros, não no relógio da máquina — sem ele o teste só
    // passava quando outro do mesmo processo já tinha deixado o relógio andar (sozinho, o campo não estava no sítio).
    for _ in 0..30 {
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || {
                let mut scene = ph2d_vector::VectorScene::new();
                paint_hero_screen(hero, vp, &mut scene, ts);
            },
        );
    }
}

fn aperta(hero: &mut HeroScreen, id: NodeId, nome: &str) -> Vec<WidgetEvent> {
    let r = hero
        .hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{nome} não foi pintado"));
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let arena = Bump::new();
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    ev
}

fn clica(hero: &mut HeroScreen, id: NodeId, nome: &str) {
    let ev = aperta(hero, id, nome);
    assert!(
        ev.contains(&WidgetEvent::Click(id)) || ev.contains(&WidgetEvent::Toggled(id)),
        "{nome}: {ev:?}"
    );
    for e in ev {
        hero.apply_event(e);
    }
}

fn pedidos(hero: &mut HeroScreen) -> Vec<PixelRequest> {
    hero.bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::Pixel(r @ PixelRequest::LayerNormal(..)) => Some(r),
            _ => None,
        })
        .collect()
}

fn info(normal: bool) -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        layers: vec![PixelLayerRow {
            id: 4,
            name: "Ink".to_owned(),
            depth: 0,
            group: false,
            collapsed: false,
            visible: true,
            locked: false,
            continuous: false,
            normal,
            reference: false,
            background: false,
            opacity: 255,
            blend: Default::default(),
            outline: None,
        }],
        active_layer: Some(4),
        ..PixelPanelInfo::default()
    }
}

#[test]
fn the_pixel_layer_normal_map_answers_a_real_pointer() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let caixa = ph2d_panel_pixel_tools::ids::LAYER_NORMAL;
    let icone = ph2d_panel_pixel_tools::ids::layer_widget_id(4, PixelLayerWidget::Normal);
    ph2d_panel_pixel_tools::set_current_pixel(info(false));
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(PANEL_PIXEL, true);
    pinta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(icone).is_none(),
        "sem Normal Map, a linha não tem o ícone"
    );
    let _ = hero.bus.drain().count();
    clica(&mut hero, caixa, "a caixa Normal Map");
    assert_eq!(pedidos(&mut hero), [PixelRequest::LayerNormal(4)]);
    ph2d_panel_pixel_tools::set_current_pixel(info(true));
    pinta(&mut hero, &mut ts);
    let _ = hero.bus.drain().count();
    clica(&mut hero, icone, "o ícone Normal Map da linha");
    assert_eq!(pedidos(&mut hero), [PixelRequest::LayerNormal(4)]);
}
