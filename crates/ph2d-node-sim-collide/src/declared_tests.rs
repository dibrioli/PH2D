//! Os gates da peça que declarou uma FORMA (doc 109 §5) — pela porta do nó, o [`super::collide`].

use super::{RADIUS_AUTO, RADIUS_FIXED, SHAPE_BOWL, SHAPE_BOX, SHAPE_DISC, SHAPE_PLANE, collide};
use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, COLLIDER_OFFSET_COLUMN, Column, Stream};

/// Uma peça em `p`, parada, com a caixa de meias `meia` e (opcional) giro e centro deslocado.
fn caixa(p: [f32; 2], meia: [f32; 2], graus: f32, desvio: Option<[f32; 2]>) -> Stream {
    let s = Stream::new(1)
        .with("P", Column::Vec2(vec![p]))
        .with("vel", Column::Vec2(vec![[0.0, 0.0]]))
        .with("rot", Column::Scalar(vec![graus]))
        .with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![meia]));
    match desvio {
        Some(d) => s.with(COLLIDER_OFFSET_COLUMN, Column::Vec2(vec![d])),
        None => s,
    }
}

const PLANO: [f32; 2] = [0.0, 1.0];

/// Um passo do nó com a forma `shape`, o obstáculo em `c`/`radius`/`meia` e o modo `mode`.
fn passo(s: &Stream, shape: i32, c: [f32; 2], radius: f32, meia: [f32; 2], mode: i32) -> [f32; 2] {
    let out = collide(
        s,
        shape,
        -2.0,
        c,
        radius,
        0.0,
        0.0,
        (mode, 0.25, 1.0),
        PLANO,
        (0.0, 0),
        meia,
    );
    match out.get("P") {
        Some(Column::Vec2(v)) => v[0],
        _ => panic!("sem P"),
    }
}

/// ⭐⭐⭐ **No chão a caixa pousa pela FACE** — deitada, pela meia altura; em pé (`90°`), pela meia
/// largura. E o CONTROLO: `Fixed` ignora a declaração e pousa pelo raio dele.
#[test]
fn a_declared_box_rests_on_the_floor_by_its_face() {
    let deitada = passo(
        &caixa([0.3, -2.1], [0.5, 0.25], 0.0, None),
        SHAPE_PLANE,
        [0.0, 0.0],
        2.0,
        [0.0, 0.0],
        RADIUS_AUTO,
    );
    assert!((deitada[1] + 1.75).abs() < 1e-5, "deitada: {deitada:?}");
    assert!(
        (deitada[0] - 0.3).abs() < 1e-6,
        "e nao escorrega: {deitada:?}"
    );

    let em_pe = passo(
        &caixa([0.3, -2.1], [0.5, 0.25], 90.0, None),
        SHAPE_PLANE,
        [0.0, 0.0],
        2.0,
        [0.0, 0.0],
        RADIUS_AUTO,
    );
    assert!((em_pe[1] + 1.5).abs() < 1e-4, "em pe': {em_pe:?}");

    let fixo = passo(
        &caixa([0.3, -2.1], [0.5, 0.25], 0.0, None),
        SHAPE_PLANE,
        [0.0, 0.0],
        2.0,
        [0.0, 0.0],
        RADIUS_FIXED,
    );
    assert!((fixo[1] + 1.75).abs() < 1e-5, "Fixed = raio 0,25: {fixo:?}");
    let fixo_em_pe = passo(
        &caixa([0.3, -2.1], [0.5, 0.25], 90.0, None),
        SHAPE_PLANE,
        [0.0, 0.0],
        2.0,
        [0.0, 0.0],
        RADIUS_FIXED,
    );
    assert!(
        (fixo_em_pe[1] + 1.75).abs() < 1e-5,
        "Fixed nao le o giro: {fixo_em_pe:?}"
    );
}

/// ⭐⭐ **O centro DESLOCADO pousa no sítio dele** — a arte mais alta que a origem pousa mais baixo.
#[test]
fn an_offset_collider_rests_where_its_centre_is() {
    let s = caixa([0.0, -2.1], [0.5, 0.25], 0.0, Some([0.0, 0.1]));
    let y = passo(&s, SHAPE_PLANE, [0.0, 0.0], 2.0, [0.0, 0.0], RADIUS_AUTO)[1];
    assert!((y + 1.85).abs() < 1e-5, "o centro a 0,1 acima de P: {y}");
}

/// ⭐⭐ **Sobre um disco e sobre uma caixa sólidos a peça pousa pela face dela.**
#[test]
fn a_declared_box_rests_on_a_disc_and_on_a_box_by_its_face() {
    let no_disco = passo(
        &caixa([0.0, 1.1], [0.5, 0.25], 0.0, None),
        SHAPE_DISC,
        [0.0, 0.0],
        1.0,
        [0.0, 0.0],
        RADIUS_AUTO,
    );
    assert!((no_disco[1] - 1.25).abs() < 1e-5, "{no_disco:?}");

    let na_caixa = passo(
        &caixa([0.0, 0.6], [0.5, 0.25], 0.0, None),
        SHAPE_BOX,
        [0.0, 0.0],
        2.0,
        [2.0, 0.5],
        RADIUS_AUTO,
    );
    assert!((na_caixa[1] - 0.75).abs() < 1e-5, "{na_caixa:?}");
}

/// ⭐⭐ **Na taça a caixa cabe INTEIRA** — os quatro cantos dentro do círculo, depois das varreduras.
#[test]
fn a_declared_box_fits_whole_inside_the_bowl() {
    let mut s = caixa([0.3, -1.9], [0.5, 0.25], 30.0, None);
    for _ in 0..40 {
        let p = passo(&s, SHAPE_BOWL, [0.0, 0.0], 2.0, [0.0, 0.0], RADIUS_AUTO);
        s.set("P", Column::Vec2(vec![p]));
    }
    let col = ph2d_contact::colisores(&s)
        .and_then(|c| c[0])
        .expect("declara");
    let p = match s.get("P") {
        Some(Column::Vec2(v)) => v[0],
        _ => unreachable!(),
    };
    for k in col.cantos(p).expect("caixa") {
        assert!(k[0].hypot(k[1]) <= 2.0 + 1e-3, "canto {k:?} fora da taca");
    }
}
