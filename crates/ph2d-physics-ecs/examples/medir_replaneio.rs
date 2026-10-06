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
//!
//! ⭐ (plano 30 §25, C e D — UMA rodada) Sem `LAMAS`, corre as duas cenas (`0` e `150` lamas) no mesmo
//! processo. Cada versão corre também num pool `rayon` de UMA thread (a web, D) — o resultado é o mesmo, só
//! o relógio muda. `PERSEGUIDORES=<k>` (por omissão `10`) agentes perseguem uma PRESA que anda em
//! círculo: a régua do atraso de quem replaneia porque o alvo andou (C2) — os tiques seguidos em que cada
//! um tem a procura pedida e por servir. E o trabalho TOTAL de procura por porta (o bloco dela mais o
//! CONTROLO): a régua de que a vez não DESPERDIÇA trabalho (C1).

use std::time::Instant;

use ph2d_ecs::stable_name_id;

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

fn cena(agentes: usize, lamas: usize) -> (SimWorld, Entity, Vec<Entity>) {
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
    for _ in 0..lamas {
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
    // A PRESA (um corpo que anda, ver [`presa_em`]) e quem a persegue: longe dela, no chão livre.
    w.spawn((
        Name::new("Presa"),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.4 },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(PRESA_C.0 + PRESA_R, PRESA_C.1)),
    ));
    let k = std::env::var("PERSEGUIDORES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10usize);
    let mut perseguidores = 0;
    while perseguidores < k {
        let (x, y) = (2.0 + a.next() * 96.0, 2.0 + a.next() * 96.0);
        if !livre(x, y) || (x - PRESA_C.0).hypot(y - PRESA_C.1) < 15.0 {
            continue;
        }
        perseguidores += 1;
        quem.push(
            w.spawn((
                Name::new("Perseguidor"),
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
                    target: NavTarget::Named(stable_name_id("Presa")),
                    ..NavAgent::default()
                },
                Transform::from_translation(Vec2::new(x, y)),
            ))
            .id(),
        );
    }
    (sim, porta, quem)
}

/// O centro e o raio do círculo da presa (`2 m/s`).
const PRESA_C: (f32, f32) = (50.0, 50.0);
const PRESA_R: f32 = 6.0;

/// Onde a presa está no tique `t`.
fn presa_no(t: u64) -> [f64; 2] {
    let a = t as f32 / 60.0 * 2.0 / PRESA_R;
    [
        f64::from(PRESA_C.0 + PRESA_R * a.cos()),
        f64::from(PRESA_C.1 + PRESA_R * a.sin()),
    ]
}

/// A presa no tique `t`.
fn presa_em(sim: &mut SimWorld, t: u64) {
    let [x, y] = presa_no(t);
    let w = sim.world_mut();
    let mut q = w.query::<(&Name, &mut Transform)>();
    for (n, mut tr) in q.iter_mut(w) {
        if n.as_str() == "Presa" {
            tr.translation = Vec2::new(x as f32, y as f32);
        }
    }
}

/// As procuras de todos os agentes até agora.
fn procuras(b: &PhysicsBridge, quem: &[Entity]) -> u64 {
    quem.iter()
        .filter_map(|&e| b.nav_agent(e))
        .map(|rt| rt.searches)
        .sum()
}

/// O que um bloco de `BLOCO` tiques mede: o pior tique em ms, o maior trabalho de procura do caminho
/// CRÍTICO num tique (a maior fatia em paralelo mais a condução), o maior número de procuras começadas
/// num tique, e o trabalho de procura TODO do bloco.
#[derive(Default)]
struct Medida {
    pior: f64,
    critico: u64,
    mais: u64,
    trabalho: u64,
}

/// (C2) O atraso dos perseguidores: os tiques SEGUIDOS em que cada um tem por acabar a procura que pediu
/// porque o ALVO ANDOU (`AMeio::persegue`) — o maior e a média das esperas. (A 3.ª rodada contava também a
/// dívida da porta, em que ele anda um caminho que ainda serve: misturava a espera inofensiva com a reacção.)
///
/// ⭐ (plano 30 §26, A) E o diagnóstico da DOBRA: o maior `recomecos` de uma procura a meio, de quem persegue
/// e das outras — ele mostrou que o pico de quem persegue primeiro é a dobra das OUTRAS.
#[derive(Default)]
struct Atraso {
    seguidos: Seguidos,
    dobra_persegue: u32,
    dobra_outras: u32,
}

