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
    let np = mesh.poly_count();
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
        let poly = mesh.poly(p);
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
        let poly = mesh.poly(w[0]);
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
    // `SEM_DOMINANCIA=1`: o CONTROLO da W9 (a procura ponderada sem a dominância entre frentes).
    s.set_front_dominance(std::env::var_os("SEM_DOMINANCIA").is_none());
    let so_grande = std::env::var_os("SO_GRANDE").is_some();
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
            m.poly_count()
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

    if so_grande {
        grande(&mut s, &params);
        return;
    }
    if std::env::var_os("DIAG_NOS").is_some() {
        diag_nos(&params);
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
                    if std::env::var_os("PIOR").is_some() && g == 0.25 && real / o > 1.0001 {
                        println!(
                            "   PIOR {:.5} peso {peso} DBG_CASO={seed},{},{},{},{},{peso}",
                            real / o,
                            a[0],
                            a[1],
                            z[0],
                            z[1]
                        );
                    }
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

    grande(&mut s, &params);
}

/// ── 4. A cena GRANDE: o custo por consulta onde ele importa ──
fn grande(s: &mut Polyanya, params: &Params) {
    let params = *params;
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
        b.mesh.poly_count()
    );
    // A ponte usa MOSAICOS: a frio, uma lama que se mexe e uma PORTA (um obstáculo) que se mexe — o
    // mínimo de 5 (W9: a montagem sem uma lista por polígono).
    for com_areas in [false, true] {
        let ars: &[Area] = if com_areas { &areas } else { &[] };
        let mut frio = f64::INFINITY;
        let mut mexe = f64::INFINITY;
        let mut porta = f64::INFINITY;
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
            let mut o2 = obs.clone();
            if let Shape::Circle { center, .. } = &mut o2[3] {
                center[0] += 0.3 + 0.01 * f64::from(k);
            }
            let i0 = Instant::now();
            tm.update_with_areas(&big, &o2, &a2);
            porta = porta.min(i0.elapsed().as_secs_f64() * 1e3);
        }
        println!(
            "   mosaicos {}: a frio {frio:.1} ms · uma lama a mexer {mexe:.2} ms · uma porta {porta:.2} ms",
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
            "   {:>13}  por consulta: {} gerados · {} expandidos · {} voltas · {} raízes · {} podadas · {} dominados · {} cortados",
            "",
            s.stats.generated / n,
            s.stats.expanded / n,
            s.stats.turns / n,
            s.stats.refractions / n,
            s.stats.pruned_refractions / n,
            s.stats.dominated / n,
            s.stats.trimmed / n
        );
    }
}

