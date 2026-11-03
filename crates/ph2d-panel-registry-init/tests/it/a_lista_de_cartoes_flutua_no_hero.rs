//! ⭐⭐ **D24 (CartoesUI, 01/11): a lista de cartões FLUTUA sobre o canvas** — no hero montado, com
//! todos os painéis registados (o `MockPanelHost` não tem layout nem canvas). O dono: *«retirar esse
//! fundo, deixar o canvas atravessar por trás; colocar o ponto onde alargar o painel na nova
//! beirada»*. E junto, o report do cartão Tool que «fechou e não reabria»: o título tem de ser da
//! lista (a shell só entrega ao canvas o que `chrome_panel_at` não reclama) e o clique alterna.
#![cfg(feature = "panel-skeleton")]

use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::screens::layout::DockSide;
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind};
use ph2d_panel_skeleton::cards::CARD_TOOL;
use ph2d_text::TextSystem;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn bones() -> ph2d_editor_core::NodeId {
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels()
            .iter()
            .find(|p| p.manifest.id == "skeleton")
            .map(|p| p.manifest.panel_node_id)
            .expect("o Bones está registado")
    })
}

/// Pinta até a mola assentar (os cartões medem-se ao pintar).
fn assenta(h: &mut HeroScreen) {
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..60 {
        scene.reset();
        ph2d_editor_core::screens::hero::paint_hero_screen(h, VIEWPORT, &mut scene, &mut text);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn clique(h: &mut HeroScreen, x: f32, y: f32) {
    let arena = bumpalo::Bump::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let ev = PointerEvent {
            x,
            y,
            pressure: 1.0,
            kind,
            source: ph2d_host::PointerSource::Mouse,
            button: PointerButton::Primary,
            timestamp_ns: 0,
        };
        let evs: Vec<_> =
            ph2d_editor_core::interaction::dispatch_pointer(&mut h.store, &h.hit_index, ev, &arena)
                .to_vec();
        for e in evs {
            h.apply_event(e);
        }
    }
}

fn hero_com_o_bones() -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_skeleton::state::set_current_bone(None);
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "skeleton");
    }
    assenta(&mut h);
    h
}

/// O canvas passa por trás da lista curta; abaixo dela o clique é do canvas; a costura de largura
/// vive na beirada da lista, só na altura dela, e não há alça de reabrir.
#[test]
fn the_card_list_floats_over_the_canvas_in_the_hero() {
    let h = hero_com_o_bones();
    let lista = h
        .store
        .panel_rect(bones())
        .expect("o Bones publicou o rect");
    let l = h.last_layout.expect("houve um quadro");
    let lado = if l.docks.float_right {
        DockSide::Right
    } else {
        assert!(
            l.docks.float_left,
            "a lista {lista:?} não flutua: {:?}",
            l.docks
        );
        DockSide::Left
    };
    assert!(
        !l.docks.left && !l.docks.right,
        "a coluna ficou reservada: {:?}",
        l.docks
    );
    assert!(
        l.draw_area.x <= lista.x && l.draw_area.x + l.draw_area.w >= lista.x + lista.w,
        "o canvas {:?} não passa por trás da lista {lista:?}",
        l.draw_area
    );
    assert_eq!(
        l.dock_reopen(lado).w,
        0.0,
        "alça de reabrir numa coluna com lista"
    );
    let costura = l.dock_seam(lado);
    assert_eq!(
        (costura.y, costura.h),
        (lista.y, lista.h),
        "a costura não é a beirada da lista"
    );
    assert!(
        l.dock_seam_at((costura.x + 1.0, lista.y + lista.h * 0.5))
            .is_some()
    );

    let abaixo = (lista.x + lista.w * 0.5, lista.y + lista.h + 40.0);
    assert!(
        abaixo.1 < VIEWPORT.h,
        "a lista curta encheu a coluna: {lista:?}"
    );
    assert_eq!(
        h.chrome_panel_at(abaixo.0, abaixo.1),
        None,
        "abaixo da lista o clique não é do canvas"
    );
}

