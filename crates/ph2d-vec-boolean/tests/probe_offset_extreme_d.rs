//! SONDA (diagnóstico, `--ignored`): o offset nos EXTREMOS do slider (`d = ±4`).
//!
//! O report (2026-07-20, "muda para round mas não para Miter/Bevel") levou o nível 18 a
//! arrastar até saturar o slider — e o app mostrou o donut ANIQUILADO num commit a `d=+4`
//! (crescer!) num run, e um SLIVER ressuscitado pelo Bevel a `d=−4` noutro. Esta sonda
//! pergunta ao MOTOR, determinístico e sem janela: o que CADA join devolve no donut do
//! smoke, numa escada de `d` até os extremos?

use ph2d_vec_boolean::offset_path;
use ph2d_vec_scene::{Contour, LineJoin, OffsetSide, VecPath, VecVertex};

/// O donut do smoke 17/18: retângulo [2.8,-1.2]-[5.2,1.2] com furo quadrado 1.4 em (4,0).
fn donut() -> VecPath {
    let rect = [[2.8, -1.2], [5.2, -1.2], [5.2, 1.2], [2.8, 1.2]]
        .map(VecVertex::corner)
        .to_vec();
    let hole = [[3.3, -0.7], [4.7, -0.7], [4.7, 0.7], [3.3, 0.7]]
        .map(VecVertex::corner)
        .to_vec();
    let mut p = VecPath {
        verts: rect,
        closed: true,
        ..VecPath::default()
    };
    p.subpaths = vec![Contour::new_closed(hole)];
    p.fill_rule = ph2d_vec_scene::FillRule::EvenOdd;
    p
}

#[test]
#[ignore = "sonda de diagnóstico — rode com -- --ignored --nocapture"]
fn probe_every_join_on_the_d_ladder() {
    let src = donut();
    for join in [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel] {
        for d in [
            -4.0, -3.0, -2.0, -1.0, -0.5, -0.26, -0.25, 0.5, 1.0, 2.0, 3.0, 3.9, 4.0,
        ] {
            let out = offset_path(&src, d, join, OffsetSide::Both);
            let verts: usize = out
                .iter()
                .map(|p| p.verts.len() + p.subpaths.iter().map(|c| c.verts.len()).sum::<usize>())
                .sum();
            let area: f64 = out.iter().map(|p| ph2d_vec_boolean::area(p).abs()).sum();
            println!("join={join:?} d={d:+.2} -> paths={} verts={verts} area={area:.4}", out.len());
        }
        println!("---");
    }
}
