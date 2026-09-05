//! Gates do [`super::TrackValues`] — o número que a dope-sheet mostra ao lado do nome.

use super::TrackValues;
use crate::{Extrap, PropKind, TrackView};
use ph2d_anim::AnimTarget;

/// Uma row mínima: o que o publicador lê é `target`, `entity` e `prop`.
fn row(target: u64, entity: u64, prop: PropKind) -> TrackView {
    TrackView {
        target: AnimTarget::new(target),
        prop,
        entity,
        missing: false,
        keys: Vec::new(),
        buffer_ghost: None,
        pre: Extrap::default(),
        post: Extrap::default(),
        expr: None,
    }
}

/// **DUAS ROWS DO MESMO OBJECTO CARREGAM NÚMEROS DIFERENTES** — a razão de a chave ser o ALVO.
///
/// ⚠️ Mutação que tem de sangrar: chavear por `entity`. Um objecto com X e Y animados mostraria o
/// mesmo número nas duas rows (a última a ser publicada), e a leitura seria *"o Y está errado"*.
#[test]
fn two_rows_of_the_same_object_carry_different_numbers() {
    let tracks = [
        row(1, 7, PropKind::TranslationX),
        row(2, 7, PropKind::TranslationY),
    ];
    let mut v = TrackValues::default();
    v.publish(&tracks, |_, p| {
        Some(if p == PropKind::TranslationX {
            3.0
        } else {
            -9.0
        })
    });
    assert_eq!(v.get(1), Some(3.0));
    assert_eq!(v.get(2), Some(-9.0));
}

/// **UM CANAL SEM NÚMERO NÃO PUBLICA ZERO** — ele não publica nada.
///
/// ⚠️ *Um zero de «não medido» e um zero de «vale zero» são o mesmo byte* (lei do repo). O
/// `TimeRemap` e o `Position` recusam na porta de amostragem por desenho, e uma entidade morta
/// recusa por acidente: nos três casos a row tem de ficar **sem número**, e não a anunciar `0,00`.
#[test]
fn a_channel_with_no_number_publishes_nothing_not_zero() {
    let tracks = [row(1, 7, PropKind::TimeRemap)];
    let mut v = TrackValues::default();
    v.publish(&tracks, |_, _| None);
    assert_eq!(v.get(1), None, "a ausencia nao pode virar um zero pintado");
}

/// **UMA ROW QUE SAIU LEVA O NÚMERO DELA** — a razão de a publicação ser uma porta que LIMPA.
///
/// ⚠️ O `bevy` recicla bits de entidade e o alvo é derivado deles: uma entrada velha não fica
/// «desactualizada», ela passa a descrever **outro objecto**.
#[test]
fn a_row_that_left_takes_its_number_with_it() {
    let mut v = TrackValues::default();
    v.publish(&[row(1, 7, PropKind::Opacity)], |_, _| Some(0.5));
    assert_eq!(v.get(1), Some(0.5));
    v.publish(&[], |_, _| Some(0.5));
    assert_eq!(v.get(1), None, "a track saiu e o numero dela ficou");
}
