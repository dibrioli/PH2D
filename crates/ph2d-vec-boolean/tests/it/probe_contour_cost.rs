//! SONDA (`--ignored`): **quanto custa um Contour de N anéis?**
//!
//! O Contour é `offset_path × N`, e o `offset_live` mediu UM offset em **0,40–1,07 ms**. Esta
//! sonda mede o N inteiro, porque é ele que decide o teto de passos — §0: medir antes de limitar.
//! O número que importa é o de um **arrasto** do slider de passos, que re-coza por frame.

use ph2d_vec_boolean::offset_path;
use ph2d_vec_scene::{Contour, FillRule, LineJoin, OffsetSide, VecPath, VecVertex};

fn poly(pts: &[[f64; 2]]) -> Vec<VecVertex> {
    pts.iter().copied().map(VecVertex::corner).collect()
}

fn closed(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

/// O donut do smoke 17: retângulo com furo quadrado.
fn donut() -> VecPath {
    let mut p = closed(poly(&[[2.8, -1.2], [5.2, -1.2], [5.2, 1.2], [2.8, 1.2]]));
    p.subpaths = vec![Contour::new_closed(poly(&[
        [3.3, -0.7],
        [4.7, -0.7],
        [4.7, 0.7],
        [3.3, 0.7],
    ]))];
    p.fill_rule = FillRule::EvenOdd;
    p
}

/// Uma estrela de 5 pontas — a fixture cara (quinas reentrantes).
fn star() -> VecPath {
    let mut pts = Vec::new();
    for i in 0..10 {
        let a = std::f64::consts::PI * 2.0 * f64::from(i) / 10.0 - std::f64::consts::FRAC_PI_2;
        let r = if i % 2 == 0 { 1.0 } else { 0.42 };
        pts.push([a.cos() * r, a.sin() * r]);
    }
    closed(poly(&pts))
}

#[test]
#[ignore = "sonda: rode com --release -- --ignored"]
fn probe_contour_cost() {
    for (name, shape) in [("donut", donut()), ("estrela", star())] {
        for join in [("Miter", LineJoin::Miter), ("Round", LineJoin::Round)] {
            for steps in [4_usize, 8, 16, 32] {
                let t0 = std::time::Instant::now();
                let mut rings = 0;
                for k in 1..=steps {
                    #[allow(clippy::cast_precision_loss)]
                    let d = 0.30 * (k as f64) / (steps as f64);
                    rings += offset_path(&shape, d, join.1, OffsetSide::Outer).len();
                }
                let dt = t0.elapsed().as_secs_f64() * 1e3;
                let flag = if dt > 8.0 {
                    "  <= PASSA O KILL 8ms"
                } else {
                    ""
                };
                eprintln!(
                    "[sonda] {name:8} {:5} steps {steps:2} -> {rings:3} paths em {dt:6.2} ms{flag}",
                    join.0
                );
            }
        }
    }
}
