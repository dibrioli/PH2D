//! ⭐⭐ **SONDA: um espelho com o plano fora da origem mantém o campo marchável?** — o instrumento
//! que achou e provou a lei da quiralidade em **8 s**, contra os **380 s** do gate dos trios.
//!
//! # Por que ela existe
//!
//! O `every_trio_of_modifiers_keeps_the_field_marchable` varre `10³` pilhas e demora `6 minutos`;
//! ele diz **que** rasgou e não deixa varrer uma hipótese. Esta sonda varre só os **pares** que
//! contêm um espelho, e varre o **plano** por quatro posições (dentro da peça, na face dela, e
//! para lá dela) — que é o alcance que o slider do artista tem.
//!
//! Foi ela que separou as duas leituras: **o espelho sozinho é exacto em todas as posições**
//! (`1,0000`), e o que rasgava era a composição `[MirrorY, Radial]` (**`223,90`**), porque um plano
//! deslocado torna a secção **quiral** — a mesma premissa que a terceira fatia da
//! [`ph2d_field_eval`] já defendia para as torções. ⇒ a cura foi uma linha na bandeira, e esta
//! sonda mediu-a verde nas quatro posições.
//!
//! ⚠️ **`probe_`: ela imprime e cala-se.** Quem afirma são os gates dos pares e dos trios.
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, UnaryKind, Xform};
use ph2d_field_eval::Field;

const HALF: [f32; 3] = [0.35, 0.35, 0.30];

thread_local! {
    static PLANO: std::cell::Cell<f32> = const { std::cell::Cell::new(1.0) };
}
fn plano(eixo: usize) -> f32 {
    -HALF[eixo] * PLANO.with(std::cell::Cell::get)
}

fn vivo(k: UnaryKind) -> Unary {
    use ph2d_field::mods::{ARRAY_AXIS, BEND_AXIS, RADIAL_AXIS, TAPER_AXIS, TWIST_AXIS};
    match k {
        UnaryKind::Shell => Unary::Shell { thickness: 0.06 },
        UnaryKind::Offset => Unary::Offset { distance: 0.05 },
        UnaryKind::Mirror => Unary::Mirror { offset: plano(0) },
        UnaryKind::MirrorY => Unary::MirrorY { offset: plano(1) },
        UnaryKind::MirrorZ => Unary::MirrorZ { offset: plano(2) },
        UnaryKind::Array => Unary::Array {
            count: 3,
            spacing: 0.5,
            joint: ph2d_field::Joint::SHARP,
            axis: ARRAY_AXIS,
        },
        UnaryKind::Radial => Unary::Radial {
            count: 6,
            joint: ph2d_field::Joint::SHARP,
            axis: RADIAL_AXIS,
        },
        UnaryKind::Taper => Unary::Taper {
            slope: 0.6,
            axis: TAPER_AXIS,
        },
        UnaryKind::Twist => Unary::Twist {
            turns: 0.35,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,
            axis: TWIST_AXIS,
        },
        UnaryKind::Bend => Unary::Bend {
            turns: 0.12,
            lower: -2.0,
            upper: 2.0,
            falloff: 0.1,
            axis: BEND_AXIS,
        },
    }
}

fn peca(mods: Vec<Unary>) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: HALF,
            round: 0.0,
            chamfer: 0.0,
        }),
    );
    n.mods = mods;
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

fn worst_gradient(doc: &FieldDoc, steps: i32) -> f64 {
    let reg = ph2d_field_eval::hybrid::Registry::default();
    let Some(bola) = ph2d_field_eval::bounds::bounding_ball(doc, &reg) else {
        return 0.0;
    };
    let (lo, hi_box) = ph2d_field_eval::bounds_clip::march_clip(bola);
    let f = Field::new(doc);
    let mut hi = 0.0f64;
    let h = 1.0e-3;
    for i in 0..=steps {
        for j in 0..=steps {
            for k in 0..=steps {
                let p = |n: i32, e: usize| {
                    let t = f64::from(n) / f64::from(steps);
                    f64::from(lo[e]) + t * f64::from(hi_box[e] - lo[e])
                };
                let (x, y, z) = (p(i, 0), p(j, 1), p(k, 2));
                let d = [
                    (f.at(x + h, y, z) - f.at(x - h, y, z)) / (2.0 * h),
                    (f.at(x, y + h, z) - f.at(x, y - h, z)) / (2.0 * h),
                    (f.at(x, y, z + h) - f.at(x, y, z - h)) / (2.0 * h),
                ];
                let g = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                if g.is_finite() && g > hi {
                    hi = g
                }
            }
        }
    }
    hi
}

#[test]
fn probe_pares_com_o_plano_de_nascimento() {
    for fracao in [1.0f32, 0.7, 0.4, 1.6] {
        PLANO.with(|c| c.set(fracao));
        println!("--- plano a {fracao}x a meia-extensao ---");
        for m in [UnaryKind::Mirror, UnaryKind::MirrorY, UnaryKind::MirrorZ] {
            println!(
                "{m:?} sozinho: {:.4}",
                worst_gradient(&peca(vec![vivo(m)]), 20)
            );
            for o in UnaryKind::ALL {
                let a = worst_gradient(&peca(vec![vivo(m), vivo(o)]), 20);
                let b = worst_gradient(&peca(vec![vivo(o), vivo(m)]), 20);
                if a > 1.02 || b > 1.02 {
                    println!("  ⛔ [{m:?},{o:?}]={a:.4}  [{o:?},{m:?}]={b:.4}");
                }
            }
        }
    }
}
