//! **EDIT SHAPE** (F7e) — a forma do QuickShape AFINADA pelos nós PRÓPRIOS do tipo, logo depois de
//! nascer (o «Edit Shape» do Procreate). Arrastar um nó NUNCA muda o tipo.
//!
//! A lei foi MEDIDA nos dois oráculos instalados — o Excalidraw `next` (a forma que o «draw shape»
//! reconhece, editada no editor) e o tldraw 5.5.2 (as formas nativas), corridos por script com o rato
//! (`oraculo_quickshape/00_resultados.md` §3i):
//!
//! - **Linha / polilinha / polígono** — um nó por canto, livre; `regular` (o `Shift`) roda o lado
//!   que chega ao nó para um múltiplo de 15° (os dois oráculos).
//! - **Rectângulo** — os 4 cantos; o OPOSTO fica fixo e os lados ficam paralelos aos de antes (a
//!   rotação fica); `regular` guarda a proporção pela MAIOR das duas razões (os dois oráculos).
//! - **Elipse / círculo** — os 4 nós nos extremos dos eixos (= as alças de lado do tldraw): o
//!   oposto fica fixo e só conta o arrasto AO LONGO do eixo; o círculo vira elipse, `regular` escala
//!   os dois eixos à volta do nó oposto (o tldraw).
//! - **Arco / biarco** — início, ÁPICE (o ponto mais longe da corda) e fim. Mover uma ponta guarda a
//!   flecha (o `bend` do tldraw); o ápice só mexe a flecha (o tldraw: a alça do meio cai na
//!   mediatriz). O arco de CÍRCULO continua de círculo (o que passa pelas pontas e pelo ápice); o de
//!   elipse e o biarco seguem a afinidade da corda (a tangência dos dois arcos sobrevive).
//! - **Seta** — os nós do cabo; a ponta é do traço e vai onde o bico for.
//! - **O corpo** — arrastar a forma entre os nós escala-a por igual (o manual do Procreate).

use super::QuickShape;
use crate::quickshape::ArcShape;
use crate::quickshape_geom::{P2, add, angle, cross, dist, dot, mul, rotate, snap, sub, unit};

/// A guarda numérica de um eixo, de uma flecha ou de um lado, em fracção do tamanho da forma: um
/// zero não teria volta (o arrasto seguinte dividiria por ele). Não é uma faixa de produto.
const MIN_FRACTION: f64 = 1e-6;

/// Um quadrilátero com as quatro quinas a 90° é um rectângulo. A tolerância é do coseno da quina e
/// só separa o rectângulo CONSTRUÍDO pela lei (exacto a 1e-15) de um quadrilátero desenhado.
const RIGHT_COS: f64 = 1e-6;

/// Os nós da forma, em mundo e na ordem do tipo (ver o cabeçalho).
#[must_use]
pub fn nodes(shape: &QuickShape) -> Vec<P2> {
    match shape {
        QuickShape::Line { a, b } => vec![*a, *b],
        QuickShape::Polygon { corners, .. } => corners.clone(),
        QuickShape::Ellipse { .. } => (0..4).map(|k| ellipse_node(shape, k)).collect(),
        QuickShape::Arc(_) | QuickShape::Biarc(_) => {
            let (s, e) = ends(shape);
            vec![s, apex(shape), e]
        }
        QuickShape::Arrow(a) => nodes(&a.shaft),
    }
}

/// **A lei do arrasto**: `shape` é a forma de quando o nó foi AGARRADO e `to` onde o nó vai (o
/// chamador soma o arrasto à posição de então, então um nó agarrado fora do centro não salta).
#[must_use]
pub fn drag(shape: &QuickShape, node: usize, to: P2, regular: bool) -> QuickShape {
    match shape {
        QuickShape::Line { a, b } => {
            let (a, b) = if node == 0 {
                (step15(*b, to, regular), *b)
            } else {
                (*a, step15(*a, to, regular))
            };
            QuickShape::Line { a, b }
        }
        QuickShape::Polygon { corners, closed } if *closed && is_rectangle(corners) => {
            QuickShape::Polygon {
                corners: drag_rectangle(corners, node, to, regular),
                closed: true,
            }
        }
        QuickShape::Polygon { corners, closed } => {
            let mut corners = corners.clone();
            let n = corners.len();
            if node < n {
                let back = if node > 0 || *closed {
                    corners[(node + n - 1) % n]
                } else {
                    corners[(node + 1).min(n - 1)]
                };
                corners[node] = step15(back, to, regular);
            }
            QuickShape::Polygon {
                corners,
                closed: *closed,
            }
        }
        QuickShape::Ellipse { .. } => drag_ellipse(shape, node, to, regular),
        QuickShape::Arc(_) | QuickShape::Biarc(_) => drag_arc(shape, node, to, regular),
        QuickShape::Arrow(a) => {
            let mut a = a.clone();
            a.shaft = Box::new(drag(&a.shaft, node, to, regular));
            QuickShape::Arrow(a)
        }
    }
}

