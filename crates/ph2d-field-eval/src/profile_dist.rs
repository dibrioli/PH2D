//! **As primitivas de distância 2D** — ponto↔segmento, segmento↔caixa, segmento↔polígono convexo.
//!
//! Irmão do [`super::profile_index`] por responsabilidade (teto de LOC): ali mora *o índice e o
//! corte*, aqui *a aritmética que eles perguntam*. ⚠️ A fórmula ponto↔segmento é a do
//! [*2D distance functions*](https://iquilezles.org/articles/distfunctions2d/) de Inigo Quilez.

use super::Edge;

pub(super) fn seg_dist2(p: [f32; 2], e: &Edge) -> f32 {
    let w = [p[0] - e.a[0], p[1] - e.a[1]];
    let h = (w[0].mul_add(e.e[0], w[1] * e.e[1]) * e.inv_ee).clamp(0.0, 1.0);
    let q = [w[0] - h * e.e[0], w[1] - h * e.e[1]];
    q[0].mul_add(q[0], q[1] * q[1])
}

/// A MENOR distância ao quadrado entre um segmento e uma caixa.
///
/// ⚠️ Entre dois convexos o par mais próximo envolve sempre um **vértice** de um deles — então os
/// seis candidatos abaixo esgotam o caso, e uma aproximação aqui seria uma aresta deitada fora que
/// podia ser a mais próxima (ver [`ProfileIndex::sd_batch_culled`]).
/// A distância² entre a aresta e um **polígono convexo** (a região como ela de facto é).
///
/// Zero quando se tocam; senão o mínimo entre a aresta e cada lado do polígono.
pub(super) fn seg_hull_dist2(e: &Edge, hull: &[[f32; 2]]) -> f32 {
    // Uma ponta dentro ⇒ tocam. ⚠️ O polígono é convexo e dado em sentido consistente, então
    // «dentro» é estar do mesmo lado de todos os lados.
    if point_in_hull(e.a, hull) || point_in_hull(e.b, hull) {
        return 0.0;
    }
    let mut best = f32::INFINITY;
    for i in 0..hull.len() {
        let (p, q) = (hull[i], hull[(i + 1) % hull.len()]);
        best = best.min(seg_seg_dist2(e.a, e.b, p, q));
    }
    best
}

fn point_in_hull(p: [f32; 2], hull: &[[f32; 2]]) -> bool {
    let (mut pos, mut neg) = (false, false);
    for i in 0..hull.len() {
        let (a, b) = (hull[i], hull[(i + 1) % hull.len()]);
        let c = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
        pos |= c > 0.0;
        neg |= c < 0.0;
    }
    !(pos && neg)
}

/// A distância² entre dois segmentos — zero se se cruzam.
fn seg_seg_dist2(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> f32 {
    let cross = |o: [f32; 2], p: [f32; 2], q: [f32; 2]| {
        (p[0] - o[0]) * (q[1] - o[1]) - (p[1] - o[1]) * (q[0] - o[0])
    };
    let (d1, d2) = (cross(a, b, c), cross(a, b, d));
    let (d3, d4) = (cross(c, d, a), cross(c, d, b));
    if (d1 > 0.0) != (d2 > 0.0) && (d3 > 0.0) != (d4 > 0.0) {
        return 0.0;
    }
    let pt = |p: [f32; 2], u: [f32; 2], v: [f32; 2]| {
        let uv = [v[0] - u[0], v[1] - u[1]];
        let len2 = uv[0].mul_add(uv[0], uv[1] * uv[1]);
        let t = if len2 > 0.0 {
            (((p[0] - u[0]) * uv[0] + (p[1] - u[1]) * uv[1]) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let q = [p[0] - u[0] - t * uv[0], p[1] - u[1] - t * uv[1]];
        q[0].mul_add(q[0], q[1] * q[1])
    };
    pt(a, c, d)
        .min(pt(b, c, d))
        .min(pt(c, a, b))
        .min(pt(d, a, b))
}

pub(super) fn seg_box_dist2(e: &Edge, lo: [f32; 2], hi: [f32; 2]) -> f32 {
    // Sobrepostos ⇒ zero, e é o caso comum perto da superfície.
    let elo = [e.a[0].min(e.b[0]), e.a[1].min(e.b[1])];
    let ehi = [e.a[0].max(e.b[0]), e.a[1].max(e.b[1])];
    if elo[0] <= hi[0] && ehi[0] >= lo[0] && elo[1] <= hi[1] && ehi[1] >= lo[1] {
        // As caixas tocam-se; o segmento pode ainda não tocar a caixa, e a conta abaixo resolve.
        let d = [
            box_dist2(e.a, lo, hi),
            box_dist2(e.b, lo, hi),
            seg_dist2([lo[0], lo[1]], e),
            seg_dist2([hi[0], lo[1]], e),
            seg_dist2([lo[0], hi[1]], e),
            seg_dist2([hi[0], hi[1]], e),
        ];
        return d.iter().fold(f32::INFINITY, |a, b| a.min(*b));
    }
    let d = [
        box_dist2(e.a, lo, hi),
        box_dist2(e.b, lo, hi),
        seg_dist2([lo[0], lo[1]], e),
        seg_dist2([hi[0], lo[1]], e),
        seg_dist2([lo[0], hi[1]], e),
        seg_dist2([hi[0], hi[1]], e),
    ];
    d.iter().fold(f32::INFINITY, |a, b| a.min(*b))
}

pub(super) fn box_dist2(p: [f32; 2], lo: [f32; 2], hi: [f32; 2]) -> f32 {
    let dx = (lo[0] - p[0]).max(0.0).max(p[0] - hi[0]);
    let dy = (lo[1] - p[1]).max(0.0).max(p[1] - hi[1]);
    dx.mul_add(dx, dy * dy)
}