/// ⭐ (plano 30 §27.3, C — MEDIR antes de propor) **Onde estão os nós** da procura ponderada na cena
/// grande (`100` lamas, `60` consultas, pesos `4` e `10`): `DIAG_NOS=1`. Por nó expandido, o `f` com que
/// saiu do heap contra o custo `C*` da resposta (o fundo `f ≈ C*` paga-o todo A* exacto; o resto é o que
/// um heurístico mais apertado cortaria), o chão contra a lama, e quantas vezes a MESMA aresta (polígono,
/// entrada) expandiu — as frentes paralelas que a dominância corta a meio (`cortados`) e sobrevivem.
fn diag_nos(params: &Params) {
    let big = vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]];
    let (obs, areas) = cena(77, 100.0, 100.0, 1_000, 100);
    let b = build_with_areas(&big, &obs, &areas, params).expect("constrói");
    let m = &b.mesh;
    let mut s = Polyanya::new();
    let mut r = Lcg(4242);
    let mut pares = Vec::new();
    while pares.len() < 60 {
        let (a, z) = (
            [r.next() * 100.0, r.next() * 100.0],
            [r.next() * 100.0, r.next() * 100.0],
        );
        if m.locate(a).is_some() && m.locate(z).is_some() && s.find_path(m, a, z).is_ok() {
            pares.push((a, z));
        }
    }
    s.stats = Default::default();
    let mut classes0 = [0u64; CLASSES.len()];
    let mut arestas0 = 0u64;
    let mut ms0 = 0.0f64;
    for &(a, z) in &pares {
        s.diag = Some(Vec::new());
        let i0 = Instant::now();
        let _ = s.find_path(m, a, z);
        ms0 += i0.elapsed().as_secs_f64() * 1e3;
        let d = s.diag.take().unwrap_or_default();
        classifica(&d, &mut classes0);
        arestas0 += d
            .iter()
            .map(|e| (e.poly, e.entry))
            .collect::<std::collections::BTreeSet<_>>()
            .len() as u64;
    }
    let n0 = s.stats.expanded.max(1);
    println!(
        "C. onde estão os nós — {} polígonos, {} consultas; o CONTROLO sem custo: {} expandidos / consulta, {} arestas distintas",
        m.poly_count(),
        pares.len(),
        s.stats.expanded / pares.len() as u64,
        arestas0 / pares.len() as u64
    );
    println!(
        "   o CONTROLO: {ms0:.0} ms as {} consultas (com o registo ligado) — o preço da fase GERAL de cada consulta ponderada",
        pares.len()
    );
    for (i, nome) in CLASSES.iter().enumerate() {
        println!(
            "     {:>5.1} %  {nome}",
            100.0 * classes0[i] as f64 / n0 as f64
        );
    }
    let mut custos_ref: Vec<f64> = Vec::new();
    const FAIXAS: [f64; 6] = [0.5, 0.8, 0.9, 0.95, 0.99, f64::INFINITY];
    for (peso, ideal) in [
        (4.0, 0),
        (4.0, 1),
        (4.0, 2),
        (10.0, 0),
        (10.0, 1),
        (10.0, 2),
    ] {
        s.ideal = ideal;
        if ideal > 0 {
            print!(
                "\n   ⭐ o TECTO de qualquer dominância contra as anteriores ({})",
                if ideal == 1 {
                    "mesmo custo"
                } else {
                    "qualquer custo"
                }
            );
        }
        let mut costs = vec![1.0; 101];
        costs[1..].iter_mut().for_each(|c| *c = peso);
        s.stats = Default::default();
        let mut faixa = [[0u64; 6]; 2];
        let (mut nos, mut arestas, mut repetidos) = (0u64, 0u64, [0u64; 2]);
        let mut por_consulta: Vec<u64> = Vec::new();
        let mut classes = [0u64; CLASSES.len()];
        let (mut ms, mut custos_v) = (0.0f64, Vec::new());
        let mut geral_n = 0u64;
        for &(a, z) in &pares {
            s.diag = Some(Vec::new());
            let i0 = Instant::now();
            let p = s.find_path_costs(m, &costs, a, z).expect("mesma ilha");
            ms += i0.elapsed().as_secs_f64() * 1e3;
            custos_v.push(p.cost);
            let d = s.diag.take().unwrap_or_default();
            por_consulta.push(d.len() as u64);
            let pond: Vec<_> = if ideal == 9 {
                d.clone()
            } else {
                d.iter().copied().filter(|e| e.ponderada).collect()
            };
            geral_n += (d.len() - pond.len()) as u64;
            classifica(&pond, &mut classes);
            let mut vistas = std::collections::BTreeSet::new();
            for e in &d {
                let lama = usize::from(e.w > 1.0);
                let x = e.f / p.cost;
                let i = FAIXAS.iter().position(|&f| x < f).unwrap_or(5);
                faixa[lama][i] += 1;
                if !vistas.insert((e.poly, e.entry)) {
                    repetidos[lama] += 1;
                }
            }
            nos += d.len() as u64;
            arestas += vistas.len() as u64;
        }
        let n = pares.len() as u64;
        let pct = |x: u64| 100.0 * x as f64 / nos.max(1) as f64;
        let mut pc = por_consulta.clone();
        pc.sort_unstable();
        println!(
            "\n   peso {peso}: {} expandidos / consulta (mediana {} · máx {}) · {} arestas distintas / consulta · {} gerados · {} dominados · {} cortados · {} raízes de fronteira",
            nos / n,
            pc[pc.len() / 2],
            pc[pc.len() - 1],
            arestas / n,
            s.stats.generated / n,
            s.stats.dominated / n,
            s.stats.trimmed / n,
            s.stats.refractions / n
        );
        if ideal == 0 {
            custos_ref = custos_v.clone();
        }
        let desvio = custos_v
            .iter()
            .zip(&custos_ref)
            .map(|(x, y)| x / y - 1.0)
            .fold(0.0f64, |m, x| if x.abs() > m.abs() { x } else { m });
        println!(
            "   {ms:.0} ms as {} consultas (com o registo ligado) · custo ÷ o de hoje: pior {desvio:+.2e}",
            pares.len()
        );
        println!(
            "   f / C*        | <0,5 | 0,5–0,8 | 0,8–0,9 | 0,9–0,95 | 0,95–0,99 | ≥0,99 | total"
        );
        for (k, nome) in [(0, "no chão (w = 1)"), (1, "na lama (w > 1)")] {
            let t: u64 = faixa[k].iter().sum();
            println!(
                "   {nome} | {:>5.1} % | {:>5.1} % | {:>5.1} % | {:>5.1} % | {:>5.1} % | {:>5.1} % | {:>5.1} %",
                pct(faixa[k][0]),
                pct(faixa[k][1]),
                pct(faixa[k][2]),
                pct(faixa[k][3]),
                pct(faixa[k][4]),
                pct(faixa[k][5]),
                pct(t)
            );
        }
        println!(
            "   a MESMA aresta outra vez: {:.1} % dos nós (chão {:.1} % · lama {:.1} %)",
            pct(repetidos[0] + repetidos[1]),
            pct(repetidos[0]),
            pct(repetidos[1])
        );
        println!(
            "   da fase GERAL (sem custos, antes da ponderada): {:.1} % dos nós",
            pct(geral_n)
        );
        println!(
            "   cada nó expandido da PONDERADA, pela 1.ª classe que o descreve (amostrado em 17 pontos do intervalo; % do total):"
        );
        for (i, nome) in CLASSES.iter().enumerate() {
            println!("     {:>5.1} %  {nome}", pct(classes[i]));
        }
    }
}

