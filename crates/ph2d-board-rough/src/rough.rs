//! O gerador e o desenhador do **rough.js 4.6.4** (`generator.js` + `renderer.js`), portados.
//!
//! Cada função pública é um método do `RoughGenerator` e devolve os mesmos `sets` (tipo + ops). A
//! aleatoriedade é UMA [`Random`] por chamada, partilhada por tudo o que a chamada desenha (o
//! `o.randomizer` do JS, que as cópias por `Object.assign` levam consigo); só a 2.ª passada de uma
//! curva usa outra, de semente `seed + 1` (`cloneOptionsAlterSeed`).

use std::f64::consts::PI;

use crate::curve_points::{curve_to_bezier, normalize, points_on_bezier_curves, points_on_path};
use crate::{P, Random, fill};

/// Como se preenche uma forma fechada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FillStyle {
    /// Riscas paralelas (o idioma do rough.js e do Excalidraw).
    Hachure,
    /// O contorno tremido, cheio.
    Solid,
    /// Riscas nos dois sentidos.
    CrossHatch,
}

/// As opções do rough.js que mudam a GEOMETRIA (as de cor ficam com quem desenha).
#[derive(Clone, Debug)]
pub struct Options {
    pub max_randomness_offset: f64,
    pub roughness: f64,
    pub bowing: f64,
    pub stroke_width: f64,
    pub curve_tightness: f64,
    pub curve_fitting: f64,
    pub curve_step_count: f64,
    /// `true` = há preenchimento (`o.fill` definido no JS).
    pub fill: bool,
    pub fill_style: FillStyle,
    pub hachure_angle: f64,
    /// `< 0` = `4 × stroke_width`.
    pub hachure_gap: f64,
    pub seed: u32,
    pub disable_multi_stroke: bool,
    pub disable_multi_stroke_fill: bool,
    pub preserve_vertices: bool,
    pub fill_shape_roughness_gain: f64,
    /// `false` = `stroke: 'none'` (só o preenchimento entra nos sets).
    pub stroke: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_randomness_offset: 2.0,
            roughness: 1.0,
            bowing: 1.0,
            stroke_width: 1.0,
            curve_tightness: 0.0,
            curve_fitting: 0.95,
            curve_step_count: 9.0,
            fill: false,
            fill_style: FillStyle::Hachure,
            hachure_angle: -41.0,
            hachure_gap: -1.0,
            seed: 0,
            disable_multi_stroke: false,
            disable_multi_stroke_fill: false,
            preserve_vertices: false,
            fill_shape_roughness_gain: 0.8,
            stroke: true,
        }
    }
}

/// Uma operação de desenho (`move` · `lineTo` · `bcurveTo`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Op {
    Move(P),
    Line(P),
    Cubic(P, P, P),
}

/// O que um set é: traço, preenchimento cheio, ou riscas que se TRAÇAM com a cor do preenchimento.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetKind {
    Path,
    FillPath,
    FillSketch,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpSet {
    pub kind: SetKind,
    pub ops: Vec<Op>,
}

/// Um segmento de caminho ABSOLUTO (o `d` de um SVG depois do `absolutize`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Seg {
    M(P),
    L(P),
    Q(P, P),
    C(P, P, P),
    Z,
}

fn path_set(ops: Vec<Op>) -> OpSet {
    OpSet {
        kind: SetKind::Path,
        ops,
    }
}

fn fill_roughness(o: &Options) -> Options {
    Options {
        disable_multi_stroke: true,
        roughness: if o.roughness != 0.0 {
            o.roughness + o.fill_shape_roughness_gain
        } else {
            0.0
        },
        ..o.clone()
    }
}

/// `RoughGenerator.line`.
pub fn line(x1: f64, y1: f64, x2: f64, y2: f64, o: &Options) -> Vec<OpSet> {
    let r = &mut Random::new(o.seed);
    vec![path_set(double_line(x1, y1, x2, y2, o, r, false))]
}

/// `RoughGenerator.rectangle` (`x, y` = canto de cima à esquerda).
pub fn rectangle(x: f64, y: f64, width: f64, height: f64, o: &Options) -> Vec<OpSet> {
    let r = &mut Random::new(o.seed);
    let points = vec![
        [x, y],
        [x + width, y],
        [x + width, y + height],
        [x, y + height],
    ];
    let outline = linear_path_ops(&points, true, o, r);
    closed_fill_then_stroke(vec![points], outline, o, r)
}

