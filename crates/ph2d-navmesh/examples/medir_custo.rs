//! A SONDA QUE ABRE A W7 (plano 30 §2.5, §4 linha W7) — *com áreas de custo (a lama), quão longe do
//! caminho mais BARATO fica cada procura, e quanto custa?* Corre-se em `--release`, com a carga ao lado:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-navmesh --release --example medir_custo
//! ```
//!
//! A régua é o ORÁCULO PONDERADO (`ph2d_nav::oracle::WeightedOracle`, cantos + Steiner nas fronteiras), cuja
//! convergência se imprime primeiro (Steiner a `0,4 / 0,2 / 0,1` m contra `0,05`): o óptimo está
//! ABAIXO de todos, e a distância entre dois espaçamentos seguidos diz quanto falta. Os candidatos:
//!
//! - **C0** — o Polyanya que IGNORA o custo (o que a W6 faz hoje), medido pelo custo do caminho dele;
//! - **C1** — o idioma da indústria (Detour, Godot): A\* sobre os polígonos pelo MEIO das arestas,
//!   com o custo da área, e o funil sem custo dentro do corredor;
//! - **C2** — o Polyanya PONDERADO desta crate (`find_path_costs`), quando existir.
//!
//! - **§4** — a cena GRANDE (`100 × 100` m, `1 000` obstáculos, `100` lamas): só o tempo.
//!
//! `DBG_CASO=semente,ax,ay,zx,zy,peso` corre UMA consulta da cena `30 × 20` e imprime a procura (os
//! contadores e o tempo) e o caminho do oráculo troço a troço — o instrumento que achou os três
//! movimentos que faltavam (dobrar no canto de custo, a recta dentro da lama, o deslize).
//!
//! ⛔ Nenhum número de tempo desta saída vale acima de `load ~5`.

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::Instant;

use ph2d_nav::cost::{cost_of, path_cost};
use ph2d_nav::geom::{dist, lerp, ord_key, orient};
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

/// Uma cena `w × h`: `n` obstáculos e `m` áreas de lama (ids `1..=m`, caixas rodadas e círculos).
fn cena(seed: u64, w: f64, h: f64, n: usize, m: usize) -> (Vec<Shape>, Vec<Area>) {
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
            Area {
                shape,
                id: (i + 1) as u16,
            }
        })
        .collect();
    (obs, areas)
}

/// C1 — o A\* da indústria: um nó por polígono, posto no MEIO da aresta por onde entrou; o custo de
/// um passo é o da área do polígono de onde sai × a distância; o alvo soma o último troço. O
/// corredor sai pelo funil SEM custo.
fn detour_like(mesh: &NavMesh, costs: &[f64], s: V2, t: V2) -> Option<Vec<V2>> {
    let ps = mesh.locate(s)?;
    let pt = mesh.locate(t)?;
    if ps == pt {
        return Some(vec![s, t]);
    }
    let wmin = costs.iter().copied().fold(f64::INFINITY, f64::min).min(1.0);
    let np = mesh.polys().len();
    let mut g = vec![f64::INFINITY; np];
    let mut pos = vec![[0.0; 2]; np];
    let mut parent = vec![u32::MAX; np];
    let mut closed = vec![false; np];
    let mut heap = BinaryHeap::new();
    g[ps as usize] = 0.0;
    pos[ps as usize] = s;
    heap.push(Reverse((ord_key(dist(s, t) * wmin), ps)));
    while let Some(Reverse((_, p))) = heap.pop() {
        if closed[p as usize] {
            continue;
        }
        closed[p as usize] = true;
        if p == pt {
            break;
        }
        let poly = &mesh.polys()[p as usize];
        let n = poly.len();
        let c = cost_of(costs, mesh.area_id(p));
        for i in 0..n {
            let Some(q) = poly.nbrs[i] else { continue };
            if closed[q as usize] {
                continue;
            }
            let mid = lerp(
                mesh.vert(poly.verts[i]),
                mesh.vert(poly.verts[(i + 1) % n]),
                0.5,
            );
            let mut ng = g[p as usize] + c * dist(pos[p as usize], mid);
            if q == pt {
                ng += cost_of(costs, mesh.area_id(q)) * dist(mid, t);
            }
            if ng < g[q as usize] {
                g[q as usize] = ng;
                pos[q as usize] = mid;
                parent[q as usize] = p;
                let h = if q == pt { 0.0 } else { dist(mid, t) * wmin };
                heap.push(Reverse((ord_key(ng + h), q)));
            }
        }
    }
    if !closed[pt as usize] {
        return None;
    }
    let mut corredor = vec![pt];
    while let Some(&c) = corredor.last()
        && c != ps
    {
        corredor.push(parent[c as usize]);
    }
    corredor.reverse();
    // Os portais (esquerda, direita) de quem sai de cada polígono para o seguinte.
    let mut portais = vec![(s, s)];
    for w in corredor.windows(2) {
        let poly = &mesh.polys()[w[0] as usize];
        let n = poly.len();
        let i = (0..n).find(|&i| poly.nbrs[i] == Some(w[1]))?;
        let (u, v) = (poly.verts[i], poly.verts[(i + 1) % n]);
        portais.push((mesh.vert(v), mesh.vert(u)));
    }
    portais.push((t, t));
    Some(funil(&portais))
}

