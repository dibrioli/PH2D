//! ⭐ **Smoke do dono 05/11 no cartão da Sprite** ([ficha](../../../../docs/GameEngineFuncional/componentes/01_sprite.md)):
//! a roda real rola o Inspector, e o Transform dobrado da imagem é um Transform como os outros (alfinete, corda, um objecto).
#![cfg(all(feature = "panel-inspector", feature = "panel-hierarchy"))]

use super::cartoes_mao::{arrasta, centro, clica, mao, pinta, solta};
use super::o_alfinete_da_sprite::{A, B, arma};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::{loose_pin_of, loose_rect_of};
use ph2d_editor_core::screens::hero::{HeroScreen, InspectorTransformInfo};
use ph2d_host::PointerKind;
use ph2d_panel_inspector as insp;

const IMAGEM: NodeId = insp::CARD_TRANSFORM_IMAGE;

fn y_de(b: u64) -> f32 {
    if b == A { 2.0 } else { 5.0 }
}

fn publica(b: u64) {
    arma(b);
    insp::set_current_inspector_transform(Some(InspectorTransformInfo {
        entity_bits: b,
        translation: [b as f32, y_de(b)],
        rotation_rad: 0.0,
        scale: [1.0, 1.0],
        skew_rad: [0.0, 0.0],
    }));
}

fn quadros(h: &mut HeroScreen, sel: u64, n: usize) {
    for _ in 0..n {
        insp::com_cada_preso(&h.store.loose_pins_of(core_ids::INSP_PANEL), publica);
        publica(sel);
        pinta(h, 1);
    }
}

fn hero(sel: u64) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    insp::set_cards_on(true);
    publica(sel);
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    h.gizmo.replace_selection(Some(sel));
    h
}

/// A roda real sobre o painel, pela mesma porta da shell.
fn roda(h: &mut HeroScreen, (x, y): (f32, f32), delta_y: f32) {
    let arena = bumpalo::Bump::new();
    let _ = h.handle_wheel(
        ph2d_host::WheelEvent {
            x,
            y,
            delta_x: 0.0,
            delta_y,
            modifiers: Default::default(),
            timestamp_ns: 0,
        },
        &arena,
    );
}

/// ⭐ Dono 05/11: «o scroll dos painéis não funciona rodando a roda do mouse».
#[test]
fn a_roda_sobre_o_inspector_da_sprite_rola_e_a_barra_acompanha() {
    let mut h = hero(A);
    for c in [
        core_ids::INSP_LIVE_RENDER_SECTION,
        core_ids::INSP_LIVE_COLOR_SECTION,
        core_ids::INSP_LIVE_SLICE_SECTION,
        core_ids::INSP_LIVE_SAMPLING_SECTION,
        IMAGEM,
        insp::CARD_ORDERING,
    ] {
        h.store.set_collapsed(c, false);
    }
    quadros(&mut h, A, 30);
    let p = core_ids::INSP_PANEL;
    let r = h.store.panel_rect(p).expect("o Inspector publicou o rect");
    let (c, v) = (h.store.panel_content_h(p), h.store.panel_visible_h(p));
    assert!(
        c.zip(v).is_some_and(|(c, v)| c > v),
        "o conteúdo tem de transbordar: {c:?} × {v:?}"
    );
    let centro_do_painel = centro(r);
    assert_eq!(
        h.chrome_panel_at(centro_do_painel.0, centro_do_painel.1),
        Some(p),
        "a shell daria zoom no canvas"
    );
    let antes: Vec<_> = h
        .hit_index
        .iter_registrations()
        .map(|(i, q)| (i, q.y))
        .collect();
    // Um entalhe: `LineDelta(0, -1)` × 16 na shell.
    for _ in 0..5 {
        roda(&mut h, centro_do_painel, -16.0);
        quadros(&mut h, A, 1);
    }
    quadros(&mut h, A, 30);
    let rolou = h.store.panel_scroll(p);
    assert!(rolou > 70.0, "cinco entalhes rolaram {rolou} px");
    let andou = h
        .hit_index
        .iter_registrations()
        .any(|(i, q)| antes.iter().any(|(a, y)| *a == i && *y > q.y + 50.0));
    assert!(andou, "o número da rolagem mudou e o conteúdo não andou");
}