/// `RoughGenerator.polygon`.
pub fn polygon(points: &[P], o: &Options) -> Vec<OpSet> {
    let r = &mut Random::new(o.seed);
    let outline = linear_path_ops(points, true, o, r);
    closed_fill_then_stroke(vec![points.to_vec()], outline, o, r)
}

fn closed_fill_then_stroke(
    polys: Vec<Vec<P>>,
    outline: Vec<Op>,
    o: &Options,
    r: &mut Random,
) -> Vec<OpSet> {
    let mut sets = Vec::new();
    if o.fill {
        sets.push(if o.fill_style == FillStyle::Solid {
            solid_fill_polygon(&polys, o, r)
        } else {
            fill::pattern(polys, o, r)
        });
    }
    if o.stroke {
        sets.push(path_set(outline));
    }
    sets
}

/// `RoughGenerator.linearPath` (aberto).
pub fn linear_path(points: &[P], o: &Options) -> Vec<OpSet> {
    let r = &mut Random::new(o.seed);
    vec![path_set(linear_path_ops(points, false, o, r))]
}

/// `RoughGenerator.ellipse` (`x, y` = CENTRO).
pub fn ellipse(x: f64, y: f64, width: f64, height: f64, o: &Options) -> Vec<OpSet> {
    let r = &mut Random::new(o.seed);
    let params = ellipse_params(width, height, o, r);
    let (estimated, outline) = ellipse_with_params(x, y, o, params, r);
    let mut sets = Vec::new();
    if o.fill {
        if o.fill_style == FillStyle::Solid {
            let (_, ops) = ellipse_with_params(x, y, o, params, r);
            sets.push(OpSet {
                kind: SetKind::FillPath,
                ops,
            });
        } else {
            sets.push(fill::pattern(vec![estimated], o, r));
        }
    }
    if o.stroke {
        sets.push(path_set(outline));
    }
    sets
}

/// `RoughGenerator.curve` (uma Catmull-Rom pelos pontos).
pub fn curve(points: &[P], o: &Options) -> Vec<OpSet> {
    let r = &mut Random::new(o.seed);
    let outline = curve_ops(points, o, r);
    let mut sets = Vec::new();
    if o.fill && points.len() >= 3 {
        if o.fill_style == FillStyle::Solid {
            let ops = curve_ops(points, &fill_roughness(o), r);
            sets.push(OpSet {
                kind: SetKind::FillPath,
                ops: merged_shape(ops),
            });
        } else {
            let bcurve = curve_to_bezier(points, 0.0);
            let poly = points_on_bezier_curves(&bcurve, 10.0, (1.0 + o.roughness) / 2.0);
            sets.push(fill::pattern(vec![poly], o, r));
        }
    }
    if o.stroke {
        sets.push(path_set(outline));
    }
    sets
}

/// `RoughGenerator.path`.
pub fn path(segs: &[Seg], o: &Options) -> Vec<OpSet> {
    let r = &mut Random::new(o.seed);
    let mut sets = Vec::new();
    if segs.is_empty() {
        return sets;
    }
    let normalized = normalize(segs);
    let polys = points_on_path(&normalized, 1.0, (1.0 + o.roughness) / 2.0);
    let shape = svg_path_ops(&normalized, o, r);
    if o.fill {
        if o.fill_style == FillStyle::Solid {
            if polys.len() == 1 {
                let ops = svg_path_ops(&normalized, &fill_roughness(o), r);
                sets.push(OpSet {
                    kind: SetKind::FillPath,
                    ops: merged_shape(ops),
                });
            } else {
                sets.push(solid_fill_polygon(&polys, o, r));
            }
        } else {
            sets.push(fill::pattern(polys, o, r));
        }
    }
    if o.stroke {
        sets.push(path_set(shape));
    }
    sets
}

fn merged_shape(ops: Vec<Op>) -> Vec<Op> {
    ops.into_iter()
        .enumerate()
        .filter(|(i, op)| *i == 0 || !matches!(op, Op::Move(_)))
        .map(|(_, op)| op)
        .collect()
}

