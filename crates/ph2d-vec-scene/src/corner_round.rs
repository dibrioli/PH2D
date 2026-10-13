//! ⭐ (F14) **O «Corner round» de QUALQUER forma** — a porta única do campo, chamada pelo
//! [`crate::cook`] por cima do contorno já feito (a tabela é `ShapeKind::corner_round_field`).
//!
//! A lei: toda quina que a GEOMETRIA vê ([`crate::corner_live`]: tangentes não colineares) leva o
//! raio `r`, no contorno principal e em todo sub-contorno FECHADO — o tipo do vértice não decide
//! (medido: a engrenagem marca os 32 cantos dos dentes como lisos). As linhas ABERTAS de um
//! composto são desenho sobre a face (as arestas do cubo, as barras da sub-rotina, as dobras da
//! faixa): não arredondam, e a ponta que tocava um contorno arredondado segue-o pela própria
//! direcção. Doc: `docs/UI_New_and_Simple/vetor_nova_ui/24_arredondar_as_quinas.md`.

use crate::{ShapeKind, VecPath, VecVertex};

impl ShapeKind {
    /// ⭐ (F14) O índice do campo «Corner round» desta forma — o ÚLTIMO dela, apenso e neutro em
    /// zero —, que o [`crate::cook`] passa à [`crate::round_shape_corners`]. `None` = não há
    /// quina viva, ou a forma já tem quem a arredonde (o raio do polígono, as pontas da estrela, o filete da
    /// nuvem). A lista das que ficam de fora, com a medida: doc 24 §2 e o censo
    /// `every_live_corner_has_a_rounder` (`ph2d-tool-vector`).
    #[must_use]
    pub const fn corner_round_field(self) -> Option<usize> {
        use ShapeKind as K;
        Some(match self {
            K::Rectangle | K::Delay => 0,
            K::Diamond
            | K::Parallelogram
            | K::HexagonFlat
            | K::Document
            | K::Display
            | K::PredefinedProcess
            | K::OffPage
            | K::NoteBracket
            | K::Bolt
            | K::Drop
            | K::Shield
            | K::Cross
            | K::Check
            | K::Banner
            | K::Bone
            | K::Heart => 1,
            K::Segment
            | K::Chevron
            | K::Trapezoid
            | K::TrapezoidFlip
            | K::Tag
            | K::IsoCone
            | K::RopeSegment => 2,
            K::Ellipse | K::Pie | K::ArrowRight | K::ArrowDouble | K::IsoCube | K::IsoPyramid => 3,
            K::ArrowBent | K::SpeechOval | K::Thought | K::Burst => 4,
            K::SpeechRect | K::Gear => 5,
            K::RoundRect
            | K::Polygon
            | K::Star
            | K::Spiral
            | K::Line
            | K::Arc
            | K::Pill
            | K::Cylinder
            | K::Junction
            | K::Brace
            | K::Cloud
            | K::Moon => return None,
        })
    }
}

/// Arredonda as quinas vivas de `p` pelo raio `r` (mundo). `r ≤ 0` (ou NaN) = `p` intacto.
#[must_use]
pub fn round_shape_corners(mut p: VecPath, r: f64) -> VecPath {
    if r.is_nan() || r <= 0.0 {
        return p;
    }
    let raw = p.clone();
    let mut changed = round_contour(&mut p.verts, p.closed, r);
    for s in p.subpaths.iter_mut().filter(|s| s.closed) {
        changed |= round_contour(&mut s.verts, true, r);
    }
    if changed {
        follow_the_outline(&raw, &mut p);
    }
    p
}

fn round_contour(verts: &mut Vec<VecVertex>, closed: bool, r: f64) -> bool {
    let mut stamped = verts.clone();
    stamped.iter_mut().for_each(|v| v.corner_radius = r);
    match crate::corner_live::round_authored_corners(&stamped, closed) {
        Some(out) if out != *verts => {
            *verts = out;
            true
        }
        _ => false,
    }
}

/// Cada contorno arredondável como par (cru, cozido).
fn outlines<'a>(raw: &'a VecPath, p: &'a VecPath) -> Vec<(&'a [VecVertex], &'a [VecVertex])> {
    let mut o = Vec::new();
    if raw.closed {
        o.push((raw.verts.as_slice(), p.verts.as_slice()));
    }
    for (a, b) in raw.subpaths.iter().zip(&p.subpaths) {
        if a.closed {
            o.push((a.verts.as_slice(), b.verts.as_slice()));
        }
    }
    o
}