/// **O corpo**: arrastar a forma de `from` para `to` escala-a por igual à volta do pivô dela (o do
/// gesto de segurar: o centro de uma forma fechada, o início de uma aberta).
#[must_use]
pub fn scale(shape: &QuickShape, from: P2, to: P2) -> QuickShape {
    let p = shape.pivot();
    let d0 = dist(from, p);
    if d0 <= f64::MIN_POSITIVE {
        return shape.clone();
    }
    shape.transformed(p, (dist(to, p) / d0).max(MIN_FRACTION), 0.0)
}

/// O ponto `to`, com `regular`, levado para que `from → to` faça um múltiplo de 15° (o mesmo
/// comprimento).
fn step15(from: P2, to: P2, regular: bool) -> P2 {
    if !regular {
        return to;
    }
    let d = sub(to, from);
    let a = snap(angle(d), super::REGULAR_STEP_DEG.to_radians());
    add(from, mul(unit(a), d[0].hypot(d[1])))
}

/// Quatro cantos com as quatro quinas a 90°.
pub(crate) fn is_rectangle(c: &[P2]) -> bool {
    c.len() == 4
        && (0..4).all(|i| {
            let (u, v) = (sub(c[(i + 1) % 4], c[i]), sub(c[(i + 3) % 4], c[i]));
            let (lu, lv) = (u[0].hypot(u[1]), v[0].hypot(v[1]));
            lu > 0.0 && lv > 0.0 && (dot(u, v) / (lu * lv)).abs() <= RIGHT_COS
        })
}

/// O canto `node` vai a `to`; o oposto fica; os lados ficam paralelos aos de antes.
fn drag_rectangle(c: &[P2], node: usize, to: P2, regular: bool) -> Vec<P2> {
    let i = node % 4;
    let o = c[(i + 2) % 4];
    // A base do rectângulo, a partir do oposto: `u` para o vizinho que partilha o x, `v` o y.
    let (eu, ev) = (sub(c[(i + 1) % 4], o), sub(c[(i + 3) % 4], o));
    let (lu, lv) = (eu[0].hypot(eu[1]), ev[0].hypot(ev[1]));
    let (u, v) = (mul(eu, 1.0 / lu), mul(ev, 1.0 / lv));
    let d = sub(to, o);
    let (mut a, mut b) = (dot(d, u), dot(d, v));
    if regular {
        let k = (a.abs() / lu).max(b.abs() / lv);
        a = k * lu * sign(a);
        b = k * lv * sign(b);
    }
    let guard = MIN_FRACTION * lu.max(lv);
    a = if a.abs() < guard { guard * sign(a) } else { a };
    b = if b.abs() < guard { guard * sign(b) } else { b };
    let mut out = c.to_vec();
    out[(i + 2) % 4] = o;
    out[(i + 1) % 4] = add(o, mul(u, a));
    out[(i + 3) % 4] = add(o, mul(v, b));
    out[i] = add(o, add(mul(u, a), mul(v, b)));
    out
}

fn sign(x: f64) -> f64 {
    if x < 0.0 { -1.0 } else { 1.0 }
}

/// O nó `k` da elipse: `+u`, `+v`, `−u`, `−v` (os eixos `rx`, `ry` rodados de `rot`).
fn ellipse_node(shape: &QuickShape, k: usize) -> P2 {
    let QuickShape::Ellipse {
        centre,
        rx,
        ry,
        rot,
    } = shape
    else {
        return [0.0; 2];
    };
    let (axis, r) = if k.is_multiple_of(2) {
        (0.0, *rx)
    } else {
        (1.0, *ry)
    };
    let s = if k < 2 { 1.0 } else { -1.0 };
    add(
        *centre,
        mul(unit(rot + axis * std::f64::consts::FRAC_PI_2), s * r),
    )
}