fn offset(min: f64, max: f64, o: &Options, r: &mut Random, gain: f64) -> f64 {
    o.roughness * gain * ((r.next_unit() * (max - min)) + min)
}

fn offset_opt(x: f64, o: &Options, r: &mut Random, gain: f64) -> f64 {
    offset(-x, x, o, r, gain)
}

pub(crate) fn double_line(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    o: &Options,
    r: &mut Random,
    filling: bool,
) -> Vec<Op> {
    let single = if filling {
        o.disable_multi_stroke_fill
    } else {
        o.disable_multi_stroke
    };
    let mut ops = line_ops(x1, y1, x2, y2, o, r, false);
    if !single {
        ops.extend(line_ops(x1, y1, x2, y2, o, r, true));
    }
    ops
}

fn line_ops(
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    o: &Options,
    r: &mut Random,
    overlay: bool,
) -> Vec<Op> {
    let length_sq = (x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2);
    let length = length_sq.sqrt();
    let gain = if length < 200.0 {
        1.0
    } else if length > 500.0 {
        0.4
    } else {
        (-0.0016668) * length + 1.233334
    };
    let mut off = o.max_randomness_offset;
    if off * off * 100.0 > length_sq {
        off = length / 10.0;
    }
    let half = off / 2.0;
    let diverge = 0.2 + r.next_unit() * 0.2;
    let mid_x = o.bowing * o.max_randomness_offset * (y2 - y1) / 200.0;
    let mid_y = o.bowing * o.max_randomness_offset * (x1 - x2) / 200.0;
    let mid_x = offset_opt(mid_x, o, r, gain);
    let mid_y = offset_opt(mid_y, o, r, gain);
    let keep = o.preserve_vertices;
    let amount = if overlay { half } else { off };
    let jitter = |r: &mut Random| offset_opt(amount, o, r, gain);
    let start = [
        x1 + if keep { 0.0 } else { jitter(r) },
        y1 + if keep { 0.0 } else { jitter(r) },
    ];
    let c1 = [
        mid_x + x1 + (x2 - x1) * diverge + jitter(r),
        mid_y + y1 + (y2 - y1) * diverge + jitter(r),
    ];
    let c2 = [
        mid_x + x1 + 2.0 * (x2 - x1) * diverge + jitter(r),
        mid_y + y1 + 2.0 * (y2 - y1) * diverge + jitter(r),
    ];
    let end = [
        x2 + if keep { 0.0 } else { jitter(r) },
        y2 + if keep { 0.0 } else { jitter(r) },
    ];
    vec![Op::Move(start), Op::Cubic(c1, c2, end)]
}

fn linear_path_ops(points: &[P], close: bool, o: &Options, r: &mut Random) -> Vec<Op> {
    let len = points.len();
    if len > 2 {
        let mut ops = Vec::new();
        for w in points.windows(2) {
            ops.extend(double_line(w[0][0], w[0][1], w[1][0], w[1][1], o, r, false));
        }
        if close {
            let (a, b) = (points[len - 1], points[0]);
            ops.extend(double_line(a[0], a[1], b[0], b[1], o, r, false));
        }
        ops
    } else if len == 2 {
        double_line(
            points[0][0],
            points[0][1],
            points[1][0],
            points[1][1],
            o,
            r,
            false,
        )
    } else {
        Vec::new()
    }
}

fn curve_ops(points: &[P], o: &Options, r: &mut Random) -> Vec<Op> {
    let mut ops = curve_with_offset(points, 1.0 * (1.0 + o.roughness * 0.2), o, r);
    if !o.disable_multi_stroke {
        let altered = &mut Random::new(o.seed.wrapping_add(1));
        ops.extend(curve_with_offset(
            points,
            1.5 * (1.0 + o.roughness * 0.22),
            o,
            altered,
        ));
    }
    ops
}

fn curve_with_offset(points: &[P], off: f64, o: &Options, r: &mut Random) -> Vec<Op> {
    let jittered = |p: P, r: &mut Random| {
        [
            p[0] + offset_opt(off, o, r, 1.0),
            p[1] + offset_opt(off, o, r, 1.0),
        ]
    };
    let mut ps = vec![jittered(points[0], r), jittered(points[0], r)];
    for i in 1..points.len() {
        ps.push(jittered(points[i], r));
        if i == points.len() - 1 {
            ps.push(jittered(points[i], r));
        }
    }
    catmull_rom(&ps, o, r)
}

