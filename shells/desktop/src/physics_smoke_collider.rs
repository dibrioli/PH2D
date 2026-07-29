//! **O COLLIDER: que forma ele tem, e ele bloqueia?** — as cenas 9 e 10.
//!
//! Irmão do [`super::physics_smoke`] pelo cap de 600 LOC, e o corte é o assunto:
//! lá moram o mundo e a autoria (queda, pilha, settings, camadas, o Inspector),
//! aqui *o que o collider É* — a forma que a escala resolve (W6) e a diferença
//! entre bloquear e apenas detectar (W7). As duas são a mesma pergunta feita ao
//! contorno da tecla **B**, que é o oráculo das duas.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use super::physics_smoke::spawn_floor;

impl crate::App {
    /// **Scene 9 (W6).** The world scale reaches the collider. Four balls drop
    /// onto the floor, each authored as a `Ball` but scaled differently — the
    /// collider (and its **B** outline) must match the SCALED sprite, not the
    /// authored radius:
    ///
    /// * a **reference** circle at unit scale;
    /// * the same circle **uniformly** 2× — a bigger circle that rests HIGHER,
    ///   because its collider grew with it;
    /// * the same circle **non-uniformly** (wide) — an **ELLIPSE**, which lands
    ///   on its wide side and rocks as it settles the way no circle would;
    /// * a circle **parented** under a 2× rig — its collider inherits the
    ///   parent's world scale (Unity/Godot), so it behaves like the uniform 2×.
    ///
    /// The oracle is the **B** overlay: it draws the RESOLVED shape, so a dead
    /// scale→collider would trace authored-size wireframes floating inside the
    /// scaled sprites. The ellipse wireframe is the headline — a `Ball` that is
    /// drawn, and rolls, as an ellipse.
    pub(crate) fn physics_smoke_scale(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // A plain `Ball` sprite: a 0.6×0.6 quad over a radius-0.3 collider, so
        // the drawing and the (unscaled) collider agree to begin with.
        let ball = |scale: Vec2, rot: f32, x: f32, hue: [f32; 4], label: &str| {
            (
                Transform {
                    translation: Vec2::new(x, 4.0),
                    rotation: rot,
                    scale,
                    ..Transform::IDENTITY
                },
                Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], hue),
                Name::new(label.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.3 },
                    restitution: 0.2,
                    ..Collider::default()
                },
            )
        };

        // Reference · uniform 2× · non-uniform (wide ⇒ ellipse, dropped tilted
        // so it visibly rocks onto its wide side).
        world.spawn(ball(
            Vec2::new(1.0, 1.0),
            0.0,
            -3.0,
            [0.55, 0.60, 0.70, 1.0],
            "Ball1x",
        ));
        world.spawn(ball(
            Vec2::new(2.0, 2.0),
            0.0,
            -1.0,
            [0.35, 0.80, 1.00, 1.0],
            "Ball2xUniform",
        ));
        world.spawn(ball(
            Vec2::new(2.2, 1.0),
            0.5,
            1.0,
            [1.00, 0.55, 0.25, 1.0],
            "BallEllipse",
        ));

        // Parented: a 2× rig (with its own small sprite so it is grabbable) and
        // a unit-scale child ball. The child's collider inherits the rig's
        // world scale, so it drops from world y = 4 (local 2 × parent scale 2)
        // and rests like the uniform 2× ball — proof the WORLD scale, not the
        // local one, reaches the collider.
        let rig = world
            .spawn((
                Transform {
                    translation: Vec2::new(3.0, 0.0),
                    scale: Vec2::new(2.0, 2.0),
                    ..Transform::IDENTITY
                },
                Sprite::atlas(WHITE_TILE_KEY, [0.15, 0.15], [0.60, 0.95, 0.55, 1.0]),
                Name::new("Rig2x"),
            ))
            .id();
        world.spawn((
            Transform::from_translation(Vec2::new(0.0, 2.0)),
            Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], [0.72, 0.55, 1.00, 1.0]),
            Name::new("BallParented"),
            ph2d_ecs::ChildOf(rig),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                restitution: 0.2,
                ..Collider::default()
            },
        ));

        eprintln!(
            "[physics-smoke 9] Four balls drop, each a `Ball` scaled differently. Press B for the \
             collider outlines — each must trace its SCALED sprite: Ball1x a small circle, \
             Ball2xUniform a bigger circle resting higher, BallEllipse an ELLIPSE (it lands wide \
             and rocks), BallParented a bigger circle that inherited its 2x rig. A dead \
             scale->collider would outline the authored radius inside every scaled sprite."
        );
    }

    /// **Scene 10 (W7).** A **sensor** (trigger) detects but does not block.
    /// Two lanes, same bar in each: on the LEFT it is a solid static platform,
    /// on the RIGHT it is a sensor. A ball drops down each lane.
    ///
    /// - Left: the ball lands ON the platform and stops.
    /// - Right: the ball passes THROUGH the sensor and continues to the floor —
    ///   and while it overlaps, the sensor's outline (B) jumps from a dim
    ///   magenta to a bright one. That colour change is the trigger firing.
    ///
    /// The magenta is the only visible reaction in this build: the trigger's
    /// overlaps are also a queryable state (`PhysicsBridge::bodies_inside`), but
    /// what a game DOES with them — a signal, a script — is the next layer.
    pub(crate) fn physics_smoke_sensor(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // A horizontal bar (static), solid on the left lane, sensor on the
        // right. Same geometry, so the only difference the artist sees is that
        // one blocks and one is passed through and lights up.
        let bar = |x: f32, is_sensor: bool, hue: [f32; 4], label: &str| {
            (
                Transform::from_translation(Vec2::new(x, 1.0)),
                Sprite::atlas(WHITE_TILE_KEY, [1.4, 0.4], hue),
                Name::new(label.to_string()),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.7,
                        half_y: 0.2,
                    },
                    is_sensor,
                    ..Collider::default()
                },
            )
        };
        world.spawn(bar(-2.0, false, [0.40, 0.42, 0.48, 1.0], "SolidPlatform"));
        world.spawn(bar(2.0, true, [0.60, 0.30, 0.58, 1.0], "SensorZone"));

        // A ball dropped down each lane, from the same height.
        let ball = |x: f32, label: &str| {
            (
                Transform::from_translation(Vec2::new(x, 4.0)),
                Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], [1.0, 0.6, 0.25, 1.0]),
                Name::new(label.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.25 },
                    ..Collider::default()
                },
            )
        };
        world.spawn(ball(-2.0, "BallOnSolid"));
        world.spawn(ball(2.0, "BallThroughSensor"));

        eprintln!(
            "[physics-smoke 10] Two lanes. LEFT: the ball lands on a solid platform and stops. \
             RIGHT: the ball passes THROUGH the sensor and lands on the floor. Press B: the sensor \
             is magenta, and it turns BRIGHT while the ball is inside it — that is the trigger \
             firing. A dead sensor would block the ball like the solid bar, or never light up."
        );
    }
}
