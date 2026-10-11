//! A volta do motor: o que o `linesweeper` devolve chega INTEIRO ao `VecPath` (BUGS #39).

use kurbo::Shape;
use linesweeper::{BinaryOp, FillRule as LsFillRule};
use ph2d_vec_scene::{ALL_SHAPES, ShapeKind, VecPath, VecVertex, cook};

use crate::{BoolOp, apply, area, binary_grouped, cut_with_line, to_bez};

const OPS: [(BinaryOp, BoolOp); 4] = [
    (BinaryOp::Union, BoolOp::Union),
    (BinaryOp::Intersection, BoolOp::Intersect),
    (BinaryOp::Difference, BoolOp::Subtract),
    (BinaryOp::Xor, BoolOp::Exclude),
];

/// A área que o MOTOR devolveu: a soma assinada dos contornos (os buracos vêm ao contrário).
fn engine_area(a: &VecPath, b: &VecPath, op: BinaryOp) -> f64 {
    binary_grouped(&to_bez(a), &to_bez(b), LsFillRule::NonZero, op)
        .expect("sweep")
        .iter()
        .flatten()
        .map(Shape::area)
        .sum()
}

fn poly(pts: &[[f64; 2]], closed: bool) -> VecPath {
    VecPath {
        verts: pts.iter().map(|&p| VecVertex::corner(p)).collect(),
        closed,
        ..VecPath::default()
    }
}

fn disc() -> VecPath {
    cook(ShapeKind::Ellipse, [-1.0, -1.0], [1.0, 1.0], &[])
}

/// O semiplano `x + y ≤ −c` como um triângulo grande: a reta corta do disco uma calota que,
/// para `c > 1`, cabe num só quarto da circunferência — uma reta e UM arco.
fn half_plane(c: f64) -> VecPath {
    poly(
        &[[-c - 20.0, 20.0], [20.0, -c - 20.0], [-30.0, -30.0]],
        true,
    )
}

/// Uma lente de DUAS âncoras: a reta `(0,0)→(1,0)` e um arco de volta.
fn lens() -> VecPath {
    let mut p = poly(&[[0.0, 0.0], [1.0, 0.0]], true);
    p.verts[1].out_handle = [1.0, 0.5];
    p.verts[0].in_handle = [0.0, 0.5];
    p
}

#[test]
fn nothing_the_engine_returns_is_lost_on_the_way_back() {
    let ellipse = cook(ShapeKind::Ellipse, [1.0, 0.5], [3.0, 2.5], &[]);
    let mut cases: Vec<(String, VecPath, VecPath)> = ALL_SHAPES
        .iter()
        .map(|&k| {
            (
                format!("{k:?} × elipse"),
                cook(k, [0.0, 0.0], [2.0, 2.0], &[]),
            )
        })
        .filter(|(_, p)| p.closed)
        .map(|(n, p)| (n, p, ellipse.clone()))
        .collect();
    for i in 0..10 {
        let c = 0.5 + 0.1 * f64::from(i);
        cases.push((format!("disco × semiplano {c:.1}"), disc(), half_plane(c)));
    }
    cases.push((
        "quadrado × lente".into(),
        poly(&[[-2.0, -2.0], [3.0, -2.0], [3.0, 3.0], [-2.0, 3.0]], true),
        lens(),
    ));
    let mut lost = Vec::new();
    for (name, a, b) in &cases {
        for (lop, op) in OPS {
            let want = engine_area(a, b, lop);
            let got: f64 = apply(a, b, op).iter().map(area).sum();
            if (want - got).abs() > 1e-6 * want.abs().max(1.0) {
                lost.push(format!("{name} {op:?}: motor {want:.5} · VecPath {got:.5}"));
            }
        }
    }
    assert!(
        lost.is_empty(),
        "a volta do motor perdeu área:\n{}",
        lost.join("\n")
    );
}

/// O corte por uma reta perto da borda: as peças somam o disco — a ponta pequena não some.
/// `c > 1`: a calota é uma reta e um arco. Fora de `c = 1` (a reta pelas âncoras), onde o próprio
/// motor erra a área em até `5e-4` (BUGS #39, medido e aceite).
#[test]
fn a_cut_near_the_rim_keeps_the_small_piece() {
    for i in 0..7 {
        let c = 0.55 + 0.1 * f64::from(i);
        let line = poly(&[[-c - 3.0, 3.0], [3.0, -c - 3.0]], false);
        let pieces = cut_with_line(&disc(), &line).expect("o corte atravessa o disco");
        let sum: f64 = pieces.iter().map(area).sum();
        assert!(
            // O motor erra ~1e-6 a 0,05 da âncora; a menor calota aqui tem 0,15.
            (sum - area(&disc())).abs() < 1e-4,
            "c {c:.2}: {} peças somam {sum:.9}, o disco tem {:.9}",
            pieces.len(),
            area(&disc())
        );
    }
}

/// O erro do PRÓPRIO motor quando a reta passa perto de uma âncora do arco: ele cresce como
/// `√(eps·L)` (absoluto) e pedir um `eps` menor parte o `linesweeper` (BUGS #39). Correr ao
/// subir o `linesweeper`.
#[test]
#[ignore = "sonda: o erro do motor perto de uma âncora"]
fn measures_the_engine_error_near_an_anchor() {
    for scale in [1.0, 100.0] {
        let disc = cook(ShapeKind::Ellipse, [-scale, -scale], [scale, scale], &[]);
        let whole = to_bez(&disc).area();
        let worst = (-12..=12)
            .map(|k| {
                let c = scale * (1.0 + 1e-4 * f64::from(k));
                let s = 20.0 * scale;
                let hp = poly(&[[-c - s, s], [s, -c - s], [-1.5 * s, -1.5 * s]], true);
                let part = |op| -> f64 {
                    binary_grouped(&to_bez(&disc), &to_bez(&hp), LsFillRule::NonZero, op)
                        .expect("sweep")
                        .iter()
                        .flatten()
                        .map(Shape::area)
                        .sum()
                };
                (part(BinaryOp::Difference) + part(BinaryOp::Intersection) - whole).abs()
            })
            .fold(0.0f64, f64::max);
        eprintln!(
            "raio {scale}: pior erro de área {worst:.3e} ({:.3e} da área)",
            worst / whole
        );
    }
}
