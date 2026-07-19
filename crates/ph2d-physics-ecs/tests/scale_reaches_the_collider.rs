//! **The world scale reaches the collider** (ADR-0131 W6).
//!
//! A sprite scaled 2× draws at twice the size; until this landed, its collider
//! did not — `body_desc` read `col.shape` verbatim, so the physical body was
//! authored-size while the drawn one grew (reported by Enio). These gates pin
//! the resolution (`scaled_shape`) and its behavioural consequence (a scaled
//! ball rests on a scaled collider), including the case that gives the fixture
//! teeth: a body under a **scaled parent** inherits the parent's world scale.
//!
//! The Ball fork is the design decision the wave carries: under *uniform* scale
//! a ball stays an exact circle; under *non-uniform* scale it becomes an
//! ellipse (a convex polygon downstream), because the collider must match the
//! ellipse the sprite draws.

use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody, ShapeDesc, scaled_shape,
};

// ---------------------------------------------------------------------------
// The resolution — pure.
// ---------------------------------------------------------------------------

/// A box takes per-axis scale natively: each half-extent multiplies by its own
/// axis. Mutation-tested — dropping either `* s` leaves that half-extent
/// authored-size.
#[test]
fn a_cuboid_inherits_per_axis_scale() {
    let out = scaled_shape(
        ColliderShape::Cuboid {
            half_x: 1.0,
            half_y: 2.0,
        },
        Vec2::new(3.0, 0.5),
    );
    assert_eq!(
        out,
        ShapeDesc::Cuboid {
            half_x: 3.0, // 1 × 3
            half_y: 1.0, // 2 × 0.5
        }
    );
}

/// Under *uniform* scale a ball stays an exact circle — a polygon would be
/// rounder-nowhere and slower. Negative scale (a flip) is still uniform: only
/// magnitude sets size, so `|−2| == |−2|` keeps it a circle.
#[test]
fn a_uniform_scale_keeps_a_ball_a_circle() {
    assert_eq!(
        scaled_shape(ColliderShape::Ball { radius: 0.5 }, Vec2::new(2.0, 2.0)),
        ShapeDesc::Ball { radius: 1.0 }
    );
    assert_eq!(
        scaled_shape(ColliderShape::Ball { radius: 0.5 }, Vec2::new(-2.0, -2.0)),
        ShapeDesc::Ball { radius: 1.0 },
        "a uniform flip is still a circle — magnitude alone sets the radius"
    );
}

/// Under *non-uniform* scale a ball is an ellipse: `rx = r·|sx|`, `ry = r·|sy|`
/// — the shape the sprite actually draws. Mutation-tested: keeping it a `Ball`
/// (any single radius) fails to distinguish the two axes.
#[test]
fn a_nonuniform_scale_makes_the_ball_an_ellipse() {
    assert_eq!(
        scaled_shape(ColliderShape::Ball { radius: 0.5 }, Vec2::new(2.0, 3.0)),
        ShapeDesc::Ellipse { rx: 1.0, ry: 1.5 }
    );
    // Signs are magnitudes here too — a non-uniform flip is the same ellipse.
    assert_eq!(
        scaled_shape(ColliderShape::Ball { radius: 0.5 }, Vec2::new(2.0, -3.0)),
        ShapeDesc::Ellipse { rx: 1.0, ry: 1.5 }
    );
}

/// **The regression guard.** Unit scale is `(1, 1)`, so every unscaled body —
/// which is every body authored before this wave — resolves to *exactly* the
/// shape it always had. A ball stays a `Ball`, not an ellipse of equal axes,
/// so the cheap-circle path and today's byte-for-byte behaviour are preserved.
#[test]
fn an_unscaled_body_resolves_byte_identically() {
    assert_eq!(
        scaled_shape(ColliderShape::Ball { radius: 0.3 }, Vec2::new(1.0, 1.0)),
        ShapeDesc::Ball { radius: 0.3 }
    );
    assert_eq!(
        scaled_shape(
            ColliderShape::Cuboid {
                half_x: 4.0,
                half_y: 0.2,
            },
            Vec2::new(1.0, 1.0),
        ),
        ShapeDesc::Cuboid {
            half_x: 4.0,
            half_y: 0.2,
        }
    );
}

