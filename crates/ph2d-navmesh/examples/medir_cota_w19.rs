//! A SONDA DA W19 (plano 30 §28.1) — *uma área mais BARATA que o chão num canto longe de tudo pesa em
//! TODA procura?* A cota global da W7 (o heurístico pelo menor custo da tabela) contra a cota pela
//! DISTÂNCIA às áreas baratas (`ph2d_nav::cota`), no MESMO processo, intercaladas, o mínimo de `5`:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-navmesh --profile smoke --example medir_cota_w19
//! ```
//!
//! 1. a cena GRANDE da `medir_custo` (`100 × 100 m`, `1 000` obstáculos, `100` lamas a `4`, `60`
//!    consultas), sem e com uma estrada de `2 × 2 m` a `0,3` num canto;
//! 2. as cenas `30 × 20` com áreas caras E baratas, a todos os pesos: o custo contra o ORÁCULO
//!    ponderado (`0,1 m`) e contra a cota global (o custo de hoje).
//!
//! ⛔ Nenhum número de tempo desta saída vale acima de `load ~5`: aí só as colunas de trabalho.

use std::time::Instant;

use ph2d_nav::cost::path_cost;
use ph2d_nav::{NavMesh, Polyanya, V2, oracle};
use ph2d_navmesh::{Area, Params, Shape, build_with_areas};

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

fn caixa(r: &mut Lcg, c: V2, hmin: f64, hmax: f64) -> Shape {
    let (hx, hy) = (
        hmin + r.next() * (hmax - hmin),
        hmin + r.next() * (hmax - hmin),
    );
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

/// A cena da `medir_custo`: `n` obstáculos e `m` áreas (ids `1..=m`); `barata(id)` recua para dentro.
fn cena(
    seed: u64,
    w: f64,
    h: f64,
    n: usize,
    m: usize,
    barata: impl Fn(u16) -> bool,
) -> (Vec<Shape>, Vec<Area>) {
    let mut r = Lcg(seed);
    let obs = (0..n)
        .map(|i| {
            let c = [r.next() * w, r.next() * h];
            if i % 3 == 0 {
                Shape::Circle {
                    center: c,
                    radius: 0.2 + r.next() * 1.0,
                }
            } else {
                caixa(&mut r, c, 0.2, 1.5)
            }
        })
        .collect();
    let areas = (0..m)
        .map(|i| {
            let c = [r.next() * w, r.next() * h];
            let shape = if i % 2 == 0 {
                caixa(&mut r, c, 1.0, 4.0)
            } else {
                Shape::Circle {
                    center: c,
                    radius: 1.0 + r.next() * 3.0,
                }
            };
            let id = (i + 1) as u16;
            Area {
                shape,
                id,
                dentro: barata(id),
            }
        })
        .collect();
    (obs, areas)
}

fn quadrado(c: V2, h: f64) -> Shape {
    Shape::Convex(vec![
        [c[0] - h, c[1] - h],
        [c[0] + h, c[1] - h],
        [c[0] + h, c[1] + h],
        [c[0] - h, c[1] + h],
    ])
}

fn main() {
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "loadavg: {}",
        load.split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    );
    let params = Params {
        agent_radius: 0.4,
        ..Params::default()
    };
    grande(&params);
    pequenas(&params);
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\nloadavg no fim: {}",
        load.split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    );
}

