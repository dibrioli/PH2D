//! The bridge folds the optional `GravityScale` component into the sim
//! (ADR-0131 W8).
//!
//! `ph2d-physics` already proves `BodyDesc::gravity_scale` reaches the solver
//! (`gravity_scale.rs` there). This is the ECS half: a body carrying the
//! optional `GravityScale` component must be spawned with that multiplier, and
//! a body without one falls normally. e2e through the bridge (components →
//! spawn → step → readback `Transform`), so a bridge that never reads the
//! component leaves the weightless body falling.
//!
//! Mutation-tested: the bridge's `world.get::<GravityScale>(e)` folded to the
//! neutral `1.0` (ignoring the component) makes the weightless ball fall like
//! the control → RED.

use ph2d_core::Vec2;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, GravityScale, PhysicsBridge, RigidBody};

#[test]
fn a_gravity_scale_component_is_folded_into_the_sim() {
    let mut sim = SimWorld::new();
    let spawn_y = 5.0_f32;

    let dynamic_ball = |x: f32| {
        (
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, spawn_y)),
        )
    };

    // Control: no GravityScale component → full gravity.
    let control = sim.world_mut().spawn(dynamic_ball(0.0)).id();
    // Weightless: the same body, plus GravityScale(0.0). Spaced 5 m away so the
    // two never touch (there is no floor — this is free fall).
    let weightless = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(5.0, spawn_y)),
            GravityScale(0.0),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    for tick in 1..=60u64 {
        bridge.dispatch(&mut sim, true, tick);
    }

    let control_y = sim.world().get::<Transform>(control).unwrap().translation.y;
    let weightless_y = sim
        .world()
        .get::<Transform>(weightless)
        .unwrap()
        .translation
        .y;

    assert!(
        control_y < spawn_y - 1.0,
        "the control (no GravityScale) should have fallen; y={control_y}"
    );
    assert!(
        (weightless_y - spawn_y).abs() < 0.01,
        "the GravityScale(0.0) body must not fall — the bridge is not reading the \
         component (y={weightless_y}, spawn was {spawn_y})"
    );
}

/// ⭐ **A curva muda a gravidade A MEIO da queda** (F30 §C; o dono, 19/10: o Auto Key grava
/// tudo — e o que se grava tem de mandar no corpo VIVO). A ponte só re-descrevia o corpo em
/// repouso: a meio do play a escala nova ficava no componente e o corpo continuava a acelerar.
/// Reafinado no lugar, a velocidade que ele trazia FICA (não é recriado) e deixa de crescer.
#[test]
fn a_escala_de_gravidade_muda_o_corpo_a_meio_da_queda() {
    let mut sim = SimWorld::new();
    let bola = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 50.0)),
        ))
        .id();
    let y = |sim: &SimWorld| sim.world().get::<Transform>(bola).unwrap().translation.y;
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    sim.world_mut().entity_mut(bola).insert(GravityScale(0.0));
    let mut passos = Vec::new();
    let mut antes = y(&sim);
    for tick in 31..=60u64 {
        bridge.dispatch(&mut sim, true, tick);
        passos.push(antes - y(&sim));
        antes = y(&sim);
    }
    let (primeiro, ultimo) = (passos[1], passos[passos.len() - 1]);
    assert!(
        primeiro > 0.05,
        "o corpo foi recriado parado — a velocidade da queda perdeu-se ({primeiro} m/passo)"
    );
    assert!(
        (ultimo - primeiro).abs() < 0.02 * primeiro,
        "a escala 0 a meio da queda não chegou ao corpo vivo: o passo foi de {primeiro} a {ultimo} m"
    );
}

/// A curva vai a 0 e VOLTA ao valor de repouso a meio da queda: o corpo volta a cair. O memo da
/// ponte (`BodyRef::rest`) anda com cada reafinação — parado no valor velho, a volta ao repouso
/// pareceria «nada mudou» e o corpo ficava preso no 0 (F30 §C, a mutação C6).
#[test]
fn a_gravidade_que_volta_ao_repouso_volta_ao_corpo() {
    let mut sim = SimWorld::new();
    let bola = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 50.0)),
        ))
        .id();
    let y = |sim: &SimWorld| sim.world().get::<Transform>(bola).unwrap().translation.y;
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=20u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    sim.world_mut().entity_mut(bola).insert(GravityScale(0.0));
    for tick in 21..=30u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    sim.world_mut().entity_mut(bola).remove::<GravityScale>();
    let mut passos = Vec::new();
    let mut antes = y(&sim);
    for tick in 31..=50u64 {
        bridge.dispatch(&mut sim, true, tick);
        passos.push(antes - y(&sim));
        antes = y(&sim);
    }
    assert!(
        passos[passos.len() - 1] > passos[1] * 1.2,
        "a gravidade voltou ao repouso e o corpo continua sem peso: {passos:?}"
    );
}
