//! A SONDA DO REPLANEIO EM MASSA (plano 30, W9; W15) — quando UMA porta muda a malha, quanto custa o
//! pior tique que se segue? Uma compilação, no perfil `smoke`, com a carga ao lado:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-physics-ecs --profile smoke --example medir_replaneio
//! ```
//!
//! A cena da `medir_nav_tique` (`100 × 100 m`, `1 000` caixas, a semente fixa) com `N` agentes do
//! mesmo raio (a mesma malha), cada um com o seu alvo longe. ⭐ (W15, a regra do dono de 05/10) As duas
//! versões vivem no MESMO processo, lado a lado, escolhidas em execução (`set_nav_slices`): A = toda
//! procura inteira no tique; B = a vez e as fatias (plano 30 §23). Em cada uma de 7 rodadas, com a
//! ordem rodada, cada versão move a porta e corre um bloco de 16 tiques (o pior tique), e depois outro
//! sem a porta mexer (o CONTROLO); vale o MÍNIMO das rodadas, a mediana ao lado só como controlo.
//!
//! (W14) `LAMAS=<n>` põe `n` áreas de lama (caixas sensoras, `LAMA_PESO`, por omissão `4`) com outra
//! semente — as caixas e os agentes ficam os mesmos: a régua do orçamento quando a procura é a PONDERADA.
//! `ORCAMENTO=<n>` muda o orçamento de trabalho por tique (por omissão o de fábrica, `20 000`).

use std::time::Instant;

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavCostArea, NavRegion, NavTarget, PhysicsBridge,
    RigidBody, TopDownPlayer,
};
use ph2d_topdown::{TopDownLaw, direction::DirectionMode};

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 40) as f32) / (1u64 << 24) as f32
    }
}

fn caixa(w: &mut bevy_ecs::world::World, kind: BodyKind, c: Vec2, hx: f32, hy: f32) -> Entity {
    w.spawn((
        RigidBody { kind },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: hx,
                half_y: hy,
            },
            ..Collider::default()
        },
        Transform::from_translation(c),
    ))
    .id()
}

fn cena(agentes: usize) -> (SimWorld, Entity, Vec<Entity>) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    w.spawn((
        Name::new("Região"),
        NavRegion {
            half_extents: [50.0, 50.0],
            obstacle_layers: u8::MAX,
        },
        Transform::from_translation(Vec2::new(50.0, 50.0)),
    ));
    let mut r = Lcg(42);
    let mut caixas = Vec::new();
    for _ in 0..1000 {
        let (x, y) = (r.next() * 100.0, r.next() * 100.0);
        let (hx, hy) = (0.2 + r.next(), 0.2 + r.next());
        caixas.push((x, y, hx + 0.6, hy + 0.6));
        caixa(w, BodyKind::Static, Vec2::new(x, y), hx, hy);
    }
    let porta = caixa(w, BodyKind::Kinematic, Vec2::new(50.3, 50.3), 0.5, 0.2);
    let env = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<f32>().ok());
    let peso = env("LAMA_PESO").unwrap_or(4.0);
    let mut l = Lcg(99);
    for _ in 0..env("LAMAS").map_or(0, |n| n as usize) {
        let (x, y) = (l.next() * 100.0, l.next() * 100.0);
        let (hx, hy) = (1.0 + l.next() * 3.0, 1.0 + l.next() * 3.0);
        w.spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: hx,
                    half_y: hy,
                },
                is_sensor: true,
                ..Collider::default()
            },
            NavCostArea {
                cost: peso,
                forbidden: false,
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    }
    let livre = |x: f32, y: f32| {
        caixas
            .iter()
            .all(|&(cx, cy, hx, hy)| (x - cx).abs() > hx || (y - cy).abs() > hy)
    };
    let mut quem = Vec::new();
    let mut a = Lcg(7);
    while quem.len() < agentes {
        let (x, y) = (2.0 + a.next() * 96.0, 2.0 + a.next() * 96.0);
        let (tx, ty) = (2.0 + a.next() * 96.0, 2.0 + a.next() * 96.0);
        if !(livre(x, y) && livre(tx, ty)) || (x - tx).hypot(y - ty) < 40.0 {
            continue;
        }
        quem.push(
            w.spawn((
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.4 },
                    ..Collider::default()
                },
                TopDownPlayer::from_law(TopDownLaw {
                    default_controls: false,
                    direction: DirectionMode::Free,
                    ..TopDownLaw::default()
                }),
                NavAgent {
                    target: NavTarget::Point([tx, ty]),
                    ..NavAgent::default()
                },
                Transform::from_translation(Vec2::new(x, y)),
            ))
            .id(),
        );
    }
    (sim, porta, quem)
}

/// As procuras de todos os agentes até agora.
fn procuras(b: &PhysicsBridge, quem: &[Entity]) -> u64 {
    quem.iter()
        .filter_map(|&e| b.nav_agent(e))
        .map(|rt| rt.searches)
        .sum()
}

/// Um bloco de `BLOCO` tiques: o pior tique em ms, o maior trabalho de procura do caminho CRÍTICO num
/// tique (a maior fatia em paralelo mais a condução), e o maior número de procuras começadas num tique.
fn bloco(
    sim: &mut SimWorld,
    b: &mut PhysicsBridge,
    quem: &[Entity],
    t: &mut u64,
) -> (f64, u64, u64) {
    let (mut pior, mut trabalho, mut mais) = (0.0f64, 0u64, 0u64);
    for _ in 0..BLOCO {
        *t += 1;
        let antes = procuras(b, quem);
        let t0 = Instant::now();
        b.dispatch(sim, true, *t);
        pior = pior.max(t0.elapsed().as_secs_f64() * 1e3);
        trabalho = trabalho.max(b.nav_search_critical_work());
        mais = mais.max(procuras(b, quem) - antes);
    }
    (pior, trabalho, mais)
}

