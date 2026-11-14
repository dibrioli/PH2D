//! ⭐ **O que a camada Normal Map recusa fica DESLIGADO à vista e diz porquê ao ponteiro real**
//! (P14c-pixel 1b): o Merge Down entre normal e cor e a caixa Normal Map fora de um sprite RGB. O
//! clique não chega ao barramento; o ponteiro em cima acende a dica (hot = o id, dica = a chave).
//! Ligados de novo, o mesmo clique chega (o controlo do teste).

use bumpalo::Bump;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::pixel_panel::{
    NewSpriteMode, PixelLayerAction, PixelLayerRow, PixelPanelInfo, PixelRequest,
};
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL, paint_hero_screen,
};
use ph2d_editor_core::widget::{ButtonState, CheckboxState};
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
    // Duas alturas de fábrica: a lei é a recusa sob o ponteiro, não a dobra. Com a faixa 36 e a fila
    // 32 da barra de menus (onda 7) o Merge Down passou para baixo da dobra da coluna (sem rect = sem
    // alvo); o artista rola até ele, o teste dá-lhe a coluna inteira.
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H * 2.0);
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

fn centro(hero: &HeroScreen, id: NodeId, nome: &str) -> (f32, f32) {
    let r = hero
        .hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{nome} não foi pintado"));
    (r.x + r.w * 0.5, r.y + r.h * 0.5)
}

/// O ponteiro pousa (Move) e a dica que o painel de dicas lê é a do id sob ele.
fn dica_no_ponteiro(hero: &mut HeroScreen, id: NodeId, nome: &str) -> Option<String> {
    let (x, y) = centro(hero, id, nome);
    let arena = Bump::new();
    for e in hero
        .handle_pointer(ptr(PointerKind::Move, x, y), &arena)
        .to_vec()
    {
        hero.apply_event(e);
    }
    assert_eq!(
        hero.store.hot_id(),
        Some(id),
        "{nome}: o ponteiro não o achou"
    );
    hero.store.tooltip_for(id).map(str::to_owned)
}

fn clica(hero: &mut HeroScreen, id: NodeId, nome: &str) -> Vec<PixelRequest> {
    let _ = hero.bus.drain().count();
    let (x, y) = centro(hero, id, nome);
    let arena = Bump::new();
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    for e in ev {
        hero.apply_event(e);
    }
    hero.bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::Pixel(
                r @ (PixelRequest::LayerNormal(..)
                | PixelRequest::LayerAction(PixelLayerAction::MergeDown)),
            ) => Some(r),
            _ => None,
        })
        .collect()
}

fn linha(id: u32, name: &str, normal: bool) -> PixelLayerRow {
    PixelLayerRow {
        id,
        name: name.to_owned(),
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
    }
}

fn info(mode: NewSpriteMode, mistura: bool) -> PixelPanelInfo {
    PixelPanelInfo {
        editing: true,
        mode,
        layers: vec![linha(5, "Normals", mistura), linha(4, "Ink", false)],
        active_layer: Some(5),
        merge_down_mixes_normal: mistura,
        ..PixelPanelInfo::default()
    }
}

fn merge_id() -> NodeId {
    ph2d_panel_pixel_tools::ids::LAYER_ACTIONS
        .iter()
        .find(|(_, a, _)| *a == PixelLayerAction::MergeDown)
        .map(|(id, ..)| *id)
        .expect("Merge Down")
}

#[test]
fn the_pixel_normal_map_refusals_answer_a_real_pointer() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let (merge, caixa) = (merge_id(), ph2d_panel_pixel_tools::ids::LAYER_NORMAL);
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(PANEL_PIXEL, true);

    // Normal Map sobre cor: o Merge Down está desligado, diz porquê e não pede nada.
    ph2d_panel_pixel_tools::set_current_pixel(info(NewSpriteMode::Rgba, true));
    pinta(&mut hero, &mut ts);
    assert_eq!(hero.store.button_state(merge), Some(ButtonState::Disabled));
    assert_eq!(
        dica_no_ponteiro(&mut hero, merge, "Merge Down").as_deref(),
        Some(ph2d_i18n::tr("panel.pixel.layer.merge_normal_tip"))
    );
    assert_eq!(clica(&mut hero, merge, "Merge Down"), []);

    // Sprite Indexed: a caixa Normal Map está desligada, diz porquê e não pede nada.
    ph2d_panel_pixel_tools::set_current_pixel(info(NewSpriteMode::Indexed, false));
    pinta(&mut hero, &mut ts);
    assert_eq!(hero.store.checkbox_visual(caixa).0, CheckboxState::Disabled);
    assert_eq!(
        dica_no_ponteiro(&mut hero, caixa, "a caixa Normal Map").as_deref(),
        Some(ph2d_i18n::tr("panel.pixel.layer.normal_needs_rgb"))
    );
    assert_eq!(clica(&mut hero, caixa, "a caixa Normal Map"), []);
    // ... e o Merge Down de duas camadas de cor voltou a ligar-se.
    assert_eq!(
        clica(&mut hero, merge, "Merge Down"),
        [PixelRequest::LayerAction(PixelLayerAction::MergeDown)]
    );

    // RGB: a mesma caixa liga-se e o clique chega.
    ph2d_panel_pixel_tools::set_current_pixel(info(NewSpriteMode::Rgba, false));
    pinta(&mut hero, &mut ts);
    assert_ne!(hero.store.checkbox_visual(caixa).0, CheckboxState::Disabled);
    assert_eq!(
        clica(&mut hero, caixa, "a caixa Normal Map"),
        [PixelRequest::LayerNormal(5)]
    );
}
