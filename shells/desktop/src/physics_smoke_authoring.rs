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
}
