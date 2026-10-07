//! O preenchimento por riscas: o `HachureFiller`/`HatchFiller` do rough.js e o `hachureLines` do
//! hachure-fill 0.5.2 (varrimento por linhas de varrer sobre os polígonos rodados).
//!
//! ⚠️ O hachure-fill RODA os polígonos NO SÍTIO e devolve-os rodados de volta — com o erro de ponto
//! flutuante das duas rotações. A 2.ª passada do tracejado cruzado lê esses pontos, não os
//! originais; a porta mantém-no (é o que o oráculo desenha).

use std::cmp::Ordering;

use crate::rough::{FillStyle, Op, OpSet, Options, SetKind, double_line};
use crate::{P, Random};

/// `patternFillPolygons`: riscas (ou riscas cruzadas) traçadas com a cor do preenchimento.
pub(crate) fn pattern(mut polys: Vec<Vec<P>>, o: &Options, r: &mut Random) -> OpSet {
    let mut ops = hachure_ops(&mut polys, o, o.hachure_angle, r);
    if o.fill_style == FillStyle::CrossHatch {
        ops.extend(hachure_ops(&mut polys, o, o.hachure_angle + 90.0, r));
    }
    OpSet {
        kind: SetKind::FillSketch,
        ops,
    }
}

fn hachure_ops(polys: &mut [Vec<P>], o: &Options, hachure_angle: f64, r: &mut Random) -> Vec<Op> {
    let angle = hachure_angle + 90.0;
    let mut gap = o.hachure_gap;
    if gap < 0.0 {
        gap = o.stroke_width * 4.0;
    }
    gap = gap.max(0.1);
    let mut skip = 1.0;
    if o.roughness >= 1.0 && r.next_unit() > 0.7 {
        skip = gap;
    }
    let mut ops = Vec::new();
    for [a, b] in hachure_lines(polys, gap, angle, skip) {
        ops.extend(double_line(a[0], a[1], b[0], b[1], o, r, true));
    }
    ops
}

fn rotate(points: &mut [P], degrees: f64) {
    let angle = (std::f64::consts::PI / 180.0) * degrees;
    let (sin, cos) = (angle.sin(), angle.cos());
    for p in points {
        let [x, y] = *p;
        *p = [(x * cos) - (y * sin), (x * sin) + (y * cos)];
    }
}

/// `hachureLines(polygons, gap, angle, stepOffset)`.
pub(crate) fn hachure_lines(polys: &mut [Vec<P>], gap: f64, angle: f64, step: f64) -> Vec<[P; 2]> {
    let gap = gap.max(0.1);
    if angle != 0.0 {
        for poly in polys.iter_mut() {
            rotate(poly, angle);
        }
    }
    let mut lines = straight_lines(polys, gap, step);
    if angle != 0.0 {
        for poly in polys.iter_mut() {
            rotate(poly, -angle);
        }
        for line in &mut lines {
            rotate(line, -angle);
        }
    }
    lines
}

#[derive(Clone, Copy)]
struct Edge {
    ymin: f64,
    ymax: f64,
    x: f64,
    islope: f64,
}

/// O sinal de `(a − b) / |a − b|` do comparador do JS (`0` quando iguais).
fn sign_cmp(a: f64, b: f64) -> Ordering {
    a.partial_cmp(&b).unwrap_or(Ordering::Equal)
}

fn straight_lines(polys: &[Vec<P>], gap: f64, step: f64) -> Vec<[P; 2]> {
    let mut edges = Vec::new();
    for poly in polys {
        let mut vertices = poly.clone();
        if vertices.first() != vertices.last() {
            vertices.push(vertices[0]);
        }
        if vertices.len() <= 2 {
            continue;
        }
        for w in vertices.windows(2) {
            let (p1, p2) = (w[0], w[1]);
            if p1[1] != p2[1] {
                let ymin = p1[1].min(p2[1]);
                edges.push(Edge {
                    ymin,
                    ymax: p1[1].max(p2[1]),
                    x: if ymin == p1[1] { p1[0] } else { p2[0] },
                    islope: (p2[0] - p1[0]) / (p2[1] - p1[1]),
                });
            }
        }
    }
    edges.sort_by(|a, b| {
        sign_cmp(a.ymin, b.ymin)
            .then(sign_cmp(a.x, b.x))
            .then(sign_cmp(a.ymax, b.ymax))
    });
    let mut lines = Vec::new();
    if edges.is_empty() {
        return lines;
    }
    let mut edges = std::collections::VecDeque::from(edges);
    let mut active: Vec<Edge> = Vec::new();
    let mut y = edges[0].ymin;
    let mut iteration = 0.0f64;
    while !active.is_empty() || !edges.is_empty() {
        while edges.front().is_some_and(|e| e.ymin <= y) {
            active.push(edges.pop_front().expect("front existe"));
        }
        active.retain(|e| e.ymax > y);
        active.sort_by(|a, b| sign_cmp(a.x, b.x));
        if (step != 1.0 || iteration % gap == 0.0) && active.len() > 1 {
            for pair in active.as_chunks::<2>().0 {
                lines.push([[js_round(pair[0].x), y], [js_round(pair[1].x), y]]);
            }
        }
        y += step;
        for e in &mut active {
            e.x += step * e.islope;
        }
        iteration += 1.0;
    }
    lines
}

/// `Math.round`: meio para CIMA (`-2.5 → -2`), não para longe do zero como o `f64::round`.
fn js_round(x: f64) -> f64 {
    let f = x.floor();
    if x - f >= 0.5 { f + 1.0 } else { f }
}