/// O funil simples (Mononen) sobre os portais `(esquerda, direita)`, com a orientação anti-horária.
fn funil(portais: &[(V2, V2)]) -> Vec<V2> {
    let mut path = vec![portais[0].0];
    let (mut apex, mut pl, mut pr) = (portais[0].0, portais[0].0, portais[0].1);
    let (mut li, mut ri) = (0usize, 0usize);
    let mut i = 1;
    while i < portais.len() {
        let (l, r) = portais[i];
        if orient(apex, pr, r) >= 0.0 {
            if apex == pr || orient(apex, pl, r) < 0.0 {
                pr = r;
                ri = i;
            } else {
                path.push(pl);
                apex = pl;
                let ai = li;
                (pl, pr) = (apex, apex);
                (li, ri) = (ai, ai);
                i = ai + 1;
                continue;
            }
        }
        if orient(apex, pl, l) <= 0.0 {
            if apex == pl || orient(apex, pr, l) > 0.0 {
                pl = l;
                li = i;
            } else {
                path.push(pr);
                apex = pr;
                let ai = ri;
                (pl, pr) = (apex, apex);
                (li, ri) = (ai, ai);
                i = ai + 1;
                continue;
            }
        }
        i += 1;
    }
    let fim = portais[portais.len() - 1].0;
    if path.last() != Some(&fim) {
        path.push(fim);
    }
    path
}

/// O espaçamento do oráculo na secção 2 (a secção 1 imprime o quanto ele ainda erra).
const ESPACO: f64 = 0.1;

