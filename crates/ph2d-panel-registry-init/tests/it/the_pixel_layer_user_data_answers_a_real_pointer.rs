//! ⭐ **O cartão User Data da camada responde a um PONTEIRO real** (P12 U, O17): no painel PIXEL, o
//! botão `User Data` ao lado do nome da camada abre o cartão (nasce fechado); um valor escrito com
//! `Enter` e o `+ Property` chegam ao barramento como `PixelRequest::UserData` da camada activa.

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::cards::user_data;
use ph2d_editor_core::pixel_panel::user_data::{CardEdit, PropRow, UserDataView};
use ph2d_editor_core::pixel_panel::{
    PixelLayerRow, PixelOwner, PixelPanelInfo, PixelRequest, PixelUserInfo,
};
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{
    KeyEvent, KeyKind, Modifiers, PointerButton, PointerEvent, PointerKind, PointerSource,
};
use ph2d_panel_pixel_tools::seam_user;

const KEY_ENTER: u32 = 0x0D;

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
    assert!(ev.contains(&WidgetEvent::Click(id)), "{nome}: {ev:?}");
    for e in ev {
        hero.apply_event(e);
    }
}

fn escreve(hero: &mut HeroScreen, id: NodeId, t: &str) {
    let mut ev = aperta(hero, id, "o campo");
    assert_eq!(
        hero.store.focus_id(),
        Some(id),
        "o clique não focou o campo"
    );
    let arena = Bump::new();
    for c in t.chars() {
        ev.extend_from_slice(hero.handle_text_input(c, &arena));
    }
    ev.extend_from_slice(hero.handle_key(
        KeyEvent {
            keycode: KEY_ENTER,
            modifiers: Modifiers::default(),
            kind: KeyKind::Down,
            timestamp_ns: 0,
        },
        &arena,
    ));
    for e in ev {
        hero.apply_event(e);
    }
}

fn pedidos(hero: &mut HeroScreen) -> Vec<PixelRequest> {
    hero.bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::Pixel(r @ PixelRequest::UserData(..)) => Some(r),
            _ => None,
        })
        .collect()
}

#[test]
fn the_pixel_layer_user_data_card_answers_a_real_pointer() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let ids = seam_user::ids();
    user_data::set_open(&ids, false);
    ph2d_panel_pixel_tools::set_current_pixel(PixelPanelInfo {
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
            reference: false,
            background: false,
            opacity: 255,
            blend: Default::default(),
        }],
        active_layer: Some(4),
        user: PixelUserInfo {
            layer: Some(UserDataView {
                rows: vec![PropRow {
                    name: "hp".to_owned(),
                    value: "3".to_owned(),
                    kind: "pixel.user.type.int",
                    editable: true,
                }],
                ..UserDataView::default()
            }),
            ..PixelUserInfo::default()
        },
        ..PixelPanelInfo::default()
    });
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(PANEL_PIXEL, true);
    pinta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(ids.value(0)).is_none(),
        "nasce fechado"
    );
    clica(&mut hero, ids.toggle, "User Data");
    pinta(&mut hero, &mut ts);
    let _ = hero.bus.drain().count();
    escreve(&mut hero, ids.value(0), "7");
    assert_eq!(
        pedidos(&mut hero),
        [PixelRequest::UserData(
            PixelOwner::Layer(4),
            CardEdit::Value("hp".into(), "37".into())
        )]
    );
    pinta(&mut hero, &mut ts);
    clica(&mut hero, ids.add, "+ Property");
    assert_eq!(
        pedidos(&mut hero),
        [PixelRequest::UserData(PixelOwner::Layer(4), CardEdit::Add)]
    );
}
