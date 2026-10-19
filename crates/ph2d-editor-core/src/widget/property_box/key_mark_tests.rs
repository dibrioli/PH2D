//! Gates do losango de animação (plano 34 §6).

use super::*;

const X: NodeId = NodeId(0xA1);
const Y: NodeId = NodeId(0xA2);

/// Um losango por linha: o estado mais forte entre os campos dela; nenhum animado ⇒ `None`.
#[test]
fn a_linha_mostra_o_estado_mais_forte() {
    set_key_marks(BTreeMap::new());
    assert_eq!(key_mark_of(&[X, Y]), None);
    set_key_marks(BTreeMap::from([(X, KeyMark::Animado), (Y, KeyMark::Chave)]));
    assert_eq!(key_mark_of(&[X, Y]), Some(KeyMark::Chave));
    set_key_marks(BTreeMap::from([(X, KeyMark::Mudado), (Y, KeyMark::Chave)]));
    assert_eq!(key_mark_of(&[X, Y]), Some(KeyMark::Mudado));
    assert_eq!(key_mark_of(&[Y]), Some(KeyMark::Chave));
}

/// Só a linha animada vira alvo do clique, e o clique devolve os campos dela.
#[test]
fn so_o_losango_pintado_e_alvo() {
    crate::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    set_key_marks(BTreeMap::from([(X, KeyMark::Animado)]));
    begin_key_mark_targets();
    let mut scene = VectorScene::new();
    let animada = Rect::new(0.0, 0.0, 10.0, 10.0);
    let parada = Rect::new(0.0, 20.0, 10.0, 10.0);
    paint_decorator_for(&mut scene, Theme::Dark, animada, &[X]);
    paint_decorator_for(&mut scene, Theme::Dark, parada, &[Y]);
    assert_eq!(
        key_mark_at(5.0, 5.0),
        Some(KeyMarkClick { fields: vec![X] })
    );
    assert_eq!(
        key_mark_at(5.0, 25.0),
        None,
        "o ponto de um campo parado nao e' alvo"
    );
}
