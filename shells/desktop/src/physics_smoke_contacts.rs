//! **Cenas de OBSERVAÇÃO e de MEIO** (`PH2D_PHYSICS_SMOKE` 25, 26).
//!
//! Its own file rather than another arm in [`crate::physics_smoke_collision`], whose
//! stated job is scenes that *author a collision outcome* — each varying the one
//! property that decides what a collision DOES. This one varies nothing: it puts
//! ordinary bodies in ordinary poses so the contact overlay has something to describe.
//! Different question, different home (and the collision file was at 465 of its 600).

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_physics_ecs::{AreaDrag, AreaEffector, BodyKind, Collider, ColliderShape, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

use crate::physics_smoke::spawn_floor;

impl crate::App {
    /// **Scene 25 (W-Contacts).** Where bodies touch, and how hard they press.
    ///
    /// LEFT: a **stack of four boxes**. Every joint gets a white cross, and the crosses
    /// grow downward — the bottom one is holding four boxes, the top one is holding
    /// one, and the marks come out in that ratio (4 : 3 : 2 : 1, the number the gate
    /// pins). It is a load meter drawn on the scene.
    ///
    /// RIGHT: a **ball resting in a V** of two tilted ramps. Two marks, one on each
    /// ramp face, at the points where the circle actually meets the slopes — not at
    /// anybody's centre, which is the answer a mark that forgot to leave the body's
    /// local frame would give.
    ///
    /// Plays immediately: the interesting state is the settled one, and a paused stack
    /// has nothing to show. Press **B** to toggle the whole physics overlay (the
    /// contact marks travel with the collider outlines — they annotate the same thing).
    pub(crate) fn physics_smoke_contacts(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // ── The stack ───────────────────────────────────────────────────────
        // Spaced 0.52 apart for a 0.02 gap that closes as they settle: dropped
        // interpenetrating, the first frames would report loads that are an artefact
        // of the spawn rather than of the weight.
        for i in 0..4 {
            let hue = [0.45 + 0.12 * i as f32, 0.62, 0.95 - 0.1 * i as f32, 1.0];
            world.spawn((
                Transform::from_translation(Vec2::new(-2.5, -0.55 + i as f32 * 0.52)),
                Sprite::atlas(WHITE_TILE_KEY, [0.5, 0.5], hue),
                Name::new(format!("Stack {i}")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.25,
                        half_y: 0.25,
                    },
                    friction: 0.6,
                    ..Collider::default()
                },
            ));
        }

        // ── The V ───────────────────────────────────────────────────────────
        // Two static ramps leaning towards each other, and a ball dropped into the
        // notch: it comes to rest touching BOTH, which is what puts a mark on each
        // slanted face.
        for (dx, rot) in [(-0.75f32, -0.45f32), (0.75, 0.45)] {
            world.spawn((
                Transform {
                    translation: Vec2::new(2.5 + dx, -0.2),
                    rotation: rot,
                    scale: Vec2::new(1.0, 1.0),
                    skew_x: 0.0,
                    skew_y: 0.0,
                },
                Sprite::atlas(WHITE_TILE_KEY, [1.6, 0.2], [0.40, 0.85, 0.55, 1.0]),
                Name::new("Ramp"),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.8,
                        half_y: 0.1,
                    },
                    friction: 0.6,
                    ..Collider::default()
                },
            ));
        }
        world.spawn((
            Transform::from_translation(Vec2::new(2.5, 1.6)),
            Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.7], [0.95, 0.80, 0.30, 1.0]),
            Name::new("Ball in the V"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.35 },
                friction: 0.6,
                ..Collider::default()
            },
        ));

        eprintln!(
            "[physics-smoke 25] Playing. WHITE CROSSES mark where bodies TOUCH, and their size is \
             the LOAD that contact is carrying. LEFT: a stack of four boxes -- the crosses grow \
             downward, because the bottom joint holds four boxes and the top one holds one (4:3:2:1, \
             the ratio the gate pins). RIGHT: a ball resting in a V of two ramps -- one mark on each \
             slanted face, at the points where the circle actually meets them, not at anybody's \
             centre. Press B to toggle the physics overlay (the marks travel with the collider \
             outlines -- they annotate the same thing). NOTE the crosses are a LOAD meter, not an \
             impact flash: the solver has already absorbed an impact by the time a frame ends, so a \
             landing reads the same as sitting still -- measured, and documented in ContactReport."
        );
    }

    /// **Scene 26 (W-AreaDrag).** The other half of a force zone: the **medium**.
    ///
    /// Three pools, all the same size, all fed the same three boxes (small, medium,
    /// large — same material, so three masses):
    ///
    /// - **VACUUM** (left, no zone at all): the control. They fall at the same rate,
    ///   because gravity does not care about mass.
    /// - **WIND** (middle, `Force Y = +3.5 N`, no drag): the force IS resisted by mass,
    ///   so the small one is thrown up and the large one still sinks — but nothing is
    ///   damped, so everything that moves keeps moving. A vacuum that blows.
    /// - **WATER** (right, the same force PLUS `Drag = 4`): now they enter and **slow
    ///   down**. Measured at t = 5 s: the small box floats at the surface (y = 2.23),
    ///   the middle one is still *sinking through* the pool (y = 0.87, and creeping),
    ///   and the heavy one has reached the floor. In the wind lane the same middle box
    ///   was already resting at the bottom by t = 1 s. Slow descent is what reads as a
    ///   liquid; the force alone reads as a fan.
    ///
    /// Two knobs, one area, and the pair is the difference between wind and water.
    /// Plays immediately. Press **B** for the collider outlines (the orange arrow on
    /// each zone is its push; a drag has no direction to draw, which is why the WATER
    /// pool looks like the WIND one until something falls in).
    pub(crate) fn physics_smoke_area_drag(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        for (cx, force, drag, tint, tag) in [
            (-2.8f32, 0.0f32, 0.0f32, [0.5, 0.5, 0.55, 0.10], "Vacuum"),
            (0.0, 3.5, 0.0, [0.55, 0.80, 0.95, 0.16], "Wind"),
            (2.8, 3.5, 4.0, [0.30, 0.55, 0.95, 0.28], "Water"),
        ] {
            // The vacuum lane gets no zone at all — a zone with nothing set would be
            // refused by `zone_effect` anyway, and a painted rectangle that does
            // nothing is exactly the "dimmed control" this repo keeps deleting.
            if force != 0.0 || drag != 0.0 {
                let mut zone = (
                    Transform::from_translation(Vec2::new(cx, 0.4)),
                    Sprite::atlas(WHITE_TILE_KEY, [2.2, 3.2], tint),
                    Name::new(format!("{tag} Zone")),
                    RigidBody {
                        kind: BodyKind::Static,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: 1.1,
                            half_y: 1.6,
                        },
                        is_sensor: true,
                        ..Collider::default()
                    },
                );
                zone.0.translation.y = 0.4;
                let e = world.spawn(zone).id();
                if force != 0.0 {
                    world.entity_mut(e).insert(AreaEffector {
                        force: [0.0, force],
                    });
                }
                if drag != 0.0 {
                    world.entity_mut(e).insert(AreaDrag(drag));
                }
            }

            // The same three masses in every lane, so the lanes are comparable.
            for (i, half) in [0.20f32, 0.30, 0.45].iter().enumerate() {
                let hue = [0.95, 0.90 - 0.25 * i as f32, 0.35, 1.0];
                world.spawn((
                    Transform::from_translation(Vec2::new(
                        cx - 0.7 + i as f32 * 0.7,
                        3.4 + i as f32 * 0.1,
                    )),
                    Sprite::atlas(WHITE_TILE_KEY, [half * 2.0, half * 2.0], hue),
                    Name::new(format!("{tag} {i}")),
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: *half,
                            half_y: *half,
                        },
                        friction: 0.4,
                        ..Collider::default()
                    },
                ));
            }
        }

        eprintln!(
            "[physics-smoke 26] Playing. Three lanes, the SAME three boxes (three sizes = three \
             masses) dropped into three media. LEFT is a VACUUM (no zone): they fall together, \
             because gravity does not care about mass. MIDDLE is WIND (Force Y = +3.5 N, no drag): \
             the force IS resisted by mass, so the small box is thrown up and the big one still \
             sinks -- but nothing is damped, so whatever moves keeps moving. RIGHT is WATER (the \
             same force PLUS Drag = 4): they enter and SLOW DOWN -- the small box floats at the \
             surface, the middle one is still sinking through the pool five seconds in (it was on \
             the bottom within one second in the wind lane), the heavy one reaches the floor. Two \
             knobs on one area, and the pair is the whole difference between a fan and a liquid. Select any zone: Section 11 shows Trigger = Sensor, Force X/Y and Drag -- \
             switch Trigger to Solid and all three rows vanish. Press B for the outlines (the \
             orange arrow is the push; drag has no direction to draw)."
        );
    }
}
