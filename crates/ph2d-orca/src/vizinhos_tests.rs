//! ⭐ (W14) **Os vizinhos por anéis são os da varrida**, lista a lista (plano 30 §22.3). O oráculo é a
//! varrida das 3 × 3 células do alcance sem perda, VERBATIM a de antes da grelha fina.

use std::collections::BTreeMap;

use crate::v2::{V2, abs_sq, len, sub};
use crate::{Agent, Crowd, Params};

/// A varrida de antes, verbatim (o `Crowd::new` e o `Crowd::neighbors` até à W13), sobre a fotografia.
fn varrida(agents: &[Agent], params: Params, i: usize) -> (Vec<u32>, Vec<(f64, u32)>) {
    let speed_bound = |a: &Agent| a.max_speed.max(len(a.vel));
    let cell = match params.neighbor_dist {
        Some(d) => d,
        None => {
            let r = agents.iter().map(|a| a.radius).fold(0.0, f64::max);
            let s = agents.iter().map(speed_bound).fold(0.0, f64::max);
            2.0 * r + 6.0 * s * params.time_horizon
        }
    };
    let cell_of = |p: V2| ((p[0] / cell).floor() as i64, (p[1] / cell).floor() as i64);
    let mut grid: BTreeMap<(i64, i64), Vec<u32>> = BTreeMap::new();
    for (i, a) in agents.iter().enumerate() {
        grid.entry(cell_of(a.pos)).or_default().push(i as u32);
    }
    let tau = params.time_horizon;
    let bounds: Vec<(f64, f64, f64)> = agents
        .iter()
        .map(|a| {
            let s = speed_bound(a);
            (a.radius, 5.0 * s * tau, s * tau)
        })
        .collect();
    let me = &agents[i];
    let (ri, ki, _) = bounds[i];
    let (cx, cy) = cell_of(me.pos);
    let mut found = Vec::new();
    for gx in cx - 1..=cx + 1 {
        for gy in cy - 1..=cy + 1 {
            let Some(list) = grid.get(&(gx, gy)) else {
                continue;
            };
            for &j in list {
                if j as usize == i || me.ignores == Some(j) {
                    continue;
                }
                let other = &agents[j as usize];
                let (rj, _, sj) = bounds[j as usize];
                let range = ri + rj + ki + sj;
                let d = abs_sq(sub(other.pos, me.pos));
                if d < range * range {
                    found.push((d, j));
                }
            }
        }
    }
    let todos = found.clone();
    let ordem = |x: &(f64, u32), y: &(f64, u32)| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1));
    if let Some(n) = params.max_neighbors
        && found.len() > n
    {
        if n == 0 {
            found.clear();
        } else {
            found.select_nth_unstable_by(n - 1, ordem);
            found.truncate(n);
        }
    }
    found.sort_unstable_by(ordem);
    (found.iter().map(|&(_, j)| j).collect(), todos)
}

struct Lcg(u64);
impl Lcg {
    fn f(&mut self, a: f64, b: f64) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        a + (b - a) * ((self.0 >> 11) as f64 / (1u64 << 53) as f64)
    }
}

fn agente(pos: V2, vel: V2, radius: f64, max_speed: f64) -> Agent {
    Agent {
        pos,
        vel,
        pref: vel,
        radius,
        max_speed,
        avoids: true,
        ignores: None,
    }
}

/// As multidões: a densa do `custo.rs` (uma grelha — EMPATES de distância), uma esparsa com raios e
/// velocidades mistos, cinco grupos, uma com um agente a `5 km` (a grelha fina alarga-se), e uma em
/// que um em cada sete ignora o seu alvo.
fn multidoes() -> Vec<(&'static str, Vec<Agent>)> {
    let mut r = Lcg(0x00f1_7a14);
    let densa: Vec<Agent> = (0..1_000)
        .map(|k| {
            let pos = [(k % 32) as f64 * 0.9 - 14.4, (k / 32) as f64 * 0.9 - 14.4];
            let l = len(pos).max(1e-9);
            agente(pos, [-pos[0] / l * 2.0, -pos[1] / l * 2.0], 0.3, 2.0)
        })
        .collect();
    let mista: Vec<Agent> = (0..400)
        .map(|_| {
            let pos = [r.f(0.0, 120.0), r.f(0.0, 120.0)];
            let vel = [r.f(-3.0, 3.0), r.f(-3.0, 3.0)];
            agente(pos, vel, r.f(0.2, 1.0), r.f(0.0, 2.5))
        })
        .collect();
    let grupos: Vec<Agent> = (0..400)
        .map(|k| {
            let c = [(k % 5) as f64 * 40.0, (k % 5) as f64 * 15.0];
            let pos = [c[0] + r.f(-4.0, 4.0), c[1] + r.f(-4.0, 4.0)];
            agente(pos, [r.f(-1.0, 1.0), r.f(-1.0, 1.0)], 0.4, 1.5)
        })
        .collect();
    let mut longe = mista.clone();
    longe.push(agente([5_000.0, -3_000.0], [0.0, 0.0], 0.5, 2.0));
    let mut ignora = grupos.clone();
    for k in (0..ignora.len()).step_by(7) {
        ignora[k].ignores = Some(((k + 1) % ignora.len()) as u32);
    }
    vec![
        ("densa", densa),
        ("mista", mista),
        ("grupos", grupos),
        ("longe", longe),
        ("ignora", ignora),
    ]
}

#[test]
fn os_vizinhos_por_aneis_sao_os_da_varrida() {
    let (mut listas, mut cheias, mut empates) = (0usize, 0usize, 0usize);
    for (nome, agents) in multidoes() {
        for n in [1usize, 3, 10, 40] {
            let p = Params {
                max_neighbors: Some(n),
                ..Params::PRODUCT
            };
            let c = Crowd::new(agents.clone(), p);
            let mut nb = Vec::new();
            for i in 0..agents.len() {
                c.neighbors(i, &mut nb);
                let (oraculo, mut todos) = varrida(&agents, p, i);
                assert_eq!(nb, oraculo, "{nome}, n = {n}, o agente {i}");
                listas += 1;
                if todos.len() > n {
                    cheias += 1;
                    // Um EMPATE na fronteira: o `n`-ésimo e o seguinte à mesma distância.
                    todos.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
                    empates += usize::from(todos[n - 1].0 == todos[n].0);
                }
            }
        }
    }
    // A população (medida: `10 404` listas, `9 472` cortadas pelo tecto — a paragem dos anéis decide —,
    // `2 089` empates na fronteira).
    assert!(
        cheias > 9_000 && empates > 2_000,
        "{listas} listas, {cheias} cortadas pelo tecto, {empates} empates na fronteira"
    );
}
