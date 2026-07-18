//! `PH2D_PHYSICS_SMOKE=<n>` — READY-TO-SEE scenes for the global rigid
//! physics (ADR-0131).
//!
//! | `n` | Wave | Scene |
//! |-----|------|-------|
//! | `1` | W1   | one **dynamic** sprite dropped above a **static** floor |
//! | `2` | W1.5 | a **pile** of bodies, for scrubbing the clock backwards |
//! | `3` | W2a  | plain sprites + a floor, for authoring bodies in the Inspector |
//! | `4` | W2b  | a world panel scene: bodies of mixed size to feel every knob |
//! | `5` | W2c  | two groups on DIFFERENT layers, for the collision matrix |
//!
//! The sprites are plain ECS entities carrying `RigidBody` + `Collider`.
//! **Nothing here touches the rapier world** — the bridge
//! (`render_loop::physics_bridge`) builds the bodies from the components,
//! steps at the `Playhead` tick, and reads the pose back into `Transform`.
//! That is deliberate ([[feedback_ready_to_smoke_example]] + the impasto
//! smoke scar): if the bridge were dead, the sprites would hang in the air
//! instead of falling — the honest failure, not a hidden pre-step.

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsJoint, RigidBody};
use ph2d_render::Sprite;

/// Static floor, centered at `y = -1` (top at `y = -0.8`). The sprite quad
/// (full size) matches the collider (half-extents).
fn spawn_floor(world: &mut bevy_ecs::world::World) {
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, -1.0)),
        Sprite::atlas(0, [8.0, 0.4], [0.40, 0.42, 0.48, 1.0]),
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 4.0,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
    ));
}

impl crate::App {
    /// Frame prologue, once. No-op without the env.
    pub(crate) fn physics_smoke(&mut self) {
        let Some(which) = std::env::var("PH2D_PHYSICS_SMOKE").ok() else {
            return;
        };
        if self.physics_smoke_done {
            return;
        }
        if self.gfx.is_none() {
            return; // world not up yet; retry next frame
        }
        self.physics_smoke_done = true;

        match which.trim() {
            "2" => self.physics_smoke_pile(),
            "3" => self.physics_smoke_author(),
            "4" => self.physics_smoke_world(),
            "5" => self.physics_smoke_layers(),
            "6" => self.physics_smoke_joints(),
            _ => self.physics_smoke_drop(),
        }

        // Play so the bridge steps the world forward — except in the
        // authoring scene, which must sit STILL until the artist has given
        // something a body (a scene already running is a scene you cannot
        // set up).
        self.playhead.rewind();
        if which.trim() != "3" {
            self.playhead.play();
        } else {
            self.playhead.pause();
        }
    }

    /// **Scene 1 (W1).** One falling body, one floor. Approved 2026-07-18.
    fn physics_smoke_drop(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());