/// O Transform dobrado da imagem solto, preso a `A`, com `B` selecionado.
fn preso() -> HeroScreen {
    let mut h = hero(A);
    h.store
        .set_card_subject(core_ids::INSP_PANEL, Some((A, "Hero".to_owned())));
    for c in [
        core_ids::INSP_LIVE_RENDER_SECTION,
        core_ids::INSP_LIVE_COLOR_SECTION,
        insp::CARD_ORDERING,
    ] {
        h.store.set_collapsed(c, true);
    }
    h.store.set_collapsed(IMAGEM, false);
    quadros(&mut h, A, 30);
    solta(&mut h, IMAGEM);
    clica(&mut h, loose_pin_of(IMAGEM));
    clica(&mut h, loose_pin_of(IMAGEM));
    assert_eq!(
        h.store.loose_card(IMAGEM).map(|c| (c.pin, c.rope)),
        Some((Some(A), true)),
        "o alfinete não prendeu com corda"
    );
    h.gizmo.replace_selection(Some(B));
    for _ in 0..10 {
        quadros(&mut h, B, 1);
        ph2d_card_rope::anda_com(
            &mut h.store,
            core_ids::INSP_PANEL,
            |_| Some((800.0, 600.0)),
            1.0 / 60.0,
        );
    }
    h
}

/// ⭐ Dono 05/11: «o transform da imagem não apareceu o pin». *Mutação: tirá-lo de `DO_OBJECTO` ⇒ o alfinete não pinta.*
#[test]
fn o_transform_da_imagem_tem_alfinete_corda_e_nome() {
    let h = preso();
    assert!(h.hit_index.rect_for(loose_pin_of(IMAGEM)).is_some());
    assert!(!h.store.loose_rope(IMAGEM).is_empty(), "a corda não nasceu");
    assert_eq!(h.store.card_subject_name(A).as_deref(), Some("Hero"));
}

/// Com outro objecto selecionado, arrastar o X publica só para o preso, com o Y dele.
#[test]
fn o_transform_da_imagem_preso_escreve_so_no_objecto_dele() {
    let mut h = preso();
    let s = h.store.panel_rect(loose_rect_of(IMAGEM)).expect("o solto");
    let r = h
        .hit_index
        .iter_registrations()
        .filter(|(i, _)| *i == insp::ids::INSP_TRANSFORM_POS_X)
        .map(|(_, r)| r)
        .find(|r| s.contains(r.x + 1.0, r.y + 1.0))
        .expect("o campo X dentro do solto");
    let fundo = (s.x + s.w * 0.5, s.y + s.h - 3.0);
    mao(&mut h, PointerKind::Down, fundo);
    mao(&mut h, PointerKind::Up, fundo);
    let _ = h.bus.drain().count();
    let (xa, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
    arrasta(&mut h, (xa, y), (xa + 40.0, y));
    let edicoes: Vec<_> = h
        .bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::InspectorTransformEdit(i) => Some(i),
            _ => None,
        })
        .collect();
    assert!(!edicoes.is_empty(), "o arrasto não publicou");
    for i in edicoes {
        assert_eq!(i.entity_bits, A, "foi a outro objecto: {i:?}");
        assert_eq!(i.translation[1], y_de(A), "o Y não é o do preso: {i:?}");
    }
}

