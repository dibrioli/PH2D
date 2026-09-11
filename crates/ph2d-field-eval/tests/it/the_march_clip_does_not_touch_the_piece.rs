//! ⭐⭐⭐ **O RECORTE DA MARCHA NÃO ENCOSTA NA PEÇA** — a cerca do
//! [`ph2d_field_eval::bounds_clip::MARCH_CLIP_PAD`].
//!
//! # Por que este gate existe (2026-09-01, report do Enio *«muitíssimo lento»*)
//!
//! A caixa do bordo passou a ser justa (as meias-extensões, e não o cubo do raio), e uma caixa justa
//! **toca** a superfície por definição. Um traçador de esferas anda o **valor** do campo: onde ele
//! entra em cima da peça o valor é zero, os passos são zero e ele fica parado — enquanto a marcha
//! honesta, de passo fixo, continua a andar. As duas divergem sem que o campo tenha deixado de ser
//! um minorante, e foi assim que `the_deformed_rosette_agrees_with_an_honest_march` foi de `6` para
//! `16` pixels.
//!
//! ⚠️ **A régua é a PROPRIEDADE, não o número**: *o campo é estritamente positivo em toda a
//! fronteira do recorte*. Um gate que comparasse a constante consigo própria não diria nada; este
//! reprova se alguém a puser a zero — e a metade de baixo prova que ele reprovaria.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, Xform};
use ph2d_field_eval::{Field, bounds, bounds_clip, hybrid::Registry};

fn peca(mods: Vec<Unary>) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.35, 0.35, 0.30],
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    n.mods = mods;
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// As quatro fixturas, e a **roseta** é a que mordeu.
fn fixturas() -> Vec<(&'static str, FieldDoc)> {
    use ph2d_field::mods::{BEND_AXIS, RADIAL_AXIS, TAPER_AXIS, TWIST_AXIS};
    vec![
        ("caixa", peca(Vec::new())),
        (
            "roseta",
            peca(vec![
                Unary::Taper {
                    slope: 0.6,
                    axis: TAPER_AXIS,
                },
                Unary::Radial {
                    count: 6,
                    joint: ph2d_field::Joint::SHARP,
                    axis: RADIAL_AXIS,
                },
            ]),
        ),
        (
            "dobrada",
            peca(vec![Unary::Bend {
                turns: 0.12,
                lower: -2.0,
                upper: 2.0,
                falloff: 0.1,
                axis: BEND_AXIS,
            }]),
        ),
        (
            "torcida",
            peca(vec![Unary::Twist {
                turns: 0.35,
                lower: -2.0,
                upper: 2.0,
                falloff: 0.1,
                axis: TWIST_AXIS,
            }]),
        ),
    ]
}

/// O menor valor do campo sobre as **seis faces** da caixa dada.
fn pior_na_fronteira(doc: &FieldDoc, lo: [f32; 3], hi: [f32; 3], n: i32) -> f64 {
    let f = Field::new(doc);
    let mut pior = f64::INFINITY;
    let em = |t: i32, e: usize| {
        let u = f64::from(t) / f64::from(n);
        f64::from(lo[e]) + u * f64::from(hi[e] - lo[e])
    };
    for eixo in 0..3 {
        let (u, v) = ((eixo + 1) % 3, (eixo + 2) % 3);
        for lado in [lo, hi] {
            for i in 0..=n {
                for j in 0..=n {
                    let mut p = [0.0f64; 3];
                    p[eixo] = f64::from(lado[eixo]);
                    p[u] = em(i, u);
                    p[v] = em(j, v);
                    let d = f.at(p[0], p[1], p[2]);
                    if d.is_finite() {
                        pior = pior.min(d);
                    }
                }
            }
        }
    }
    pior
}

/// ⭐⭐⭐ **A metade de cima: com a margem, a fronteira do recorte está FORA da peça.**
#[test]
fn the_march_clip_has_the_margin_it_was_measured_to_need() {
    let reg = Registry::default();
    let mut maus = Vec::new();
    for (nome, doc) in fixturas() {
        let bola = bounds::bounding_ball(&doc, &reg).expect("bordo");
        let (lo, hi) = bounds_clip::march_clip(bola);
        let pior = pior_na_fronteira(&doc, lo, hi, 24);
        if pior <= 0.0 {
            maus.push(format!(
                "{nome}: o campo vale {pior:.6} na fronteira do recorte"
            ));
        }
    }
    assert!(
        maus.is_empty(),
        "o recorte da marcha ENCOSTA na peça — um raio que entra ali lê um passo de zero e fica \
         parado, e a imagem deixa de concordar com a marcha honesta: {}",
        maus.join(" · ")
    );
}

/// ⛔⛔ **A metade de baixo: SEM a margem, ele encosta** — a prova de que o gate acima não é vazio.
///
/// ⚠️ Sem esta metade, apagar a margem deixaria o gate de cima **verde por acidente** em fixturas
/// cuja caixa por acaso não toca a peça. *Uma cerca que não sabe demonstrar o que impede é uma
/// afirmação, não um gate.*
#[test]
fn without_the_margin_the_clip_does_touch_the_piece() {
    let reg = Registry::default();
    let mut encostam = 0usize;
    for (_, doc) in fixturas() {
        let bola = bounds::bounding_ball(&doc, &reg).expect("bordo");
        let (lo, hi) = bola.aabb();
        if pior_na_fronteira(&doc, lo, hi, 24) <= 1e-4 {
            encostam += 1;
        }
    }
    assert!(
        encostam >= 2,
        "só {encostam} fixturas encostam sem a margem — a população desta metade evaporou, e com \
         ela a prova de que a margem faz alguma coisa"
    );
}
