//! **A zona numa forma FILHA** (GEF02 B6, ADR-0199): o vento sopra no frame da FORMA, e uma forma
//! EXTRA (peça) com componentes de área é uma zona própria — o efector é por collider.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Entity, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaEffector, BodyKind, Collider, ColliderShape, PhysicsBridge, PhysicsSettings, RigidBody,
};

fn sensor(half: f32) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: half,
            half_y: half,
        },
        density: 1.0,
        is_sensor: true,
        ..Collider::default()
    }
}

fn em(x: f32, y: f32, rotation: f32) -> Transform {
    let mut t = Transform::from_translation(Vec2::new(x, y));
    t.rotation = rotation;
    t
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
            em(0.0, 0.0, 0.0),
        ))
        .id()
}

/// 60 tiques sem gravidade, depois rebobina a 0 e repete: devolve as duas posições finais.
fn play_twice(sim: &mut SimWorld, e: Entity) -> ((f32, f32), (f32, f32)) {
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    });
    let run = |bridge: &mut PhysicsBridge, sim: &mut SimWorld| {
        for t in 1..=60 {
            bridge.dispatch(sim, true, t);
        }
        let p = sim.world().get::<Transform>(e).unwrap().translation;
        (p.x, p.y)
    };
    let first = run(&mut bridge, sim);
    bridge.dispatch(sim, false, 0);
    let second = run(&mut bridge, sim);
    (first, second)
}

#[test]
fn a_zone_on_a_rotated_child_shape_pushes_in_the_shapes_frame_and_a_rewind_preserves_it() {
    let mut sim = SimWorld::new();
    let zona = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            em(0.0, 0.0, 0.0),
        ))
        .id();
    sim.world_mut().spawn((
        sensor(3.0),
        AreaEffector { force: [3.0, 0.0] },
        em(0.0, 0.0, std::f32::consts::FRAC_PI_2),
        ChildOf(zona),
    ));
    let e = drifter(&mut sim);
    let ((x, y), (x2, y2)) = play_twice(&mut sim, e);
    assert!(
        y > 0.5 && x.abs() < 1e-3,
        "a forma filha rodada 1/4 de volta sopra para +Y; o corpo está em ({x}, {y})"
    );
    assert!(
        (x2 - x).abs() < 1e-6 && (y2 - y).abs() < 1e-6,
        "o rewind repete a viagem: ({x}, {y}) -> ({x2}, {y2})"
    );
}

#[test]
fn an_extra_shape_zone_pushes_and_a_rewind_preserves_it() {
    let mut sim = SimWorld::new();
    let zona = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            em(0.0, 0.0, 0.0),
        ))
        .id();
    // A forma PRINCIPAL é um sensor inerte longe dali; a SEGUNDA (peça) é a zona.
    sim.world_mut()
        .spawn((sensor(0.5), em(50.0, 50.0, 0.0), ChildOf(zona)));
    sim.world_mut().spawn((
        sensor(3.0),
        AreaEffector { force: [3.0, 0.0] },
        em(0.0, 0.0, std::f32::consts::FRAC_PI_2),
        ChildOf(zona),
    ));
    let e = drifter(&mut sim);
    let ((x, y), (x2, y2)) = play_twice(&mut sim, e);
    assert!(
        y > 0.5 && x.abs() < 1e-3,
        "a peça-zona rodada sopra para +Y; o corpo está em ({x}, {y})"
    );
    assert!(
        (x2 - x).abs() < 1e-6 && (y2 - y).abs() < 1e-6,
        "o rewind re-arma a peça-zona: ({x}, {y}) -> ({x2}, {y2})"
    );
}