#[derive(Default)]
struct Seguidos {
    por: std::collections::BTreeMap<Entity, u32>,
    max: u32,
    soma: u64,
    n: u64,
}

impl Seguidos {
    fn conta(&mut self, e: Entity, agora: bool) {
        let s = self.por.entry(e).or_default();
        if agora {
            *s += 1;
        } else if *s > 0 {
            (self.max, self.soma, self.n) = (self.max.max(*s), self.soma + u64::from(*s), self.n + 1);
            *s = 0;
        }
    }

    fn media(&self) -> f64 {
        self.soma as f64 / self.n.max(1) as f64
    }
}

impl Atraso {
    fn mede(&mut self, b: &PhysicsBridge, quem: &[Entity], perseguidores: &[Entity]) {
        for &e in perseguidores {
            let espera = b
                .nav_agent(e)
                .is_some_and(|rt| rt.a_meio.is_some_and(|a| a.persegue));
            self.seguidos.conta(e, espera);
        }
        for &e in quem {
            let Some(rt) = b.nav_agent(e) else { continue };
            let Some(a) = rt.a_meio else { continue };
            let d = if a.persegue {
                &mut self.dobra_persegue
            } else {
                &mut self.dobra_outras
            };
            *d = (*d).max(rt.recomecos);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn bloco(
    sim: &mut SimWorld,
    b: &mut PhysicsBridge,
    quem: &[Entity],
    perseguidores: &[Entity],
    t: &mut u64,
    pool: &rayon::ThreadPool,
    atraso: &mut Atraso,
) -> Medida {
    let mut m = Medida::default();
    for _ in 0..BLOCO {
        *t += 1;
        presa_em(sim, *t);
        let antes = procuras(b, quem);
        let tique = *t;
        let t0 = Instant::now();
        pool.install(|| b.dispatch(sim, true, tique));
        m.pior = m.pior.max(t0.elapsed().as_secs_f64() * 1e3);
        m.critico = m.critico.max(b.nav_search_critical_work());
        m.trabalho += b.nav_search_work();
        m.mais = m.mais.max(procuras(b, quem) - antes);
        atraso.mede(b, quem, perseguidores);
    }
    m
}

/// O bloco depois de cada movimento da porta: o pior tique cai nos primeiros.
const BLOCO: usize = 16;
/// As rodadas intercaladas (a ordem das versões roda a cada uma).
const RODADAS: usize = 7;

/// Uma versão: o nome, as fatias ligadas, quantas procuras a meio em paralelo, se corre num pool de UMA
/// thread (a web), o alvo à vista sem procura (§25, C2), as de quem persegue em vagas a mais (§26, A4) e se
/// um recomeço delas dobra a fatia (A5).
type Versao = (&'static str, bool, usize, bool, bool, (bool, bool));

/// As versões, lado a lado. As rodadas do plano 30 §25 mediram também `8 · 4 · 2` em paralelo e um tecto
/// do trabalho do passo em paralelo; a 1.ª da §26, quem persegue primeiro sem a dobra (A1) e o fim do
/// caminho a seguir o alvo (A2) — recusados (`target/prova/w16/…`, `target/prova/w17/…`).
const VERSOES: [Versao; 6] = [
    ("B produto            ", true, 16, false, true, (false, true)),
    ("B produto · 1t       ", true, 16, true, true, (false, true)),
    ("A4 vagas a mais      ", true, 16, false, true, (true, true)),
    ("A4 · 1t              ", true, 16, true, true, (true, true)),
    ("A5 A4 sem a dobra    ", true, 16, false, true, (true, false)),
    ("A5 · 1t              ", true, 16, true, true, (true, false)),
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
        let (mut sim, _porta, _quem) = cena(n, 0);
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
    // (`set_nav_slices`, `set_nav_parallel`, o pool): intercaladas em blocos curtos com a ordem rodada; o
    // MÍNIMO das rodadas, a mediana ao lado só como controlo.
    let orc: u64 = std::env::var("ORCAMENTO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20_000);
    let lamas: Vec<usize> = std::env::var("LAMAS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map_or(vec![0, 150], |n| vec![n]);
    let um = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("o pool de uma thread");
    let todos = rayon::ThreadPoolBuilder::new()
        .build()
        .expect("o pool de todos os núcleos");
    println!(
        "# loadavg: {} · orçamento {orc} · {} threads no pool de todos",
        loadavg(),
        todos.current_num_threads()
    );
    println!(
        "# lamas | versão | agentes | pior tique depois da porta (ms, mín · mediana de {RODADAS}) | crítico (máx) | procuras num tique (máx) | CONTROLO sem porta (mín · mediana) | trabalho por porta (mediana) | atraso do perseguidor (tiques: máx · média) | dobra (recomeços máx: quem persegue · as outras) | falta (m)"
    );
    for &l in &lamas {
        for &n in &[10usize, 50, 200] {
            let mut v: Vec<_> = VERSOES
                .iter()
                .map(|&(_, fatias, paralelas, uma, vista, (a_mais, dobra))| {
                    let (mut sim, porta, quem) = cena(n, l);
                    let persegue: Vec<Entity> = quem
                        .iter()
                        .copied()
                        .filter(|&e| {
                            sim.world()
                                .get::<Name>(e)
                                .is_some_and(|x| x.as_str() == "Perseguidor")
                        })
                        .collect();
                    let mut b = PhysicsBridge::new();
                    b.set_nav_replan_budget(orc);
                    b.set_nav_slices(fatias);
                    b.set_nav_parallel(paralelas);
                    b.set_nav_sight(vista);
                    b.set_nav_chase(a_mais, dobra);
                    let pool = if uma { &um } else { &todos };
                    let mut t = 0u64;
                    for _ in 0..30 {
                        t += 1;
                        presa_em(&mut sim, t);
                        pool.install(|| b.dispatch(&mut sim, true, t));
                    }
                    (sim, porta, quem, persegue, b, t, pool, Atraso::default())
                })
                .collect();
            let nv = v.len();
            let mut porta = vec![Vec::new(); nv];
            let mut ctl = vec![Vec::new(); nv];
            let mut total = vec![Vec::new(); nv];
            let mut trab = vec![0u64; nv];
            let mut proc = vec![0u64; nv];
            for r in 0..RODADAS {
                for k in 0..nv {
                    let i = (k + r) % nv;
                    let (sim, p, quem, persegue, b, t, pool, atraso) = &mut v[i];
                    sim.world_mut()
                        .get_mut::<Transform>(*p)
                        .expect("a porta")
                        .translation = Vec2::new(50.3 + ((r + 1) % 2) as f32, 50.3);
                    let m = bloco(sim, b, quem, persegue, t, pool, atraso);
                    porta[i].push(m.pior);
                    trab[i] = trab[i].max(m.critico);
                    proc[i] = proc[i].max(m.mais);
                    let c = bloco(sim, b, quem, persegue, t, pool, atraso);
                    ctl[i].push(c.pior);
                    total[i].push((m.trabalho + c.trabalho) as f64);
                }
            }
            for i in 0..nv {
                let (sim, _, quem, _, b, _, _, atraso) = &v[i];
                // O que falta, em média, a cada agente com um ponto por alvo (a régua de que a vez não os
                // atrasa): pelo caminho, ou — quem ainda espera o 1.º — a direito até ao alvo.
                let (soma, conta) = quem
                    .iter()
                    .filter_map(|&e| {
                        let NavTarget::Point(t) = sim.world().get::<NavAgent>(e)?.target else {
                            return None;
                        };
                        let p = sim.world().get::<Transform>(e)?.translation;
                        let p = [f64::from(p.x), f64::from(p.y)];
                        let rt = b.nav_agent(e)?;
                        if !rt.path.is_empty() {
                            return Some(rt.remaining(p));
                        }
                        Some((p[0] - f64::from(t[0])).hypot(p[1] - f64::from(t[1])))
                    })
                    .fold((0.0, 0usize), |(s, c), x| (s + x, c + 1));
                let falta = soma / conta.max(1) as f64;
                let esperam = quem
                    .iter()
                    .filter(|&&e| b.nav_agent(e).is_none_or(|r| r.path.is_empty()))
                    .count();
                let (pm, pd) = min_med(&mut porta[i]);
                let (cm, cd) = min_med(&mut ctl[i]);
                let (_, td) = min_med(&mut total[i]);
                println!(
                    "{l:>3} | {} | {n:>4} | {pm:>7.2} · {pd:>7.2} | {:>7} | {:>4} | {cm:>7.2} · {cd:>7.2} | {td:>9.0} | {:>3} · {:.2} | {} · {} | {falta:.2} ({esperam} sem caminho)",
                    VERSOES[i].0,
                    trab[i],
                    proc[i],
                    atraso.seguidos.max,
                    atraso.seguidos.media(),
                    atraso.dobra_persegue,
                    atraso.dobra_outras,
                );
            }
        }
    }
    println!("# loadavg: {}", loadavg());
}
