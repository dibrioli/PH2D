//! F14: o «Corner round» de qualquer forma (`round_shape_corners`). Doc 24.

use super::round_shape_corners;
use crate::corner_live::corner_at;
use crate::{ALL_SHAPES, ShapeKind, VecPath, VecVertex, cook};

const ASPECTS: [f64; 3] = [0.25, 1.0, 4.0];
const RADII: [f64; 4] = [0.05, 0.3, 1.0, 50.0];

fn raw(k: ShapeKind, aspect: f64) -> VecPath {
    cook(k, [0.0, 0.0], [4.0, 4.0 * aspect], &k.defaults())
}

/// Os contornos que arredondam: o principal e os sub-contornos fechados.
fn rounded_contours(p: &VecPath) -> Vec<(&[VecVertex], bool)> {
    let mut c = vec![(p.verts.as_slice(), p.closed)];
    c.extend(
        p.subpaths
            .iter()
            .filter(|s| s.closed)
            .map(|s| (s.verts.as_slice(), true)),
    );
    c
}

/// O seno EXACTO da viragem no vértice `i` (as alças, ou o vizinho quando a alça é nula).
fn turn_sine(v: &[VecVertex], i: usize) -> Option<f64> {
    let n = v.len();
    let (p, q, me) = (&v[(i + n - 1) % n], &v[(i + 1) % n], &v[i]);
    let dir = |cands: [[f64; 2]; 2]| {
        cands
            .into_iter()
            .map(|c| [c[0] - me.anchor[0], c[1] - me.anchor[1]])
            .find(|d| d[0].hypot(d[1]) > 1e-12)
    };
    let (a, b) = (
        dir([me.in_handle, p.out_handle])?,
        dir([me.out_handle, q.in_handle])?,
    );
    Some((a[0] * b[1] - a[1] * b[0]).abs() / (a[0].hypot(a[1]) * b[0].hypot(b[1])))
}

