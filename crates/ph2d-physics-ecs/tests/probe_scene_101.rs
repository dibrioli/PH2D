//! **SONDA — a cena 101 pela porta do produto** (os dois reports do smoke).
//!
//! Reproduz a geometria da cena `PH2D_PHYSICS_SMOKE=101` headless e imprime a
//! TRAJETÓRIA, porque as duas queixas do Enio são sobre *para onde o personagem
//! vai*, e nenhum endpoint responde isso:
//!
//! 1. o CIANO (Spring) *"pula-pula"* e **SOBE** a rampa parado;
//! 2. o LARANJA (Snap) *"ao pousar se aproxima da rampa"* numa direção que
//!    parece a NORMAL dela.
//!
//! ⚠️ A geometria é copiada da cena, não da fixture `platform_scene`: o que se
//! mede aqui é o que o artista viu.
//!
//! `cargo test -p ph2d-physics-ecs --release --test probe_scene_101 -- --ignored --nocapture`
#![allow(dead_code)]

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerMode,
    RigidBody,
};

/// A altura de flutuação da cena 101.
const FLOAT: f32 = 0.9;
/// O amortecimento da cena 101 — um QUARTO do teto.
const DAMPING: f32 = 0.25;
/// A inclinação da rampa da cena, em radianos (desce para a DIREITA).
const SLOPE: f32 = -30.0 * core::f32::consts::PI / 180.0;

fn slab(sim: &mut SimWorld, name: &str, at: Vec2, half: [f32; 2], rot: f32) {
    sim.world_mut().spawn((
        Name::new(name.to_string()),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: half[0],
                half_y: half[1],
            },
            ..Collider::default()
        },
        Transform {
            rotation: rot,
            ..Transform::from_translation(at)
        },
    ));
}

fn player(sim: &mut SimWorld, name: &str, at: Vec2, kinematic: bool, damping: f32) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new(name.to_string()),
        Transform::from_translation(at),
        RigidBody {
            kind: if kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            ..Collider::default()
        },
        LockRotation,
        PlatformPlayer {
            float_height: FLOAT,
            spring_damping: damping,
            ..PlatformPlayer::default()
        },
    ));
    if kinematic {
        e.insert(PlayerMode::Kinematic);
    }
    e.id()
}

fn at(sim: &SimWorld, e: Entity) -> (f32, f32) {
    let t = sim.world().get::<Transform>(e).expect("transform");
    (t.translation.x, t.translation.y)
}

/// A cena 101, ou a metade dela que importa: chão + rampa + os dois.
fn scene_101(damping: f32) -> (SimWorld, PhysicsBridge, Entity, Entity) {
    let mut sim = SimWorld::new();
    slab(&mut sim, "Floor", Vec2::new(0.0, -0.5), [16.0, 0.5], 0.0);
    slab(&mut sim, "Ramp30", Vec2::new(-7.0, 1.3), [4.0, 0.5], SLOPE);
    let spring = player(&mut sim, "Spring", Vec2::new(-7.5, 2.5), false, damping);
    let snap = player(&mut sim, "Snap", Vec2::new(-6.0, 2.0), true, damping);
    (sim, PhysicsBridge::new(), spring, snap)
}

