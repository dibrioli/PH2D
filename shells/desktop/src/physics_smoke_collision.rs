//! **The smoke scenes that author a COLLISION OUTCOME** (`PH2D_PHYSICS_SMOKE` 19,
//! 20, 21, 23). Each varies the ONE property that decides what a collision does —
//! including, in scene 23, whether it happens at all from the side you arrive on.
//!
//! Split out of [`crate::physics_smoke_props`] for the shell's 600-LOC cap. Scenes
//! 19/20 launch a mover into a row of targets and vary whether it plows THROUGH —
//! its WEIGHT (mass, scene 19) or its PRIORITY (dominance, scene 20): a heavy body
//! plows through by mass; a LIGHT body plows through by dominance, which mass alone
//! cannot. Scene 21 varies how hard a body BOUNCES off a dead floor — its material
//! COMBINE rule: a superball only bounces off a plain floor if it is told to take
//! the MAX of the two, not the default average.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, CombineRule, Dominance, InitialVelocity, MassOverride,
    MaterialCombine, OneWayPlatform, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::physics_smoke::spawn_floor;

impl crate::App {
    /// **Scene 19 (W-Mass).** Two lanes prove mass MATTERS. Each lane has a bowling
    /// ball launched into a row of five light pins. The TOP lane's ball is HEAVY (a
    /// manual `MassOverride` of 30 kg) — it plows straight through the whole row and
    /// keeps going. The BOTTOM lane's ball is the SAME SIZE but auto-mass (density
    /// ≈ light) — it stops dead at the first pin. Density and mass are the same
    /// quantity by two roads, so an artist wanting a "heavy" object sets the mass
    /// directly (Unity's manual mass) instead of reverse-engineering a density.
    ///
    /// Runs PAUSED at t=0. Play to launch both balls; select the heavy one and see
    /// **Mass: Auto | Manual → 30 kg** in §11 (Dynamic-only). Flip it to Auto to
    /// watch it stop like the light one.
    pub(crate) fn physics_smoke_mass(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // A ball at `(x, y)` with an optional launch and mass override.
        let mut ball = |x: f32, y: f32, vx: f32, mass: Option<f32>, hue: [f32; 4], label: &str| {
            let base = (
                Transform::from_translation(Vec2::new(x, y)),
                Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], hue),
                Name::new(label.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.25 },
                    // Low friction so the pins scatter cleanly rather than grip.
                    friction: 0.2,
                    ..Collider::default()
                },
                InitialVelocity {
                    linvel: [vx, 0.0],
                    angvel: 0.0,
                },
            );
            match mass {
                Some(m) => {
                    world.spawn((base, MassOverride(m)));
                }
                None => {
                    world.spawn(base);
                }
            };
        };

        // Two lanes at different heights. Each: a launcher ball on the left + five
        // light pins in a row. The launcher differs only in mass.
        for (lane_y, launcher_mass, hue, tag) in [
            (2.2f32, Some(30.0), [0.4, 0.8, 0.95, 1.0], "Heavy"),
            (0.6f32, None, [0.95, 0.55, 0.25, 1.0], "Light"),
        ] {
            ball(
                -4.0,
                lane_y,
                5.0,
                launcher_mass,
                hue,
                &format!("{tag} Ball"),
            );
            for i in 0..5u32 {
                ball(
                    i as f32 * 0.7,
                    lane_y,
                    0.0,
                    None,
                    [0.75, 0.75, 0.80, 1.0],
                    &format!("{tag} Pin {i}"),
                );
            }
        }

        eprintln!(
            "[physics-smoke 19] Paused at t=0. Press Play. Two lanes, each a ball launched into five \
             light pins. TOP (CYAN) ball is HEAVY (manual Mass 30 kg): it plows through the whole row \
             and keeps going. BOTTOM (ORANGE) ball is the SAME SIZE but auto-mass (light): it STOPS \
             at the first pin. Select the heavy ball and see Mass: Auto | Manual = 30 kg in §11 \
             (Dynamic-only); flip it to Auto to watch it stop like the light one. Mass is the same \
             quantity as density by another road — you set the weight directly instead of a density."
        );
    }

    /// **Scene 20 (W-Dominance).** The counterpoint to the mass scene: here a LIGHT
    /// ball bulldozes HEAVY ones by DECREE, not by weight. Two lanes, each a light
    /// ball launched into a row of heavy balls. The TOP lane's ball has **Dominance
    /// 5**, so it plows through the whole heavy row and keeps going — the heavy balls
    /// treat it as infinitely massive. The BOTTOM lane's ball is identical but
    /// **Dominance 0** (neutral), so it BOUNCES off the first heavy ball, as mass
    /// alone dictates. Dominance is orthogonal to mass — the unstoppable mover.
    ///
    /// Runs PAUSED at t=0. Play; select the top ball and see **Dominance = 5** in §11
    /// (Dynamic-only). Set it to 0 to watch the light ball bounce like the bottom one.
    pub(crate) fn physics_smoke_dominance(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // Two lanes: same LIGHT mover, same HEAVY row, only the dominance differs.
        for (lane_y, dominance, hue, tag) in [
            (2.2f32, 5i8, [0.5, 0.85, 0.55, 1.0], "Dominant"),
            (0.6f32, 0i8, [0.95, 0.55, 0.25, 1.0], "Neutral"),
        ] {
            // The light mover (auto mass), launched right. Dominance is attached only
            // when non-zero (the neutral mover carries none).
            let base = (
                Transform::from_translation(Vec2::new(-4.0, lane_y)),
                Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], hue),
                Name::new(format!("{tag} Mover")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.25 },
                    friction: 0.2,
                    ..Collider::default()
                },
                InitialVelocity {
                    linvel: [5.0, 0.0],
                    angvel: 0.0,
                },
            );
            if dominance != 0 {
                world.spawn((base, Dominance(dominance)));
            } else {
                world.spawn(base);
            }
            // The row of HEAVY balls (manual mass, neutral dominance) at rest.
            for i in 0..4u32 {
                world.spawn((
                    Transform::from_translation(Vec2::new(i as f32 * 0.8, lane_y)),
                    Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], [0.75, 0.75, 0.80, 1.0]),
                    Name::new(format!("{tag} Heavy {i}")),
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Ball { radius: 0.3 },
                        friction: 0.2,
                        ..Collider::default()
                    },
                    MassOverride(15.0),
                ));
            }
        }

        eprintln!(
            "[physics-smoke 20] Paused at t=0. Press Play. Two lanes, each a LIGHT ball launched into \
             a row of HEAVY balls. TOP (GREEN) mover has Dominance 5: it PLOWS through the whole heavy \
             row and keeps going — the heavy balls treat it as infinitely massive. BOTTOM (ORANGE) \
             mover is identical but Dominance 0: it BOUNCES off the first heavy ball, as its light \
             mass dictates. Select the top mover and see Dominance = 5 in §11 (Dynamic-only); set it \
             to 0 to watch the light ball bounce like the bottom one. Dominance is a collision \
             PRIORITY — orthogonal to mass, the unstoppable mover a light body cannot be otherwise."
        );
    }

    /// **Scene 21 (W-Material).** Two superballs (Bounce 1.0) dropped from the same
    /// height onto the SAME plain floor (Bounce 0.0). The only difference is how each
    /// ball's restitution COMBINES with the floor's. The TOP ball has **Bounce
    /// Combine = Max**, so rapier takes the greater of the two coefficients (1.0) and
    /// it bounces back near its drop height, again and again. The BOTTOM ball is
    /// identical but leaves the default **Average**, which halves with the dead floor
    /// (0.5) — it returns to only a quarter of the drop and dies in a couple of hops.
    ///
    /// This is the whole reason to expose the rule: without it, an artist who sets
    /// Bounce = 1.0 on a ball and drops it on an ordinary floor gets a feeble bounce
    /// and nothing on the ball alone can fix it.
    ///
    /// Runs PAUSED at t=0. Play; select the top ball and see **Bounce Combine = Max**
    /// in §11 (offered for any body, not Dynamic-only). Set it to Average to watch it
    /// die like the bottom one.
    pub(crate) fn physics_smoke_material(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // Two superballs at the same height; only the restitution combine differs.
        for (x, combine, hue, tag) in [
            (
                -1.5f32,
                Some(CombineRule::Max),
                [0.5, 0.85, 0.55, 1.0],
                "Max",
            ),
            (1.5f32, None, [0.95, 0.55, 0.25, 1.0], "Average"),
        ] {
            let base = (
                Transform::from_translation(Vec2::new(x, 3.5)),
                Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], hue),
                Name::new(format!("{tag} Superball")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.25 },
                    // A perfect superball; the floor below is the DEFAULT dead one.
                    restitution: 1.0,
                    friction: 0.2,
                    ..Collider::default()
                },
            );
            // The combine rule is attached only when non-default (Max); the Average
            // ball carries no component, exactly as leaving the toggle alone does.
            match combine {
                Some(rule) => {
                    world.spawn((
                        base,
                        MaterialCombine {
                            restitution: rule,
                            friction: CombineRule::Average,
                        },
                    ));
                }
                None => {
                    world.spawn(base);
                }
            };
        }

        eprintln!(
            "[physics-smoke 21] Paused at t=0. Press Play. Two superballs (Bounce 1.0) dropped onto \
             the SAME plain floor (Bounce 0.0). LEFT (GREEN) has Bounce Combine = Max: it takes the \
             greater of the two coefficients (1.0) and bounces back near its drop height, over and \
             over. RIGHT (ORANGE) leaves the default Average, which halves with the dead floor (0.5): \
             it returns to a quarter of the drop and dies in a couple of hops. Select the LEFT ball \
             and see Bounce Combine = Max in §11 (offered for any body, not Dynamic-only); set it to \
             Average to watch it die like the right one. This is the only way to make a superball \
             bounce off an ordinary floor."
        );
    }

    /// **Scene 23 (W-OneWay).** The jump-through platform. Two lanes, each a ball
    /// launched straight UP at a platform hanging over it. The LEFT platform is
    /// **One-Way**: the ball passes clean through it on the way up, then falls back and
    /// LANDS on top. The RIGHT platform is identical but solid: the ball bonks its
    /// underside and drops back to the floor.
    ///
    /// This is the iconic 2D platformer collider, and the reason it is a per-collider
    /// property rather than a body kind: both platforms here are Static.
    ///
    /// Runs PAUSED at t=0. Play; select the LEFT platform and see **One-Way = On** in
    /// §11 (offered for any body kind, not Dynamic-only). Turn it Off to watch its ball
    /// bonk like the right one. Press **B** for the collider outlines.
    pub(crate) fn physics_smoke_one_way(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        for (x, one_way, hue, tag) in [
            (-2.5f32, true, [0.5, 0.85, 0.55, 1.0], "One-Way"),
            (2.5f32, false, [0.95, 0.55, 0.25, 1.0], "Solid"),
        ] {
            // The platform overhead. Static — a platform is scenery, which is exactly
            // why one-way is a COLLIDER property and not a body kind.
            let plat = (
                Transform::from_translation(Vec2::new(x, 1.5)),
                Sprite::atlas(WHITE_TILE_KEY, [2.4, 0.2], hue),
                Name::new(format!("{tag} Platform")),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 1.2,
                        half_y: 0.1,
                    },
                    friction: 0.4,
                    ..Collider::default()
                },
            );
            if one_way {
                world.spawn((plat, OneWayPlatform));
            } else {
                world.spawn(plat);
            }

            // The jumper underneath, launched straight up hard enough to clear the
            // platform's top face.
            world.spawn((
                Transform::from_translation(Vec2::new(x, 0.0)),
                Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], [0.85, 0.85, 0.9, 1.0]),
                Name::new(format!("{tag} Jumper")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.25 },
                    friction: 0.4,
                    ..Collider::default()
                },
                InitialVelocity {
                    linvel: [0.0, 7.0],
                    angvel: 0.0,
                },
            ));
        }

        eprintln!(
            "[physics-smoke 23] Paused at t=0. Press Play. Two lanes, each a ball launched straight \
             UP at the platform hanging over it. LEFT (GREEN) platform is One-Way: the ball passes \
             clean THROUGH it going up, then falls back and LANDS on top of it. RIGHT (ORANGE) is \
             identical but solid: the ball bonks the underside and drops back to the floor. Select the \
             LEFT platform and see One-Way = On in §11 (offered for ANY body kind, not Dynamic-only \
             -- both platforms here are Static, which is the whole reason it is a collider property). \
             Turn it Off to watch its ball bonk like the right one. Press B for the collider outlines."
        );
    }
}
