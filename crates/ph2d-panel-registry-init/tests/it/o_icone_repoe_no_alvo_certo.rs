//! ⭐ **O duplo clique no ícone repõe a fábrica no alvo certo** (F13), por gesto real: a reposição pede ao
//! mundo o mesmo que escrever o valor à mão, e é desse pedido que o Auto Key vive (`autokey_pass` compara a
//! pose do objecto editado). Uma seleção, várias, misto e cartão preso.

use super::*;
use ph2d_editor_core::InspectorTransformInfo;
use ph2d_editor_core::widget::card_field::handle_rect;
use ph2d_panel_inspector::CARD_TRANSFORM;

const X: NodeId = insp::ids::INSP_TRANSFORM_SCALE_X;

fn info(bits: u64) -> InspectorTransformInfo {
    InspectorTransformInfo {
        entity_bits: bits,
        translation: [0.0, 0.0],
        rotation_rad: 0.0,
        scale: [2.5, 1.5],
        skew_rad: [0.0, 0.0],
    }
}

/// O que o painel publica do objecto primário, longe da fábrica; o preso, se o há, idem.
fn publica(m: &mut Mesa) {
    let pins = m.h.store.loose_pins_of(INSP_PANEL);
    insp::com_cada_preso(&pins, |b| {
        insp::set_current_inspector_transform(Some(info(b)));
    });
    insp::set_current_inspector_transform(Some(info(m.bits[m.sel[0]])));
    insp::set_current_inspector_selecionados(m.sel.len());
    pinta(&mut m.h, 2);
    m.h.bus.drain().for_each(drop);
}

/// As edições do Transform que o gesto deixou, e por quem elas se espalham.
fn edicoes(m: &mut Mesa) -> (Vec<InspectorTransformInfo>, Vec<u64>) {
    let e =
        m.h.bus
            .drain()
            .filter_map(|a| match a {
                EditorAction::InspectorTransformEdit(i) => Some(i),
                _ => None,
            })
            .collect();
    let n = m.h.gizmo.selected_len();
    let fan = m.h.store.card_fan_out(n, m.h.gizmo.iter_selected());
    (e, fan)
}

/// A pega do Scale X: a da coluna, ou a do solto `k`.
fn pega(m: &Mesa, k: Option<NodeId>) -> (f32, f32) {
    let r = match k {
        None => m.h.hit_index.rect_for(X).expect("o Scale X"),
        Some(k) => {
            let (x, y) = m.no_solto(k, X);
            m.h.hit_index.hit_with_rect(x, y).expect("o campo").1
        }
    };
    centro(handle_rect(r))
}

/// Dois toques no ícone, com o relógio da mão parado: o duplo clique.
fn duplo(m: &mut Mesa, p: (f32, f32)) {
    let _ = m.h.store.record_pointer_down(Some(NodeId(1)), 0);
    for _ in 0..2 {
        mao(&mut m.h, PointerKind::Down, p);
        mao(&mut m.h, PointerKind::Up, p);
    }
    assert_eq!(m.h.store.focus_id(), Some(X), "o ícone não abriu a escrita");
}

fn mesa(sel: &[usize]) -> Mesa {
    let mut m = Mesa::nova();
    m.h.store.set_collapsed(CARD_TRANSFORM, false);
    m.escolhe(sel);
    // O cartão acaba de abrir: até a mola parar, o corpo recorta o campo.
    for _ in 0..400 {
        m.quadro();
        if !m.h.store.card_list_mut(INSP_PANEL).stack.moving() {
            break;
        }
    }
    publica(&mut m);
    m
}

/// Os pedidos da reposição e os de escrever `1` à mão, partindo do mesmo sítio.
fn reposicao_e_escrita(
    m: &mut Mesa,
    k: Option<NodeId>,
) -> [(Vec<InspectorTransformInfo>, Vec<u64>); 2] {
    let p = pega(m, k);
    duplo(m, p);
    let repor = edicoes(m);
    m.h.store.set_focus(None);
    publica(m);
    let _ = m.h.store.record_pointer_down(Some(NodeId(1)), 0);
    mao(&mut m.h, PointerKind::Down, p);
    mao(&mut m.h, PointerKind::Up, p);
    assert_eq!(m.h.store.focus_id(), Some(X));
    m.teclas("1");
    let escrever = edicoes(m);
    [repor, escrever]
}

