//! ⭐⭐ **AS FORMAS CUJOS VÉRTICES O ARTISTA AUTORA** (W131–W132) — o triângulo escaleno e o
//! polígono de `N` vértices.
//!
//! # Por que elas saíram do irmão
//!
//! O [`super::primitive_tree`] responde por *qual fórmula cada forma usa*, e ele chegou aos **700**
//! do gate de LOC quando a W131 lhe trouxe o triângulo. ⚠️ **A cura é partir para irmão, nunca uma
//! entrada na allowlist**, e o corte é por responsabilidade: destas duas o artista **digita os
//! pontos**, e nenhuma outra primitiva desta casa é assim (o [`ph2d_field::Primitive::Extrude`] tem
//! pontos e o dono deles é o **editor vetorial**, que é precisamente a distinção que faz o polígono
//! ser primitiva própria em vez de mais um perfil).
//!
//! ⚠️ **O divisor da aresta continua a viver na porta** ([`super::primitive_tree::primitive`]) —
//! *uma lei escrita em dois sítios ainda não é uma lei.*

use fidget::context::Tree;
use ph2d_field::Primitive;

/// A árvore de cada uma das duas.
///
/// # Panics
/// Nunca — o `match` do chamador já garante que `p` é uma delas.
pub(crate) fn by_vertices(p: &Primitive) -> Tree {
    match p {
        // ─────────────────────────── W131 ───────────────────────────
        Primitive::Triangle {
            a,
            b,
            c,
            half_height,
            round,
            chamfer,
        } => {
            let f = |q: &[f32; 2]| [f64::from(q[0]), f64::from(q[1])];
            crate::ops_triangle::sd_triangle(
                f(a),
                f(b),
                f(c),
                f64::from(*half_height),
                f64::from(*round),
                f64::from(*chamfer),
            )
        }
        // ─────────────────────────── W132 ───────────────────────────
        // ⭐⭐⭐ **O polígono é o EXTRUDE, e a partilha é o achado da wave.** A distância a um
        // polígono simples — `min` sobre os segmentos com o sinal do enrolamento — é exactamente o
        // que a [`crate::profile::sd_extrude`] já calcula, e ela traz de graça três coisas que uma
        // segunda fórmula perderia: a **especialização por ladrilho** (que deixa cair as arestas
        // que aquela região não pode ver), a **abertura morfológica** do filete e o aro chanfrado.
        //
        // ⚠️ *Duas cópias da mesma lei divergem no dia em que uma delas mudar* — e aqui a que
        // divergiria seria a mais nova, que é a que ninguém está a olhar.
        Primitive::Polygon {
            profile,
            half_height,
            round,
            chamfer,
        } => crate::profile::sd_extrude(
            profile,
            f64::from(*half_height),
            f64::from(*round),
            f64::from(*chamfer),
        ),
        _ => unreachable!("by_vertices só é chamada pelas duas formas de vértices autorados"),
    }
}