/// **REPORT 1 — o ciano pula, e para que LADO ele anda?**
///
/// ⚠️ O gate `the_kinematic_player_does_not_creep_up_a_ramp` mede
/// `(x - x0).abs()`, então ele **nunca soube a direção**. Aqui a coluna é
/// assinada.
#[test]
#[ignore = "sonda"]
fn probe_the_cyan_bounces_and_which_way_it_walks() {
    for damping in [0.25_f32, 1.0] {
        let (mut sim, mut bridge, spring, snap) = scene_101(damping);
        for t in 1..=120u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let (sx0, _) = at(&sim, spring);
        let (nx0, _) = at(&sim, snap);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for t in 121..=720u64 {
            bridge.dispatch(&mut sim, true, t);
            let (_, sy) = at(&sim, spring);
            // A altura ACIMA da rampa sob o personagem, para o quique aparecer
            // sem a rampa a mascarar.
            let (sx, _) = at(&sim, spring);
            let surface = 1.3 + 0.5 / SLOPE.cos() + (sx + 7.0) * SLOPE.tan();
            let h = sy - surface;
            lo = lo.min(h);
            hi = hi.max(h);
        }
        let (sx1, sy1) = at(&sim, spring);
        let (nx1, ny1) = at(&sim, snap);
        println!(
            "damping {damping:.2}:  CIANO deriva {:+.4} m (x {sx0:.3} -> {sx1:.3}, y {sy1:.3})  \
             quique {:.4} m (de {lo:.4} a {hi:.4})   |   LARANJA deriva {:+.4} m (y {ny1:.3})",
            sx1 - sx0,
            hi - lo,
            nx1 - nx0,
        );
    }
}

/// **O QUIQUE DE 2,9 m — a tabela do `RideConfig` diz 199 mm em `d = 0,25`.**
///
/// Ela mede *"queda de 1,5 m NO PLANO"*. A cena 101 é uma RAMPA, e nasce com o
/// personagem **abaixo** da altura de flutuação. Esta sonda separa as duas
/// causas: a rampa e o berço.
#[test]
#[ignore = "sonda"]
fn probe_what_makes_the_bounce_29_metres() {
    for (label, slope) in [("plano", 0.0_f32), ("rampa 30", SLOPE)] {
        for cradle in ["no repouso", "como a cena 101"] {
            for d in [0.25_f32, 0.5, 1.0] {
                let mut sim = SimWorld::new();
                slab(&mut sim, "Ramp", Vec2::new(-7.0, 1.3), [4.0, 0.5], slope);
                let x = -7.5;
                let surface = 1.3 + 0.5 / slope.cos() + (x + 7.0) * slope.tan();
                let y = if cradle == "no repouso" {
                    surface + FLOAT
                } else {
                    2.5
                };
                let who = player(&mut sim, "P", Vec2::new(x, y), false, d);
                let mut bridge = PhysicsBridge::new();
                let (mut lo, mut hi) = (f32::MAX, f32::MIN);
                let mut last_peak = 0.0_f32;
                for t in 1..=600u64 {
                    bridge.dispatch(&mut sim, true, t);
                    let (px, py) = at(&sim, who);
                    let s = 1.3 + 0.5 / slope.cos() + (px + 7.0) * slope.tan();
                    let h = py - s;
                    if t > 60 {
                        lo = lo.min(h);
                        hi = hi.max(h);
                    }
                    if t > 540 {
                        last_peak = last_peak.max(h);
                    }
                }
                println!(
                    "{label:9} / berco {cradle:14} / d {d:.2}:  quique {:7.1} mm   \
                     (pico no fim {:7.1} mm sobre o repouso)   berco {:.3} vs repouso {:.3}",
                    (hi - lo) * 1000.0,
                    (last_peak - FLOAT) * 1000.0,
                    y - surface,
                    FLOAT,
                );
            }
        }
    }
}