/// O `_curve` do rough.js (sem `closePoint`, que nenhum caminho daqui usa).
fn catmull_rom(points: &[P], o: &Options, r: &mut Random) -> Vec<Op> {
    let len = points.len();
    let mut ops = Vec::new();
    if len > 3 {
        let s = 1.0 - o.curve_tightness;
        ops.push(Op::Move(points[1]));
        let mut i = 1;
        while i + 2 < len {
            let v = points[i];
            let b1 = [
                v[0] + (s * points[i + 1][0] - s * points[i - 1][0]) / 6.0,
                v[1] + (s * points[i + 1][1] - s * points[i - 1][1]) / 6.0,
            ];
            let b2 = [
                points[i + 1][0] + (s * points[i][0] - s * points[i + 2][0]) / 6.0,
                points[i + 1][1] + (s * points[i][1] - s * points[i + 2][1]) / 6.0,
            ];
            ops.push(Op::Cubic(b1, b2, points[i + 1]));
            i += 1;
        }
    } else if len == 3 {
        ops.push(Op::Move(points[1]));
        ops.push(Op::Cubic(points[1], points[2], points[2]));
    } else if len == 2 {
        ops.extend(double_line(
            points[0][0],
            points[0][1],
            points[1][0],
            points[1][1],
            o,
            r,
            false,
        ));
    }
    ops
}

#[derive(Clone, Copy)]
struct EllipseParams {
    increment: f64,
    rx: f64,
    ry: f64,
}

fn ellipse_params(width: f64, height: f64, o: &Options, r: &mut Random) -> EllipseParams {
    let hw = width / 2.0;
    let hh = height / 2.0;
    let psq = (PI * 2.0 * ((hw * hw + hh * hh) / 2.0).sqrt()).sqrt();
    let step_count = o
        .curve_step_count
        .max((o.curve_step_count / 200f64.sqrt()) * psq)
        .ceil();
    let increment = (PI * 2.0) / step_count;
    let mut rx = hw.abs();
    let mut ry = hh.abs();
    let fit = 1.0 - o.curve_fitting;
    rx += offset_opt(rx * fit, o, r, 1.0);
    ry += offset_opt(ry * fit, o, r, 1.0);
    EllipseParams { increment, rx, ry }
}

fn ellipse_with_params(
    x: f64,
    y: f64,
    o: &Options,
    p: EllipseParams,
    r: &mut Random,
) -> (Vec<P>, Vec<Op>) {
    let inner = offset(0.4, 1.0, o, r, 1.0);
    let overlap = p.increment * offset(0.1, inner, o, r, 1.0);
    let (ap1, cp1) = ellipse_points(p, x, y, 1.0, overlap, o, r);
    let mut ops = catmull_rom(&ap1, o, r);
    if !o.disable_multi_stroke && o.roughness != 0.0 {
        let (ap2, _) = ellipse_points(p, x, y, 1.5, 0.0, o, r);
        ops.extend(catmull_rom(&ap2, o, r));
    }
    (cp1, ops)
}