/// ⭐ Um vértice LISO (seno exacto zero) nunca se mexe: o motor só vê a quina que a geometria
/// tem. Com o `acos` de antes, os lisos da elipse esticada (balão oval, pensamento, gota, cone)
/// liam-se como quinas de `2e-8` rad e arredondavam — o contorno achatava.
#[test]
fn a_straight_vertex_never_moves() {
    for &k in ALL_SHAPES {
        for aspect in ASPECTS {
            let p = raw(k, aspect);
            for r in RADII {
                let out = round_shape_corners(p.clone(), r);
                for ((src, closed), (dst, _)) in
                    rounded_contours(&p).into_iter().zip(rounded_contours(&out))
                {
                    if !closed {
                        continue;
                    }
                    for i in 0..src.len() {
                        if turn_sine(src, i).is_some_and(|s| s == 0.0) {
                            assert!(
                                dst.iter().any(|v| v.anchor == src[i].anchor),
                                "{k:?} aspecto {aspect} r {r}: o liso {i} mexeu"
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Zero, negativo ou NaN = a forma intacta, byte a byte.
#[test]
fn zero_is_the_shape() {
    for &k in ALL_SHAPES {
        let p = raw(k, 1.0);
        for r in [0.0, -0.3, f64::NAN] {
            assert_eq!(round_shape_corners(p.clone(), r), p, "{k:?} r {r}");
        }
    }
}

/// Com raio, não sobra quina viva nos contornos que arredondam (saturado incluído).
#[test]
fn every_corner_rounds() {
    for &k in ALL_SHAPES {
        for aspect in ASPECTS {
            for r in RADII {
                let out = round_shape_corners(raw(k, aspect), r);
                for (vs, closed) in rounded_contours(&out) {
                    let live: Vec<usize> = (0..vs.len())
                        .filter(|&i| corner_at(vs, closed, i).is_some())
                        .collect();
                    assert!(live.is_empty(), "{k:?} aspecto {aspect} r {r}: {live:?}");
                }
            }
        }
    }
}

/// A distância de `e` ao contorno fechado `vs` (amostrado a 256 por troço).
fn distance(vs: &[VecVertex], e: [f64; 2]) -> f64 {
    let n = vs.len();
    let mut best = f64::INFINITY;
    for i in 0..n {
        let (a, b) = (&vs[i], &vs[(i + 1) % n]);
        let c = [a.anchor, a.out_handle, b.in_handle, b.anchor];
        let gap = |t: f64| {
            let q = super::bezier(&c, t);
            (q[0] - e[0]).hypot(q[1] - e[1])
        };
        // A amostra mais perto, refinada pela secção áurea sobre a cúbica (a corda mente ~1e-6).
        let s = (0..=256)
            .min_by(|&a, &b| gap(a as f64 / 256.0).total_cmp(&gap(b as f64 / 256.0)))
            .unwrap_or(0);
        let (mut lo, mut hi) = ((s as f64 - 1.0) / 256.0, (s as f64 + 1.0) / 256.0);
        (lo, hi) = (lo.max(0.0), hi.min(1.0));
        for _ in 0..80 {
            let (m1, m2) = (lo + 0.382 * (hi - lo), hi - 0.382 * (hi - lo));
            if gap(m1) < gap(m2) {
                hi = m2;
            } else {
                lo = m1;
            }
        }
        best = best.min(gap(0.5 * (lo + hi)));
    }
    best
}

/// Quantas vezes o segmento aberto `a → b` (sem as pontas) atravessa o contorno fechado `vs`.
fn crossings(vs: &[VecVertex], a: [f64; 2], b: [f64; 2]) -> usize {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy);
    if len < 1e-9 {
        return 0;
    }
    let shrink = 1e-6 / len;
    let (a, b) = (
        [a[0] + dx * shrink, a[1] + dy * shrink],
        [b[0] - dx * shrink, b[1] - dy * shrink],
    );
    let side = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    };
    let n = vs.len();
    let mut count = 0;
    for i in 0..n {
        let (u, w) = (&vs[i], &vs[(i + 1) % n]);
        let c = [u.anchor, u.out_handle, w.in_handle, w.anchor];
        for s in 0..256 {
            let (p, q) = (
                super::bezier(&c, s as f64 / 256.0),
                super::bezier(&c, (s + 1) as f64 / 256.0),
            );
            if side(a, b, p) * side(a, b, q) < 0.0 && side(p, q, a) * side(p, q, b) < 0.0 {
                count += 1;
            }
        }
    }
    count
}

/// ⭐ As linhas desenhadas sobre a face (as arestas do cubo e da pirâmide, as dobras da faixa,
/// as barras da sub-rotina): a ponta que tocava um contorno continua a tocá-lo — nem fica no
/// ar dentro do canto arredondado, nem espeta para fora dele.
#[test]
fn the_lines_on_the_face_keep_touching_the_outline() {
    let mut seen = 0;
    for &k in ALL_SHAPES {
        for aspect in ASPECTS {
            let p = raw(k, aspect);
            let outlines = |q: &VecPath| -> Vec<Vec<VecVertex>> {
                rounded_contours(q)
                    .into_iter()
                    .filter(|(_, c)| *c)
                    .map(|(v, _)| v.to_vec())
                    .collect()
            };
            for r in RADII {
                let out = round_shape_corners(p.clone(), r);
                for (s, t) in p.subpaths.iter().zip(&out.subpaths) {
                    if s.closed || s.verts.len() < 2 {
                        continue;
                    }
                    for end in [0, s.verts.len() - 1] {
                        let touched = outlines(&p)
                            .iter()
                            .position(|o| distance(o, s.verts[end].anchor) < 1e-9);
                        let Some(o) = touched else { continue };
                        seen += 1;
                        let d = distance(&outlines(&out)[o], t.verts[end].anchor);
                        assert!(
                            d < 1e-6,
                            "{k:?} aspecto {aspect} r {r}: a ponta {end} a {d}"
                        );
                        // E é o corte MAIS PERTO: entre a ponta de antes e a de agora não há
                        // contorno (a recta da aresta do cubo corta-o também do outro lado).
                        let (a, b) = (s.verts[end].anchor, t.verts[end].anchor);
                        assert_eq!(
                            crossings(&outlines(&out)[o], a, b),
                            0,
                            "{k:?} aspecto {aspect} r {r}: a ponta {end} saltou o contorno"
                        );
                    }
                }
            }
        }
    }
    assert!(seen > 0, "o censo varreu linhas sobre a face");
}

/// ⭐ (A, o rectângulo) O «Corner round» do rectângulo É o raio do rectângulo arredondado — a mesma
/// conta (recuo `r` a 90°, a alça do arco de um quarto), da quina viva e do gerador — a menos do
/// `KAPPA` que o gerador escreve com 8 casas (`3e-11` no ponto: `corner_tests.rs`, «resíduo»).
#[test]
fn the_rectangle_corner_round_is_the_round_rect_radius() {
    for (w, h) in [(4.0, 4.0), (6.0, 2.0), (1.0, 5.0)] {
        for r in [0.0, 0.2, 0.7, 1.0, 9.0] {
            let rect = cook(ShapeKind::Rectangle, [0.0, 0.0], [w, h], &[r]);
            let round = cook(ShapeKind::RoundRect, [0.0, 0.0], [w, h], &[r]);
            let pts = |p: &VecPath| -> Vec<[f64; 2]> {
                p.verts
                    .iter()
                    .flat_map(|v| [v.anchor, v.in_handle, v.out_handle])
                    .collect()
            };
            let (a, mut b) = (pts(&rect), pts(&round));
            assert_eq!(a.len(), b.len(), "{w}×{h} r {r}");
            for p in a {
                let near = b
                    .iter()
                    .position(|q| (p[0] - q[0]).abs() < 1e-9 && (p[1] - q[1]).abs() < 1e-9);
                let Some(i) = near else {
                    panic!("{w}×{h} r {r}: {p:?} não está no rectângulo arredondado");
                };
                b.swap_remove(i);
            }
        }
    }
}

/// ⭐ A régua INDEPENDENTE do motor: toda quina que as tangentes mostram (seno ≠ 0, e não uma
/// cúspide de ângulo zero) é uma quina para o `corner_at`. O motor recusava todo contorno de
/// menos de 3 vértices, e o coração (dois troços) não tinha quinas para ninguém — nem para o
/// censo, que lia pelo mesmo predicado.
#[test]
fn the_engine_sees_every_corner_the_geometry_has() {
    for &k in ALL_SHAPES {
        for aspect in ASPECTS {
            let p = raw(k, aspect);
            for (vs, closed) in rounded_contours(&p) {
                if !closed {
                    continue;
                }
                for i in 0..vs.len() {
                    if turn_sine(vs, i).is_some_and(|s| s > 1e-6) {
                        assert!(
                            corner_at(vs, true, i).is_some(),
                            "{k:?} aspecto {aspect}: a quina {i} é invisível ao motor"
                        );
                    }
                }
            }
        }
    }
}

/// Um FURO com quinas (um sub-contorno fechado) arredonda como o contorno de fora.
#[test]
fn a_closed_hole_rounds_too() {
    let square = |lo: f64, hi: f64| -> Vec<VecVertex> {
        [[lo, lo], [hi, lo], [hi, hi], [lo, hi]]
            .into_iter()
            .map(VecVertex::corner)
            .collect()
    };
    let p = VecPath {
        verts: square(0.0, 4.0),
        closed: true,
        subpaths: vec![crate::Contour::new_closed(square(1.0, 3.0))],
        ..VecPath::default()
    };
    let out = round_shape_corners(p, 0.3);
    for (vs, closed) in rounded_contours(&out) {
        assert_eq!(vs.len(), 8, "quatro quinas, dois pontos cada");
        assert!((0..vs.len()).all(|i| corner_at(vs, closed, i).is_none()));
    }
}
