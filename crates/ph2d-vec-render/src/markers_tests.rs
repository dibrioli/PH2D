//! Testes de `markers.rs` — a COSTURA de render das pontas (arquivo irmão).
//!
//! A geometria da ponta em si é gateada em `ph2d_vec_scene::marker`. O que se prova AQUI é o
//! que só existe no render: que o `StrokeSpec` do usuário (`marker_scale` / `marker_round`)
//! chega íntegro na cabeça **e** no recuo da linha — os dois pela MESMA medida. Errar isso não
//! quebra a compilação: quebra o desenho (a linha atravessa a seta, ou fica um vão).

use super::*;
use ph2d_vec_scene::{Rgba8, VecVertex};

/// Uma linha horizontal, da esquerda para a direita: a ponta do FIM olha para `+x`.
fn line(x1: f64) -> VecPath {
    VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([x1, 0.0])],
        closed: false,
        ..VecPath::default()
    }
}

fn spec(end: Marker, scale: f64, round: f64) -> StrokeSpec {
    let mut s = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 2.0);
    s.marker_end = end;
    s.marker_scale = scale;
    s.marker_round = round;
    s
}

/// O quanto a cabeça avança para TRÁS do bico, no contorno real (as cúbicas, não as âncoras).
fn head_depth(geo: &VecPath, tip_x: f64) -> f64 {
    const STEPS: usize = 32;
    let n = geo.verts.len();
    let segs = if geo.closed { n } else { n - 1 };
    let mut deepest = f64::MIN;
    for i in 0..segs {
        let (a, b) = (&geo.verts[i], &geo.verts[(i + 1) % n]);
        let (p0, p1, p2, p3) = (a.anchor, a.out_handle, b.in_handle, b.anchor);
        for k in 0..=STEPS {
            let u = k as f64 / STEPS as f64;
            let v = 1.0 - u;
            let x = v * v * v * p0[0]
                + 3.0 * v * v * u * p1[0]
                + 3.0 * v * u * u * p2[0]
                + u * u * u * p3[0];
            deepest = deepest.max(tip_x - x);
        }
    }
    deepest
}

/// **O gate que morde: a linha para EXATAMENTE nas costas da cabeça, em qualquer `scale`.**
///
/// O recuo (`stroked_line`) e a cabeça (`head`) têm de ler o MESMO `marker_scale`. Se o recuo
/// ignorar o `scale`, uma cabeça 2.5× maior engole o fim do traço e a linha reaparece
/// ATRAVESSANDO a seta; se a cabeça ignorar, sobra um vão entre o fim da linha e a ponta.
/// Nenhum dos dois quebra a compilação — só o desenho.
#[test]
fn the_trimmed_line_meets_the_head_at_every_scale() {
    let path = line(60.0);
    for m in [Marker::Triangle, Marker::Diamond, Marker::CircleOpen] {
        for scale in [0.5, 1.0, 2.5] {
            for round in [0.0, 0.5, 1.0] {
                let s = spec(m, scale, round);
                let (marker, geo) = head(&path, &s, false).expect("a ponta existe");
                assert_eq!(marker, m);
                let trimmed = super::stroked_line(&path, &s).expect("sobra linha");
                let end_x = trimmed.verts.last().expect("tem vertices").anchor[0];

                let gap = head_depth(&geo, 60.0) - (60.0 - end_x);
                assert!(
                    gap.abs() < 1e-3 * s.width * s.marker_scale,
                    "{m:?} (scale {scale}, round {round}): a linha termina em x={end_x} e a \
                     cabeça vai ate x={} — {} de {}",
                    60.0 - head_depth(&geo, 60.0),
                    if gap > 0.0 { "VAO" } else { "sobreposicao" },
                    gap.abs()
                );
            }
        }
    }
}

/// O `marker_round` do usuário chega na cabeça: com ele a ponta perde as quinas vivas (mais
/// vértices, handles não-degenerados). Um `0.0` cravado no `build` passaria despercebido — a
/// cena renderiza, só que afiada para sempre.
#[test]
fn the_users_round_reaches_the_head() {
    let path = line(60.0);
    let sharp = head(&path, &spec(Marker::Triangle, 1.0, 0.0), false).expect("existe");
    let round = head(&path, &spec(Marker::Triangle, 1.0, 0.6), false).expect("existe");
    assert_eq!(sharp.1.verts.len(), 3, "afiada: um vertice por quina");
    assert_eq!(round.1.verts.len(), 6, "arredondada: dois por quina");
    assert!(
        round
            .1
            .verts
            .iter()
            .all(|v| v.in_handle != v.anchor || v.out_handle != v.anchor),
        "sobrou quina viva: o marker_round nao chegou no build"
    );
}

/// Sem ponta, sem recuo: a esmagadora maioria dos paths não paga nada por esta feature — e a
/// linha continua chegando inteira na sua extremidade.
#[test]
fn a_line_without_markers_is_not_trimmed() {
    let path = line(60.0);
    let s = StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 2.0);
    assert!(!s.has_markers());
    let trimmed = super::stroked_line(&path, &s).expect("a linha inteira");
    assert_eq!(
        trimmed.verts.last().expect("tem vertices").anchor,
        [60.0, 0.0]
    );
    assert!(head(&path, &s, false).is_none(), "sem ponta nao ha cabeca");
}

/// Um contorno FECHADO não tem extremo onde pôr uma ponta — nem com o seletor marcado.
#[test]
fn a_closed_path_has_no_ends_to_decorate() {
    let mut path = line(60.0);
    path.closed = true;
    assert!(head(&path, &spec(Marker::Triangle, 2.0, 0.0), false).is_none());
    assert!(head(&path, &spec(Marker::Triangle, 2.0, 0.0), true).is_none());
}