/// ⭐ Report do dono (01/11): o cartão Tool fechou e não reabria. No hero, com o canvas por trás, o
/// título é da lista (a shell não o entrega ao canvas) e quatro cliques alternam a dobra.
#[test]
fn the_tool_card_folds_and_unfolds_in_the_hero() {
    let mut h = hero_com_o_bones();
    for vez in 0..4 {
        let grip = ph2d_editor_core::ids::grip_de(CARD_TOOL);
        let pega = h
            .hit_index
            .rect_for(grip)
            .unwrap_or_else(|| panic!("clique {vez}: o título do Tool não foi registado"));
        let p = (pega.x + 10.0, pega.y + pega.h * 0.5);
        assert_eq!(
            h.chrome_panel_at(p.0, p.1),
            Some(bones()),
            "clique {vez}: o título do Tool cai no canvas — a shell entregá-lo-ia à ferramenta"
        );
        let antes = h.store.is_collapsed(CARD_TOOL);
        clique(&mut h, p.0, p.1);
        assert_ne!(
            h.store.is_collapsed(CARD_TOOL),
            antes,
            "clique {vez}: o título do Tool não mudou a dobra (estava dobrado: {antes})"
        );
        assenta(&mut h);
    }
}

/// ⭐ C3 aberto («a lista ainda não rola»): numa janela baixa a lista do Bones passa do fundo; a
/// roda sobre ela rola os cartões (o título do Bone sobe — o do Tool, no topo, só se recorta) e a
/// lista publica a altura que tem.
#[test]
fn a_long_card_list_scrolls_under_the_wheel() {
    const BAIXA: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 500.0,
    };
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_skeleton::state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(
        20.0, 1.0,
    )));
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "skeleton");
    }
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let mut pinta = |h: &mut HeroScreen| {
        for _ in 0..40 {
            scene.reset();
            ph2d_editor_core::screens::hero::paint_hero_screen(h, BAIXA, &mut scene, &mut text);
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    };
    pinta(&mut h);
    let grip = ph2d_editor_core::ids::grip_de(ph2d_panel_skeleton::cards::CARD_BONE);
    let antes = h.hit_index.rect_for(grip).expect("o título do Bone").y;
    let lista = h
        .store
        .panel_rect(bones())
        .expect("o Bones publicou o rect");
    let (Some(conteudo), Some(visivel)) = (
        h.store.panel_content_h(bones()),
        h.store.panel_visible_h(bones()),
    ) else {
        panic!("a lista não publicou as alturas da rolagem");
    };
    assert!(
        conteudo > visivel,
        "a lista cabe na janela baixa: {conteudo} ≤ {visivel}"
    );
    let arena = bumpalo::Bump::new();
    let roda = ph2d_host::WheelEvent {
        x: lista.x + lista.w * 0.5,
        y: lista.y + lista.h * 0.5,
        delta_x: 0.0,
        // Roda para baixo = delta negativo (a convenção do `wheel_panel`).
        delta_y: -120.0,
        modifiers: Default::default(),
        timestamp_ns: 0,
    };
    let _ = ph2d_editor_core::interaction::dispatch_wheel(&mut h.store, roda, &arena);
    pinta(&mut h);
    assert!(
        h.store.panel_scroll(bones()) > 0.0,
        "a roda não rolou a lista"
    );
    let depois = h.hit_index.rect_for(grip).map(|r| r.y);
    assert!(
        depois.is_none_or(|y| y < antes),
        "o título do Bone não subiu: {antes} → {depois:?}"
    );
}

/// A barra de rolagem da lista tem a SUA faixa: com a lista a transbordar, nenhum cartão chega
/// por baixo dela (antes ela pintava sobre a beirada direita dos cartões).
#[test]
fn the_scroll_bar_has_its_own_lane_beside_the_cards() {
    const BAIXA: Rect = Rect {
        x: 0.0,
        y: 0.0,
        w: 1600.0,
        h: 500.0,
    };
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_skeleton::state::set_current_bone(Some(ph2d_skeleton::bend::BoneSpec::straight(
        20.0, 1.0,
    )));
    let mut h = HeroScreen::new(ph2d_editor_core::NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "skeleton");
    }
    let mut scene = ph2d_vector::VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    for _ in 0..40 {
        scene.reset();
        ph2d_editor_core::screens::hero::paint_hero_screen(&mut h, BAIXA, &mut scene, &mut text);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let barra = h
        .hit_index
        .rect_for(ph2d_editor_core::panel::cards::scroll_bar_of(bones()))
        .expect("a lista transborda e tem barra");
    for card in [CARD_TOOL, ph2d_panel_skeleton::cards::CARD_BONE] {
        let r = h.hit_index.rect_for(card).expect("o cabeçalho do cartão");
        assert!(
            r.x + r.w <= barra.x,
            "o cartão {card:?} ({r:?}) passa por baixo da barra ({barra:?})"
        );
    }
}
