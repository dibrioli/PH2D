//! **SONDA — o que a ÁGUA faz ao personagem, e o que ela faz a uma jangada.**
//!
//! Report do Enio (2026-08-07): *"nosso player atual não interage corretamente
//! com a água (como a jangada faz)"*.
//!
//! Esta sonda **não conserta nada** — ela nomeia o defeito com números antes de
//! a wave ser escrita, que é a política deste módulo desde a W11. O CONTROLE é
//! uma cápsula **idêntica** sem o `PlatformPlayer`: mesma forma, mesma
//! densidade, mesma poça. Tudo o que os dois fizerem diferente é do player.
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_player_in_water -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaBuoyancy, AreaDrag, AreaEffector, BodyKind, Collider, ColliderShape, LockRotation,
    PhysicsBridge, PlatformPlayer, PlayerInput, RigidBody,
};

/// A cápsula das fixtures do player.
const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;
/// A altura de flutuação das fixtures do player (`platform_scene`).
const FLOAT: f32 = 0.9;
/// A densidade do fluido, 4× a do corpo — o mesmo par do `compound_zone`.
const FLUID: f32 = 4.0;

fn capsule() -> Collider {
    Collider {
        shape: ColliderShape::Capsule {
            half_height: HALF_H,
            radius: RADIUS,
        },
        density: 1.0,
        ..Collider::default()
    }
}

/// A poça: um sensor com empuxo e arrasto, do `y = -6` ao `y = 0`.
///
/// ⚠️ **O arrasto não é enfeite** — empuxo sem resistência é uma mola sem
/// amortecimento, e o repouso que este oráculo lê nunca chegaria (a lição já
/// escrita no `compound_zone`).
fn pool(sim: &mut SimWorld, current: f32) {
    let mut e = sim.world_mut().spawn((
        Name::new("Pool"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 3.0,
            },
            ..Collider::default()
        },
        AreaBuoyancy(FLUID),
        AreaDrag(0.6),
        Transform::from_translation(Vec2::new(0.0, -3.0)),
    ));
    if current != 0.0 {
        e.insert(AreaEffector {
            force: [current, 0.0],
        });
    }
}

/// Um fundo sólido em `y = floor_top - 0.5`, ou nenhum.
fn floor(sim: &mut SimWorld, floor_top: f32) {
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, floor_top - 0.5)),
    ));
}

/// `player = false` ⇒ o CONTROLE (uma cápsula solta).
fn subject(sim: &mut SimWorld, player: bool, y: f32) -> Entity {
    subject_with(sim, player, y, false)
}

/// `neutral_gravity` zera os multiplicadores do pulo — a ablação que separa
/// *"o player pesa outra coisa"* de *"o player é empurrado por outra coisa"*.
///
/// ⚠️ **Ela lê `+0,0000` na árvore de HOJE, e isso NÃO é *"não faz diferença"*:**
/// enquanto o raio de chão vê a poça, o personagem está **de pé** e os
/// multiplicadores do pulo são inertes por definição. A ablação só separa depois
/// da primeira metade da cura — medida com ela, a linha `g=1` cai de `0,3446`
/// para `0,2174`, que é o controle **ao quarto decimal**. *Um instrumento
/// mascarado por outro defeito lê como um resultado.*
fn subject_with(sim: &mut SimWorld, player: bool, y: f32, neutral_gravity: bool) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        capsule(),
        LockRotation,
        Transform::from_translation(Vec2::new(0.0, y)),
    ));
    if player {
        let base = PlatformPlayer {
            float_height: FLOAT,
            ..PlatformPlayer::default()
        };
        e.insert(if neutral_gravity {
            PlatformPlayer {
                takeoff_gravity: 1.0,
                peak_gravity: 1.0,
                fall_gravity: 1.0,
                cut_gravity: 1.0,
                ..base
            }
        } else {
            base
        });
    }
    e.id()
}

fn pose(sim: &SimWorld) -> (f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Subject" {
            found = Some((t.translation.x, t.translation.y));
        }
    }
    found.expect("o sujeito tem de existir")
}

/// Roda `ticks` e devolve a pose final.
fn run(
    sim: &mut SimWorld,
    bridge: &mut PhysicsBridge,
    who: Entity,
    is_player: bool,
    input: PlayerInput,
    ticks: u64,
) -> (f32, f32) {
    if is_player {
        bridge.set_player_input(who, input);
    }
    for t in 1..=ticks {
        bridge.dispatch(sim, true, t);
    }
    pose(sim)
}

/// **A poça FUNDA** — nada ao alcance do sensor de chão.
///
/// A pergunta: o personagem boia na mesma linha d'água que a cápsula solta?
#[test]
#[ignore = "sonda de medição"]
fn measure_deep_water() {
    println!("\n=== POÇA FUNDA (sem fundo ao alcance) ===");
    println!("{:<12} {:>10} {:>10}", "sujeito", "y final", "vs controle");
    let mut control_y = 0.0;
    for (label, player, neutral) in [
        ("capsula", false, false),
        ("player", true, false),
        ("player g=1", true, true),
    ] {
        let mut sim = SimWorld::new();
        pool(&mut sim, 0.0);
        let who = subject_with(&mut sim, player, 0.5, neutral);
        let mut bridge = PhysicsBridge::new();
        let (_, y) = run(
            &mut sim,
            &mut bridge,
            who,
            player,
            PlayerInput::default(),
            600,
        );
        if !player {
            control_y = y;
            println!("{label:<12} {y:>10.4} {:>10}", "-");
        } else {
            println!("{label:<12} {y:>10.4} {:>+10.4}", y - control_y);
        }
    }
}

/// **A poça RASA** — o fundo está ao alcance da perna.
///
/// A pergunta: a perna finge que a água não existe?
#[test]
#[ignore = "sonda de medição"]
fn measure_shallow_water() {
    println!("\n=== POÇA RASA (fundo em y = -2, dentro da água) ===");
    println!(
        "{:<12} {:>10} {:>12}",
        "sujeito", "y final", "acima do fundo"
    );
    for player in [false, true] {
        let mut sim = SimWorld::new();
        pool(&mut sim, 0.0);
        floor(&mut sim, -2.0);
        let who = subject(&mut sim, player, 0.5);
        let mut bridge = PhysicsBridge::new();
        let (_, y) = run(
            &mut sim,
            &mut bridge,
            who,
            player,
            PlayerInput::default(),
            600,
        );
        println!(
            "{:<12} {:>10.4} {:>12.4}",
            if player { "player" } else { "capsula" },
            y,
            y + 2.0
        );
    }
}

/// **A CORRENTE** — a água empurra para a direita.
///
/// A pergunta: o personagem PARADO é levado pela correnteza, como a jangada?
#[test]
#[ignore = "sonda de medição"]
fn measure_current() {
    println!("\n=== CORRENTE (AreaEffector 8 N para a direita, 5 s) ===");
    println!("{:<20} {:>10} {:>10}", "sujeito", "x final", "y final");
    for (label, player, deep) in [
        ("capsula (controle)", false, true),
        ("player", true, true),
        ("capsula, poca rasa", false, false),
        ("player, poca rasa", true, false),
    ] {
        let mut sim = SimWorld::new();
        pool(&mut sim, 8.0);
        if !deep {
            floor(&mut sim, -2.0);
        }
        let who = subject(&mut sim, player, 0.5);
        let mut bridge = PhysicsBridge::new();
        let (x, y) = run(
            &mut sim,
            &mut bridge,
            who,
            player,
            PlayerInput::default(),
            300,
        );
        println!("{label:<20} {x:>10.4} {y:>10.4}");
    }
}