fn ellipse_points(
    p: EllipseParams,
    cx: f64,
    cy: f64,
    off: f64,
    overlap: f64,
    o: &Options,
    r: &mut Random,
) -> (Vec<P>, Vec<P>) {
    let EllipseParams { increment, rx, ry } = p;
    let mut core = Vec::new();
    let mut all = Vec::new();
    if o.roughness == 0.0 {
        let increment = increment / 4.0;
        all.push([cx + rx * (-increment).cos(), cy + ry * (-increment).sin()]);
        let mut angle = 0.0;
        while angle <= PI * 2.0 {
            let q = [cx + rx * angle.cos(), cy + ry * angle.sin()];
            core.push(q);
            all.push(q);
            angle += increment;
        }
        all.push([cx + rx * 0f64.cos(), cy + ry * 0f64.sin()]);
        all.push([cx + rx * increment.cos(), cy + ry * increment.sin()]);
    } else {
        let jit = |r: &mut Random| offset_opt(off, o, r, 1.0);
        let rad = offset_opt(0.5, o, r, 1.0) - (PI / 2.0);
        all.push([
            jit(r) + cx + 0.9 * rx * (rad - increment).cos(),
            jit(r) + cy + 0.9 * ry * (rad - increment).sin(),
        ]);
        let end = PI * 2.0 + rad - 0.01;
        let mut angle = rad;
        while angle < end {
            let q = [
                jit(r) + cx + rx * angle.cos(),
                jit(r) + cy + ry * angle.sin(),
            ];
            core.push(q);
            all.push(q);
            angle += increment;
        }
        all.push([
            jit(r) + cx + rx * (rad + PI * 2.0 + overlap * 0.5).cos(),
            jit(r) + cy + ry * (rad + PI * 2.0 + overlap * 0.5).sin(),
        ]);
        all.push([
            jit(r) + cx + 0.98 * rx * (rad + overlap).cos(),
            jit(r) + cy + 0.98 * ry * (rad + overlap).sin(),
        ]);
        all.push([
            jit(r) + cx + 0.9 * rx * (rad + overlap * 0.5).cos(),
            jit(r) + cy + 0.9 * ry * (rad + overlap * 0.5).sin(),
        ]);
    }
    (all, core)
}

/// O `svgPath` do rough.js sobre um caminho já normalizado (só `M`, `L`, `C`, `Z`).
fn svg_path_ops(segs: &[Seg], o: &Options, r: &mut Random) -> Vec<Op> {
    let mut ops = Vec::new();
    let mut first = [0.0, 0.0];
    let mut current = [0.0, 0.0];
    for seg in segs {
        match *seg {
            Seg::M(p) => {
                current = p;
                first = p;
            }
            Seg::L(p) => {
                ops.extend(double_line(current[0], current[1], p[0], p[1], o, r, false));
                current = p;
            }
            Seg::C(c1, c2, p) => {
                ops.extend(bezier_to(c1, c2, p, current, o, r));
                current = p;
            }
            Seg::Z => {
                ops.extend(double_line(
                    current[0], current[1], first[0], first[1], o, r, false,
                ));
                current = first;
            }
            Seg::Q(..) => unreachable!("normalize converte Q em C"),
        }
    }
    ops
}

fn bezier_to(c1: P, c2: P, p: P, current: P, o: &Options, r: &mut Random) -> Vec<Op> {
    let base = if o.max_randomness_offset != 0.0 {
        o.max_randomness_offset
    } else {
        1.0
    };
    let ros = [base, base + 0.3];
    let iterations = if o.disable_multi_stroke { 1 } else { 2 };
    let keep = o.preserve_vertices;
    let mut ops = Vec::new();
    for (i, &ro) in ros.iter().enumerate().take(iterations) {
        if i == 0 || keep {
            ops.push(Op::Move(current));
        } else {
            let mx = current[0] + offset_opt(ros[0], o, r, 1.0);
            let my = current[1] + offset_opt(ros[0], o, r, 1.0);
            ops.push(Op::Move([mx, my]));
        }
        let f = if keep {
            p
        } else {
            [
                p[0] + offset_opt(ro, o, r, 1.0),
                p[1] + offset_opt(ro, o, r, 1.0),
            ]
        };
        let a = [
            c1[0] + offset_opt(ro, o, r, 1.0),
            c1[1] + offset_opt(ro, o, r, 1.0),
        ];
        let b = [
            c2[0] + offset_opt(ro, o, r, 1.0),
            c2[1] + offset_opt(ro, o, r, 1.0),
        ];
        ops.push(Op::Cubic(a, b, f));
    }
    ops
}

fn solid_fill_polygon(polys: &[Vec<P>], o: &Options, r: &mut Random) -> OpSet {
    let mut ops = Vec::new();
    let off = o.max_randomness_offset;
    for points in polys {
        if points.len() > 2 {
            let jit = |q: P, r: &mut Random| {
                [
                    q[0] + offset_opt(off, o, r, 1.0),
                    q[1] + offset_opt(off, o, r, 1.0),
                ]
            };
            ops.push(Op::Move(jit(points[0], r)));
            for &q in &points[1..] {
                ops.push(Op::Line(jit(q, r)));
            }
        }
    }
    OpSet {
        kind: SetKind::FillPath,
        ops,
    }
}
