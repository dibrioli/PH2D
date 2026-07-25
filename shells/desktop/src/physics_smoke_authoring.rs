//! **The smoke scenes for a canvas AUTHORING gesture** — dragging a physics
//! handle, not watching the solver (`PH2D_PHYSICS_SMOKE` 38).
//!
//! Sibling of [`crate::physics_smoke_rigs`] (which demos the rigs) — split off it
//! under the shell's 600-LOC cap, and the seam is real: those scenes prove the
//! SOLVER does the right thing, and these prove an EDIT gesture on the canvas
//! does. The first of them is the joint-anchor point gizmo.

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

impl crate::App {
    /// **Scene 38 (W-JointAnchor).** A pendulum, PAUSED, so the artist can drag
    /// its pivot and see the pivot move rather than a swing.
    ///
    /// The gesture the scene exists for: select the pin in the Hierarchy → an
    /// amber DOT appears at its anchor → drag the dot → the pivot (dot + the
    /// amber overlay link) relocates, and pressing Play hangs the plank from the
    /// new spot. Until this wave the anchor was authorable only by typing into
    /// the Inspector's Position fields; the dot is the canvas handle.
    ///
    /// Paused on purpose: a swinging pendulum would drag the eye away from the
    /// one thing being demonstrated, which is that the DOT is grabbable and moves
    /// the pivot. The overlay (key `B`) shows the amber link following.
    pub(crate) fn physics_smoke_joint_anchor(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        // A static hook, a dynamic plank pinned to it at the hook's point — the
        // plank hangs from the pivot, not from its own centre.
        world.spawn((
            Transform::from_translation(Vec2::new(0.0, 4.4)),
            Sprite::atlas(WHITE_TILE_KEY, [0.18, 0.18], [0.75, 0.75, 0.8, 1.0]),
            Name::new("Hook".to_string()),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.09 },
                ..Collider::default()
            },
        ));
        world.spawn((
            Transform::from_translation(Vec2::new(0.7, 4.4)),
            Sprite::atlas(WHITE_TILE_KEY, [1.4, 0.18], [0.95, 0.6, 0.2, 1.0]),
            Name::new("Plank".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 0.7,
                    half_y: 0.09,
                },
                ..Collider::default()
            },
        ));
        // The pin. Its `Transform.translation` IS the anchor the dot grabs.
        world.spawn((
            Transform::from_translation(Vec2::new(0.0, 4.4)),
            Name::new("Pivot".to_string()),
            PhysicsJoint {
                body_a: stable_name_id("Hook"),
                body_b: stable_name_id("Plank"),
                ..PhysicsJoint::default()
            },
        ));

        eprintln!(
            "[physics-smoke 38] A pendulum, PAUSED. The gesture:\n  \
               1. Select 'Pivot' in the Hierarchy -> an AMBER DOT appears at the\n     \
                  pivot (top-left of the plank). Press B to see the amber joint link too.\n  \
               2. DRAG the dot. The pivot moves -- the dot and the amber link follow the\n     \
                  cursor. The plank stays put (paused); it re-hangs when you Play.\n  \
               3. Ctrl+Z undoes the whole drag in ONE press (it is a Transform move,\n     \
                  the same as moving a sprite).\n  \
               4. Play (Physics is armed): the plank now swings from the NEW pivot.\n\
             Before this wave the anchor was only typeable in the Inspector's Position\n\
             fields; the dot is the canvas handle."
        );
    }

    /// **Scene 40 (joint authoring, end to end).** Three PLAIN physics bodies,
    /// no joint yet — the artist AUTHORS one from scratch. The scene answers
    /// "where is the UI to create a joint?": it has been there since W3; this
    /// walks the whole loop (create -> pick kind -> tune -> drag the anchor ->
    /// re-pick a body), which no scene demonstrated before, so an artist thought
    /// it was missing.
    ///
    /// PAUSED: authoring a joint on a scene that is already falling would fight
    /// the artist.
    pub(crate) fn physics_smoke_author_joint(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        // Two bodies to JOIN (a static hook, a dynamic plank) and a spare static
        // anchor (Post) to demonstrate RE-PICKING a body afterwards.
        for (name, x, kind, w, h, colour) in [
            (
                "Hook",
                -1.5f32,
                BodyKind::Static,
                0.2f32,
                0.2f32,
                [0.75, 0.75, 0.8, 1.0],
            ),
            (
                "Plank",
                -0.4,
                BodyKind::Dynamic,
                1.2,
                0.18,
                [0.95, 0.6, 0.2, 1.0],
            ),
            (
                "Post",
                1.7,
                BodyKind::Static,
                0.2,
                0.2,
                [0.4, 0.85, 0.55, 1.0],
            ),
        ] {
            world.spawn((
                Transform::from_translation(Vec2::new(x, 4.5)),
                Sprite::atlas(WHITE_TILE_KEY, [w, h], colour),
                Name::new(name.to_string()),
                RigidBody { kind },
                Collider {
                    shape: if h < 0.19 {
                        ColliderShape::Cuboid {
                            half_x: w * 0.5,
                            half_y: h * 0.5,
                        }
                    } else {
                        ColliderShape::Ball { radius: w * 0.5 }
                    },
                    ..Collider::default()
                },
            ));
        }

        eprintln!(
            "[physics-smoke 40] Three bodies, NO joint yet -- author one end to end.\n\
             PAUSED. The UI to create a joint has existed since W3; this is the walk:\n  \
               1. Select Hook AND Plank (click Hook, then Ctrl-click Plank).\n  \
               2. Inspector > Physics Body grows 'Join Selected Bodies' -> click it.\n     \
                  A Pin joint appears in the Hierarchy, and Plank now hangs from Hook.\n  \
               3. Select the joint -> the 'Physics Joint' section (12) appears. Switch\n     \
                  its Kind (Pin/Spring/Rope/Weld); turn Limits or a Motor on and tune.\n  \
               4. Press B: an amber DOT sits at the pivot -- DRAG it to move the anchor.\n  \
               5. RE-PICK a body: with the joint still selected, Ctrl-click 'Post'. The\n     \
                  section grows 'Set Body A: Post' and 'Set Body B: Post' -- click\n     \
                  'Set Body A'. The joint now hangs from POST instead of Hook, without\n     \
                  deleting and re-making it. (The buttons are DIMMED until a single body\n     \
                  is selected alongside the joint.)\n  \
               6. Play: the plank swings from wherever you left the anchor.\n\
             Ctrl+Z undoes each step; Ctrl+S / Ctrl+O: the joint survives a round trip."
        );
    }
}
