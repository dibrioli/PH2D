//! A SONDA DO TIQUE DA NAVEGAÇÃO (plano 30, W6) — quanto custa a ponte por tique quando NADA muda
//! (o preço de olhar para os obstáculos e ver que a malha está em dia) e quando UMA porta pára.
//! Corre-se em `--release`, com a carga ao lado:
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo run -p ph2d-physics-ecs --release --example medir_nav_tique
//! ```
//!
//! A cena: uma região de `100 × 100 m`, `n` caixas estáticas (a semente fixa), um agente a andar.
//! ⛔ Nenhum número desta saída vale acima de `load ~5`.

use std::time::Instant;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
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

fn cena(n: usize) -> (SimWorld, ph2d_ecs::Entity) {
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
    for _ in 0..n {
        let (x, y) = (r.next() * 100.0, r.next() * 100.0);
        if (x - 2.0).abs() < 3.0 && (y - 2.0).abs() < 3.0 {
            continue;
        }
        w.spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.2 + r.next(),
                    half_y: 0.2 + r.next(),
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    }
    let porta = w
        .spawn((
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.5,
                    half_y: 0.2,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(50.3, 50.3)),
        ))
        .id();
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
            target: NavTarget::Point([98.0, 98.0]),
            ..NavAgent::default()
        },
        Transform::from_translation(Vec2::new(2.0, 2.0)),
    ));
    (sim, porta)
}

fn main() {
    let load = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("# loadavg: {}", load.trim());
    println!(
        "# n_obst | tique_sem_nav_us | tique_parado_us | tique_porta_ms | controlo_ms   (minimo)"
    );
    for &n in &[100usize, 1000] {
        // O CONTROLO: a mesma cena sem agente (a ponte não olha para a navegação).
        let (mut s0, _) = cena(n);
        {
            let w = s0.world_mut();
            let mut q = w.query_filtered::<ph2d_ecs::Entity, bevy_ecs::query::With<NavAgent>>();
            let es: Vec<_> = q.iter(w).collect();
            for e in es {
                w.despawn(e);
            }
        }
        let mut b0 = PhysicsBridge::new();
        let mut sem = f64::INFINITY;
        for t in 1..=60u64 {
            let t0 = Instant::now();
            b0.dispatch(&mut s0, true, t);
            if t > 5 {
                sem = sem.min(t0.elapsed().as_secs_f64() * 1e6);
            }
        }
        let (mut sim, porta) = cena(n);
        let mut b = PhysicsBridge::new();
        let mut parado = f64::INFINITY;
        for t in 1..=60u64 {
            let t0 = Instant::now();
            b.dispatch(&mut sim, true, t);
            if t > 5 {
                parado = parado.min(t0.elapsed().as_secs_f64() * 1e6);
            }
        }
        // A porta alterna entre dois sítios. ⚠️ O tique que a RECORTA não é um tique fixo depois de
        // a mover (o que anda deixa de recortar num tique, o que pára volta a recortar noutro): a
        // régua é o MAIOR tique da janela de oito que se segue a cada movimento. ⛔ E o 1.º
        // movimento é REAL: a 1.ª redacção punha a porta onde ela já estava, nada mudava, e o
        // mínimo escolhia esse caso (`0,43 ms` lido por `11 ms`) — o CONTROLO abaixo é a janela sem
        // movimento nenhum, e a coluna da porta tem de ficar acima dela.
        let mut com_porta = f64::INFINITY;
        let mut t = 60u64;
        for k in 0..10 {
            sim.world_mut()
                .get_mut::<Transform>(porta)
                .expect("a porta")
                .translation = Vec2::new(50.3 + ((k + 1) % 2) as f32, 50.3);
            let mut pior = 0.0f64;
            for _ in 0..8 {
                t += 1;
                let t0 = Instant::now();
                b.dispatch(&mut sim, true, t);
                pior = pior.max(t0.elapsed().as_secs_f64() * 1e3);
            }
            com_porta = com_porta.min(pior);
        }
        let mut sem_mexer = 0.0f64;
        for _ in 0..8 {
            t += 1;
            let t0 = Instant::now();
            b.dispatch(&mut sim, true, t);
            sem_mexer = sem_mexer.max(t0.elapsed().as_secs_f64() * 1e3);
        }
        println!("{n:>8} | {sem:>16.1} | {parado:>15.1} | {com_porta:>14.3} | {sem_mexer:>9.3}");
    }
    println!(
        "# loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
