//! The bridge folds the optional `AreaForceWorldAxes` marker into the sim, and a rewind
//! RE-ARMS it (the frame of a force zone, W-AreaFrame).
//!
//! `ph2d-physics` proves the kernel turns the wind with the zone. This is the ECS half,
//! tested through the OUTCOME an artist would check: a ROTATED wind column carries a body
//! along the direction it was turned to, and marking it "world axes" sends the same body
//! the old way instead. After a scrub back to t=0 both replay identically — which they can
//! only if the frame rode the `BodyDesc` the world rebuilds from (law 2).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaEffector, AreaForceWorldAxes, BodyKind, Collider, ColliderShape, PhysicsBridge,
    PhysicsSettings, RigidBody,
};

/// No world gravity: the zone is then the only thing acting on the body, so where it
/// ends up IS the direction of the push.
fn zero_gravity() -> PhysicsSettings {
    PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    }
}

/// A static SENSOR box carrying a force along its own +X, turned by `rotation` radians.
/// `world_axes` attaches the marker — the escape that pins the push back to the world.
fn zone(sim: &mut SimWorld, rotation: f32, world_axes: bool) {
    let e = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 3.0,
                    half_y: 3.0,
                },
                density: 1.0,
                is_sensor: true,
                ..Collider::default()
            },
            Transform {
                translation: Vec2::new(0.0, 0.0),
                rotation,
                scale: Vec2::new(1.0, 1.0),
                skew_x: 0.0,
                skew_y: 0.0,
            },
            AreaEffector { force: [3.0, 0.0] },
        ))
        .id();
    if world_axes {
        sim.world_mut().entity_mut(e).insert(AreaForceWorldAxes);
    }
}

fn drifter(sim: &mut SimWorld) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id()
}

fn play_to(bridge: &mut PhysicsBridge, sim: &mut SimWorld, tick: u64) {
    let from = bridge.last_stepped();
    for t in (from + 1)..=tick {
        bridge.dispatch(sim, true, t);
    }
}

fn pos_of(sim: &SimWorld, e: Entity) -> (f32, f32) {
    let t = sim.world().get::<Transform>(e).unwrap().translation;
    (t.x, t.y)
}

/// Turn the zone a quarter turn: the body must leave along **+Y**, not +X.
#[test]
fn the_bridge_folds_the_zones_frame_and_a_rewind_preserves_it() {
    let mut sim = SimWorld::new();
    zone(&mut sim, std::f32::consts::FRAC_PI_2, false);
    let e = drifter(&mut sim);
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    play_to(&mut bridge, &mut sim, 60);

    let (x, y) = pos_of(&sim, e);
    assert!(
        y > 0.5 && x.abs() < 1e-3,
        "a zone turned a quarter turn must carry the body along +Y — it is at ({x}, {y}); \
         the bridge is not folding the zone's frame into the sim"
    );

    // Scrub back to t=0 and replay: the same trip, to the same place.
    bridge.dispatch(&mut sim, false, 0);
    play_to(&mut bridge, &mut sim, 60);
    let (x2, y2) = pos_of(&sim, e);
    assert!(
        (y2 - y).abs() < 1e-3 && (x2 - x).abs() < 1e-3,
        "after a rewind the frame was not re-armed (({x}, {y}) -> ({x2}, {y2})) — it was \
         read once and lost on the scrub"
    );
}

/// The marker's half: the SAME rotated zone, pinned to world axes, pushes the old way.
#[test]
fn the_marker_pins_the_push_to_world_axes_and_a_rewind_preserves_it() {
    let mut sim = SimWorld::new();
    zone(&mut sim, std::f32::consts::FRAC_PI_2, true);
    let e = drifter(&mut sim);
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    play_to(&mut bridge, &mut sim, 60);

    let (x, y) = pos_of(&sim, e);
    assert!(
        x > 0.5 && y.abs() < 1e-3,
        "with the marker the push stays on world +X whatever the zone's pose — the body \
         is at ({x}, {y})"
    );

    bridge.dispatch(&mut sim, false, 0);
    play_to(&mut bridge, &mut sim, 60);
    let (x2, y2) = pos_of(&sim, e);
    assert!(
        (x2 - x).abs() < 1e-3 && (y2 - y).abs() < 1e-3,
        "after a rewind the marker was not re-armed (({x}, {y}) -> ({x2}, {y2}))"
    );
}

/// **The marker ALONE is not a zone.** It describes the frame of a force, so on a body
/// carrying no force there is nothing for it to qualify — and registering that body as a
/// zone would cost the substep walk (and could WAKE a sleeping body) for a push of zero.
/// It is deliberately outside the bridge's `any`, and this is the gate on that.
#[test]
fn the_marker_alone_does_not_make_a_zone() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 3.0,
                half_y: 3.0,
            },
            density: 1.0,
            is_sensor: true,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        AreaForceWorldAxes,
    ));
    let e = drifter(&mut sim);
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    play_to(&mut bridge, &mut sim, 60);
    let (x, y) = pos_of(&sim, e);
    assert!(
        x.abs() < 1e-6 && y.abs() < 1e-6,
        "a marker with no force is not a zone; nothing should have moved, but the body \
         is at ({x}, {y})"
    );
}

/// **Espelhar o sprite da zona espelha o vento — pela ESCALA do `Transform`** (W-AreaMirror).
///
/// A metade ECS: a lateralidade não vem de um componente novo, vem do `Transform` que o
/// artista já manipula quando vira um sprite. `scale::body_desc` a dobra ao lado da linha
/// que já dobra a escala sincada no offset (W-Offset) — *"um flip espelha o que tem lado"*
/// —, então o precedente e a regra nova moram juntos e não podem divergir.
///
/// E o rewind: a `mirror` rida o `BodyDesc` que o mundo reconstrói, então um scrub até t=0
/// tem de reproduzir a MESMA viagem. Sem isso a zona espelhada voltaria a soprar para o
/// lado original depois de arrastar a régua — em silêncio, e só na segunda corrida.
#[test]
fn a_mirrored_zone_blows_the_other_way_and_a_rewind_preserves_it() {
    let run = |sx: f32, rewind: bool| {
        let mut sim = SimWorld::new();
        let e = sim
            .world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 3.0,
                        half_y: 3.0,
                    },
                    density: 1.0,
                    is_sensor: true,
                    ..Collider::default()
                },
                AreaEffector { force: [3.0, 0.0] },
            ))
            .id();
        // O gesto do artista: virar o sprite. Só o SINAL de x muda.
        sim.world_mut().entity_mut(e).insert(Transform {
            translation: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::new(sx, 1.0),
            skew_x: 0.0,
            skew_y: 0.0,
        });
        let b = drifter(&mut sim);
        let mut bridge = PhysicsBridge::new();
        bridge.set_settings(zero_gravity());
        play_to(&mut bridge, &mut sim, 40);
        if rewind {
            bridge.dispatch(&mut sim, false, 0);
            play_to(&mut bridge, &mut sim, 40);
        }
        pos_of(&sim, b).0
    };
    let plain = run(1.0, false);
    assert!(
        plain > 0.5,
        "controle: sem espelho a bola vai para +X ({plain})"
    );
    let flipped = run(-1.0, false);
    assert!(
        flipped < -0.5,
        "virar o sprite da zona tem de virar a correia: a bola está em {flipped}, e a \
         ponte não está dobrando o sinal da escala no frame da zona"
    );
    let replayed = run(-1.0, true);
    assert!(
        (replayed - flipped).abs() < 1e-3,
        "depois de um rewind o espelho não foi re-armado ({flipped} -> {replayed})"
    );
}