fn drag_ellipse(shape: &QuickShape, node: usize, to: P2, regular: bool) -> QuickShape {
    let QuickShape::Ellipse { rx, ry, rot, .. } = *shape else {
        return shape.clone();
    };
    let k = node % 4;
    let o = ellipse_node(shape, (k + 2) % 4);
    let d = unit(angle(sub(ellipse_node(shape, k), o)));
    let (r_own, r_other) = if k.is_multiple_of(2) {
        (rx, ry)
    } else {
        (ry, rx)
    };
    let span = dot(sub(to, o), d);
    let r_new = (span.abs() / 2.0).max(MIN_FRACTION * rx.max(ry));
    let r_side = if regular {
        r_other * r_new / r_own
    } else {
        r_other
    };
    let (rx, ry) = if k.is_multiple_of(2) {
        (r_new, r_side)
    } else {
        (r_side, r_new)
    };
    QuickShape::Ellipse {
        centre: add(o, mul(d, sign(span) * r_new)),
        rx,
        ry,
        rot,
    }
}

/// As pontas de um arco ou biarco: onde o desenho começou e onde acabou.
fn ends(shape: &QuickShape) -> (P2, P2) {
    match shape {
        QuickShape::Arc(a) => (a.at(a.start), a.at(a.start + a.sweep)),
        QuickShape::Biarc(b) => (b[0].at(b[0].start), b[1].at(b[1].start + b[1].sweep)),
        _ => ([0.0; 2], [0.0; 2]),
    }
}

/// Os arcos de um arco ou biarco.
fn pieces(shape: &QuickShape) -> &[ArcShape] {
    match shape {
        QuickShape::Arc(a) => std::slice::from_ref(a),
        QuickShape::Biarc(b) => b,
        _ => &[],
    }
}

/// O ÁPICE: o ponto da curva mais longe da corda. Num arco é o meio do parâmetro (a tangente ali é
/// paralela à corda num círculo, e uma afinidade leva paralelas a paralelas); no biarco, procura-se
/// (amostras e refinamento pela secção áurea no melhor intervalo).
fn apex(shape: &QuickShape) -> P2 {
    if let QuickShape::Arc(a) = shape {
        return a.at(a.start + a.sweep / 2.0);
    }
    let (s, e) = ends(shape);
    let n = sub(e, s);
    let height = |a: &ArcShape, t: f64| cross(n, sub(a.at(t), s)).abs();
    let mut best = (0.0, s);
    for a in pieces(shape) {
        const N: usize = 64;
        let step = a.sweep / N as f64;
        let i = (0..=N)
            .max_by(|&i, &j| {
                let (ti, tj) = (a.start + step * i as f64, a.start + step * j as f64);
                height(a, ti).total_cmp(&height(a, tj))
            })
            .unwrap_or(0);
        let (mut lo, mut hi) = (
            a.start + step * i.saturating_sub(1) as f64,
            a.start + step * (i + 1).min(N) as f64,
        );
        for _ in 0..48 {
            let (m1, m2) = (lo + (hi - lo) * 0.382, lo + (hi - lo) * 0.618);
            if height(a, m1) < height(a, m2) {
                lo = m1;
            } else {
                hi = m2;
            }
        }
        let t = 0.5 * (lo + hi);
        if height(a, t) >= best.0 {
            best = (height(a, t), a.at(t));
        }
    }
    best.1
}

/// Arco e biarco: as pontas guardam a flecha; o ápice só a mexe. Ver o cabeçalho.
fn drag_arc(shape: &QuickShape, node: usize, to: P2, regular: bool) -> QuickShape {
    let (s, e) = ends(shape);
    let chord = sub(e, s);
    let l = chord[0].hypot(chord[1]);
    if l <= f64::MIN_POSITIVE {
        return shape.clone();
    }
    let (u, n) = (
        mul(chord, 1.0 / l),
        mul(rotate(chord, std::f64::consts::FRAC_PI_2), 1.0 / l),
    );
    let h = dot(sub(apex(shape), s), n);
    let (s2, e2, h2) = match node {
        0 => (step15(e, to, regular), e, h),
        1 => (s, e, dot(sub(to, s), n)),
        _ => (s, step15(s, to, regular), h),
    };
    let chord2 = sub(e2, s2);
    let l2 = chord2[0].hypot(chord2[1]).max(MIN_FRACTION * l);
    let u2 = mul(chord2, 1.0 / l2);
    let n2 = rotate(u2, std::f64::consts::FRAC_PI_2);
    let guard = MIN_FRACTION * l2;
    let h2 = if h2.abs() < guard {
        guard * sign(h2)
    } else {
        h2
    };
    if let QuickShape::Arc(a) = shape
        && is_circle(a)
    {
        return QuickShape::Arc(circle_through(s2, e2, h2, a.sweep.signum()));
    }
    // A afinidade da corda: ao longo dela escala `l2/l`, através dela `h2/h`.
    let (ku, kn) = (l2 / l, h2 / h);
    let lin = [
        [
            ku * u2[0] * u[0] + kn * n2[0] * n[0],
            ku * u2[0] * u[1] + kn * n2[0] * n[1],
        ],
        [
            ku * u2[1] * u[0] + kn * n2[1] * n[0],
            ku * u2[1] * u[1] + kn * n2[1] * n[1],
        ],
    ];
    let map = Affine {
        lin,
        t: sub(s2, apply(&lin, s)),
    };
    match shape {
        QuickShape::Arc(a) => QuickShape::Arc(map.arc(a)),
        QuickShape::Biarc(b) => QuickShape::Biarc(b.map(|a| map.arc(&a))),
        _ => shape.clone(),
    }
}

