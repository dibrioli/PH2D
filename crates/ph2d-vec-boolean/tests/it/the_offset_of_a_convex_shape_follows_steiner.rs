//! **Steiner** — o Offset para FORA de uma forma convexa tem área fechada:
//! `A + L·d + d²·(θ/2 + Σ c(φ))`, com `θ` a curva das partes lisas, `φ` a curva de cada quina e
//! `c = tan(φ/2)` no Miter, `sen(φ)/2` no Bevel. Sem grelha e sem juiz: a conta é a verdade.
//!
//! Prende as duas curas do BUGS #39 no Offset Bevel/Miter: a reta vizinha de uma quina curva
//! segue a TANGENTE (`expand_corner.rs`), e um ponto a `1e-17` do anterior não vira aresta com
//! direção de ruído (`expand_band.rs`, `polylines`).

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use kurbo::{Affine, ParamCurveArclen, Point, Vec2};
use ph2d_vec_boolean::{area, offset_path};
use ph2d_vec_scene::{LineJoin, OffsetSide, ShapeKind, VecPath, VecVertex, cook};

use super::raster::vecpath_bez;

/// Um vértice com handles absolutos.
fn vx(a: Point, i: Point, o: Point) -> VecVertex {
    VecVertex {
        in_handle: [i.x, i.y],
        out_handle: [o.x, o.y],
        ..VecVertex::corner([a.x, a.y])
    }
}

/// O sector de disco de abertura `span` (raio `1`, centro na origem) — com `span = π` é o meio
/// disco; a corda fecha-o. Arcos de no máximo 90° por cúbica.
fn sector(span: f64, with_centre: bool) -> Vec<VecVertex> {
    let n = (span / FRAC_PI_2).ceil() as usize;
    let step = span / n as f64;
    let k = 4.0 / 3.0 * (step / 4.0).tan();
    let at = |a: f64| Point::new(a.cos(), a.sin());
    let tan = |a: f64| Vec2::new(-a.sin(), a.cos());
    let mut v: Vec<VecVertex> = (0..=n)
        .map(|j| {
            let a = step * j as f64;
            let p = at(a);
            let i = if j == 0 { p } else { p - tan(a) * k };
            let o = if j == n { p } else { p + tan(a) * k };
            vx(p, i, o)
        })
        .collect();
    if with_centre {
        v.push(VecVertex::corner([0.0, 0.0]));
    }
    v
}

fn turned(verts: Vec<VecVertex>, deg: f64) -> VecPath {
    let t = Affine::rotate(deg.to_radians());
    let m = |p: [f64; 2]| {
        let q = t * Point::new(p[0], p[1]);
        [q.x, q.y]
    };
    VecPath {
        verts: verts
            .into_iter()
            .map(|v| VecVertex {
                anchor: m(v.anchor),
                in_handle: m(v.in_handle),
                out_handle: m(v.out_handle),
                ..v
            })
            .collect(),
        closed: true,
        ..VecPath::default()
    }
}

/// `(nome, forma, curvas das quinas)` — as quinas contadas à mão, a curva lisa é o resto de 2π.
fn corpus() -> Vec<(&'static str, VecPath, Vec<f64>)> {
    let segment_120 = 2.0 * PI / 3.0;
    vec![
        (
            "elipse",
            cook(ShapeKind::Ellipse, [0.0, 0.0], [2.0, 1.3], &[]),
            vec![],
        ),
        // O círculo do catálogo: o `flatten` emite nele um ponto a `1e-17` do fim de uma curva.
        (
            "círculo",
            cook(ShapeKind::Ellipse, [0.0, 0.0], [2.0, 2.0], &[]),
            vec![],
        ),
        (
            "meio disco",
            turned(sector(PI, false), 17.0),
            vec![FRAC_PI_2; 2],
        ),
        (
            "quarto de disco",
            turned(sector(FRAC_PI_2, true), 17.0),
            vec![FRAC_PI_2; 3],
        ),
        (
            "segmento de 120°",
            turned(sector(segment_120, false), 17.0),
            // A corda faz com a tangente metade do ângulo central: a curva é π − α/2.
            vec![PI - segment_120 / 2.0; 2],
        ),
    ]
}

fn perimeter(p: &VecPath) -> f64 {
    vecpath_bez(p).segments().map(|s| s.arclen(1e-12)).sum()
}

fn steiner(p: &VecPath, corners: &[f64], d: f64, join: LineJoin) -> f64 {
    let smooth = TAU - corners.iter().sum::<f64>();
    let c: f64 = corners
        .iter()
        .map(|&phi| match join {
            LineJoin::Miter => (phi / 2.0).tan(),
            _ => phi.sin() / 2.0,
        })
        .sum();
    area(p) + perimeter(p) * d + d * d * (smooth / 2.0 + c)
}

/// A tolerância de achatamento do Offset (`expand::REL_TOL`, relativa à diagonal da forma).
const FLATTEN_REL: f64 = 1e-4;

/// O que o achatamento PODE perder: cada corda fica por dentro do arco a no máximo `tol`, e o
/// segmento parabólico entre as duas tem `⅔·tol·corda` — sobre a borda da forma e a do offset.
fn flatten_bound(p: &VecPath, corners: &[f64], d: f64) -> f64 {
    let b = kurbo::Shape::bounding_box(&vecpath_bez(p));
    let tol = b.width().hypot(b.height()) * FLATTEN_REL;
    let l = perimeter(p);
    let smooth = TAU - corners.iter().sum::<f64>();
    2.0 / 3.0 * tol * (2.0 * l + smooth * d)
}

/// `(caso, erro relativo, limite relativo)` de cada caso.
fn errors() -> Vec<(String, f64, f64)> {
    let mut out = Vec::new();
    for (name, p, corners) in corpus() {
        for join in [LineJoin::Miter, LineJoin::Bevel] {
            for d in [0.02, 0.1, 0.4, 1.0] {
                let want = steiner(&p, &corners, d, join);
                let got: f64 = offset_path(&p, d, join, OffsetSide::Both)
                    .iter()
                    .map(area)
                    .sum();
                let bound = flatten_bound(&p, &corners, d) / want;
                out.push((format!("{name} {join:?} d={d}"), (got - want) / want, bound));
            }
        }
    }
    out
}

/// Medido (11/10): com as curas o pior é `0,033 %` (o achatamento, sempre para menos); sem a
/// tangente honesta o Bevel perde `0,10–0,44 %` a `d ≥ 0,4` (meio disco, quarto, segmento).
#[test]
fn the_offset_of_a_convex_shape_follows_steiner() {
    let bad: Vec<String> = errors()
        .into_iter()
        .filter(|(_, e, bound)| e.abs() > *bound)
        .map(|(n, e, b)| format!("{n}: {:+.4} % (limite {:.4} %)", 100.0 * e, 100.0 * b))
        .collect();
    assert!(
        bad.is_empty(),
        "o Offset saiu da conta de Steiner:\n{}",
        bad.join("\n")
    );
}

#[test]
#[ignore = "sonda: o erro de cada caso"]
fn prints_the_steiner_errors() {
    for (n, e, b) in errors() {
        println!("{n}\t{:+.5} %\tlimite {:.5} %", 100.0 * e, 100.0 * b);
    }
}
