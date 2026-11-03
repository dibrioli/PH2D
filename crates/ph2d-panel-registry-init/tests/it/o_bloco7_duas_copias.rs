//! ⭐ **T54 — duas cópias soltas da mesma secção** (`docs/CartoesUI/01_decisoes_do_dono.md`): com
//! o Transform preso (com corda) ao Circle, escolher o Square e soltar o Transform de novo cria a
//! cópia 2 — a 1 fica presa ao Circle com a corda, a 2 prende-se ao Square, cada uma com os seus
//! números e o gesto em cada uma edita o objecto dela.
#![cfg(feature = "panel-inspector")]

use super::cartoes_mao::{arrasta, clica, mao, pinta, solta};
use super::o_bloco5_abertos::escolhe;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{WidgetEvent, loose_instance_of, loose_pin_of, loose_rect_of};
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_host::PointerKind;

const CIRCULO: u64 = 11;
const QUADRADO: u64 = 12;

fn publica(h: &HeroScreen) {
    let pins = h.store.loose_pins_of(ph2d_editor_core::ids::INSP_PANEL);
    ph2d_panel_inspector::com_cada_preso(&pins, |b| match b {
        CIRCULO => escolhe(CIRCULO, "Circle", 1.0),
        QUADRADO => escolhe(QUADRADO, "Square", 9.0),
        _ => {}
    });
}

fn quadros(h: &mut HeroScreen, n: usize) {
    for _ in 0..n {
        publica(h);
        pinta(h, 1);
        ph2d_card_rope::anda_com(
            &mut h.store,
            ph2d_editor_core::ids::INSP_PANEL,
            |_| Some((800.0, 600.0)),
            1.0 / 60.0,
        );
    }
}

fn pos_x(h: &HeroScreen, k: NodeId) -> Option<f64> {
    let x = ph2d_panel_inspector::ids::INSP_TRANSFORM_POS_X;
    match h.store.pinned_states(k).and_then(|e| e.estado(x)) {
        Some(ph2d_editor_core::interaction::InteractiveState::NumberInput { value, .. }) => {
            Some(*value)
        }
        // Sem diferença guardada o preso É a seleção: a caixa mostra o número da lista (T61).
        _ => h.store.number_value(x),
    }
}

/// O objecto que o gesto no solto `k` edita (pressão no fundo do cartão + `ValueChanged`).
fn edita(h: &mut HeroScreen, k: NodeId) -> Option<u64> {
    let r = h
        .store
        .panel_rect(loose_rect_of(k))
        .expect("o solto está no canvas");
    let dentro = (r.x + r.w * 0.5, r.y + r.h - 3.0);
    mao(h, PointerKind::Down, dentro);
    mao(h, PointerKind::Up, dentro);
    let _ = h.bus.drain().count();
    h.apply_event(WidgetEvent::ValueChanged(
        ph2d_panel_inspector::ids::INSP_TRANSFORM_POS_X,
    ));
    h.bus.drain().find_map(|a| match a {
        EditorAction::InspectorTransformEdit(i) => Some(i.entity_bits),
        _ => None,
    })
}

/// *Mutação: `detach_card` chaveado pela secção (`free_loose_instance` → `section`) ⇒ a cópia 1 é
/// reescrita: um só solto, sem alfinete.*
#[test]
fn a_second_transform_card_pins_to_another_object_and_the_first_keeps_its_rope() {
    use ph2d_panel_inspector::CARD_TRANSFORM;
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    ph2d_panel_inspector::set_current_inspector_sprite(None);
    escolhe(CIRCULO, "Circle", 1.0);
    let mut h = HeroScreen::new(NodeId(1));
    let ids: Vec<&'static str> = ph2d_editor_core::panel::with_registry_ref(|reg| {
        reg.panels().iter().map(|p| p.manifest.id).collect()
    });
    for n in ids {
        h.panel_visibility.insert(n, n == "inspector");
    }
    pinta(&mut h, 30);
    let pega = h
        .hit_index
        .rect_for(ph2d_editor_core::ids::grip_de(CARD_TRANSFORM))
        .expect("o título do Transform na lista");
    solta(&mut h, CARD_TRANSFORM);
    clica(&mut h, loose_pin_of(CARD_TRANSFORM));
    clica(&mut h, loose_pin_of(CARD_TRANSFORM));
    let um = h.store.loose_card(CARD_TRANSFORM).map(|c| (c.pin, c.rope));
    assert_eq!(
        um,
        Some((Some(CIRCULO), true)),
        "a cópia 1 não ficou presa com corda"
    );
    escolhe(QUADRADO, "Square", 9.0);
    quadros(&mut h, 10);
    assert!(
        !h.store.loose_rope(CARD_TRANSFORM).is_empty(),
        "a corda da cópia 1 não nasceu"
    );
    let canvas = h.last_layout.expect("houve um quadro").canvas;
    arrasta(
        &mut h,
        (pega.x + 12.0, pega.y + pega.h * 0.5),
        (canvas.x + canvas.w * 0.6, canvas.y + canvas.h * 0.6),
    );
    quadros(&mut h, 5);
    let dois = loose_instance_of(CARD_TRANSFORM, 1);
    let soltos = h.store.loose_cards_of(ph2d_editor_core::ids::INSP_PANEL);
    assert_eq!(soltos.len(), 2, "não há duas cópias: {soltos:?}");
    let um = h.store.loose_card(CARD_TRANSFORM).map(|c| (c.pin, c.rope));
    assert_eq!(
        um,
        Some((Some(CIRCULO), true)),
        "a cópia 1 perdeu o alfinete ou a corda"
    );
    assert!(
        !h.store.loose_rope(CARD_TRANSFORM).is_empty(),
        "a corda da cópia 1 partiu-se"
    );
    assert_eq!(
        h.store.loose_card(dois).map(|c| c.section),
        Some(CARD_TRANSFORM)
    );
    clica(&mut h, loose_pin_of(dois));
    assert_eq!(h.store.loose_card(dois).and_then(|c| c.pin), Some(QUADRADO));
    quadros(&mut h, 5);
    let (a, b) = (pos_x(&h, CARD_TRANSFORM), pos_x(&h, dois));
    assert!(
        a.is_some() && b.is_some() && a != b,
        "os números não são de cada um: {a:?} × {b:?}"
    );
    escolhe(77, "Other", 4.0);
    quadros(&mut h, 3);
    assert_eq!(edita(&mut h, CARD_TRANSFORM), Some(CIRCULO));
    assert_eq!(edita(&mut h, dois), Some(QUADRADO));
    assert!(
        !h.store.loose_rope(CARD_TRANSFORM).is_empty(),
        "a corda da cópia 1 partiu-se"
    );
}
