//! **Kill-criterion** (plano §7, 05_W5 §4): 0,5 ms por quadro para 1 000 — a COLA (cadeias de pinos com condutor
//! + quadriláteros, o caminho da ezpz) e a PILHA (1 000 itens, nunca medida desde a W1), na MESMA rodada:
//! variantes no mesmo processo, intercaladas em blocos, resumo = o MÍNIMO (`docs/DevOps/MEDIR_VELOCIDADE.md`).
//!
//! `#[ignore]`: o relógio só vale sozinho e no perfil `smoke` —
//! `bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-constraints-ecs --test it kill -- --ignored --nocapture`.

use std::time::{Duration, Instant};

use bevy_ecs::entity::Entity;
use ph2d_constraints::{Canais, CopyTransform, Kind, Space};
use ph2d_constraints_cola::Cola;
use ph2d_constraints_ecs::{Colas, Constraints, colar, evaluate};
use ph2d_ecs::{Locked, SimWorld, StableId, Transform};
use ph2d_preview_drive::{Driven, PreviewDrive};

const N: u64 = 1_000;
const TECTO: Duration = Duration::from_micros(500);
const BLOCOS: usize = 7;
const QUADROS: u32 = 40;

#[expect(clippy::cast_possible_truncation, reason = "o `Transform` da casa é f32")]
fn tf(x: f64, y: f64, rot: f64) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x as f32;
    t.translation.y = y as f32;
    t.rotation = rot as f32;
    t
}

fn pose(t: Transform) -> ph2d_constraints::Pose2 {
    ph2d_constraints::Pose2 {
        x: f64::from(t.translation.x),
        y: f64::from(t.translation.y),
        rot: f64::from(t.rotation),
        sx: 1.0,
        sy: 1.0,
    }
}

/// 1 000 objectos colados: 100 quadriláteros (400 objectos, ezpz) e 60 cadeias de 10 pinos (fecho). Devolve os
/// condutores (a manivela de cada quadrilátero, a cabeça de cada cadeia).
fn cena_cola(sim: &mut SimWorld) -> Vec<Entity> {
    let mut condutores = Vec::new();
    let mut colas = Colas::default();
    let mut id = 0u64;
    let mut novo = |sim: &mut SimWorld, t: Transform| {
        id += 1;
        (sim.world_mut().spawn((StableId(id), t)).id(), id)
    };
    for k in 0..100 {
        let ox = f64::from(k) * 10.0;
        let a = (3.5f64.powi(2) - 9.0 + 9.0) / 6.0;
        let h = (3.5f64.powi(2) - a * a).sqrt();
        let ts = [tf(ox, 0.0, 0.0), tf(ox, 0.0, 0.0), tf(ox + 1.0, 0.0, h.atan2(a)), tf(ox + 4.0, 0.0, h.atan2(a - 3.0))];
        let es: Vec<(Entity, u64)> = ts.iter().map(|&t| novo(sim, t)).collect();
        sim.world_mut().entity_mut(es[0].0).insert(Locked);
        let p: Vec<_> = ts.iter().map(|&t| pose(t)).collect();
        for (i, j, m) in [(0, 1, [ox, 0.0]), (1, 2, [ox + 1.0, 0.0]), (2, 3, [ox + 1.0 + a, h]), (0, 3, [ox + 4.0, 0.0])] {
            colas.push(|n| Cola::pino(n, es[i].1, es[j].1, p[i], p[j], m));
        }
        condutores.push(es[1].0);
    }
    for k in 0..60 {
        let oy = 20.0 + f64::from(k) * 2.0;
        let es: Vec<(Entity, u64, Transform)> = (0..10)
            .map(|i| {
                let t = tf(f64::from(i) * 2.0, oy, 0.0);
                let (e, s) = novo(sim, t);
                (e, s, t)
            })
            .collect();
        for w in es.windows(2) {
            let m = [f64::from(w[1].2.translation.x), oy];
            colas.push(|n| Cola::pino(n, w[0].1, w[1].1, pose(w[0].2), pose(w[1].2), m));
        }
        condutores.push(es[0].0);
    }
    let dono = sim.world_mut().spawn(StableId(1_000_000)).id();
    sim.world_mut().entity_mut(dono).insert(colas);
    condutores
}

/// 1 000 itens: 500 donos com Copy Location + Copy Rotation, cada um a seguir o anterior.
fn cena_pilha(sim: &mut SimWorld) -> Entity {
    let raiz = sim.world_mut().spawn((StableId(1), tf(0.0, 0.0, 0.0))).id();
    for i in 2..=N / 2 + 1 {
        let e = sim.world_mut().spawn((StableId(i), tf(0.0, 0.0, 0.0))).id();
        let mut c = Constraints::default();
        for canais in [Canais::LOCATION, Canais::ROTATION] {
            let k = Kind::CopyTransform(CopyTransform { canais, space: Space::World, ..CopyTransform::default() });
            c.push(k, Some(StableId(i - 1)));
        }
        sim.world_mut().entity_mut(e).insert(c);
    }
    raiz
}

/// O condutor anda (como a curva, pelo `PreviewDrive`).
#[expect(clippy::cast_possible_truncation, reason = "o `Transform` da casa é f32")]
fn conduz(sim: &mut SimWorld, drive: &mut PreviewDrive, e: Entity, f: u32) {
    let agora = *sim.world().get::<Transform>(e).unwrap();
    let mut novo = agora;
    novo.rotation = f64::from(f % 48).to_radians() as f32 * 7.5;
    drive.driven(e, Driven::SolverPose(agora), Driven::SolverPose(novo));
    *sim.world_mut().get_mut::<Transform>(e).unwrap() = novo;
}

#[test]
#[ignore = "relógio: corre sozinho, --profile smoke"]
fn glue_frame_under_half_a_millisecond_per_thousand() {
    let (mut sc, mut dc) = (SimWorld::new(), PreviewDrive::default());
    let condutores = cena_cola(&mut sc);
    let (mut sp, mut dp) = (SimWorld::new(), PreviewDrive::default());
    let raiz = cena_pilha(&mut sp);
    colar(&mut sc, &mut dc);
    evaluate(&mut sp, &mut dp);

    let (mut cola, mut pilha) = (Duration::MAX, Duration::MAX);
    let mut f = 0;
    for bloco in 0..BLOCOS {
        for variante in [bloco % 2, 1 - bloco % 2] {
            for _ in 0..QUADROS {
                f += 1;
                let t = if variante == 0 {
                    for &c in &condutores {
                        conduz(&mut sc, &mut dc, c, f);
                    }
                    let t0 = Instant::now();
                    colar(&mut sc, &mut dc);
                    let t = t0.elapsed();
                    dc.settle();
                    t
                } else {
                    conduz(&mut sp, &mut dp, raiz, f);
                    let t0 = Instant::now();
                    evaluate(&mut sp, &mut dp);
                    let t = t0.elapsed();
                    dp.settle();
                    t
                };
                let m = if variante == 0 { &mut cola } else { &mut pilha };
                *m = (*m).min(t);
            }
        }
    }
    println!("kill: cola {cola:?} / 1 000 objectos · pilha {pilha:?} / 1 000 itens (mínimo de {BLOCOS}×{QUADROS})");
    assert!(cola <= TECTO, "cola {cola:?} > {TECTO:?}");
    assert!(pilha <= TECTO, "pilha {pilha:?} > {TECTO:?}");
}
