//! A SONDA DE CUSTO da W0/W2 (plano 30 §8.4) — corre-se em `--release`, com a carga ao lado:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-navmesh --release --example medir
//! ```
//!
//! Imprime, para 10/100/1000 obstáculos numa região de 100×100 m (raio 0,4 m): o MÍNIMO de cinco
//! construções (ms), os polígonos com e sem fusão, e o custo de uma procura (µs, média de 2 000
//! pares) sobre a malha fundida e sobre a crua. ⛔ Nenhum número desta saída vale acima de `load ~5`.

use std::time::Instant;

use ph2d_nav::Polyanya;
use ph2d_navmesh::{Params, Shape, build};

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

fn cena(n: usize, seed: u64) -> Vec<Shape> {
    let mut r = Lcg(seed);
    (0..n)
        .map(|i| {
            let c = [r.next() * 100.0, r.next() * 100.0];
            if i % 3 == 0 {
                Shape::Circle {
                    center: c,
                    radius: 0.2 + r.next() * 1.3,
                }
            } else {
                let (hx, hy) = (0.2 + r.next() * 1.8, 0.2 + r.next() * 1.8);
                let (a, b) = (r.next() * 2.0 - 1.0, r.next() * 2.0 - 1.0);
                let l = (a * a + b * b).sqrt().max(1e-6);
                let (co, si) = (a / l, b / l);
                Shape::Convex(
                    [[-hx, -hy], [hx, -hy], [hx, hy], [-hx, hy]]
                        .iter()
                        .map(|p| [c[0] + p[0] * co - p[1] * si, c[1] + p[0] * si + p[1] * co])
                        .collect(),
                )
            }
        })
        .collect()
}

fn main() {
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("# loadavg: {}", load.trim());
    println!(
        "# n_obst | lados | build_min_ms | aneis | v_aneis | tri | polys | q_fundida_us | q_crua_us"
    );
    let reg = vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]];
    for &n in &[10usize, 100, 1000] {
        for &lados in &[16u32, 32] {
            let obs = cena(n, 42);
            let mut best = f64::INFINITY;
            let mut built = None;
            for _ in 0..5 {
                let t0 = Instant::now();
                let b = build(
                    &reg,
                    &obs,
                    &Params {
                        agent_radius: 0.4,
                        disk_sides: lados,
                        ..Params::default()
                    },
                )
                .expect("constrói");
                best = best.min(t0.elapsed().as_secs_f64() * 1e3);
                built = Some(b);
            }
            let b = built.expect("cinco corridas");
            let crua = build(
                &reg,
                &obs,
                &Params {
                    agent_radius: 0.4,
                    disk_sides: lados,
                    merge: false,
                    ..Params::default()
                },
            )
            .expect("constrói");
            let mut r = Lcg(7);
            let mut pares = Vec::new();
            while pares.len() < 2_000 {
                let a = [r.next() * 100.0, r.next() * 100.0];
                let c = [r.next() * 100.0, r.next() * 100.0];
                if b.mesh.locate(a).is_some() && b.mesh.locate(c).is_some() {
                    pares.push((a, c));
                }
            }
            let mut s = Polyanya::new();
            let t0 = Instant::now();
            let mut soma = 0.0;
            for &(a, c) in &pares {
                soma += s.find_path(&b.mesh, a, c).map(|p| p.length).unwrap_or(0.0);
            }
            let qf = t0.elapsed().as_secs_f64() * 1e6 / pares.len() as f64;
            let t0 = Instant::now();
            let mut soma2 = 0.0;
            for &(a, c) in &pares {
                soma2 += s
                    .find_path(&crua.mesh, a, c)
                    .map(|p| p.length)
                    .unwrap_or(0.0);
            }
            let qc = t0.elapsed().as_secs_f64() * 1e6 / pares.len() as f64;
            assert!(
                (soma - soma2).abs() < 1e-6 * soma.max(1.0),
                "as duas malhas têm de dar os mesmos caminhos"
            );
            println!(
                "{n:>8} | {lados:>5} | {best:>12.3} | {:>5} | {:>7} | {:>5} | {:>5} | {qf:>12.1} | {qc:>9.1}",
                b.stats.rings, b.stats.ring_verts, b.stats.triangles, b.stats.polygons
            );
        }
    }
    println!(
        "# loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
