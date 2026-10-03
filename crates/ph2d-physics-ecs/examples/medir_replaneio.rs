//! A SONDA DO REPLANEIO EM MASSA (plano 30, W9) — quando UMA porta muda a malha, quantos agentes
//! replaneiam no MESMO tique, e quanto custa esse tique? Corre-se em `--release`, com a carga ao lado:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-physics-ecs --release --example medir_replaneio
//! ```
//!
//! A cena da `medir_nav_tique` (`100 × 100 m`, `1 000` caixas, a semente fixa) com `N` agentes do
//! mesmo raio (a mesma malha), cada um com o seu alvo longe. A porta alterna entre dois sítios; a
//! régua é o PIOR tique (e o maior número de procuras num tique) da janela de oito que se segue a cada
//! movimento, e o CONTROLO é a mesma janela sem a porta mexer.
//! ⛔ Nenhum número de tempo desta saída vale acima de `load ~5`; as procuras não dependem da carga.

use std::time::Instant;

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, NavAgent, NavRegion, NavTarget, PhysicsBridge, RigidBody,
    TopDownPlayer,
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

/// Os agentes que a fila ainda deve, e quantos deles têm o caminho PARTIDO.
fn devidos(b: &PhysicsBridge, quem: &[Entity]) -> (usize, usize) {
    let rts: Vec<_> = quem.iter().filter_map(|&e| b.nav_agent(e)).collect();
    (
        rts.iter().filter(|rt| rt.owed > 0).count(),
        rts.iter().filter(|rt| rt.owed > 0 && rt.broken).count(),
    )
}

/// Uma janela de `JANELA` tiques: o pior em ms, o maior número de procuras num tique, o tique em que
/// a fila ficou vazia e o tique em que o último PARTIDO foi servido (`0` = nunca houve).
fn janela(
    sim: &mut SimWorld,
    b: &mut PhysicsBridge,
    quem: &[Entity],
    t: &mut u64,
) -> (f64, u64, usize, usize) {
    let (mut pior, mut mais, mut vazia, mut partidos) = (0.0f64, 0u64, 0usize, 0usize);
    for k in 1..=JANELA {
        *t += 1;
        let antes = procuras(b, quem);
        let t0 = Instant::now();
        b.dispatch(sim, true, *t);
        pior = pior.max(t0.elapsed().as_secs_f64() * 1e3);
        mais = mais.max(procuras(b, quem) - antes);
        let (d, p) = devidos(b, quem);
        if d > 0 {
            vazia = k + 1;
        }
        if p > 0 {
            partidos = k + 1;
        }
    }
    (pior, mais, vazia, partidos)
}

/// A janela depois de cada movimento da porta: a fila mais longa medida esvazia dentro dela.
const JANELA: usize = 96;

fn main() {
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("# loadavg: {}", load.trim());
    println!(
        "# orçamento | agentes | pior tique com a porta (ms, mediana · máx de 6) | procuras num tique (máx) | tiques até a fila esvaziar · até o último partido (máx) | CONTROLO: pior tique · procuras"
    );
    let orcamentos: Vec<u64> = std::env::var("ORCAMENTOS").map_or_else(
        |_| vec![u64::MAX, 40_000, 20_000, 10_000],
        |v| {
            v.split(',')
                .map(|x| x.parse().expect("um número"))
                .collect()
        },
    );
    for &orc in &orcamentos {
        for &n in &[10usize, 50, 200] {
            let (mut sim, porta, quem) = cena(n);
            let mut b = PhysicsBridge::new();
            b.set_nav_replan_budget(orc);
            let mut t = 0u64;
            for _ in 0..30 {
                t += 1;
                b.dispatch(&mut sim, true, t);
            }
            let mut piores = Vec::new();
            let (mut mais, mut vazia, mut partidos) = (0u64, 0usize, 0usize);
            for k in 0..6 {
                sim.world_mut()
                    .get_mut::<Transform>(porta)
                    .expect("a porta")
                    .translation = Vec2::new(50.3 + ((k + 1) % 2) as f32, 50.3);
                let (p, m, v, pt) = janela(&mut sim, &mut b, &quem, &mut t);
                piores.push(p);
                mais = mais.max(m);
                vazia = vazia.max(v);
                partidos = partidos.max(pt);
            }
            let (ctl, ctl_m, _, _) = janela(&mut sim, &mut b, &quem, &mut t);
            piores.sort_by(f64::total_cmp);
            let nome = if orc == u64::MAX {
                "sem fila".to_string()
            } else {
                orc.to_string()
            };
            println!(
                "{nome:>11} | {n:>7} | {:>8.2} · {:>8.2} | {mais:>6} | {vazia:>4} · {partidos:>3} | {ctl:>8.2} · {ctl_m}",
                piores[piores.len() / 2],
                piores[piores.len() - 1]
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
