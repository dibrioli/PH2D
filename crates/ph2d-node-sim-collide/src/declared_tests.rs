//! Os gates da peça que declarou uma FORMA (doc 109 §5) — pela porta do nó, o [`super::collide`].

use super::{RADIUS_AUTO, RADIUS_FIXED, SHAPE_BOWL, SHAPE_BOX, SHAPE_DISC, SHAPE_PLANE, collide};
use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_OFFSET_COLUMN, Column, INV_INERTIA_COLUMN, Stream,
};

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

/// ⭐⭐⭐ **Uma caixa INCLINADA roda no chão; de chapa, não** (doc 109 §6 — *«precisa destravar a
/// rot»*). O ponto do contacto é o do SUPORTE: numa caixa deitada ele é o meio da face (binário
/// zero) e numa inclinada é a quina (binário que a deita).
///
/// ⚠️ **E o `Lock Rotation` (a coluna a zero) trava-a**: nem roda, nem a coluna `rot` nasce.
#[test]
fn a_tilted_box_turns_on_the_floor_and_flat_or_locked_does_not() {
    let passo_de = |graus: f32, travada: bool| -> (Option<f32>, f32) {
        let mut s = caixa([0.0, -2.1], [0.5, 0.25], graus, None);
        if travada {
            s = s.with(INV_INERTIA_COLUMN, Column::Scalar(vec![0.0]));
        }
        let out = collide(
            &s,
            SHAPE_PLANE,
            -2.0,
            [0.0, 0.0],
            2.0,
            0.0,
            0.0,
            (RADIUS_AUTO, 0.25, 1.0),
            PLANO,
            (0.0, 0),
            [0.0, 0.0],
        );
        let y = match out.get("P") {
            Some(Column::Vec2(v)) => v[0][1],
            _ => panic!("sem P"),
        };
        let rot = match out.get("rot") {
            Some(Column::Scalar(v)) => Some(v[0]),
            _ => None,
        };
        (rot, y)
    };
    let angulo = |graus: f32, travada: bool| passo_de(graus, travada).0;
    let inclinada = angulo(20.0, false).expect("o angulo sai no stream");
    assert!(
        (inclinada - 20.0).abs() > 0.05,
        "a quina da' binario: {inclinada}"
    );
    // ⚠️ O ângulo é comparado com o de ENTRADA, e não com a ausência da coluna: ela vem no stream
    // (é o giro autorado da peça) e o nó copia-a — o que este gate mede é se ela MUDOU.
    assert_eq!(
        angulo(0.0, false),
        Some(0.0),
        "de chapa o binario e' zero: a peca nao roda"
    );
    assert_eq!(angulo(20.0, true), Some(20.0), "travada nao roda");

    // ⭐⭐⭐ **A correcção REPARTE-SE entre empurrar e rodar** (doc 109 §6): com a rotação livre a
    // caixa sobe MENOS do que travada, porque parte do empurrão virou giro. ⚠️ É este gate que
    // apanha a massa efectiva apagada (`k = 1`): sem ela as duas subiriam exactamente o mesmo, e o
    // resto do teste continuaria verde — medido, essa mutação SOBREVIVEU a tudo o resto.
    let (livre, travada) = (passo_de(20.0, false).1, passo_de(20.0, true).1);
    assert!(
        travada > livre + 1e-4,
        "travada sobe {travada}, livre sobe {livre} — o empurrao tem de repartir-se"
    );
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
