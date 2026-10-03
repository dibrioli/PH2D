//! A SONDA DA W6 (plano 30 §2.4, §4 linha W6) — *quanto custa refazer a malha quando UMA porta muda?*
//! Corre-se em `--release`, com a carga ao lado:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-navmesh --release --example medir_mudanca
//! ```
//!
//! Imprime, para 10/100/1000 obstáculos numa região de 100×100 m (raio 0,4 m, a cena da `medir`),
//! o MÍNIMO de cinco construções partido pelas fases (recuo · união · diferença · triangulação ·
//! fusão · a `NavMesh`), e a [`TiledMesh`] a `5/10/20/25 m`: a frio, UMA porta a aparecer e a sumir
//! (o mínimo de dez), quantos mosaicos ela refez, e o custo de uma procura na malha montada contra a
//! inteira — a alternativa do §2.4 se a construção inteira não couber no tique.
//! ⛔ Nenhum número desta saída vale acima de `load ~5`.

use std::time::Instant;

use clipper2_rust::{FillRule, Path64, Paths64, Point64, difference_64, union_subjects_64};
use ph2d_nav::Polyanya;
use ph2d_nav::{NavMesh, V2};
use ph2d_navmesh::{
    Corner, DISK_SIDES, Params, Shape, TiledMesh, build, inflate, lattice, triangulate,
};

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

/// A MESMA cena da `medir` (semente, formas e tamanhos), num quadrado de `lado` m.
fn cena(n: usize, seed: u64, lado: f64) -> Vec<Shape> {
    let mut r = Lcg(seed);
    (0..n)
        .map(|i| {
            let c = [r.next() * lado, r.next() * lado];
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

fn to_path(ring: &[lattice::P]) -> Path64 {
    ring.iter().map(|&(x, y)| Point64::new(x, y)).collect()
}

/// As fases da `build`, cronometradas (ms): recuo, união, diferença, triangulação, fusão, NavMesh.
fn fases(reg: &[V2], obs: &[Shape], r: f64, encolhe: bool) -> [f64; 6] {
    let mut t = [0.0; 6];
    let mut marca = Instant::now();
    let mut lap = |i: usize, t: &mut [f64; 6]| {
        t[i] = marca.elapsed().as_secs_f64() * 1e3;
        marca = Instant::now();
    };
    let regiao = inflate::inset_region(reg, if encolhe { r } else { 0.0 });
    let holes: Paths64 = obs
        .iter()
        .map(|o| inflate::inflate(o, r, Corner::Round, DISK_SIDES))
        .filter(|ring| ring.len() >= 3)
        .map(|ring| to_path(&ring))
        .collect();
    lap(0, &mut t);
    let merged = union_subjects_64(&holes, FillRule::NonZero);
    lap(1, &mut t);
    let walk = difference_64(&vec![to_path(&regiao)], &merged, FillRule::NonZero);
    lap(2, &mut t);
    let rings: Vec<Vec<lattice::P>> = walk
        .iter()
        .map(|p| p.iter().map(|q| (q.x, q.y)).collect())
        .collect();
    let (pts, tris) = triangulate::triangulate(&rings).expect("triangula");
    lap(3, &mut t);
    let polys = triangulate::merge_convex(&pts, &tris);
    lap(4, &mut t);
    let verts: Vec<V2> = pts.iter().map(|&p| lattice::to_world(p)).collect();
    let _ = NavMesh::from_polygons(verts, polys);
    lap(5, &mut t);
    t
}

fn main() {
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("# loadavg: {}", load.trim());
    let r = 0.4;
    let reg = vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]];
    println!(
        "# n_obst | build_min_ms | recuo | uniao | diferenca | triang | fusao | navmesh   (ms, minimo de 5 por fase)"
    );
    for &n in &[10usize, 100, 1000] {
        let obs = cena(n, 42, 100.0);
        let mut best = f64::INFINITY;
        for _ in 0..5 {
            let t0 = Instant::now();
            let _ = build(
                &reg,
                &obs,
                &Params {
                    agent_radius: r,
                    ..Params::default()
                },
            )
            .expect("constrói");
            best = best.min(t0.elapsed().as_secs_f64() * 1e3);
        }
        let mut f = [f64::INFINITY; 6];
        for _ in 0..5 {
            let g = fases(&reg, &obs, r, true);
            for i in 0..6 {
                f[i] = f[i].min(g[i]);
            }
        }
        println!(
            "{n:>8} | {best:>12.3} | {:>5.2} | {:>5.2} | {:>9.2} | {:>6.2} | {:>5.2} | {:>7.2}",
            f[0], f[1], f[2], f[3], f[4], f[5]
        );
    }

    println!(
        "# MOSAICOS (TiledMesh): a frio, e UMA porta a mudar (um obstaculo de 1 m que aparece e some)"
    );
    println!(
        "# n_obst | T_m | mosaicos | frio_ms | porta_ms | refeitos | polys | q_us | q_inteira_us"
    );
    for &n in &[100usize, 1000] {
        let obs = cena(n, 42, 100.0);
        let p = Params {
            agent_radius: r,
            ..Params::default()
        };
        let inteira = build(&reg, &obs, &p).expect("constrói").mesh;
        let mut com = obs.clone();
        com.push(Shape::Convex(vec![
            [50.2, 50.2],
            [51.2, 50.2],
            [51.2, 50.6],
            [50.2, 50.6],
        ]));
        for &tm in &[5.0f64, 10.0, 15.0, 20.0, 25.0, 33.4, 50.0] {
            let mut frio = f64::INFINITY;
            for _ in 0..5 {
                let mut t = TiledMesh::new(p, tm);
                let t0 = Instant::now();
                t.update(&reg, &obs);
                frio = frio.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            let mut t = TiledMesh::new(p, tm);
            t.update(&reg, &obs);
            let mut porta = f64::INFINITY;
            let mut refeitos = 0;
            for k in 0..10 {
                let entrada = if k % 2 == 0 { &com } else { &obs };
                let t0 = Instant::now();
                t.update(&reg, entrada);
                porta = porta.min(t0.elapsed().as_secs_f64() * 1e3);
                refeitos = refeitos.max(t.stats().rebuilt);
            }
            let q = |m: &NavMesh| {
                let mut rr = Lcg(7);
                let mut pares = Vec::new();
                while pares.len() < 2_000 {
                    let a = [rr.next() * 100.0, rr.next() * 100.0];
                    let c = [rr.next() * 100.0, rr.next() * 100.0];
                    if m.locate(a).is_some()
                        && m.locate(c).is_some()
                        && inteira.locate(a).is_some()
                        && inteira.locate(c).is_some()
                    {
                        pares.push((a, c));
                    }
                }
                let mut s = Polyanya::new();
                let t0 = Instant::now();
                for &(a, c) in &pares {
                    let _ = s.find_path(m, a, c);
                }
                t0.elapsed().as_secs_f64() * 1e6 / pares.len() as f64
            };
            println!(
                "{n:>8} | {tm:>3} | {:>8} | {frio:>7.2} | {porta:>8.3} | {refeitos:>8} | {:>5} | {:>4.0} | {:>12.0}",
                t.stats().tiles,
                t.mesh().poly_count(),
                q(t.mesh()),
                q(&inteira)
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