/// **A ATRIBUIÇÃO — o chute morro-acima é dos DOIS modos ou só do Snap?**
///
/// A lei da caminhada, com `drive = 0`, freia a velocidade **ao longo da
/// tangente da rampa**; um corpo em QUEDA tem componente tangencial em qualquer
/// inclinação, então o freio a lê como *"estou a escorregar"* e empurra
/// morro-ACIMA. A previsão é `v · sin θ · cos θ` de velocidade horizontal.
#[test]
#[ignore = "sonda"]
fn probe_whether_the_uphill_kick_is_both_modes() {
    for kinematic in [false, true] {
        for drop in [0.5_f32, 1.5, 3.0] {
            let mut sim = SimWorld::new();
            slab(&mut sim, "Ramp30", Vec2::new(-7.0, 1.3), [4.0, 0.5], SLOPE);
            let surface = 1.3 + 0.5 / SLOPE.cos();
            let who = player(
                &mut sim,
                "P",
                Vec2::new(-7.0, surface + FLOAT + drop),
                kinematic,
                1.0,
            );
            let mut bridge = PhysicsBridge::new();
            let x0 = at(&sim, who).0;
            for t in 1..=240u64 {
                bridge.dispatch(&mut sim, true, t);
            }
            let (x1, y1) = at(&sim, who);
            let v = (2.0 * 9.81 * drop).sqrt();
            println!(
                "{:9}  queda {drop:.1} m (v {v:.2} m/s):  desvio {:+.4} m  (previsto morro-acima \
                 {:.4} m/s)   repouso y {y1:.3}",
                if kinematic { "SNAP" } else { "SPRING" },
                x1 - x0,
                v * 0.5 * 0.866,
            );
        }
    }
}

/// **O QUE A CENA PODE MOSTRAR NO DEFAULT** — os gates correm detunados de
/// propósito; o artista corre no default. Esta sonda mede o que SOBRA lá.
#[test]
#[ignore = "sonda"]
fn probe_what_differs_at_the_shipping_default() {
    let flat = |kinematic: bool, drop: f32| -> (f32, f32) {
        let mut sim = SimWorld::new();
        slab(&mut sim, "Floor", Vec2::new(0.0, -0.5), [16.0, 0.5], 0.0);
        let who = player(&mut sim, "P", Vec2::new(0.0, FLOAT + drop), kinematic, 1.0);
        let mut bridge = PhysicsBridge::new();
        let mut worst = f32::INFINITY;
        for t in 1..=600u64 {
            bridge.dispatch(&mut sim, true, t);
            if t > 30 {
                worst = worst.min(at(&sim, who).1);
            }
        }
        (at(&sim, who).1, worst)
    };
    let (dyn_rest, _) = flat(false, 0.0);
    let (kin_rest, _) = flat(true, 0.0);
    println!(
        "repouso no plano:  SPRING {dyn_rest:.4}   SNAP {kin_rest:.4}   (diferenca {:.4} m)",
        dyn_rest - kin_rest
    );
    println!(
        "{:>8} {:>14} {:>14}",
        "queda", "afunda SPRING", "afunda SNAP"
    );
    for drop in [0.5_f32, 2.0, 5.0, 10.0] {
        println!(
            "{drop:>8.1} {:>14.4} {:>14.4}",
            (dyn_rest - flat(false, drop).1).max(0.0),
            (kin_rest - flat(true, drop).1).max(0.0),
        );
    }
}

/// **ABLAÇÃO — o chute morro-acima é do FREIO da caminhada?**
///
/// Com `acceleration = 0` a `walk` devolve `Motor::default()` no primeiro `if`,
/// então o freio some — e é um knob que o artista tem. Se o desvio some com
/// ele, a causa é o freio a ler a QUEDA como *"estou a escorregar"*.
#[test]
#[ignore = "sonda"]
fn probe_whether_the_walk_brake_is_the_uphill_kick() {
    for kinematic in [false, true] {
        for accel in [60.0_f32, 0.0] {
            let mut sim = SimWorld::new();
            slab(&mut sim, "Ramp30", Vec2::new(-7.0, 1.3), [4.0, 0.5], SLOPE);
            let surface = 1.3 + 0.5 / SLOPE.cos();
            let who = player(
                &mut sim,
                "P",
                Vec2::new(-7.0, surface + FLOAT + 1.5),
                kinematic,
                1.0,
            );
            {
                let mut e = sim.world_mut().entity_mut(who);
                if let Some(mut p) = e.get_mut::<PlatformPlayer>() {
                    p.acceleration = accel;
                    p.air_acceleration = 0.0;
                }
            }
            let mut bridge = PhysicsBridge::new();
            let x0 = at(&sim, who).0;
            for t in 1..=240u64 {
                bridge.dispatch(&mut sim, true, t);
            }
            println!(
                "{:7}  acceleration {accel:5.1}:  desvio {:+.4} m",
                if kinematic { "SNAP" } else { "SPRING" },
                at(&sim, who).0 - x0,
            );
        }
    }
}

