//! ⭐ **Sprite ▸ Properties… responde a um PONTEIRO real** (P12 U, O17): no painel PIXEL, o botão
//! `Properties…` do menu Sprite pede `PixelSpriteAction::Properties`; aberta a secção, o botão
//! `User Data` abre o cartão do sprite e o texto escrito com `Enter` chega ao barramento como
//! `PixelRequest::UserData(PixelOwner::Sprite, _)`.

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::cards::user_data;
use ph2d_editor_core::pixel_panel::user_data::{CardEdit, UserDataView};
use ph2d_editor_core::pixel_panel::{
    PixelOwner, PixelPanelInfo, PixelRequest, PixelSpriteAction, PixelUserInfo, sprite_props,
};
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId};
use ph2d_host::{
    KeyEvent, KeyKind, Modifiers, PointerButton, PointerEvent, PointerKind, PointerSource,
};
use ph2d_panel_pixel_tools::{ids, seam_user};

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
    for _ in 0..3 {
        let mut scene = ph2d_vector::VectorScene::new();
        paint_hero_screen(hero, vp, &mut scene, ts);
    }
}

fn clica(hero: &mut HeroScreen, id: NodeId, nome: &str) {
    let ev = aperta(hero, id, nome);
    assert!(ev.contains(&WidgetEvent::Click(id)), "{nome}: {ev:?}");
    for e in ev {
        hero.apply_event(e);
    }
}

fn escreve(hero: &mut HeroScreen, id: NodeId, t: &str) {
    let mut ev = aperta(hero, id, "o texto");
    assert_eq!(
        hero.store.focus_id(),
        Some(id),
        "o clique não focou o texto"
    );
    let arena = Bump::new();
    for c in t.chars() {
        ev.extend_from_slice(hero.handle_text_input(c, &arena));
    }
    ev.extend_from_slice(hero.handle_key(
        KeyEvent {
            keycode: 0x0D,
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

fn pedidos(hero: &mut HeroScreen) -> Vec<PixelRequest> {
    hero.bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::Pixel(
                r @ (PixelRequest::UserData(..)
                | PixelRequest::Sprite(PixelSpriteAction::Properties)),
            ) => Some(r),
            _ => None,
        })
        .collect()
}

#[test]
fn the_pixel_sprite_properties_answers_a_real_pointer() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let ud = seam_user::ids_sprite();
    sprite_props::set_open(false);
    user_data::set_open(&ud, false);
    ph2d_panel_pixel_tools::set_current_pixel(PixelPanelInfo {
        editing: true,
        size: (32, 16),
        user: PixelUserInfo {
            sprite: Some(UserDataView::default()),
            ..PixelUserInfo::default()
        },
        ..PixelPanelInfo::default()
    });
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(PANEL_PIXEL, true);
    // Os outros cartões dobrados: o Sprite fica acima da dobra para a mão o apanhar.
    for c in [
        ids::PIXEL_SEC_OPTIONS,
        ids::PIXEL_SEC_BRUSHES,
        ids::PIXEL_SEC_TRANSFORM,
        ids::PIXEL_SEC_COLORS,
        ids::PIXEL_SEC_LAYERS,
        ids::PIXEL_SEC_PALETTE,
        ids::PIXEL_SEC_PICKER,
        ids::PIXEL_SEC_SYMMETRY,
        ids::PIXEL_SEC_SELECT,
        ids::PIXEL_SEC_EDIT,
        ids::PIXEL_SEC_SPRITE,
    ] {
        hero.store.set_collapsed(c, c != ids::PIXEL_SEC_SPRITE);
    }
    pinta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(ud.toggle).is_none(),
        "nasce fechada"
    );
    let _ = hero.bus.drain().count();
    let [.., (props, _, _)] = ids::SPRITE_ACTIONS;
    clica(&mut hero, props, "Properties…");
    assert_eq!(
        pedidos(&mut hero),
        [PixelRequest::Sprite(PixelSpriteAction::Properties)]
    );
    // A resposta da família (pixel-scenes `sprite_props` prova-a): a secção abre.
    sprite_props::set_open(true);
    pinta(&mut hero, &mut ts);
    assert!(
        hero.hit_index.rect_for(ud.text).is_none(),
        "o cartão nasce fechado"
    );
    clica(&mut hero, ud.toggle, "User Data");
    pinta(&mut hero, &mut ts);
    let _ = hero.bus.drain().count();
    escreve(&mut hero, ud.text, "hero");
    assert_eq!(
        pedidos(&mut hero),
        [PixelRequest::UserData(
            PixelOwner::Sprite,
            CardEdit::Text("hero".into())
        )]
    );
    sprite_props::set_open(false);
}