        // Dynamic body dropped from y = 4 → settles at y ≈ -0.5
        // (floor_top -0.8 + half height 0.3).
        //
        // ⚠️ The collider is a CUBOID matching the sprite quad, not a ball.
        // A sprite is a textured square, so a ball collider under it draws as
        // a box and behaves as a circle — the mismatch Enio reported. The
        // outline overlay makes any such mismatch visible; a demo scene
        // should not contain one to begin with.
        gfx.sim.world_mut().spawn((
            Transform::from_translation(Vec2::new(0.0, 4.0)),
            Sprite::atlas(0, [0.6, 0.6], [1.0, 0.5, 0.2, 1.0]),
            Name::new("FallingSprite"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.3,
                    half_y: 0.3,
                },
                density: 1.0,
                ..Collider::default()
            },
        ));

        eprintln!(
            "[physics-smoke 1] FallingSprite (dynamic box) dropped above Floor (static). \
             It should fall and settle on the floor. A dead bridge leaves it hanging in the air."
        );
    }

    /// **Scene 2 (W1.5).** A pile, because a pile is the scene where a wrong
    /// scrub is *visible*: mid-fall the bodies are strewn across the air, and
    /// settled they are a heap. A scrub that quietly replayed from the wrong
    /// state would show one when the ruler says the other.
    ///
    /// Opens the timeline panel itself — asking the artist to press `L` before
    /// the smoke can be run is exactly the assembly a ready-to-smoke scene is
    /// supposed to remove ([[feedback_ready_to_smoke_example]]).
    fn physics_smoke_pile(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());

        // 12 bodies on a staggered grid, so they tumble against each other on
        // the way down instead of dropping in tidy columns.
        for i in 0..12u32 {
            let col = (i % 4) as f32;
            let row = (i / 4) as f32;
            let x = col * 0.9 - 1.35
                + if (row as u32).is_multiple_of(2) {
                    0.0
                } else {
                    0.28
                };
            let y = 1.6 + row * 1.1;
            // Every collider matches its sprite quad. A sprite is a
            // textured SQUARE, so a ball collider under one draws as a box
            // and rolls like a circle — the thing Enio caught. Two sizes,
            // both boxes, so the pile still stacks unevenly and tips over.
            let hue = 0.25 + 0.06 * (i % 5) as f32;
            let (sprite, shape) = if i % 3 != 2 {
                (
                    Sprite::atlas(0, [0.56, 0.56], [1.0, hue + 0.2, hue, 1.0]),
                    ColliderShape::Cuboid {
                        half_x: 0.28,
                        half_y: 0.28,
                    },
                )
            } else {
                (
                    Sprite::atlas(0, [0.6, 0.6], [hue, 0.62, 0.95, 1.0]),
                    ColliderShape::Cuboid {
                        half_x: 0.3,
                        half_y: 0.3,
                    },
                )
            };
            gfx.sim.world_mut().spawn((
                Transform::from_translation(Vec2::new(x, y)),
                sprite,
                Name::new(format!("Body{i:02}")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape,
                    density: 1.0,
                    ..Collider::default()
                },
            ));
        }

        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("timeline", true);
        }

        eprintln!(
            "[physics-smoke 2] 12 bodies falling onto Floor. Let them settle, then DRAG THE \
             PLAYHEAD BACKWARDS on the timeline ruler: the pile must rebuild exactly as it fell, \
             with no stall. (Timeline panel opened for you; `L` toggles it.)"
        );
    }

    /// **Scene 4 (W2b).** The world panel, with a scene built so every knob on
    /// it changes something you can SEE.
    ///
    /// Opens the panel itself — asking the artist to find `W` first is exactly
    /// the assembly a ready-to-smoke scene removes
    /// ([[feedback_ready_to_smoke_example]]).
    ///
    /// The scene is chosen per knob, not for looks:
    /// - **bodies of three sizes**, so *Air Drag* can separate the heavy from
    ///   the light. ⚠️ *Damping* deliberately will NOT: it is a uniform decay,
    ///   and the first smoke failed precisely because that knob was labelled
    ///   "Air Drag" (Enio: *"todos os objetos grandes e pequenos caem na mesma
    ///   velocidade"*). The scene proves both behaviours;
    /// - **a tall stack**, because *Sub-steps* and *Iterations* are visible in
    ///   how a stack settles and in how deep a fast body sinks on impact — a
    ///   single body shows neither;
    /// - **bodies that come to rest**, because *Sleep* is only observable on
    ///   something that stops.
    fn physics_smoke_world(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());

        // Three sizes × four columns: a stack that settles unevenly (so it
        // topples a little, like scene 2) and has small bodies that a drag
        // value slows down long before it touches the big ones.
        let sizes = [0.7f32, 0.45, 0.28];
        for i in 0..12u32 {
            let col = (i % 4) as f32;
            let row = (i / 4) as f32;
            let s = sizes[(i % 3) as usize];
            let x = col * 0.9 - 1.35;
            let y = 2.0 + row * 1.1;
            let hue = 0.25 + 0.18 * (i % 3) as f32;
            gfx.sim.world_mut().spawn((
                Transform::from_translation(Vec2::new(x, y)),
                Sprite::atlas(0, [s, s], [hue, 0.55, 0.85, 1.0]),
                Name::new(format!("Body{i:02}")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: s * 0.5,
                        half_y: s * 0.5,
                    },
                    density: 1.0,
                    ..Collider::default()
                },
            ));
        }

        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 4] 12 bodies of three sizes falling onto Floor, with the PHYSICS \
             panel open (`W` toggles it).\n\
             Try, in order:\n\
               · Gravity Y -> 0      : everything stops falling, mid-air.\n\
               · Gravity X           : the pile slides sideways.\n\
               · Air Drag / Density  : the BIG bodies fall fastest (drag scales\n\
                                       with cross-section, mass resists it).\n\
               · Damping / Linear    : everything slows EQUALLY (uniform decay;\n\
                                       mass cannot enter it -- this is the knob\n\
                                       whose old 'Air Drag' label was the bug).\n\
               · Sub-steps           : less sink on impact (watch a body land).\n\
               · Sleep / Delay -> 0  : the settled pile freezes sooner.\n\
               · Show Colliders      : must agree with the `B` key, always.\n\
               · Reset to Defaults   : everything back, in one click.\n\
             Then Ctrl+S, Ctrl+O: the settings must come back with the project."
        );
    }

    /// **Scene 5 (W2c).** Two groups, two layers, one floor.
    ///
    /// The scene exists to make ONE cell of the matrix visible: the left group
    /// is on layer 0 with the floor, the right group on layer 1. Turning off the
    /// `(1, 0)` cell drops the right group through the floor while the left
    /// group sits — and the right group still stacks on ITSELF, which is what
    /// separates "layers work" from "collisions broke".
    ///
    /// Colour follows LAYER, not position: same-coloured groups would make the
    /// artist read positions to infer a rule.
    fn physics_smoke_layers(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());

        for (layer, x0, hue) in [(0u8, -2.2f32, 0.30f32), (1, 0.8, 0.85)] {
            for i in 0..6u32 {
                let col = (i % 2) as f32;
                let row = (i / 2) as f32;
                gfx.sim.world_mut().spawn((
                    Transform::from_translation(Vec2::new(x0 + col * 0.7, 1.5 + row * 0.8)),
                    Sprite::atlas(0, [0.5, 0.5], [hue, 0.55, 0.9, 1.0]),
                    Name::new(format!("L{layer}_Body{i}")),
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: 0.25,
                            half_y: 0.25,
                        },
                        density: 1.0,
                        layer,
                        ..Collider::default()
                    },
                ));
            }
        }

        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("physics", true);
        }

        eprintln!(
            "[physics-smoke 5] LEFT group on layer 0 (with the floor), RIGHT group on layer 1.\n\
             The physics panel is open (`W`); expand 'Collision Layers'.\n\
             Try:\n\
               · click cell (1,0) : the RIGHT group falls THROUGH the floor, the left\n\
                                    group stays, and the right group still stacks on itself.\n\
               · click it again   : they land again -- and this must work on the bodies\n\
                                    ALREADY in the scene, not only on new ones.\n\
               · click cell (1,1) : the right group stops colliding with ITSELF.\n\
               · Inspector > Layer: select one body and move it to another layer.\n\
               · Ctrl+S, Ctrl+O   : the matrix comes back with the project."
        );
    }

    /// **Scene 6 (W3).** The three things joints exist for, side by side: a
    /// **pendulum**, a **chain**, and a **ragdoll**.
    ///
    /// Three and not one, because each answers a different question. The
    /// pendulum says the anchor is where the artist put it (it is pinned at
    /// the plank's END, so a version that used body centres would hang it from
    /// its middle). The chain says links that overlap at their pins do not
    /// fight each other. The ragdoll says limits hold — its knees only bend
    /// one way.
    fn physics_smoke_joints(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        // A static anchor body — everything hangs off one of these.
        let hook = |world: &mut ph2d_ecs::World, name: &str, x: f32, y: f32| {
            world.spawn((
                Transform::from_translation(Vec2::new(x, y)),
                Sprite::atlas(0, [0.16, 0.16], [0.75, 0.75, 0.8, 1.0]),
                Name::new(name.to_string()),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.08 },
                    ..Collider::default()
                },
            ));
        };
        let limb = |world: &mut ph2d_ecs::World,
                    name: &str,
                    x: f32,
                    y: f32,
                    hw: f32,
                    hh: f32,
                    hue: [f32; 4]| {
            world.spawn((
                Transform::from_translation(Vec2::new(x, y)),
                Sprite::atlas(0, [hw * 2.0, hh * 2.0], hue),
                Name::new(name.to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: hw,
                        half_y: hh,
                    },
                    ..Collider::default()
                },
            ));
        };
        let pin = |world: &mut ph2d_ecs::World, name: &str, a: &str, b: &str, x: f32, y: f32| {
            world.spawn((
                Transform::from_translation(Vec2::new(x, y)),
                Name::new(name.to_string()),
                PhysicsJoint {
                    body_a: stable_name_id(a),
                    body_b: stable_name_id(b),
                    ..PhysicsJoint::default()
                },
            ));
        };

        // --- the pendulum: pinned at the plank's LEFT END, not its centre ---
        hook(world, "PendHook", -4.0, 4.5);
        limb(world, "Plank", -3.4, 4.5, 0.6, 0.09, [0.95, 0.6, 0.2, 1.0]);
        pin(world, "PendPin", "PendHook", "Plank", -4.0, 4.5);

        // --- the chain: links that OVERLAP at their pins ---
        hook(world, "ChainHook", -0.6, 4.8);
        let mut prev = "ChainHook".to_string();
        for i in 0..6u32 {
            let y = 4.5 - i as f32 * 0.42;
            let name = format!("Link{i}");
            limb(world, &name, -0.6, y, 0.1, 0.22, [0.35, 0.75, 0.95, 1.0]);
            pin(world, &format!("ChainPin{i}"), &prev, &name, -0.6, y + 0.24);
            prev = name;
        }

        // --- the ragdoll: a torso on a hook, two arms, two legs with LIMITS ---
        hook(world, "DollHook", 3.0, 5.0);
        limb(world, "Torso", 3.0, 4.4, 0.18, 0.42, [0.9, 0.4, 0.5, 1.0]);
        pin(world, "NeckPin", "DollHook", "Torso", 3.0, 4.82);
        for (name, dx, hw, hh, limited) in [
            ("ArmL", -0.3f32, 0.26f32, 0.07f32, false),
            ("ArmR", 0.3, 0.26, 0.07, false),
            ("LegL", -0.11, 0.08, 0.32, true),
            ("LegR", 0.11, 0.08, 0.32, true),
        ] {
            let (x, y, px, py) = if limited {
                (3.0 + dx, 3.7, 3.0 + dx, 4.02)
            } else {
                (3.0 + dx * 1.6, 4.6, 3.0 + dx * 0.6, 4.6)
            };
            limb(world, name, x, y, hw, hh, [0.9, 0.55, 0.45, 1.0]);
            world.spawn((
                Transform::from_translation(Vec2::new(px, py)),
                Name::new(format!("{name}Pin")),
                PhysicsJoint {
                    body_a: stable_name_id("Torso"),
                    body_b: stable_name_id(name),
                    // A knee bends one way. Without limits the ragdoll is a
                    // bag of sticks, which is the difference the eye reads as
                    // "alive" versus "broken".
                    limits_enabled: limited,
                    limit_min: -0.15,
                    limit_max: 1.4,
                    ..PhysicsJoint::default()
                },
            ));
        }

        eprintln!(
            "[physics-smoke 6] Three rigs, playing. Press B for the overlay: joints are the\n\
             AMBER links (colliders stay green/cyan).\n\
             Watch:\n  \
               · PENDULUM (left)  : it hangs from the plank's END, not its middle, and swings.\n  \
               · CHAIN (middle)   : six links that OVERLAP at their pins and do not fight.\n  \
               · RAGDOLL (right)  : the legs are limited -- knees bend one way, not both.\n\
             Then try the authoring:\n  \
               · select the plank, Inspector > Physics Joint is NOT there (it is a body).\n  \
               · select 'PendPin' in the Hierarchy -> the Joint section appears. Switch it to\n    \
                 Spring: the plank starts bouncing on a spring instead of hanging rigid.\n  \
               · select TWO bodies -> Physics Body grows a 'Join Selected Bodies' button.\n  \
               · Ctrl+Z after any of it, and Ctrl+S / Ctrl+O: the joints survive both."
        );
    }

    /// **Scene 3 (W2).** A floor and three plain sprites with NO physics.
    ///
    /// The whole point is the empty state: select a sprite, open **Physics
    /// Body** in the Inspector, click **Add Physics Body**, press Play. Before
    /// W2 there was no gesture anywhere in the editor that could do that.
    fn physics_smoke_author(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());

        // Three different aspect ratios, because the collider Add derives is
        // the sprite's own box — a mistake there is invisible on a square.
        for (i, (w, h)) in [(1.6f32, 0.5f32), (0.5, 1.4), (0.9, 0.9)]
            .into_iter()
            .enumerate()
        {
            let hue = 0.3 + 0.2 * i as f32;
            gfx.sim.world_mut().spawn((
                Transform::from_translation(Vec2::new(i as f32 * 1.8 - 1.8, 2.0 + i as f32 * 0.6)),
                Sprite::atlas(0, [w, h], [1.0, hue, 0.35, 1.0]),
                Name::new(format!("Prop{i}")),
            ));
        }

        eprintln!(
            "[physics-smoke 3] Three plain sprites and a floor, clock PAUSED. Select a sprite, \
             open the Inspector's 'Physics Body' section, click 'Add Physics Body', then press \
             Play: it should fall and land. The collider is boxed to the sprite, so B (collider \
             outlines) should trace each sprite exactly."
        );
    }
}