/// **O QUE A CORREÇÃO DE ORDEM REMOVE** — o chute do tique de CONTATO é um só e
/// fica; o que somem são os tiques SEGUINTES, em que a lei relia uma queda que o
/// integrador já ia apagar.
#[test]
#[ignore = "sonda"]
fn probe_the_travel_after_the_contact_tick() {
    for drop in [0.5_f32, 1.5, 3.0] {
        let mut sim = SimWorld::new();
        slab(&mut sim, "Ramp30", Vec2::new(-7.0, 1.3), [4.0, 0.5], SLOPE);
        let surface = 1.3 + 0.5 / SLOPE.cos();
        let who = player(
            &mut sim,
            "P",
            Vec2::new(-7.0, surface + FLOAT + drop),
            true,
            1.0,
        );
        let mut bridge = PhysicsBridge::new();
        let mut prev = at(&sim, who);
        let mut contact = None;
        let mut after = 0.0_f32;
        let mut moving_ticks = 0u32;
        for t in 1..=240u64 {
            bridge.dispatch(&mut sim, true, t);
            let now = at(&sim, who);
            let dx = now.0 - prev.0;
            if contact.is_none() && dx.abs() > 1.0e-4 {
                contact = Some(t);
            } else if contact.is_some() {
                after += dx;
                if dx.abs() > 1.0e-4 {
                    moving_ticks += 1;
                }
            }
            prev = now;
        }
        println!(
            "queda {drop:.1} m:  contato no tique {:?}   deslocamento DEPOIS dele {after:+.5} m   \
             ({moving_ticks} tiques ainda a andar)",
            contact
        );
    }
}

/// **REPORT 2 — para onde o laranja vai ao POUSAR na rampa?**
///
/// Imprime o passo por tique. A normal EXTERNA da rampa é `(+0,500, +0,866)`;
/// a interna (o que a seta do Enio desenha) é `(−0,500, −0,866)`.
#[test]
#[ignore = "sonda"]
fn probe_where_the_orange_goes_when_it_lands() {
    let mut sim = SimWorld::new();
    slab(&mut sim, "Ramp30", Vec2::new(-7.0, 1.3), [4.0, 0.5], SLOPE);
    // Largado 1,5 m acima da superfície, no meio da rampa.
    let x = -7.0;
    let surface = 1.3 + 0.5 / SLOPE.cos();
    let who = player(&mut sim, "Snap", Vec2::new(x, surface + 1.5), true, DAMPING);
    let mut bridge = PhysicsBridge::new();
    let mut prev = at(&sim, who);
    println!("  tick        x        y        dx        dy    direcao");
    for t in 1..=90u64 {
        bridge.dispatch(&mut sim, true, t);
        let now = at(&sim, who);
        let (dx, dy) = (now.0 - prev.0, now.1 - prev.1);
        let len = (dx * dx + dy * dy).sqrt();
        if len > 1.0e-5 {
            println!(
                "  {t:4}  {:7.4}  {:7.4}  {dx:+8.5}  {dy:+8.5}   ({:+.3}, {:+.3})",
                now.0,
                now.1,
                dx / len,
                dy / len
            );
        }
        prev = now;
    }
    let surface_under = 1.3 + 0.5 / SLOPE.cos() + (prev.0 + 7.0) * SLOPE.tan();
    println!(
        "  repouso: x {:.4}  y {:.4}  altura sobre a rampa {:.4} (a vertical de repouso e' 0,531)",
        prev.0,
        prev.1,
        prev.1 - surface_under
    );
}