fn quantil(v: &mut [f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
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
    let (w, h) = (30.0, 20.0);
    let reg = vec![[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]];
    let params = Params {
        agent_radius: 0.4,
        ..Params::default()
    };
    let pesos = [1.5, 2.0, 4.0, 10.0];
    let mut s = Polyanya::new();
    if let Ok(caso) = std::env::var("DBG_CASO") {
        let v: Vec<f64> = caso.split(',').map(|x| x.parse().unwrap()).collect();
        let (obs, areas) = cena(v[0] as u64, w, h, 10, 4);
        let b = build_with_areas(&reg, &obs, &areas, &params).expect("constrói");
        let m = &b.mesh;
        let costs = [1.0, v[5], v[5], v[5], v[5]];
        let (a, z) = ([v[1], v[2]], [v[3], v[4]]);
        let mut pa = Vec::new();
        let mut pz = Vec::new();
        m.locate_all(a, &mut pa);
        m.locate_all(z, &mut pz);
        eprintln!(
            "s em {:?} (áreas {:?}), t em {:?}",
            pa,
            pa.iter().map(|&p| m.area_id(p)).collect::<Vec<_>>(),
            pz
        );
        s.stats = Default::default();
        let i0 = Instant::now();
        let p = s.find_path_costs(m, &costs, a, z);
        let dt = i0.elapsed().as_secs_f64() * 1e6;
        eprintln!(
            "C2 {:?}\nstats {:?}\n{dt:.0} µs, {} polígonos",
            p,
            s.stats,
            m.polys().len()
        );
        let orc = oracle::WeightedOracle::new(m, &costs, 0.1);
        let (op, oc) = orc.shortest(a, z).unwrap();
        eprintln!("oráculo {oc}");
        let mut here = Vec::new();
        for w2 in op.windows(2) {
            m.locate_all(w2[1], &mut here);
            let sc = ph2d_nav::cost::segment_cost(m, &costs, w2[0], w2[1]);
            eprintln!(
                "  -> {:?} polys {:?} áreas {:?} custo {:?} comp {:.3}",
                w2[1],
                here,
                here.iter().map(|&q| m.area_id(q)).collect::<Vec<_>>(),
                sc,
                dist(w2[0], w2[1])
            );
        }
        return;
    }

    // ── 1. A convergência do oráculo ──
    println!(
        "\n1. O ORÁCULO: o custo com Steiner a 0,4 / 0,2 / 0,1 m nas fronteiras, relativo a 0,05 m"
    );
    let espacos = [0.4, 0.2, 0.1];
    let mut dif: Vec<Vec<f64>> = vec![Vec::new(); espacos.len()];
    let mut nos = 0;
    for seed in 1..=3u64 {
        let (obs, areas) = cena(seed, w, h, 10, 4);
        let b = build_with_areas(&reg, &obs, &areas, &params).expect("constrói");
        let m = &b.mesh;
        let costs = [1.0, 4.0, 4.0, 4.0, 4.0];
        let fino = oracle::WeightedOracle::new(m, &costs, 0.05);
        nos = nos.max(fino.node_count());
        let grossos: Vec<_> = espacos
            .iter()
            .map(|&e| oracle::WeightedOracle::new(m, &costs, e))
            .collect();
        let mut r = Lcg(seed * 977);
        let mut n = 0;
        while n < 8 {
            let (a, z) = ([r.next() * w, r.next() * h], [r.next() * w, r.next() * h]);
            let Some((_, o)) = fino.shortest(a, z) else {
                continue;
            };
            n += 1;
            for (k, g) in grossos.iter().enumerate() {
                dif[k].push(g.shortest(a, z).map_or(f64::NAN, |x| x.1) / o - 1.0);
            }
        }
    }
    for (k, e) in espacos.iter().enumerate() {
        let v = &mut dif[k];
        let neg = v.iter().filter(|&&x| x < -1e-12).count();
        println!(
            "   {e} m: máx {:+.2e} · mediana {:+.2e} · abaixo do fino: {neg} de {}",
            quantil(v, 1.0),
            quantil(v, 0.5),
            v.len()
        );
    }
    println!("   (o grafo fino tem até {nos} nós)");

    // ── 2. Os candidatos contra o oráculo (k = 32) ──
    println!(
        "\n2. Custo do caminho / oráculo a {ESPACO} m (média · p95 · máx), e o tempo por consulta"
    );
    println!(
        "   {:>5} | {:>28} | {:>28} | {:>7} | {:>7}",
        "peso", "C0 Polyanya sem custo", "C1 A*+funil (indústria)", "C0 µs", "C1 µs"
    );
    let grelhas = [0.5, 0.25, 0.1];
    let mut linhas_c2: Vec<String> = Vec::new();
    for &peso in &pesos {
        let costs = [1.0, peso, peso, peso, peso];
        let (mut r0, mut r1) = (Vec::new(), Vec::new());
        let mut r2: Vec<Vec<f64>> = vec![Vec::new(); grelhas.len()];
        let mut t2 = vec![0.0; grelhas.len()];
        let mut refr = vec![0u64; grelhas.len()];
        let (mut t0, mut t1) = (0.0, 0.0);
        let mut n_tot = 0usize;
        for seed in 1..=8u64 {
            let (obs, areas) = cena(seed, w, h, 10, 4);
            let b = build_with_areas(&reg, &obs, &areas, &params).expect("constrói");
            let m = &b.mesh;
            let orc = oracle::WeightedOracle::new(m, &costs, ESPACO);
            let mut r = Lcg(seed * 31 + 7);
            let mut n = 0;
            while n < 20 {
                let (a, z) = ([r.next() * w, r.next() * h], [r.next() * w, r.next() * h]);
                if m.locate(a).is_none() || m.locate(z).is_none() {
                    continue;
                }
                let Some((_, o)) = orc.shortest(a, z) else {
                    continue;
                };
                n += 1;
                let i0 = Instant::now();
                let p0 = s.find_path(m, a, z).expect("mesma ilha");
                t0 += i0.elapsed().as_secs_f64();
                let i1 = Instant::now();
                let p1 = detour_like(m, &costs, a, z).expect("mesma ilha");
                t1 += i1.elapsed().as_secs_f64();
                r0.push(path_cost(m, &costs, &p0.points).expect("C0 dentro") / o);
                r1.push(path_cost(m, &costs, &p1).expect("C1 dentro") / o);
                for (k, &g) in grelhas.iter().enumerate() {
                    s.set_steiner_spacing(Some(g));
                    let antes = s.stats.refractions;
                    let i2 = Instant::now();
                    let p2 = s.find_path_costs(m, &costs, a, z).expect("mesma ilha");
                    t2[k] += i2.elapsed().as_secs_f64();
                    refr[k] += s.stats.refractions - antes;
                    let real = path_cost(m, &costs, &p2.points).expect("C2 dentro");
                    assert!(
                        (real - p2.cost).abs() <= 1e-9 * real.max(1.0),
                        "C2 diz {} e custa {real}",
                        p2.cost
                    );
                    r2[k].push(real / o);
                }
                s.set_steiner_spacing(None);
            }
            n_tot += n;
        }
        let f = |v: &mut Vec<f64>| {
            let media = v.iter().sum::<f64>() / v.len() as f64;
            format!(
                "{:.4} · {:.4} · {:.4}",
                media,
                quantil(v, 0.95),
                quantil(v, 1.0)
            )
        };
        println!(
            "   {:>5} | {:>28} | {:>28} | {:>7.1} | {:>7.1}",
            peso,
            f(&mut r0),
            f(&mut r1),
            t0 / n_tot as f64 * 1e6,
            t1 / n_tot as f64 * 1e6
        );
        for (k, &g) in grelhas.iter().enumerate() {
            linhas_c2.push(format!(
                "   {:>5} | {:>5} m | {:>28} | {:>8.1} | {:>8.1}",
                peso,
                g,
                f(&mut r2[k]),
                t2[k] / n_tot as f64 * 1e6,
                refr[k] as f64 / n_tot as f64
            ));
        }
        let abaixo = r0.iter().chain(&r1).filter(|&&x| x < 1.0 - 1e-9).count();
        if abaixo > 0 {
            println!("   ⚠️ {abaixo} caminhos ABAIXO do oráculo — a régua não é o piso");
        }
    }
    println!(
        "\n3. C2 — o Polyanya PONDERADO (refracção + polimento de Snell), por passo da grelha"
    );
    println!(
        "   {:>5} | {:>7} | {:>28} | {:>8} | {:>8}",
        "peso", "grelha", "custo / oráculo", "µs", "raízes"
    );
    for l in linhas_c2 {
        println!("{l}");
    }

    // ── 4. A cena GRANDE: o custo por consulta onde ele importa ──
    println!(
        "\n4. A cena GRANDE (100 × 100 m, 1 000 obstáculos, 100 lamas a peso 4): µs por consulta"
    );
    let big = vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]];
    let (obs, areas) = cena(77, 100.0, 100.0, 1_000, 100);
    let i0 = Instant::now();
    let b = build_with_areas(&big, &obs, &areas, &params).expect("constrói");
    println!(
        "   construção {:.1} ms · {} polígonos",
        i0.elapsed().as_secs_f64() * 1e3,
        b.mesh.polys().len()
    );
    // A ponte usa MOSAICOS: a frio, e uma lama que se mexe (o mínimo de 5).
    for com_areas in [false, true] {
        let ars: &[Area] = if com_areas { &areas } else { &[] };
        let mut frio = f64::INFINITY;
        let mut mexe = f64::INFINITY;
        for k in 0..5 {
            let mut tm = ph2d_navmesh::TiledMesh::new(params, ph2d_navmesh::TILE_M);
            let i0 = Instant::now();
            tm.update_with_areas(&big, &obs, ars);
            frio = frio.min(i0.elapsed().as_secs_f64() * 1e3);
            let mut a2 = ars.to_vec();
            if let Some(Area {
                shape: Shape::Circle { center, .. },
                ..
            }) = a2.get_mut(1)
            {
                center[0] += 0.3 + 0.01 * f64::from(k);
            }
            let i0 = Instant::now();
            tm.update_with_areas(&big, &obs, &a2);
            mexe = mexe.min(i0.elapsed().as_secs_f64() * 1e3);
        }
        println!(
            "   mosaicos {}: a frio {frio:.1} ms · uma lama a mexer {mexe:.2} ms",
            if com_areas {
                "com 100 lamas"
            } else {
                "sem lamas    "
            }
        );
    }
    let m = &b.mesh;
    let mut costs = vec![1.0; 101];
    costs[1..].iter_mut().for_each(|c| *c = 4.0);
    let mut r = Lcg(4242);
    let mut pares = Vec::new();
    while pares.len() < 40 {
        let (a, z) = (
            [r.next() * 100.0, r.next() * 100.0],
            [r.next() * 100.0, r.next() * 100.0],
        );
        if m.locate(a).is_some() && m.locate(z).is_some() && s.find_path(m, a, z).is_ok() {
            pares.push((a, z));
        }
    }
    for (nome, g) in [
        ("C0 sem custo", None),
        ("C2 a 0,25 m", Some(0.25)),
        ("C2 a 0,1 m", Some(0.1)),
    ] {
        let mut ts: Vec<f64> = Vec::new();
        s.stats = Default::default();
        for &(a, z) in &pares {
            let i0 = Instant::now();
            match g {
                None => {
                    let _ = s.find_path(m, a, z);
                }
                Some(g) => {
                    s.set_steiner_spacing(Some(g));
                    let _ = s.find_path_costs(m, &costs, a, z);
                }
            }
            ts.push(i0.elapsed().as_secs_f64() * 1e6);
        }
        s.set_steiner_spacing(None);
        println!(
            "   {nome:>13}: mediana {:>8.0} · p95 {:>8.0} · máx {:>8.0}",
            quantil(&mut ts.clone(), 0.5),
            quantil(&mut ts.clone(), 0.95),
            quantil(&mut ts, 1.0)
        );
        let n = pares.len() as u64;
        println!(
            "   {:>13}  por consulta: {} gerados · {} expandidos · {} voltas · {} raízes · {} podadas",
            "",
            s.stats.generated / n,
            s.stats.expanded / n,
            s.stats.turns / n,
            s.stats.refractions / n,
            s.stats.pruned_refractions / n
        );
    }
}
