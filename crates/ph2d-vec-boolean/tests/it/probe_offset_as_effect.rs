//! SONDA (`--ignored`): **cabe o offset na fronteira de um `PathEffect`?**
//!
//! Um efeito da pilha é `VecPath -> VecPath` ([`ph2d_vec_scene::effect`], invariante 1), e
//! [`offset_path`] devolve `Vec<VecPath>`. Esta sonda mede QUANDO o vetor tem mais de um
//! elemento, e se juntar tudo num compound `EvenOdd` descreve o MESMO conjunto de pontos.

use ph2d_vec_boolean::{area, offset_path};
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

/// DUAS ilhas disjuntas num caminho só — o compound que não é rosquinha.
fn two_islands() -> VecPath {
    let mut p = closed(poly(&[[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]]));
    p.subpaths = vec![Contour::new_closed(poly(&[
        [5.0, 0.0],
        [7.0, 0.0],
        [7.0, 2.0],
        [5.0, 2.0],
    ]))];
    p.fill_rule = FillRule::EvenOdd;
    p
}

/// Um HALTERE: dois bulbos ligados por um pescoço fino. Encolher além do pescoço PARTE a
/// forma em duas — é o caso em que um path de entrada vira dois de saída.
fn dumbbell() -> VecPath {
    closed(poly(&[
        [0.0, 0.0],
        [2.0, 0.0],
        [2.0, 0.9],
        [5.0, 0.9],
        [5.0, 0.0],
        [7.0, 0.0],
        [7.0, 2.0],
        [5.0, 2.0],
        [5.0, 1.1],
        [2.0, 1.1],
        [2.0, 2.0],
        [0.0, 2.0],
    ]))
}

fn star() -> VecPath {
    let mut v = Vec::new();
    for i in 0..10 {
        let a = std::f64::consts::PI * f64::from(i) / 5.0 - std::f64::consts::FRAC_PI_2;
        let r = if i % 2 == 0 { 1.2 } else { 0.55 };
        v.push([a.cos() * r, a.sin() * r]);
    }
    closed(poly(&v))
}

/// Junta N caminhos num compound `EvenOdd` — a tradução candidata para a fronteira do efeito.
///
/// Legítima porque tudo o que sai do sweep está regularizado: um ponto está no conjunto sse
/// um número ÍMPAR de contornos o cerca, qualquer que seja o aninhamento.
fn merge_even_odd(paths: &[VecPath]) -> Option<VecPath> {
    let mut it = paths.iter();
    let first = it.next()?;
    let mut out = first.clone();
    out.fill_rule = FillRule::EvenOdd;
    for p in it {
        out.subpaths.push(Contour {
            verts: p.verts.clone(),
            closed: p.closed,
        });
        out.subpaths.extend(p.subpaths.iter().cloned());
    }
    Some(out)
}

#[test]
#[ignore = "sonda de diagnóstico — rode com -- --ignored --nocapture"]
fn probe_how_many_paths_an_offset_returns() {
    let cases: [(&str, VecPath); 5] = [
        ("donut", donut()),
        ("two_islands", two_islands()),
        ("dumbbell", dumbbell()),
        ("star", star()),
        (
            "square",
            closed(poly(&[[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]])),
        ),
    ];
    let mut worst = 1usize;
    for (name, src) in &cases {
        for side in [OffsetSide::Outer, OffsetSide::Inner, OffsetSide::Both] {
            for join in [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel] {
                for d in [-0.8, -0.6, -0.45, -0.3, -0.1, 0.1, 0.3, 0.6, 1.0] {
                    let out = offset_path(src, d, join, side);
                    if out.len() > 1 {
                        worst = worst.max(out.len());
                        println!(
                            "{name} side={side:?} join={join:?} d={d:+.2} -> PATHS={}",
                            out.len()
                        );
                    }
                }
            }
        }
    }
    println!("pior caso de paths devolvidos: {worst}");
}

/// O compound `EvenOdd` descreve o MESMO conjunto que os N caminhos? Oráculo de APARÊNCIA:
/// uma grade densa de pontos, cada um perguntado aos dois lados.
#[test]
#[ignore = "sonda de diagnóstico — rode com -- --ignored --nocapture"]
fn probe_merging_the_paths_keeps_the_same_point_set() {
    let cases: [(&str, VecPath); 3] = [
        ("two_islands", two_islands()),
        ("dumbbell", dumbbell()),
        ("donut", donut()),
    ];
    for (name, src) in &cases {
        for d in [-0.6, -0.45, -0.3, 0.3, 0.6] {
            let out = offset_path(src, d, LineJoin::Round, OffsetSide::Both);
            let Some(merged) = merge_even_odd(&out) else {
                continue;
            };
            let a_sum: f64 = out.iter().map(|p| area(p).abs()).sum();
            let a_merged = area(&merged).abs();
            println!(
                "{name} d={d:+.2} paths={} area_sum={a_sum:.6} area_merged={a_merged:.6} \
                 delta={:.2e}",
                out.len(),
                (a_sum - a_merged).abs()
            );
        }
    }
}

#[test]
#[ignore = "sonda de PERF — rode com --release -- --ignored --nocapture"]
fn probe_offset_cost_per_call() {
    for (name, src) in [("donut", donut()), ("star", star())] {
        let _ = offset_path(&src, 0.3, LineJoin::Round, OffsetSide::Both);
        for join in [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel] {
            for d in [0.1, 0.3, 0.6, 1.0] {
                let n = 31;
                let mut ts: Vec<f64> = (0..n)
                    .map(|_| {
                        let t0 = std::time::Instant::now();
                        let _ = offset_path(&src, d, join, OffsetSide::Both);
                        t0.elapsed().as_secs_f64() * 1e3
                    })
                    .collect();
                ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
                println!("{name} join={join:?} d={d:+.2} -> {:.3} ms", ts[n / 2]);
            }
        }
    }
}