/// O bloco depois de cada movimento da porta: o pior tique cai nos primeiros.
const BLOCO: usize = 16;
/// As rodadas intercaladas (a ordem das versões roda a cada uma).
const RODADAS: usize = 7;

/// As versões, lado a lado: o nome, as fatias ligadas, e quantas procuras a meio em paralelo.
const VERSOES: [(&str, bool, usize); 4] = [
    ("A inteira", false, 0),
    ("B série  ", true, 0),
    ("B par 8  ", true, 8),
    ("B par 16 ", true, 16),
];

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// O mínimo e a mediana.
fn min_med(v: &mut [f64]) -> (f64, f64) {
    v.sort_by(f64::total_cmp);
    (v[0], v[v.len() / 2])
}

fn main() {
    // `FASES=<n>`: o tique com `n` agentes e NADA a mudar (o CONTROLO), a média de 30.
    if let Ok(n) = std::env::var("FASES") {
        let n: usize = n.parse().expect("um número");
        let (mut sim, _porta, _quem) = cena(n);
        let mut b = PhysicsBridge::new();
        let mut t = 0u64;
        for _ in 0..30 {
            t += 1;
            b.dispatch(&mut sim, true, t);
        }
        let t0 = Instant::now();
        for _ in 0..30 {
            t += 1;
            b.dispatch(&mut sim, true, t);
        }
        println!(
            "{n} agentes, nada a mudar: {:.2} ms por tique",
            t0.elapsed().as_secs_f64() * 1e3 / 30.0
        );
        return;
    }
    // ⭐ (W15, a regra do dono de 05/10) As versões no MESMO processo, escolhidas em execução
    // (`set_nav_slices`, `set_nav_parallel`): A = toda procura inteira no tique; B = a vez e as fatias,
    // em série ou com `n` procuras a meio em paralelo. Intercaladas em blocos curtos com a ordem rodada;
    // o MÍNIMO das rodadas, a mediana ao lado só como controlo.
    let orc: u64 = std::env::var("ORCAMENTO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20_000);
    println!("# loadavg: {} · orçamento {orc}", loadavg());
    println!(
        "# versão | agentes | pior tique depois da porta (ms, mín · mediana de {RODADAS}) | trabalho crítico num tique (máx) | procuras num tique (máx) | CONTROLO sem porta: pior tique (mín · mediana) | falta (m)"
    );
    for &n in &[10usize, 50, 200] {
        let mut v: Vec<(SimWorld, Entity, Vec<Entity>, PhysicsBridge, u64)> = VERSOES
            .iter()
            .map(|&(_, fatias, paralelas)| {
                let (mut sim, porta, quem) = cena(n);
                let mut b = PhysicsBridge::new();
                b.set_nav_replan_budget(orc);
                b.set_nav_slices(fatias);
                b.set_nav_parallel(paralelas);
                let mut t = 0u64;
                for _ in 0..30 {
                    t += 1;
                    b.dispatch(&mut sim, true, t);
                }
                (sim, porta, quem, b, t)
            })
            .collect();
        let nv = v.len();
        let mut porta = vec![Vec::new(); nv];
        let mut ctl = vec![Vec::new(); nv];
        let mut trab = vec![0u64; nv];
        let mut proc = vec![0u64; nv];
        for r in 0..RODADAS {
            for k in 0..nv {
                let i = (k + r) % nv;
                let (sim, p, quem, b, t) = &mut v[i];
                sim.world_mut()
                    .get_mut::<Transform>(*p)
                    .expect("a porta")
                    .translation = Vec2::new(50.3 + ((r + 1) % 2) as f32, 50.3);
                let (ms, w, m) = bloco(sim, b, quem, t);
                porta[i].push(ms);
                trab[i] = trab[i].max(w);
                proc[i] = proc[i].max(m);
                let (ms, _, _) = bloco(sim, b, quem, t);
                ctl[i].push(ms);
            }
        }
        for i in 0..nv {
            let (sim, _, quem, b, _) = &v[i];
            // O que falta, em média, a cada agente no fim (a régua de que a vez não os atrasa): pelo
            // caminho, ou — quem ainda espera o 1.º — a direito até ao alvo.
            let falta = quem
                .iter()
                .filter_map(|&e| {
                    let p = sim.world().get::<Transform>(e)?.translation;
                    let p = [f64::from(p.x), f64::from(p.y)];
                    let rt = b.nav_agent(e)?;
                    if !rt.path.is_empty() {
                        return Some(rt.remaining(p));
                    }
                    let NavTarget::Point(t) = sim.world().get::<NavAgent>(e)?.target else {
                        return None;
                    };
                    Some((p[0] - f64::from(t[0])).hypot(p[1] - f64::from(t[1])))
                })
                .sum::<f64>()
                / quem.len() as f64;
            let esperam = quem
                .iter()
                .filter(|&&e| b.nav_agent(e).is_none_or(|r| r.path.is_empty()))
                .count();
            let (pm, pd) = min_med(&mut porta[i]);
            let (cm, cd) = min_med(&mut ctl[i]);
            println!(
                "{} | {n:>4} | {pm:>7.2} · {pd:>7.2} | {:>7} | {:>4} | {cm:>7.2} · {cd:>7.2} | {falta:.2} ({esperam} sem caminho)",
                VERSOES[i].0, trab[i], proc[i]
            );
        }
    }
    println!("# loadavg: {}", loadavg());
}