/// Um arco de círculo (os dois raios iguais, a qualquer rotação).
fn is_circle(a: &ArcShape) -> bool {
    (a.rx - a.ry).abs() <= 1e-9 * a.rx.abs().max(a.ry.abs())
}

/// O arco de círculo de `s` a `e` cujo ápice fica a `h` da corda (do lado `+n`, a corda rodada de
/// 90°). O ângulo varrido é `4·atan(2|h|/l)`; o sentido, o que passa pelo ápice; `hint` desempata o
/// meio-círculo exacto.
fn circle_through(s: P2, e: P2, h: f64, hint: f64) -> ArcShape {
    let chord = sub(e, s);
    let l = chord[0].hypot(chord[1]);
    let n = mul(rotate(chord, std::f64::consts::FRAC_PI_2), 1.0 / l);
    let r = (l * l / 4.0 + h * h) / (2.0 * h.abs());
    let top = add(mul(add(s, e), 0.5), mul(n, h));
    let centre = sub(top, mul(n, sign(h) * r));
    let theta = 4.0 * (2.0 * h.abs() / l).atan();
    let start = angle(sub(s, centre));
    let arc = |sweep: f64| ArcShape {
        centre,
        rx: r,
        ry: r,
        rot: 0.0,
        start,
        sweep,
    };
    let miss = |sweep: f64| dist(arc(sweep).at(start + sweep / 2.0), top);
    let (pos, neg) = (miss(theta), miss(-theta));
    let sweep = if (pos - neg).abs() <= 1e-12 * l {
        theta * sign(hint)
    } else if pos < neg {
        theta
    } else {
        -theta
    };
    arc(sweep)
}

/// `p ↦ lin·p + t`.
struct Affine {
    lin: [[f64; 2]; 2],
    t: P2,
}

fn apply(m: &[[f64; 2]; 2], p: P2) -> P2 {
    [
        m[0][0] * p[0] + m[0][1] * p[1],
        m[1][0] * p[0] + m[1][1] * p[1],
    ]
}

impl Affine {
    fn point(&self, p: P2) -> P2 {
        add(apply(&self.lin, p), self.t)
    }

    /// A imagem EXACTA de um arco de elipse (uma afinidade leva elipses a elipses): `J = lin·R·diag`
    /// decomposto em `R(φ)·diag(sx, sy)·R(θ)` (a SVD fechada de 2×2); o parâmetro anda `θ`, e um
    /// espelho (`sy < 0`) inverte o sentido.
    fn arc(&self, a: &ArcShape) -> ArcShape {
        let (c, s) = (a.rot.cos(), a.rot.sin());
        let m = self.lin;
        let j = [
            [
                (m[0][0] * c + m[0][1] * s) * a.rx,
                (-m[0][0] * s + m[0][1] * c) * a.ry,
            ],
            [
                (m[1][0] * c + m[1][1] * s) * a.rx,
                (-m[1][0] * s + m[1][1] * c) * a.ry,
            ],
        ];
        let (e, f) = ((j[0][0] + j[1][1]) / 2.0, (j[0][0] - j[1][1]) / 2.0);
        let (g, h) = ((j[1][0] + j[0][1]) / 2.0, (j[1][0] - j[0][1]) / 2.0);
        let (q, r) = (e.hypot(h), f.hypot(g));
        let (a1, a2) = (g.atan2(f), h.atan2(e));
        let (theta, phi) = ((a2 - a1) / 2.0, (a2 + a1) / 2.0);
        let (sx, sy) = (q + r, q - r);
        let (start, sweep) = if sy < 0.0 {
            (-(a.start + theta), -a.sweep)
        } else {
            (a.start + theta, a.sweep)
        };
        ArcShape {
            centre: self.point(a.centre),
            rx: sx,
            ry: sy.abs(),
            rot: phi,
            start,
            sweep,
        }
    }
}

#[cfg(test)]
#[path = "quickshape_edit_tests.rs"]
mod tests;
