//! **The smoke scene for per-body damping (drag)** — `PH2D_PHYSICS_SMOKE=22`.
//!
//! Its own file (damping is not a collision), and it shows both axes of drag in
//! isolation: a floaty ball with heavy LINEAR damping drifts down beside a plain one
//! that drops fast, and a box with heavy ANGULAR damping spins to a stop beside a
//! plain one that spins forever. The spinning pair hovers (`GravityScale(0)`) so the
//! spin is the only motion — angular damping with nothing else to read.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, DampMode, DampingOverride, GravityScale, InitialVelocity,
    RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::physics_smoke::spawn_floor;

impl crate::App {
    /// **Scene 22 (W-Damping).** Two demos of per-body drag.
    ///
    /// LEFT — falling: a GREEN ball with heavy **Linear Damping** (4.0) drifts down
    /// like a feather (its fall levels off at terminal velocity g/drag ≈ 2.5 m/s),
    /// while an identical ORANGE ball with no damping drops fast and lands first.
    ///
    /// RIGHT — spinning: a GREEN box with heavy **Angular Damping** (4.0) is spun at
    /// t=0 and quickly winds to a stop, while an identical ORANGE box spins forever.
    /// Both hover (`GravityScale 0`), so the spin is the only motion. Press **B** to
    /// see the collider outlines rotate.
    ///
    /// Runs PAUSED at t=0. Play; select the GREEN ball and see **Linear Damping = 4**
    /// (or the GREEN box and **Angular Damping = 4**, **Damp Mode = Combine**) in §11
    /// (Dynamic-only). Zero the damping to watch it behave like the orange one.
    pub(crate) fn physics_smoke_damping(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // Falling pair: same drop height, only the linear damping differs.
        for (x, linear, hue, tag) in [
            (-3.0f32, 4.0f32, [0.5, 0.85, 0.55, 1.0], "Floaty"),
            (-1.5f32, 0.0f32, [0.95, 0.55, 0.25, 1.0], "Plain"),
        ] {
            let base = (
                Transform::from_translation(Vec2::new(x, 4.0)),
                Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], hue),
                Name::new(format!("{tag} Faller")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.25 },
                    friction: 0.2,
                    ..Collider::default()
                },
            );
            if linear != 0.0 {
                world.spawn((
                    base,
                    DampingOverride {
                        linear,
                        angular: 0.0,
                        mode: DampMode::Combine,
                    },
                ));
            } else {
                world.spawn(base);
            }
        }

        // Spinning pair: hovering (GravityScale 0) so the spin is the only motion;
        // only the angular damping differs.
        for (x, angular, hue, tag) in [
            (2.0f32, 4.0f32, [0.5, 0.85, 0.55, 1.0], "Damped"),
            (3.5f32, 0.0f32, [0.95, 0.55, 0.25, 1.0], "Free"),
        ] {
            let base = (
                Transform::from_translation(Vec2::new(x, 2.5)),
                Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], hue),
                Name::new(format!("{tag} Spinner")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.3,
                        half_y: 0.3,
                    },
                    friction: 0.2,
                    ..Collider::default()
                },
                // Hover so the box does not fall — the spin is the whole demo.
                GravityScale(0.0),
                InitialVelocity {
                    linvel: [0.0, 0.0],
                    angvel: 5.0,
                },
            );
            if angular != 0.0 {
                world.spawn((
                    base,
                    DampingOverride {
                        linear: 0.0,
                        angular,
                        mode: DampMode::Combine,
                    },
                ));
            } else {
                world.spawn(base);
            }
        }

        eprintln!(
            "[physics-smoke 22] Paused at t=0. Press Play. TWO demos of per-body drag. \
             LEFT (falling): the GREEN ball has Linear Damping 4 and drifts down like a feather \
             (terminal velocity ~2.5 m/s), while the identical ORANGE ball drops fast and lands \
             first. RIGHT (spinning): the GREEN box has Angular Damping 4 and winds to a stop, \
             while the identical ORANGE box spins forever; both hover (Gravity Scale 0) so the \
             spin is the only motion — press B to see the collider outlines rotate. Select the \
             GREEN ball shows Linear Damping = 4 in §11 (Dynamic-only); the GREEN box shows Angular \
             Damping = 4, Damp Mode = Combine. Zero the damping to watch it match the orange one."
        );
    }
}
