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

const Y_CIRCULO: f32 = 2.0;
const Y_QUADRADO: f32 = 5.0;

fn com_y(bits: u64, x: f32, y: f32) {
    ph2d_panel_inspector::set_current_inspector_transform(Some(
        ph2d_editor_core::screens::hero::InspectorTransformInfo {
            entity_bits: bits,
            translation: [x, y],
            rotation_rad: 0.0,
            scale: [1.0, 1.0],
            skew_rad: [0.0, 0.0],
        },
    ));
}

fn publica(h: &HeroScreen) {
    let pins = h.store.loose_pins_of(ph2d_editor_core::ids::INSP_PANEL);
    ph2d_panel_inspector::com_cada_preso(&pins, |b| match b {
        CIRCULO => {
            escolhe(CIRCULO, "Circle", 1.0);
            com_y(CIRCULO, 1.0, Y_CIRCULO);
        }
        QUADRADO => {
            escolhe(QUADRADO, "Square", 9.0);
            com_y(QUADRADO, 9.0, Y_QUADRADO);
        }
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
    let (mut h, dois) = duas_copias();
    let a = ph2d_panel_inspector::CARD_TRANSFORM;
    assert_eq!(edita(&mut h, a), Some(CIRCULO));
    assert_eq!(edita(&mut h, dois), Some(QUADRADO));
    assert!(
        !h.store.loose_rope(a).is_empty(),
        "a corda da cópia 1 partiu-se"
    );
}

/// O que um arrasto no campo X do solto `k` publica.
fn arrasta_o_x(
    h: &mut HeroScreen,
    k: NodeId,
) -> ph2d_editor_core::screens::hero::InspectorTransformInfo {
    use ph2d_card_stack::metrics as m;
    let s = h.store.panel_rect(loose_rect_of(k)).expect("o solto");
    let r = h
        .hit_index
        .iter_registrations()
        .filter(|(i, _)| *i == ph2d_panel_inspector::ids::INSP_TRANSFORM_POS_X)
        .map(|(_, r)| r)
        .find(|r| s.contains(r.x + 1.0, r.y + 1.0))
        .expect("o campo X dentro do solto");
    // Dois `Down` seguidos no mesmo id e no mesmo instante (`mao` não tem relógio) seriam um duplo clique.
    let fundo = (s.x + s.w * 0.5, s.y + s.h - 3.0);
    mao(h, PointerKind::Down, fundo);
    mao(h, PointerKind::Up, fundo);
    let _ = h.bus.drain().count();
    let (xa, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
    arrasta(h, (xa, y), (xa + 40.0, y));
    let i = h
        .bus
        .drain()
        .filter_map(|a| match a {
            EditorAction::InspectorTransformEdit(i) => Some(i),
            _ => None,
        })
        .next_back()
        .unwrap_or_else(|| {
            panic!(
                "o arrasto em {k:?} não publicou: campo {r:?} · solto {s:?} · sob a mão {:?}",
                h.hit_index.hit(xa, y)
            )
        });
    quadros(h, 2);
    i
}

/// ⭐ **Dono 05/11: mover um objecto no X não pode mexer no cartão do outro.** Cada solto preso escreve só o campo
/// que a mão mexeu: o resto é do objecto dele, nunca da caixa partilhada do `WidgetStore`.
/// *Mutação: `commit_transform_edit` ler o Y do `store` ⇒ o Y do Square vai parar no Circle.*
#[test]
fn arrastar_o_x_de_um_preso_nao_traz_o_y_do_outro() {
    let (mut h, dois) = duas_copias();
    for (k, bits, x, y) in [
        (
            ph2d_panel_inspector::CARD_TRANSFORM,
            CIRCULO,
            1.0,
            Y_CIRCULO,
        ),
        (dois, QUADRADO, 9.0, Y_QUADRADO),
        (
            ph2d_panel_inspector::CARD_TRANSFORM,
            CIRCULO,
            1.0,
            Y_CIRCULO,
        ),
    ] {
        let i = arrasta_o_x(&mut h, k);
        assert_eq!(i.entity_bits, bits);
        assert_eq!(i.translation[1], y, "o Y de {bits} mudou: {i:?}");
        assert!(
            (i.translation[0] - x).abs() < 2.0 && i.translation[0] != x,
            "o X de {bits} tem de andar a partir do dele: {i:?}"
        );
    }
}

/// ⭐ O preso que também é a seleção não tinha diferença guardada: o cartão dele mostrava o X que a mão arrastava no outro.
#[test]
fn arrastar_num_cartao_nao_muda_o_numero_que_o_outro_mostra() {
    let (mut h, dois) = duas_copias();
    let a = ph2d_panel_inspector::CARD_TRANSFORM;
    // O Square passa a ser também a seleção.
    com_y(QUADRADO, 9.0, Y_QUADRADO);
    h.gizmo.replace_selection(Some(QUADRADO));
    quadros(&mut h, 5);
    let antes = pos_x(&h, dois);
    assert!(antes.is_some());
    // No MEIO do arrasto: é aí que a caixa partilhada tem o número da mão.
    use ph2d_card_stack::metrics as m;
    let solto = h.store.panel_rect(loose_rect_of(a)).expect("o solto");
    let r = h
        .hit_index
        .iter_registrations()
        .filter(|(i, _)| *i == ph2d_panel_inspector::ids::INSP_TRANSFORM_POS_X)
        .map(|(_, r)| r)
        .find(|r| solto.contains(r.x + 1.0, r.y + 1.0))
        .expect("o campo X dentro do solto");
    let (xa, y) = (r.x + m::ICON_CELL * 0.5, r.y + r.h * 0.5);
    mao(&mut h, PointerKind::Down, (xa, y));
    pinta(&mut h, 2);
    for i in 1..=4 {
        mao(&mut h, PointerKind::Move, (xa + 10.0 * i as f32, y));
        quadros(&mut h, 1);
        assert_eq!(pos_x(&h, dois), antes, "o número do outro cartão andou");
    }
    mao(&mut h, PointerKind::Up, (xa + 40.0, y));
    assert_eq!(pos_x(&h, dois), antes, "o número do outro cartão andou");
}

/// O Transform preso ao Circle e a cópia 2 presa ao Square, com outro objecto seleccionado.
fn duas_copias() -> (HeroScreen, NodeId) {
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
    (h, dois)
}
