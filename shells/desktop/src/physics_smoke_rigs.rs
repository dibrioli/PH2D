//! **The smoke scenes where bodies are RELATED** — to each other, or to the
//! Hierarchy (`PH2D_PHYSICS_SMOKE` 6, 7 and 8).
//!
//! Sibling of [`crate::physics_smoke`], which keeps the prologue and the scenes
//! about a body on its own (a drop, a pile, an empty state, the world knobs).
//! Split under the shell's 600-LOC cap, and the seam is a real one: everything
//! here needs a SECOND thing to be meaningful — a joint needs two bodies, a bake
//! needs the timeline, a parented body needs an ancestor.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsJoint, RigidBody};
use ph2d_render::Sprite;

impl crate::App {
    /// **Scene 6 (W3).** The three things joints exist for, side by side: a
    /// **pendulum**, a **chain**, and a **ragdoll**.
    ///
    /// Three and not one, because each answers a different question. The
    /// pendulum says the anchor is where the artist put it (it is pinned at
    /// the plank's END, so a version that used body centres would hang it from
    /// its middle). The chain says links that overlap at their pins do not
    /// fight each other. The ragdoll says limits hold — its knees only bend
    /// one way.
    pub(crate) fn physics_smoke_joints(&mut self) {
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

    /// **Scene 7 (W4).** A drop worth baking, with the clock PAUSED.
    ///
    /// The whole gesture is: select, Bake, watch the curve replay. So the
    /// scene is built to make the bake VISIBLE rather than merely correct —
    /// bodies whose motion has shape (a bounce, a roll, a spin), because a
    /// curve that only says "went down" proves nothing about the fit.
    ///
    /// Paused on purpose: the sim starts at the pose the artist sees, and
    /// baking a scene that has already been running would bake from tick 0
    /// anyway — the picture and the curve would disagree about where the
    /// motion began, which is the confusing kind of correct.
    pub(crate) fn physics_smoke_bake(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        // A ramp: a static box tilted so what lands on it ROLLS. Rotation is
        // the third baked channel and the easiest one to get wrong in silence.
        world.spawn((
            Transform {
                rotation: -0.32,
                ..Transform::from_translation(Vec2::new(-2.2, 1.1))
            },
            Sprite::atlas(0, [3.2, 0.24], [0.40, 0.42, 0.48, 1.0]),
            Name::new("Ramp".to_string()),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.6,
                    half_y: 0.12,
                },
                ..Collider::default()
            },
        ));

        // The ball rolls down the ramp: X, Y and rotation all move, so all
        // three tracks are written and any one of them being wrong shows.
        world.spawn((
            Transform::from_translation(Vec2::new(-3.2, 2.4)),
            Sprite::atlas(0, [0.5, 0.5], [0.95, 0.6, 0.2, 1.0]),
            Name::new("Roller".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                restitution: 0.2,
                friction: 0.7,
                ..Collider::default()
            },
        ));

