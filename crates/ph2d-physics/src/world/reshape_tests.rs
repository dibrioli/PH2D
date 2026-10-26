//! Os gates de `reshape_body` (plano 09).
use super::*;
use crate::world::shape::ShapeDesc;
use rapier2d::dynamics::RigidBodyType;

const STAND: ShapeDesc = ShapeDesc::Capsule {
    half_height: 0.3,
    radius: 0.2,
};
const LOW: ShapeDesc = ShapeDesc::Capsule {
    half_height: 0.1,
    radius: 0.2,
};

/// ⚠️ `BodyDesc` não tem `Default` de propósito — o molde do `character_tests`.
fn desc() -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: STAND,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    }
}

/// Encolher mantém a massa de pé; repor a forma autorada devolve a massa do spawn bit a bit; a
/// mesma translação é um no-op.
#[test]
fn reshaping_keeps_the_mass_and_standing_again_is_the_spawn() {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    let rest = desc();
    let h = w.spawn_body(rest);
    let m0 = w.body_mass(h);
    assert!(w.reshape_body(h, &rest, LOW, [0.0, -0.2]));
    assert!(!w.reshape_body(h, &rest, LOW, [0.0, -0.2]), "idempotente");
    assert_eq!(
        w.body_mass(h).to_bits(),
        m0.to_bits(),
        "agachado pesa o mesmo"
    );
    assert!(w.reshape_body(h, &rest, STAND, [0.0, 0.0]));
    assert_eq!(w.body_mass(h).to_bits(), m0.to_bits(), "de pé = o spawn");
}

/// A varredura do tecto pergunta pela forma ACTUAL: um tecto a 0,55 acima do centro trava a
/// cápsula de pé (topo 0,5) e deixa passar a encolhida (topo 0,1, base no mesmo sítio).
#[test]
fn the_sweep_sees_the_new_shape() {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    w.add_static_cuboid(0.0, 0.55 + 0.5, 2.0, 0.5);
    let rest = desc();
    let h = w.spawn_body(rest);
    w.step();
    assert!(w.sweep_body(h, [0.0, 1.0], 0.1, 0).is_some(), "de pé toca");
    w.reshape_body(h, &rest, LOW, [0.0, -0.2]);
    w.step();
    assert!(
        w.sweep_body(h, [0.0, 1.0], 0.3, 0).is_none(),
        "encolhido cabe"
    );
    assert!(
        w.sweep_body(h, [0.0, 1.0], 0.5, 0).is_some(),
        "e sobe até ele"
    );
}

/// Encolher pelo topo deixa a BASE no sítio (com a translação a descer `s/2`) e esgota-se no troço
/// recto: a cápsula vira bola, a bola não encolhe.
#[test]
fn shrinking_from_the_top_keeps_the_base() {
    let half = |s: ShapeDesc| match s {
        ShapeDesc::Ball { radius } => radius,
        ShapeDesc::Cuboid { half_y, .. } => half_y,
        ShapeDesc::Capsule {
            half_height,
            radius,
        } => half_height + radius,
        _ => unreachable!(),
    };
    let base = |s: ShapeDesc, oy: f32| oy - half(s);
    let top = |s: ShapeDesc, oy: f32| oy + half(s);
    for (shape, s) in [
        (STAND, 0.25),
        (STAND, 0.6),
        (
            ShapeDesc::Cuboid {
                half_x: 0.3,
                half_y: 0.5,
            },
            0.4,
        ),
    ] {
        let low = shape.shrunk_from_top(s);
        let oy = -0.5 * s.min(shape.top_shrink_room());
        assert!(
            (base(low, oy) - base(shape, 0.0)).abs() < 1e-6,
            "{shape:?} {s}"
        );
        assert!((top(shape, 0.0) - top(low, oy) - s.min(shape.top_shrink_room())).abs() < 1e-6);
    }
    assert_eq!(STAND.shrunk_from_top(0.6), ShapeDesc::Ball { radius: 0.2 });
    let ball = ShapeDesc::Ball { radius: 0.4 };
    assert_eq!(
        (ball.top_shrink_room(), ball.shrunk_from_top(0.3)),
        (0.0, ball)
    );
}