/// *Mutação: tirar o `ValueChanged` do `down_on_handle` ⇒ a reposição não pede nada.*
#[test]
fn the_reset_asks_the_same_as_writing_with_one_object() {
    let mut m = mesa(&[0]);
    let [repor, escrever] = reposicao_e_escrita(&mut m, None);
    assert_eq!(repor.0.len(), 1, "um pedido, um passo: {:?}", repor.0);
    assert_eq!(repor.0[0].entity_bits, m.bits[0]);
    assert_eq!(repor.0[0].scale, [1.0, 1.5]);
    assert_eq!(repor.0, escrever.0);
    assert_eq!(repor.1, escrever.1);
}

/// Várias formas: o pedido espalha-se por todas, como a escrita.
#[test]
fn the_reset_fans_out_like_writing_with_many_objects() {
    let mut m = mesa(&[1, 2]);
    let [repor, escrever] = reposicao_e_escrita(&mut m, None);
    assert_eq!(
        repor.1,
        [m.bits[1], m.bits[2]],
        "a reposição não se espalhou"
    );
    assert_eq!(repor, escrever);
}

/// Campo misto: o 1.º toque do duplo clique abre vazio e sai sem escrever nada; só a reposição pede.
#[test]
fn the_reset_of_a_mixed_field_asks_once_and_clears_the_mix() {
    let mut m = mesa(&[1, 2]);
    m.h.store.blank_number_input(X);
    let p = pega(&m, None);
    duplo(&mut m, p);
    let (e, fan) = edicoes(&mut m);
    assert_eq!(e.len(), 1, "{e:?}");
    assert_eq!(e[0].scale, [1.0, 1.5]);
    assert_eq!(fan, [m.bits[1], m.bits[2]]);
    assert!(!m.h.store.number_input_mixed(X), "o misto ficou");
}

/// Cartão preso a A com B e C escolhidos: a reposição e a escrita vão a A, sem espalhar.
/// *Mutação: `card_fan_out` sem a guarda do preso ⇒ o pedido espalha-se pela seleção.*
#[test]
fn the_reset_and_the_write_on_a_pinned_card_go_to_its_object() {
    let mut m = mesa(&[0]);
    let k = m.prende(CARD_TRANSFORM, 0, &[1, 2]);
    publica(&mut m);
    let [repor, escrever] = reposicao_e_escrita(&mut m, Some(k));
    assert_eq!(repor.0.len(), 1, "{:?}", repor.0);
    assert_eq!(repor.0[0].entity_bits, m.bits[0], "a reposição não foi a A");
    assert_eq!(repor.0[0].scale, [1.0, 1.5]);
    assert_eq!(repor, escrever);
    assert!(repor.1.is_empty(), "espalhou pela seleção: {:?}", repor.1);
}

/// Depois da reposição no preso, o que se escreve e sai do foco por um clique fora é ainda de A.
#[test]
fn what_is_typed_after_the_reset_on_a_pinned_card_stays_of_its_object() {
    let mut m = mesa(&[0]);
    let k = m.prende(CARD_TRANSFORM, 0, &[1]);
    publica(&mut m);
    let p = pega(&m, Some(k));
    duplo(&mut m, p);
    let _ = edicoes(&mut m);
    let arena = bumpalo::Bump::new();
    for e in dispatch_text_input(&mut m.h.store, '3', &arena).to_vec() {
        m.h.apply_event(e);
    }
    let s = m.h.store.panel_rect(loose_rect_of(k)).expect("o solto");
    mao(&mut m.h, PointerKind::Down, (s.x - 40.0, 880.0));
    let (e, _) = edicoes(&mut m);
    let ultima = e.last().expect("a escrita perdeu-se ao sair do foco");
    assert_eq!(ultima.entity_bits, m.bits[0], "o commit foi à seleção");
    assert_eq!(ultima.scale[0], 3.0);
}
