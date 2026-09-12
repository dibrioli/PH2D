//! ⭐ **DE QUEM É ESTE PIXEL** — a metade do gizmo que **aponta**, separada da que **projecta**.
//!
//! ⚠️ **O corte é por responsabilidade, e ele já estava escrito nos comentários do pai**: o
//! [`super`] responde *«onde ficam as alças»* e este *«qual delas o cursor apanhou»*. O ficheiro
//! comum passou as `600` do gate de LOC do shell na W133, e ⛔ *split, nunca allowlist*.
//!
//! ⚠️ **Módulo-filho por `#[path]`**, como a lei do arrasto e as alças de vértice: ele vê os tipos e
//! as constantes do pai por `use super::*`, e o `pub use` mantém `ph2d_viewport3d::gizmo::pick` a
//! resolver em todos os chamadores.

use super::*;

/// **De quem é este ponto?** — `None` quando nenhuma alça o reclama.
pub fn pick(projected: &[Projected], p: [f32; 2]) -> Option<Handle> {
    projected
        .iter()
        .find(|h| h.live && hits(&h.shape, p))
        .map(|h| h.handle)
}

fn hits(shape: &Shape, p: [f32; 2]) -> bool {
    match shape {
        Shape::Disc { center, radius } => dist(*center, p) <= *radius,
        Shape::Quad(q) => point_in_quad(*q, p),
        // ⚠️ A haste começa DEPOIS da folga: sem isto as três setas disputariam o centro com o
        // disco, e qual ganha dependeria da ordem da lista em vez da geometria.
        Shape::Arrow { from, to } => {
            let d = [to[0] - from[0], to[1] - from[1]];
            let len = (d[0] * d[0] + d[1] * d[1]).sqrt();
            if len <= INNER_PX {
                return false;
            }
            let u = [d[0] / len, d[1] / len];
            let start = [from[0] + u[0] * INNER_PX, from[1] + u[1] * INNER_PX];
            dist_to_segment(start, *to, p) <= GRAB_PX
        }
        Shape::Arc(pts) => pts
            .windows(2)
            .any(|w| dist_to_segment(w[0], w[1], p) <= GRAB_PX),
        // O punho é o quadrado do fim; o traço até ele é decoração e não se agarra.
        Shape::Grip { to, .. } => {
            (p[0] - to[0]).abs() <= GRIP_HALF_PX + GRAB_PX * 0.5
                && (p[1] - to[1]).abs() <= GRIP_HALF_PX + GRAB_PX * 0.5
        }
        // ⚠️ A folga é a mesma do punho, e pela mesma razão — ver [`VERTEX_HALF_PX`].
        Shape::Point { center } => {
            (p[0] - center[0]).abs() <= VERTEX_HALF_PX + GRAB_PX * 0.5
                && (p[1] - center[1]).abs() <= VERTEX_HALF_PX + GRAB_PX * 0.5
        }
    }
}

fn dist_to_segment(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> f32 {
    let d = [b[0] - a[0], b[1] - a[1]];
    let dd = d[0].mul_add(d[0], d[1] * d[1]);
    if dd <= f32::MIN_POSITIVE {
        return dist(a, p);
    }
    let t = ((p[0] - a[0]) * d[0] + (p[1] - a[1]) * d[1]) / dd;
    let t = t.clamp(0.0, 1.0);
    dist([a[0] + d[0] * t, a[1] + d[1] * t], p)
}
/// ⚠️ **Por produto vetorial, e não por «está dentro da caixa»**: o quadrilátero é um quadrado do
/// MUNDO já projetado, então ele é um losango qualquer na tela. Um teste de caixa alinhada
/// reclamaria pixels que não são dele — e como as três alças de plano se tocam nos cantos, o gesto
/// escolheria a errada exatamente onde a diferença importa.
fn point_in_quad(q: [[f32; 2]; 4], p: [f32; 2]) -> bool {
    let mut positive = false;
    let mut negative = false;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        if cross > 0.0 {
            positive = true;
        }
        if cross < 0.0 {
            negative = true;
        }
    }
    !(positive && negative)
}