// ---------------------------------------------------------------------------
// The consequence — behavioural, through the real sim.
// ---------------------------------------------------------------------------

/// Drop `ball` onto a floor and run to rest; return its **world** resting y.
/// The floor's top surface is at y = 0.1, so a body of vertical half-extent `h`
/// rests with its centre at ≈ 0.1 + h.
fn rest_y(mut sim: SimWorld, ball: ph2d_ecs::Entity) -> f32 {
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=300u64 {
        bridge.dispatch(&mut sim, true, tick);
    }
    // WORLD pose (rapier space), so this reads the same whether or not the body
    // is parented.
    bridge.body_pose(ball).expect("ball has a body").1
}

fn floor(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
}

/// **A scaled ball rests on its scaled collider** — proof the descriptor
/// actually became a bigger collider in the sim, not just a bigger number.
///
/// A radius-0.5 ball scaled 2× is a radius-1.0 collider, so it rests with its
/// centre at ≈ 1.1 (floor 0.1 + radius 1.0) — half a metre higher than the
/// unscaled 0.6. Mutation-tested: dropping the scale from `body_desc` rests
/// both at 0.6 and the gap vanishes.
#[test]
fn a_scaled_dynamic_ball_rests_on_its_scaled_collider() {
    let ball = |sim: &mut SimWorld, scale: f32| {
        sim.world_mut()
            .spawn((
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.5 },
                    density: 1.0,
                    ..Collider::default()
                },
                Transform {
                    translation: Vec2::new(0.0, 6.0),
                    rotation: 0.0,
                    scale: Vec2::new(scale, scale),
                    skew_x: 0.0,
                    skew_y: 0.0,
                },
            ))
            .id()
    };

    let mut a = SimWorld::new();
    floor(&mut a);
    let e = ball(&mut a, 1.0);
    let unscaled = rest_y(a, e);

    let mut b = SimWorld::new();
    floor(&mut b);
    let e = ball(&mut b, 2.0);
    let scaled = rest_y(b, e);

    assert!(
        (unscaled - 0.6).abs() < 0.05,
        "unscaled ball rested at {unscaled}, expected ≈ 0.6 (floor + 0.5)"
    );
    assert!(
        (scaled - 1.1).abs() < 0.05,
        "2× ball rested at {scaled}, expected ≈ 1.1 (floor + 1.0) — the scale \
         did not reach the collider"
    );
    assert!(
        scaled - unscaled > 0.4,
        "the scaled ball is not sitting higher ({scaled} vs {unscaled}) — its \
         collider is the same size"
    );
}

/// **A parented body's collider uses its PARENT's world scale.**
///
/// The child carries unit local scale; the 2× lives on the rig. The collider
/// inherits it through the composed chain (like Unity/Godot), so the child
/// rests exactly where a root 2× ball does. This is the fixture that has teeth:
/// reading the raw local scale would leave the child authored-size, and every
/// root-body test above would still pass ([[feedback_a_condition_that_enumerates_its_readers_rots]]).
///
/// The rig has **no translation** (only scale), so the world resting height is
/// governed purely by the inherited scale, not by a parent offset.
#[test]
fn a_parented_bodys_collider_uses_the_parents_world_scale() {
    let mut sim = SimWorld::new();
    floor(&mut sim);

    let rig = sim
        .world_mut()
        .spawn((Transform {
            translation: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::new(2.0, 2.0),
            skew_x: 0.0,
            skew_y: 0.0,
        },))
        .id();
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.5 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 6.0)),
            ChildOf(rig),
        ))
        .id();

    let y = rest_y(sim, ball);
    assert!(
        (y - 1.1).abs() < 0.05,
        "the parented ball rested at world y={y}; a radius-0.5 ball under a 2× \
         parent must rest at ≈ 1.1 (floor + 1.0) — the parent's world scale did \
         not reach the collider"
    );
}