/// 1. A cena GRANDE: a estrada longe, as duas cotas.
fn grande(params: &Params) {
    println!("\n1. A cena GRANDE (100 × 100 m, 1 000 obstáculos, 100 lamas a 4, 60 consultas)");
    let big = vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]];
    let (obs, areas) = cena(77, 100.0, 100.0, 1_000, 100, |_| false);
    let sem = build_with_areas(&big, &obs, &areas, params)
        .expect("constrói")
        .mesh;
    let mut com_areas = areas.clone();
    com_areas.push(Area {
        shape: quadrado([2.0, 98.0], 1.0),
        id: 101,
        dentro: true,
    });
    let com = build_with_areas(&big, &obs, &com_areas, params)
        .expect("constrói")
        .mesh;
    let mut costs = vec![4.0; 102];
    costs[0] = 1.0;
    costs[101] = 0.3;
    let mut r = Lcg(4242);
    let mut probe = Polyanya::new();
    let mut pares = Vec::new();
    while pares.len() < 60 {
        let (a, z) = (
            [r.next() * 100.0, r.next() * 100.0],
            [r.next() * 100.0, r.next() * 100.0],
        );
        if [&sem, &com].iter().all(|m| {
            m.locate(a).is_some() && m.locate(z).is_some() && probe.find_path(m, a, z).is_ok()
        }) {
            pares.push((a, z));
        }
    }
    let versoes: [(&str, &NavMesh, bool); 4] = [
        ("sem estrada · cota global", &sem, true),
        ("sem estrada · cota nova  ", &sem, false),
        ("com estrada · cota global", &com, true),
        ("com estrada · cota nova  ", &com, false),
    ];
    let mut ms = [f64::INFINITY; 4];
    let mut medidas = [(0u64, 0u64, 0u64, 0.0f64); 4];
    for _ in 0..5 {
        for (k, &(_, m, global)) in versoes.iter().enumerate() {
            let mut s = Polyanya::new();
            s.set_cota_global(global);
            let mut total = 0.0;
            let i0 = Instant::now();
            for &(a, z) in &pares {
                total += s
                    .find_path_costs(m, &costs, a, z)
                    .map_or(f64::NAN, |p| p.cost);
            }
            ms[k] = ms[k].min(i0.elapsed().as_secs_f64() * 1e3);
            medidas[k] = (s.stats.expanded, s.stats.generated, s.stats.work(), total);
        }
    }
    let n = pares.len() as u64;
    println!(
        "   {:>26} | {:>10} | {:>10} | {:>10} | {:>8} | {:>12}",
        "versão", "expandidos", "gerados", "trabalho", "ms (mín)", "custo total"
    );
    for (k, (nome, _, _)) in versoes.iter().enumerate() {
        let (e, g, w, c) = medidas[k];
        println!(
            "   {nome:>26} | {:>10} | {:>10} | {:>10} | {:>8.1} | {c:>12.6}",
            e / n,
            g / n,
            w / n,
            ms[k]
        );
    }
    let razao = medidas[3].0 as f64 / medidas[1].0 as f64;
    println!("   kill-criterion: expandidos com/sem a estrada (cota nova) = {razao:.3} (≤ 1,1)");
}

/// 2. As cenas `30 × 20` com áreas caras e baratas: o custo contra o oráculo e contra a cota global.
fn pequenas(params: &Params) {
    println!(
        "\n2. As cenas 30 × 20 (ids 2 e 4 BARATOS, 1 e 3 caros): custo / oráculo a 0,1 m e nova − global"
    );
    let (w, h) = (30.0, 20.0);
    let reg = vec![[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]];
    println!(
        "   {:>5} {:>6} | {:>22} | {:>10} | {:>18}",
        "caro", "barato", "nova / oráculo (máx)", "|nova−glob|", "expandidos glob→nova"
    );
    for &caro in &[1.5, 2.0, 4.0, 10.0] {
        for &barato in &[0.3, 0.6, 0.9] {
            let costs = [1.0, caro, barato, caro, barato];
            let (mut pior, mut dif) = (0.0f64, 0.0f64);
            let (mut eg, mut en) = (0u64, 0u64);
            let mut abaixo = 0;
            for seed in 1..=6u64 {
                let (obs, areas) = cena(seed, w, h, 10, 4, |id| id % 2 == 0);
                let m = build_with_areas(&reg, &obs, &areas, params)
                    .expect("constrói")
                    .mesh;
                let orc = oracle::WeightedOracle::new(&m, &costs, 0.1);
                let mut nova = Polyanya::new();
                let mut glob = Polyanya::new();
                glob.set_cota_global(true);
                let mut r = Lcg(seed * 31 + 7);
                let mut k = 0;
                while k < 12 {
                    let (a, z) = ([r.next() * w, r.next() * h], [r.next() * w, r.next() * h]);
                    let Some((_, o)) = orc.shortest(a, z) else {
                        continue;
                    };
                    if m.locate(a).is_none() || m.locate(z).is_none() {
                        continue;
                    }
                    k += 1;
                    let (e0, e1) = (nova.stats.expanded, glob.stats.expanded);
                    let pn = nova.find_path_costs(&m, &costs, a, z).expect("mesma ilha");
                    let pg = glob.find_path_costs(&m, &costs, a, z).expect("mesma ilha");
                    en += nova.stats.expanded - e0;
                    eg += glob.stats.expanded - e1;
                    let andado = path_cost(&m, &costs, &pn.points).expect("na malha");
                    assert!((andado - pn.cost).abs() <= 1e-9 * andado.max(1.0));
                    pior = pior.max(pn.cost / o);
                    abaixo += usize::from(pn.cost < o * (1.0 - 1e-9));
                    let d = (pn.cost - pg.cost) / pg.cost.max(1.0);
                    if d.abs() > 1e-9 {
                        println!(
                            "   DIFERE seed {seed} {a:?} → {z:?}: nova {} glob {} oráculo {o} ({:+.2e})",
                            pn.cost, pg.cost, d
                        );
                    }
                    dif = dif.max(d.abs());
                }
            }
            println!(
                "   {caro:>5} {barato:>6} | {pior:>22.6} | {dif:>10.2e} | {eg:>8} → {en:<8}  (abaixo do oráculo: {abaixo})"
            );
        }
    }
}