#[test]
fn a_roda_rola_o_inspector_com_um_cartao_preso_no_canvas() {
    let mut h = preso();
    for c in [
        core_ids::INSP_LIVE_RENDER_SECTION,
        core_ids::INSP_LIVE_COLOR_SECTION,
        core_ids::INSP_LIVE_SLICE_SECTION,
        core_ids::INSP_LIVE_SAMPLING_SECTION,
        insp::CARD_ORDERING,
    ] {
        h.store.set_collapsed(c, false);
    }
    quadros(&mut h, B, 30);
    let p = core_ids::INSP_PANEL;
    let r = h.store.panel_rect(p).expect("o rect");
    let (c, v) = (h.store.panel_content_h(p), h.store.panel_visible_h(p));
    assert!(c.zip(v).is_some_and(|(c, v)| c > v), "{c:?} × {v:?}");
    for _ in 0..5 {
        roda(&mut h, centro(r), -16.0);
        quadros(&mut h, B, 1);
    }
    quadros(&mut h, B, 30);
    assert!(
        h.store.panel_scroll(p) > 70.0,
        "{}",
        h.store.panel_scroll(p)
    );
}

/// Abre tudo o que o Inspector da sprite tem, para a lista transbordar; devolve o centro dele.
fn inspector_que_transborda(h: &mut HeroScreen) -> (f32, f32) {
    for c in [
        core_ids::INSP_LIVE_RENDER_SECTION,
        core_ids::INSP_LIVE_COLOR_SECTION,
        core_ids::INSP_LIVE_SLICE_SECTION,
        core_ids::INSP_LIVE_SAMPLING_SECTION,
        IMAGEM,
        insp::CARD_ORDERING,
    ] {
        h.store.set_collapsed(c, false);
    }
    quadros(h, A, 30);
    let r = h.store.panel_rect(core_ids::INSP_PANEL).expect("o rect");
    centro(r)
}

fn cinco_entalhes(h: &mut HeroScreen, p: (f32, f32)) -> f32 {
    for _ in 0..5 {
        roda(h, p, -16.0);
        quadros(h, A, 1);
    }
    quadros(h, A, 30);
    h.store.panel_scroll(core_ids::INSP_PANEL)
}

/// ⭐⭐ **A causa da roda morta** (dono 05/11): a superfície do grafo de um layout anterior (mais largo, docas mais
/// estreitas) ficava no store, e `dispatch_wheel` perguntava por ela ANTES de perguntar quem está por cima: a coluna da
/// doca que caía nela dava zoom num grafo invisível. *Mutação: tirar `dona_do_ponto` de `dispatch_wheel`.*
#[test]
fn a_roda_e_do_painel_por_cima_de_uma_superficie_de_grafo() {
    let mut h = hero(A);
    let p = inspector_que_transborda(&mut h);
    h.store.set_graph_canvas(
        core_ids::MOTION_GRAPH_PANEL,
        ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 900.0),
    );
    let rolou = cinco_entalhes(&mut h, p);
    assert!(
        rolou > 70.0,
        "rolou {rolou} px: a roda foi ao grafo debaixo"
    );
}

/// O grafo aberto continua a dar zoom debaixo da mão, e fechado larga a superfície.
/// *Mutação: tirar os `remove_graph_canvas` do Motion Graph ⇒ a superfície fica.*
#[test]
fn o_grafo_aberto_da_zoom_e_fechado_larga_a_superficie() {
    use ph2d_editor_core::screens::layout::CenterSplit;
    let mut h = hero(A);
    h.panel_visibility.insert("motion_graph", true);
    h.view.center_split = CenterSplit::Vertical { t: 0.5 };
    let p = inspector_que_transborda(&mut h);
    let g = core_ids::MOTION_GRAPH_PANEL;
    let dentro = centro(h.store.panel_rect(g).expect("o grafo abriu"));
    assert_eq!(h.store.graph_surface_at(dentro.0, dentro.1), Some(g));
    let _ = h.store.take_graph_zoom(g);
    roda(&mut h, dentro, -16.0);
    assert!(
        h.store.take_graph_zoom(g).is_some(),
        "o grafo não recebeu a roda"
    );
    h.panel_visibility.insert("motion_graph", false);
    h.view.center_split = CenterSplit::None;
    quadros(&mut h, A, 5);
    assert!(
        h.store.graph_surface_at(dentro.0, dentro.1).is_none(),
        "o grafo fechado guarda a superfície"
    );
    assert!(cinco_entalhes(&mut h, p) > 70.0);
}
