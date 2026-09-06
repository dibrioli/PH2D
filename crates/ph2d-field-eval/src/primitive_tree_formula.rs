//! ⭐ **AS FORMAS POR FÓRMULA baixam aqui** (W125–W128) — o cilindro com bojo, a superquadrática e
//! a superfórmula de Gielis.
//!
//! # Por que elas saíram do irmão
//!
//! O [`super::primitive_tree`] responde por *qual fórmula cada forma usa*; ele chegou aos **700** do
//! gate de LOC quando a superfórmula trouxe os oito números dela. ⚠️ **A cura é partir para irmão,
//! nunca uma entrada na allowlist**, e o corte é por responsabilidade: estas três não têm filete e
//! os números delas são **adimensionais**.
//!
//! ⚠️ **O divisor da aresta continua a viver na porta** ([`super::primitive_tree::primitive`]) —
//! *uma lei escrita em dois sítios ainda não é uma lei.*

use fidget::context::Tree;
use ph2d_field::Primitive;

/// A árvore de cada uma das três.
///
/// # Panics
/// Nunca — o `match` do chamador já garante que `p` é uma delas.
pub(crate) fn formula(p: &Primitive) -> Tree {
    match *p {
        // ─────────────────────────── W125 ───────────────────────────
        // ⚠️ **Sem `round`/`chamfer`** — o bojo já É o arredondamento desta forma, e um segundo
        // controlo sobre a mesma aresta daria dois nomes ao mesmo raio.
        Primitive::RoundedCylinder {
            radius,
            bulge,
            half_height,
        } => crate::ops_exact::sd_rounded_cylinder(
            f64::from(radius),
            f64::from(bulge),
            f64::from(half_height),
        ),
        // ─────────────────────────── W127 ───────────────────────────
        Primitive::Superquadric {
            half,
            exponent_top,
            exponent_side,
        } => crate::ops_super::sd_superquadric(
            [f64::from(half[0]), f64::from(half[1]), f64::from(half[2])],
            f64::from(exponent_top),
            f64::from(exponent_side),
        ),
        // ─────────────────────────── W128 ───────────────────────────
        Primitive::Superformula {
            half,
            top_symmetry,
            top_n1,
            top_n2,
            top_n3,
            side_symmetry,
            side_n1,
            side_n2,
            side_n3,
        } => {
            let cv = |m: f32, n1: f32, n2: f32, n3: f32| crate::ops_gielis::Curve {
                symmetry: f64::from(m),
                n1: f64::from(n1),
                n2: f64::from(n2),
                n3: f64::from(n3),
            };
            crate::ops_gielis::sd_superformula(
                [f64::from(half[0]), f64::from(half[1]), f64::from(half[2])],
                cv(top_symmetry, top_n1, top_n2, top_n3),
                cv(side_symmetry, side_n1, side_n2, side_n3),
            )
        }
        _ => unreachable!("o chamador só encaminha as três formas por fórmula"),
    }
}
