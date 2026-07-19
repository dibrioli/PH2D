//! **The smoke scenes about ONE body's own property** — what a single collider
//! *is*, rather than how bodies relate (`PH2D_PHYSICS_SMOKE` 12 and 13).
//!
//! Sibling of [`crate::physics_smoke`] (the prologue and the basics) and
//! [`crate::physics_smoke_rigs`] (everything that needs a SECOND thing to mean
//! anything — a joint, the timeline, an ancestor). The seam here is real and not
//! merely an overflow: each of these scenes varies **one property of one body**
//! and shows what changes — the gravity multiplier, the collider shape.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, GravityScale, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::physics_smoke::spawn_floor;

impl crate::App {
    /// **Scene 13 (capsule).** A capsule and a box, identical in every way that
    /// matters, run at the same stair. The box catches on a step; the capsule
    /// rides over it — which is the whole reason 2D characters are capsules.
    ///
    /// Gravity is tilted to supply the walk: a body has no authored initial
    /// velocity in this engine, so the sideways component IS the push.
    pub(crate) fn physics_smoke_capsule(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // A stair of four steps, each 0.15 m up — low enough that a capsule
        // rides them and high enough that a square corner catches.
        for i in 0..4u32 {
            let top = 0.15 * (i + 1) as f32;
            let x0 = i as f32 * 1.2;
            world.spawn((
                Transform::from_translation(Vec2::new(x0 + 0.6, top * 0.5)),
                Sprite::atlas(WHITE_TILE_KEY, [1.2, top], [0.40, 0.42, 0.48, 1.0]),
                Name::new(format!("Step{i}")),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.6,
                        half_y: top * 0.5,
                    },
                    ..Collider::default()
                },
            ));
        }
        // The approach: flat ground before the stair.
        world.spawn((
            Transform::from_translation(Vec2::new(-2.0, -0.05)),
            Sprite::atlas(WHITE_TILE_KEY, [4.0, 0.1], [0.40, 0.42, 0.48, 1.0]),
            Name::new("Approach"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 2.0,
                    half_y: 0.05,
                },
                ..Collider::default()
            },
        ));

        // Two runners with the SAME total half-extent (0.25), same start, same
        // friction: the only difference is the shape, which is the point.
        world.spawn((
            Transform::from_translation(Vec2::new(-3.0, 0.25)),
            Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], [0.95, 0.45, 0.25, 1.0]),
            Name::new("BoxRunner"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.25,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(-3.0, 1.0)),
            Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.5], [0.35, 0.85, 0.55, 1.0]),
            Name::new("CapsuleRunner"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.10,
                    radius: 0.15,
                },
                ..Collider::default()
            },
        ));

        // The walk. Tilted gravity, so both runners are pushed at the stair —
        // read-modify-write through the one door, so every other world setting
        // keeps whatever it had.
        let mut settings = gfx.physics.settings();
        settings.gravity_x = 4.0;
        gfx.physics.set_settings(settings);

        eprintln!(
            "[physics-smoke 13] A stair, and two runners pushed at it by tilted gravity. \
             The GREEN capsule rides up the steps; the ORANGE box catches on a square corner \
             and stops. Press B: the capsule outline is a stadium (two round caps), the box a \
             rectangle. That difference is the whole reason a 2D character is a capsule."
        );
    }

    /// **Scene 12 (W8).** Four bodies dropped from the same height, differing
    /// only in per-body **Gravity Scale**. The whole point is that the same
    /// world gravity does four different things, which no single global gravity
    /// can express — so a dead multiplier would make all four fall together.
    /// (Here rather than in the prologue file only because that file is full.)
    pub(crate) fn physics_smoke_gravity(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // Same body every time (a cuboid matching the sprite quad, so the
        // outline never disagrees with the art); the tuple's last slot is the
        // optional `GravityScale`, absent on the control.
        let body = |x: f32, hue: [f32; 4], label: &str| {
            (
                Transform::from_translation(Vec2::new(x, 2.5)),
                Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], hue),
                Name::new(label.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.3,
                        half_y: 0.3,
                    },
                    ..Collider::default()
                },
            )
        };
        // Control: no GravityScale component → full gravity, falls and settles.
        world.spawn(body(-3.0, [1.0, 0.55, 0.2, 1.0], "Normal (1.0)"));
        // Weightless: hangs exactly where it was dropped.
        world.spawn((
            body(-1.0, [0.4, 0.6, 0.9, 1.0], "Weightless (0.0)"),
            GravityScale(0.0),
        ));
        // Heavy: falls fastest, settles first.
        world.spawn((
            body(1.0, [0.75, 0.2, 0.2, 1.0], "Heavy (2.0)"),
            GravityScale(2.0),
        ));
        // Balloon: negative scale → floats UP, off the top of the view.
        world.spawn((
            body(3.0, [0.45, 0.85, 0.85, 1.0], "Balloon (-0.3)"),
            GravityScale(-0.3),
        ));

        eprintln!(
            "[physics-smoke 12] Four bodies, one gravity, four fates: the orange one falls \
             normally, the blue one is WEIGHTLESS (hangs in the air), the red one is HEAVY \
             (falls fastest), the cyan one is a BALLOON (floats UP off the top). Select each \
             and see the Gravity Scale row in §11 (Dynamic only). A dead multiplier makes all \
             four fall together."
        );
    }
}