        // Two boxes that bounce and topple — a second body to bake at the same
        // time, which is the case where a fan-out would have shown up as three
        // undo steps for one click.
        for (i, x) in [1.4f32, 2.1].into_iter().enumerate() {
            world.spawn((
                Transform::from_translation(Vec2::new(x, 3.4 + i as f32 * 1.1)),
                Sprite::atlas(0, [0.6, 0.6], [0.35, 0.75, 0.95, 1.0]),
                Name::new(format!("Box{i}")),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.3,
                        half_y: 0.3,
                    },
                    restitution: 0.45,
                    ..Collider::default()
                },
            ));
        }

        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("timeline", true);
        }

        eprintln!(
            "[physics-smoke 7] A roller, a ramp and two boxes. Clock PAUSED, timeline open.
             Press Play once to SEE the motion, then Ctrl+Z nothing -- just rewind and:
                 1. select Roller + both boxes (marquee or Ctrl-click).
                 2. Inspector > Physics Body > 'Bake 5.0s to Timeline'.
             What must happen:
                 · the timeline fills with a FEW keys per channel, in aligned columns --
                     not one key per frame (that would be unusable, and is the bug).
                 · the Body chip flips to KINEMATIC and the toast says so. Press B: the
                     outlines turn VIOLET (they were cyan -- the solver no longer owns them).
                 · press Play: the objects replay the SAME motion, now driven by the curves.
                 · ONE Ctrl+Z removes the whole bake (all the keys, one press).
             Then the two things the bake is FOR:
                 · grab a key in the timeline and drag it -- the motion is yours to edit now.
                 · the baked bodies still SHOVE: they are kinematic, not ghosts.
             And the reason the transport has a PHYSICS toggle (it is ON here only
             because this is a physics demo; a real project opens with it off):
                 · UNCHECK it and press Play. The baked motion still plays -- it is
                     ANIMATION now, and that is precisely what a bake buys you.
                 · the un-baked box keeps whatever pose it has instead of falling:
                     one clock, and you decide whether the solver hears it.
             Range: it says 5.0s because nothing is animated yet. Arm a loop in the
             transport and the button follows it."
        );
    }

    /// **Scene 8 (W5).** Physics bodies that are CHILDREN in the Hierarchy.
    ///
    /// The bug this scene exists to show was silent: the solver speaks WORLD
    /// and `Transform` is LOCAL, so a parented body simulated at its local
    /// coordinates while it drew at its composed ones. Measured before the fix,
    /// a ball authored at local `(0, 4)` under a rig at `(5, 0)` fell down the
    /// `x = 0` line and drew at `x = 5` — the collider was simply not where the
    /// sprite was, and nothing errored.
    ///
    /// The scene is built so a regression is unmissable rather than subtle:
    /// each rig sits over its OWN short pedestal, far from the origin. A body
    /// that reverts to local coordinates misses its pedestal entirely and falls
    /// forever, in plain view.
    ///
    /// Three depths, because the walk is recursive and a one-level fixture
    /// cannot tell "composes the parent" from "composes the whole chain": one
    /// body one level down, one two levels down, and one under a rig that is
    /// ROTATED — the case where getting the order wrong still lands near enough
    /// to look right until it swings.
    pub(crate) fn physics_smoke_parented(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        let world = gfx.sim.world_mut();

        // (rig x, extra depth, rig rotation, label)
        let rigs = [
            (-3.0_f32, 0_u32, 0.0_f32, "One"),
            (0.0, 1, 0.0, "Two"),
            (3.0, 0, 0.45, "Tilted"),
        ];
        for (x, depth, rot, label) in rigs {
            // A pedestal under each rig — narrow on purpose. It is the oracle:
            // a body in the wrong space cannot land on it.
            world.spawn((
                Transform::from_translation(Vec2::new(x, -1.0)),
                Sprite::atlas(0, [1.6, 0.4], [0.40, 0.42, 0.48, 1.0]),
                Name::new(format!("Pedestal{label}")),
                RigidBody {
                    kind: BodyKind::Static,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.8,
                        half_y: 0.2,
                    },
                    ..Collider::default()
                },
            ));

            // The rig: a plain entity with no physics of its own, exactly like
            // a group node an artist would parent things under.
            let mut parent = world
                .spawn((
                    Transform {
                        translation: Vec2::new(x, 0.0),
                        rotation: rot,
                        ..Transform::IDENTITY
                    },
                    Name::new(format!("Rig{label}")),
                ))
                .id();
            for level in 0..depth {
                parent = world
                    .spawn((
                        Transform::from_translation(Vec2::new(0.0, 0.0)),
                        Name::new(format!("Rig{label}Sub{level}")),
                        ph2d_ecs::ChildOf(parent),
                    ))
                    .id();
            }

            // LOCAL (0, 3): directly above its own pedestal only once the rig's
            // offset is composed in. Read as world it is above the ORIGIN.
            world.spawn((
                Transform::from_translation(Vec2::new(0.0, 3.0)),
                Sprite::atlas(0, [0.5, 0.5], [0.86, 0.62, 0.30, 1.0]),
                Name::new(format!("Ball{label}")),
                ph2d_ecs::ChildOf(parent),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.25 },
                    restitution: 0.3,
                    ..Collider::default()
                },
            ));
        }

        eprintln!(
            "[physics-smoke 8] Three rigs, each with a physics ball parented UNDER it,
             each over its own narrow pedestal. Clock plays immediately.
             What must happen:
                 · every ball lands on the pedestal BELOW ITS OWN RIG and stays there.
                 · press B: each outline sits exactly on its sprite, at every depth.
                     A collider drawn away from its ball is the whole bug, visible.
                 · the tilted rig's ball lands on the tilted rig's pedestal -- rotation
                     in the chain is composed, not dropped.
             The regression is unmissable by construction: a body that falls back to
             reading its LOCAL pose as world drops down the x = 0 line, misses the
             narrow pedestal it was drawn above, and falls out of frame forever.
             Then, to see it is not a trick: drag a RIG in the viewport. Its ball
             follows, keeps its collider, and still collides -- the pose is composed
             every frame, not baked at spawn."
        );
    }
}