/// As classes do diagnóstico, por ordem: a 1.ª que se aplica.
const CLASSES: [&str; 7] = [
    "refracta (o custo muda na aresta: sem dominância)",
    "1.º nesta aresta",
    "dominado INTEIRO pelas frentes anteriores do MESMO custo (a dominância de hoje deixou-o)",
    "dominado inteiro pelas anteriores contando as de OUTRO custo",
    "dominado inteiro só contando as frentes POSTERIORES (a ordem)",
    "dominado em parte (por alguma frente, de qualquer tempo)",
    "não dominado (uma frente que conta)",
];

/// Classifica cada nó expandido contra as outras frentes da MESMA aresta (polígono, entrada): `y` está
/// dominado por `A` se `A` cobre `y` e chega lá por menos ou igual (`g_A + w_A·|ρ_A − y|`).
fn classifica(d: &[ph2d_nav::polyanya::Expandido], classes: &mut [u64; CLASSES.len()]) {
    use std::collections::BTreeMap;
    let mut por_aresta: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for (i, e) in d.iter().enumerate() {
        if !e.refrata {
            por_aresta.entry((e.poly, e.entry)).or_default().push(i);
        }
    }
    let cobre = |a: &ph2d_nav::polyanya::Expandido, y: V2| {
        let (l, r) = (a.left, a.right);
        let len = dist(l, r);
        len > 0.0 && (dist(l, y) + dist(y, r) - len).abs() <= 1e-7 * (1.0 + len)
    };
    let custo = |a: &ph2d_nav::polyanya::Expandido, y: V2| a.g + a.w * dist(a.rho, y);
    for (i, e) in d.iter().enumerate() {
        if e.refrata {
            classes[0] += 1;
            continue;
        }
        let irmaos = &por_aresta[&(e.poly, e.entry)];
        if irmaos[0] == i {
            classes[1] += 1;
            continue;
        }
        let ys: Vec<V2> = (0..17)
            .map(|k| lerp(e.left, e.right, f64::from(k) / 16.0))
            .collect();
        let dominado = |filtro: &dyn Fn(usize) -> bool| {
            ys.iter()
                .map(|&y| {
                    let meu = custo(e, y);
                    irmaos.iter().any(|&j| {
                        j != i
                            && filtro(j)
                            && cobre(&d[j], y)
                            && custo(&d[j], y) <= meu + 1e-9 * (1.0 + meu)
                    })
                })
                .filter(|&b| b)
                .count()
        };
        let n = ys.len();
        if dominado(&|j| j < i && d[j].w == e.w) == n {
            classes[2] += 1;
        } else if dominado(&|j| j < i) == n {
            classes[3] += 1;
        } else if dominado(&|_| true) == n {
            classes[4] += 1;
        } else if dominado(&|_| true) > 0 {
            classes[5] += 1;
        } else {
            classes[6] += 1;
        }
    }
}