/// As pontas das linhas abertas que pousavam num contorno passam ao ponto onde a recta da
/// ponta (a direcção do troço que chega a ela) corta o contorno cozido, o mais perto da ponta.
fn follow_the_outline(raw: &VecPath, p: &mut VecPath) {
    let mut moves = Vec::new();
    for (j, s) in p.subpaths.iter().enumerate().filter(|(_, s)| !s.closed) {
        let n = s.verts.len();
        if n < 2 {
            continue;
        }
        for end in [0, n - 1] {
            let Some(u) = end_direction(&s.verts, end) else {
                continue;
            };
            let e = s.verts[end].anchor;
            for (cru, cozido) in outlines(raw, p) {
                let tol = 1e-7 * diagonal(cru).max(1.0);
                let on_it = hits(cru, e, u).into_iter().any(|t| t.abs() <= tol);
                if !on_it {
                    continue;
                }
                let best = hits(cozido, e, u)
                    .into_iter()
                    .min_by(|a, b| a.abs().total_cmp(&b.abs()));
                if let Some(t) = best {
                    moves.push((j, end, [u[0] * t, u[1] * t]));
                }
                break;
            }
        }
    }
    for (j, end, d) in moves {
        let v = &mut p.subpaths[j].verts[end];
        for q in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
            q[0] += d[0];
            q[1] += d[1];
        }
    }
}

/// O versor que SAI da linha pela ponta `end` (do troço que chega a ela para fora).
fn end_direction(verts: &[VecVertex], end: usize) -> Option<[f64; 2]> {
    let first = end == 0;
    let v = &verts[end];
    let w = &verts[if first { 1 } else { end - 1 }];
    let toward = if first {
        [v.out_handle, w.in_handle, w.anchor]
    } else {
        [v.in_handle, w.out_handle, w.anchor]
    };
    toward.into_iter().find_map(|q| {
        let d = [v.anchor[0] - q[0], v.anchor[1] - q[1]];
        let l = d[0].hypot(d[1]);
        (l > 1e-12).then(|| [d[0] / l, d[1] / l])
    })
}

fn diagonal(verts: &[VecVertex]) -> f64 {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for v in verts {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    (hi[0] - lo[0]).hypot(hi[1] - lo[1])
}

/// Onde a recta `e + t·u` corta o contorno fechado `verts`: os `t` (distância com sinal).
fn hits(verts: &[VecVertex], e: [f64; 2], u: [f64; 2]) -> Vec<f64> {
    const STEPS: usize = 32;
    let nrm = [-u[1], u[0]];
    let n = verts.len();
    let mut out = Vec::new();
    for i in 0..n {
        let (a, b) = (&verts[i], &verts[(i + 1) % n]);
        let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
        let side = |t: f64| {
            let q = bezier(&c, t);
            nrm[0] * (q[0] - e[0]) + nrm[1] * (q[1] - e[1])
        };
        let mut push = |t: f64| {
            let q = bezier(&c, t);
            out.push(u[0] * (q[0] - e[0]) + u[1] * (q[1] - e[1]));
        };
        for k in 0..STEPS {
            let (mut lo, mut hi) = (k as f64 / STEPS as f64, (k + 1) as f64 / STEPS as f64);
            let (flo, fhi) = (side(lo), side(hi));
            if flo == 0.0 {
                push(lo);
                continue;
            }
            if flo * fhi > 0.0 {
                continue;
            }
            for _ in 0..64 {
                let mid = 0.5 * (lo + hi);
                if side(lo) * side(mid) <= 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }
            push(0.5 * (lo + hi));
        }
    }
    out
}

fn bezier(c: &[[f64; 2]; 4], t: f64) -> [f64; 2] {
    let m = 1.0 - t;
    let w = [m * m * m, 3.0 * m * m * t, 3.0 * m * t * t, t * t * t];
    [
        w[0] * c[0][0] + w[1] * c[1][0] + w[2] * c[2][0] + w[3] * c[3][0],
        w[0] * c[0][1] + w[1] * c[1][1] + w[2] * c[2][1] + w[3] * c[3][1],
    ]
}

#[cfg(test)]
#[path = "corner_round_tests.rs"]
mod tests;
