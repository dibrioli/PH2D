//! **A travessia** do catálogo de formas — módulo irmão de [`crate::shapes`] (teto de LOC).
//!
//! A tabela vive lá; aqui vivem as perguntas que se fazem A ela: qual é o descritor de uma
//! forma, como um valor cruza a fronteira UI↔mundo, onde ele satura, e — para um campo de
//! ESCOLHA — qual é a próxima opção e como ela se chama.

use crate::shapes::{FieldDesc, FieldUnit, SHAPES, ShapeDesc, ShapeGroup};
use ph2d_vec_scene::{ShapeKind, ShapeValues};

/// O descritor de `kind` (todo `ShapeKind` tem um — o gate abaixo garante).
#[must_use]
pub fn desc(kind: ShapeKind) -> &'static ShapeDesc {
    SHAPES.iter().find(|d| d.kind == kind).unwrap_or(&SHAPES[0])
}

/// As formas de uma família, na ordem do catálogo.
pub fn shapes_in(group: ShapeGroup) -> impl Iterator<Item = &'static ShapeDesc> {
    SHAPES.iter().filter(move |d| d.group == group)
}

/// Valores autorados (UI) → valores de DOCUMENTO (mundo). Só os campos `Px` viajam;
/// contagens, razões e ângulos são os mesmos dos dois lados.
#[must_use]
pub fn to_world(kind: ShapeKind, ui: &ShapeValues, px_to_world: f64) -> ShapeValues {
    let mut out = *ui;
    for (i, f) in desc(kind).fields.iter().enumerate() {
        if f.unit == FieldUnit::Px {
            out[i] = ui[i] * px_to_world;
        }
    }
    out
}

/// Valores de DOCUMENTO (mundo) → valores autorados (UI) — o inverso exato de
/// [`to_world`]. `px_to_world` degenerado devolve os campos `Px` zerados (em vez de
/// infinito).
#[must_use]
pub fn to_ui(kind: ShapeKind, world: &ShapeValues, px_to_world: f64) -> ShapeValues {
    let mut out = *world;
    for (i, f) in desc(kind).fields.iter().enumerate() {
        if f.unit == FieldUnit::Px {
            out[i] = if px_to_world > 0.0 {
                world[i] / px_to_world
            } else {
                0.0
            };
        }
    }
    out
}

/// Clampa UM valor autorado à faixa do campo dele — a porta única de todo campo descrito
/// por um [`FieldDesc`]: os de forma, os do CONECTOR (`connector::clamp_to` delega aqui) e
/// os das PONTAS do traço (Head Size / Head Round). Um clamp por família divergiria da
/// faixa que o `set_number_range` registra na caixa, e o campo passaria a mentir.
#[must_use]
pub fn clamp_to(f: &FieldDesc, v: f64) -> f64 {
    v.clamp(f.min, f.max)
}

/// Clampa cada campo à faixa dele (e arredonda as contagens). Aplicado a toda autoria,
/// então nem digitação nem save corrompido produzem forma inválida.
pub fn clamp(kind: ShapeKind, v: &mut ShapeValues) {
    for (i, f) in desc(kind).fields.iter().enumerate() {
        let mut x = v[i].clamp(f.min, f.max);
        // Uma contagem e uma escolha são INTEIRAS: meio lado de polígono, ou "1,4 de ponto
        // de vista", não existem.
        if matches!(f.unit, FieldUnit::Count | FieldUnit::Choice(_)) {
            x = x.round();
        }
        v[i] = x;
    }
}

/// A opção SEGUINTE de um campo de escolha (o clique cicla). `None` se o campo não é uma
/// escolha — o painel então trata o clique como o que ele é: nada.
#[must_use]
pub fn next_choice(kind: ShapeKind, index: usize, current: f64) -> Option<f64> {
    let f = desc(kind).fields.get(index)?;
    let FieldUnit::Choice(names) = f.unit else {
        return None;
    };
    let n = names.len().max(1) as f64;
    Some((current.round() + 1.0).rem_euclid(n))
}

/// O RÓTULO da opção escolhida (o que o botão mostra). `None` se o campo não é uma escolha.
#[must_use]
pub fn choice_label(kind: ShapeKind, index: usize, current: f64) -> Option<&'static str> {
    let f = desc(kind).fields.get(index)?;
    let FieldUnit::Choice(names) = f.unit else {
        return None;
    };
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "o valor e clampado a [0, n-1] antes de chegar aqui"
    )]
    let i = current.round().clamp(0.0, (names.len() - 1) as f64) as usize;
    names.get(i).copied()
}
