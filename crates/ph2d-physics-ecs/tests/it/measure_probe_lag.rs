//! **QUANTO O SENSOR PUBLICADO ATRASA O CORPO** — a sonda que responde ao report
//! do Enio (*"temos um drift dos gizmos dos sensores do platformer ao se mover o
//! Player"*) com um número em vez de uma hipótese.
//!
//! Rodar:
//! ```text
//! cargo test -p ph2d-physics-ecs --release --test measure_probe_lag -- --ignored --nocapture
//! ```

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    ProbeKind, ProbeShape, RigidBody,
};

const FLOAT: f32 = 0.9;

struct Rig {
    sim: SimWorld,
    bridge: PhysicsBridge,
    player: ph2d_ecs::Entity,
}

fn rig() -> Rig {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 60.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    let player = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
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
                cling_distance: 0.1,
                wall_reach: 0.15,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, FLOAT)),
        ))
        .id();
    Rig {
        sim,
        bridge: PhysicsBridge::new(),
        player,
    }
}

fn leg_origin_x(bridge: &PhysicsBridge) -> f32 {
    for m in bridge.player_probe_marks() {
        if m.kind == ProbeKind::Ground
            && let ProbeShape::Ray { origin, .. } = m.shape
        {
            return origin[0];
        }
    }
    f32::NAN
}

#[test]
#[ignore = "sonda: rode com --ignored --nocapture"]
fn measure_how_far_the_published_sensor_lags_the_body() {
    let mut r = rig();
    let mut t = 0u64;
    println!("tick |   corpo x |  sensor x |   atraso");
    for i in 0..90u64 {
        r.bridge.set_player_input(
            r.player,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
        let body = r
            .sim
            .world()
            .get::<Transform>(r.player)
            .map(|x| x.translation.x)
            .unwrap_or(f32::NAN);
        let sensor = leg_origin_x(&r.bridge);
        if i % 10 == 9 || i < 3 {
            println!("{t:4} | {body:9.4} | {sensor:9.4} | {:8.4}", body - sensor);
        }
    }
}
