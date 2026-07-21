//! **The smoke scenes where a MOVER hits a ROW** — isolating what makes a body plow
//! through a collision (`PH2D_PHYSICS_SMOKE` 19 and 20).
//!
//! Split out of [`crate::physics_smoke_props`] for the shell's 600-LOC cap, and the
//! seam is real: both scenes launch a mover into a row of targets and vary the ONE
//! thing that decides whether it plows through — its WEIGHT (mass, scene 19) or its
//! PRIORITY (dominance, scene 20). The counterpoint is the point: a heavy body plows
//! through by mass; a LIGHT body plows through by dominance, which mass alone cannot.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, Dominance, InitialVelocity, MassOverride, RigidBody,
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
}
