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
use ph2d_physics_ecs::{
    BodyKind, Ccd, Collider, ColliderShape, GravityScale, InitialVelocity, LockPositionX,
    LockPositionY, LockRotation, RigidBody,
};
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

    /// **Scene 14 (W9).** Three balls LAUNCHED from the same spot at t=0, each
    /// with a different authored initial velocity — one arcs over a wall, one
    /// spins in place, one is at rest for contrast. Until this, a body could only
    /// start still; the earlier scenes had to tilt gravity to fake a push.
    ///
    /// Runs PAUSED at t=0 so the yellow velocity arrows are visible before Play:
    /// press Play to fire them. Arm the transport's Physics toggle first.
    pub(crate) fn physics_smoke_launch(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // A wall to arc over, so the launch is legible as a trajectory.
        world.spawn((
            Transform::from_translation(Vec2::new(2.0, 0.4)),
            Sprite::atlas(WHITE_TILE_KEY, [0.3, 2.0], [0.40, 0.42, 0.48, 1.0]),
            Name::new("Wall"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.15,
                    half_y: 1.0,
                },
                ..Collider::default()
            },
        ));

        let ball = |x: f32, hue: [f32; 4], label: &str| {
            (
                Transform::from_translation(Vec2::new(x, 0.0)),
                Sprite::atlas(WHITE_TILE_KEY, [0.4, 0.4], hue),
                Name::new(label.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.2 },
                    ..Collider::default()
                },
            )
        };
        // Projectile: up and to the right, arcs over the wall.
        world.spawn((
            ball(-1.0, [1.0, 0.55, 0.2, 1.0], "Projectile"),
            InitialVelocity {
                linvel: [5.0, 6.0],
                angvel: 0.0,
            },
        ));
        // Spinner: no travel, just tumbles (watch its rotation guide whirl).
        world.spawn((
            ball(-3.0, [0.4, 0.6, 0.9, 1.0], "Spinner"),
            InitialVelocity {
                linvel: [0.0, 0.0],
                angvel: 12.0,
            },
        ));
        // At rest: no component — falls straight, for contrast.
        world.spawn(ball(-4.0, [0.5, 0.85, 0.55, 1.0], "AtRest"));

        eprintln!(
            "[physics-smoke 14] Paused at t=0: press B to see the YELLOW initial-velocity arrows. \
             The orange Projectile is launched up-and-right (arcs over the wall), the blue Spinner \
             tumbles in place, the green AtRest just falls. Select each and see the Init Vel / Spin \
             rows in §11. Press Play to fire them. The arrows vanish once the sim steps — a live \
             body's velocity is no longer the authored launch."
        );
    }

    /// **Scene 15 (W-CCD).** Two identical fast balls fired at two identical thin
    /// walls; the only difference is one has **Continuous** collision detection and
    /// the other **Discrete** (the default). The discrete ball is moving too fast
    /// to be tested at a pose that overlaps the wall, so it tunnels clean THROUGH
    /// and flies off; the continuous one is swept and stopped against the wall.
    ///
    /// Runs PAUSED at t=0 so the two yellow launch arrows are visible (they are
    /// identical — the difference is not the launch). Press Play to fire.
    pub(crate) fn physics_smoke_ccd(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // One lane: a thin, tall static wall and a small ball five metres to its
        // left, launched right at 160 m/s. `ccd` attaches the CCD marker.
        let mut lane = |y: f32, hue: [f32; 4], label: &str, ccd: bool| {
            world.spawn((
                Transform::from_translation(Vec2::new(0.0, y)),
                Sprite::atlas(WHITE_TILE_KEY, [0.1, 2.0], [0.40, 0.42, 0.48, 1.0]),
                Name::new(format!("Wall {label}")),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.05,
                        half_y: 1.0,
                    },
                    ..Collider::default()
                },
            ));
            let ball = (
                Transform::from_translation(Vec2::new(-5.0, y)),
                Sprite::atlas(WHITE_TILE_KEY, [0.3, 0.3], hue),
                Name::new(label.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.15 },
                    ..Collider::default()
                },
                InitialVelocity {
                    linvel: [160.0, 0.0],
                    angvel: 0.0,
                },
            );
            // The marker's PRESENCE is the flag — attach it only on the
            // continuous ball, exactly as the Inspector's "Continuous" chip does.
            if ccd {
                world.spawn((ball, Ccd));
            } else {
                world.spawn(ball);
            }
        };
        // Top lane: continuous — stopped by the wall. Bottom: discrete — tunnels.
        lane(3.5, [0.5, 0.85, 0.55, 1.0], "Continuous", true);
        lane(1.0, [0.95, 0.45, 0.25, 1.0], "Discrete", false);

        eprintln!(
            "[physics-smoke 15] Paused at t=0: press B to see the two YELLOW arrows (identical — \
             both balls launch at 160 m/s). Press Play. The GREEN Continuous ball is swept and \
             STOPS against its wall; the ORANGE Discrete ball is moving too fast to be caught and \
             TUNNELS clean through its wall, flying off screen. Select each and see the \
             Collision: Discrete | Continuous row in §11 (Dynamic-only). It is fast — the story is \
             in the end state: one wall keeps its ball, the other does not."
        );
    }

    /// **Scene 16 (W-LockRot).** Two identical boxes on two mirror-image slopes;
    /// the only difference is one has **Freeze Rotation** and the other does not.
    /// Both slide down under gravity — but the free box tips over and TUMBLES down
    /// its slope, while the locked one slides down staying perfectly UPRIGHT (the
    /// reason a 2D character has its rotation frozen).
    ///
    /// Runs PAUSED at t=0. Press Play; the boxes diverge (free left, locked right)
    /// so they never meet.
    pub(crate) fn physics_smoke_lock_rotation(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // A wide floor well below the slopes, so a tumbled box has somewhere to land.
        world.spawn((
            Transform::from_translation(Vec2::new(0.0, -3.0)),
            Sprite::atlas(WHITE_TILE_KEY, [40.0, 0.4], [0.30, 0.32, 0.38, 1.0]),
            Name::new("Floor"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 20.0,
                    half_y: 0.2,
                },
                friction: 0.4,
                ..Collider::default()
            },
        ));

        // A 32° ramp (`deg` sign sets which way is downhill) and a box resting on
        // its upper end. `lock` freezes the box's rotation.
        let mut slope = |cx: f32, deg: f32, box_x: f32, hue: [f32; 4], label: &str, lock: bool| {
            let rot = deg.to_radians();
            world.spawn((
                Transform {
                    translation: Vec2::new(cx, 1.0),
                    rotation: rot,
                    scale: Vec2::new(1.0, 1.0),
                    skew_x: 0.0,
                    skew_y: 0.0,
                },
                Sprite::atlas(WHITE_TILE_KEY, [6.0, 0.3], [0.40, 0.42, 0.48, 1.0]),
                Name::new(format!("Ramp {label}")),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 3.0,
                        half_y: 0.15,
                    },
                    friction: 0.4,
                    ..Collider::default()
                },
            ));
            let body = (
                Transform::from_translation(Vec2::new(box_x, 2.4)),
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
                    friction: 0.4,
                    ..Collider::default()
                },
            );
            // The marker's PRESENCE is the flag — attach it only on the locked box.
            if lock {
                world.spawn((body, LockRotation));
            } else {
                world.spawn(body);
            }
        };
        // Left slope tilts down-left: the ORANGE Free box slides left and tumbles.
        slope(-3.0, 32.0, -1.5, [0.95, 0.45, 0.25, 1.0], "Free", false);
        // Right slope tilts down-right: the GREEN Locked box slides right, upright.
        slope(3.0, -32.0, 1.5, [0.5, 0.85, 0.55, 1.0], "Locked", true);

        eprintln!(
            "[physics-smoke 16] Paused at t=0. Press Play. Both boxes slide down their slopes, but \
             the ORANGE Free box (left) TIPS OVER and tumbles down; the GREEN Locked box (right) \
             slides down staying perfectly UPRIGHT — that is Freeze Rotation, the reason a 2D \
             character does not fall over. Select each and see the Rotation: Free | Locked row in \
             §11 (Dynamic-only). Press B to see the collider outlines."
        );
    }

    /// **Scene 17 (W-Offset).** Two tall "character" sprites dropped on a floor;
    /// the only difference is where the collider sits. The GREEN one has its box
    /// collider offset down to its FEET, so it comes to rest standing ON the floor;
    /// the ORANGE one's collider is centred on the sprite, so the sprite SINKS to
    /// its waist (its middle rests on the floor, the legs go through it). That is
    /// what a collider offset is for — the collider is almost never centred on the
    /// art.
    ///
    /// Runs PAUSED at t=0 (press B to see the outlines in their two places). Play
    /// to drop them.
    pub(crate) fn physics_smoke_offset(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // A tall character sprite with a foot-sized box collider at `offset`.
        let mut character = |x: f32, offset: [f32; 2], hue: [f32; 4], label: &str| {
            world.spawn((
                Transform::from_translation(Vec2::new(x, 3.0)),
                // 0.6 × 1.6 — clearly taller than the little foot collider.
                Sprite::atlas(WHITE_TILE_KEY, [0.6, 1.6], hue),
                Name::new(label.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    // A foot-sized box: full width, short.
                    shape: ColliderShape::Cuboid {
                        half_x: 0.3,
                        half_y: 0.2,
                    },
                    // Freeze rotation so the tall sprite does not tip over on
                    // landing — this scene is about WHERE the collider is, not spin.
                    offset,
                    ..Collider::default()
                },
                LockRotation,
            ));
        };
        // Collider at the FEET (0.6 below centre): the sprite stands on the floor.
        character(
            -1.5,
            [0.0, -0.6],
            [0.5, 0.85, 0.55, 1.0],
            "Grounded (foot offset)",
        );
        // Collider CENTRED: the sprite sinks to its waist.
        character(1.5, [0.0, 0.0], [0.95, 0.45, 0.25, 1.0], "Sunk (centred)");

        eprintln!(
            "[physics-smoke 17] Paused at t=0: press B to see the two collider outlines — the GREEN \
             character's box is at its FEET, the ORANGE one's is at its CENTRE. Press Play. The \
             GREEN one lands standing ON the floor; the ORANGE one SINKS to its waist because its \
             collider (its middle) is what rests on the floor. Select each and see the Offset X/Y \
             rows in §11. The collider is almost never centred on the art — that is why offset exists."
        );
    }

    /// **Scene 18 (W-LockPos).** Three identical balls, each launched sideways at
    /// t=0 (`InitialVelocity`); the only difference is which position axis is frozen
    /// (Freeze Position X/Y — the rest of Unity/Godot's constraint trio, beside the
    /// Freeze Rotation of scene 16). The GREEN one is free and ARCS down-right. The
    /// CYAN one has **Freeze Position X**, so the sideways launch is dropped and it
    /// falls STRAIGHT DOWN in its lane (a rail-locked actor). The ORANGE one has
    /// **Freeze Position Y**, so gravity cannot pull it down and it GLIDES sideways
    /// at a constant height forever (a floating platform).
    ///
    /// Runs PAUSED at t=0 (press B to see the outlines). Play to launch them.
    pub(crate) fn physics_smoke_freeze_position(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();
        spawn_floor(world);

        // A ball launched at `linvel`, optionally with a frozen X or Y axis.
        let mut ball = |x: f32,
                        y: f32,
                        linvel: [f32; 2],
                        lock_x: bool,
                        lock_y: bool,
                        hue: [f32; 4],
                        label: &str| {
            let base = (
                Transform::from_translation(Vec2::new(x, y)),
                Sprite::atlas(WHITE_TILE_KEY, [0.6, 0.6], hue),
                Name::new(label.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.3 },
                    ..Collider::default()
                },
                InitialVelocity {
                    linvel,
                    angvel: 0.0,
                },
            );
            // The markers' PRESENCE is the flag — attach only the frozen axes.
            match (lock_x, lock_y) {
                (false, false) => {
                    world.spawn(base);
                }
                (true, false) => {
                    world.spawn((base, LockPositionX));
                }
                (false, true) => {
                    world.spawn((base, LockPositionY));
                }
                (true, true) => {
                    world.spawn((base, LockPositionX, LockPositionY));
                }
            };
        };
        // GREEN free: the launch arcs it down and to the right.
        ball(
            -5.0,
            4.0,
            [3.0, 0.0],
            false,
            false,
            [0.5, 0.85, 0.55, 1.0],
            "Free",
        );
        // CYAN X-locked: the launch is dropped, it falls straight down at x=0.
        ball(
            0.0,
            5.0,
            [3.0, 0.0],
            true,
            false,
            [0.4, 0.8, 0.95, 1.0],
            "Freeze X",
        );
        // ORANGE Y-locked: gravity cannot hold it, it glides sideways at y=2.
        ball(
            -6.0,
            2.0,
            [1.5, 0.0],
            false,
            true,
            [0.95, 0.55, 0.25, 1.0],
            "Freeze Y",
        );

        eprintln!(
            "[physics-smoke 18] Paused at t=0. Press Play. All three balls are launched sideways, \
             but: the GREEN one is free and ARCS down-right; the CYAN one has Freeze Position X, so \
             the sideways launch is cancelled and it falls STRAIGHT DOWN; the ORANGE one has Freeze \
             Position Y, so gravity cannot pull it down and it GLIDES sideways at a constant height. \
             Select each and see the Freeze X | Freeze Y rows in §11 (Dynamic-only). Press B for the \
             collider outlines. Freeze Position is the rest of the constraint trio beside Freeze \
             Rotation (scene 16)."
        );
    }
}
