//! **The smoke scene for baking a JOINTED rig** (`PH2D_PHYSICS_SMOKE` 39).
//!
//! Sibling of [`crate::physics_smoke_rigs`], split off under the shell's 600-LOC
//! cap. Baking a coupled rig is coherent only as a WHOLE — bake one link and the
//! un-baked dynamic neighbours freeze when the Physics toggle is off — so this
//! scene proves that selecting ONE link bakes the entire articulated group
//! (`ph2d_physics_ecs::jointed_group`, walked from the joint graph).

use crate::physics_smoke::spawn_floor;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform, stable_name_id};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

impl crate::App {
    /// **Scene 39 (bake a joint).** A three-link chain hung SIDEWAYS from a
    /// static hook, PAUSED — a bake taken mid-swing bakes a half-fallen scene
    /// (scene 7's rule).
    ///
    /// The one thing it proves: selecting ONE link and baking pulls in the WHOLE
    /// chain. `jointed_group` walks the joint edges, so all three links become
    /// kinematic curves and replay as a coherent swing with Physics off. Without
    /// the expansion only the selected link would animate and the other two would
    /// freeze — the half-baked state the feature exists to prevent.
    ///
    /// Sideways on purpose: a chain already hanging straight down does not move,
    /// and then there is nothing to bake. Measured over the 5 s default range the
    /// links whip down and travel far (Link1 ~0.7 m, Link2 ~2.5 m, Link3 ~4.2 m
    /// at their worst), so every link writes tracks and the toast reads
    /// "Baked 3 bodies" — not 1.
    pub(crate) fn physics_smoke_bake_joint(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        // A static hook the chain hangs from.
        world.spawn((
            Transform::from_translation(Vec2::new(-2.0, 4.2)),
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
        // Three dynamic links reaching out to the right, so the chain swings when
        // released. Jointed bodies do not collide with each other, so the links
        // fold freely as it whips down.
        for (i, name) in ["Link1", "Link2", "Link3"].iter().enumerate() {
            world.spawn((
                Transform::from_translation(Vec2::new(-1.3 + i as f32 * 0.9, 4.2)),
                Sprite::atlas(WHITE_TILE_KEY, [0.44, 0.44], [0.95, 0.6, 0.2, 1.0]),
                Name::new((*name).to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.22 },
                    friction: 0.4,
                    ..Collider::default()
                },
            ));
        }
        // Pins at each junction: Hook-Link1, Link1-Link2, Link2-Link3.
        for (name, a, b, x) in [
            ("Pin0", "Hook", "Link1", -1.65f32),
            ("Pin1", "Link1", "Link2", -0.85),
            ("Pin2", "Link2", "Link3", 0.05),
        ] {
            world.spawn((
                Transform::from_translation(Vec2::new(x, 4.2)),
                Name::new(name.to_string()),
                PhysicsJoint {
                    body_a: stable_name_id(a),
                    body_b: stable_name_id(b),
                    kind: JointKind::Pin,
                    ..PhysicsJoint::default()
                },
            ));
        }

        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.panel_visibility.insert("timeline", true);
        }

        eprintln!(
            "[physics-smoke 39] A three-link chain hung sideways from a hook. Clock\n\
             PAUSED, timeline open.\n  \
               1. Press Play once to SEE the chain whip down, then rewind.\n  \
               2. Select ONE link only -- say Link3, the end (click it in the\n     \
                  Hierarchy, or on the canvas).\n  \
               3. Inspector > Physics Body > 'Bake 5.0s to Timeline'.\n\
             What must happen -- the whole point of the scene:\n  \
               · the toast reads 'Baked 3 bodies', NOT 1. Selecting one link baked\n     \
                 the WHOLE chain: a bake walks the joints and pulls in every link it\n     \
                 is coupled to. The static Hook is a boundary and is NOT baked.\n  \
               · press B: all three link outlines turn VIOLET (kinematic now).\n  \
               · UNCHECK the transport's Physics toggle and press Play. The whole\n     \
                 chain replays the swing as ANIMATION.\n\
             Why it matters: without the group expansion, baking Link3 would leave\n\
             Link1 and Link2 Dynamic -- and with Physics off they would FREEZE mid-air\n\
             while Link3 played, the joint stretching between a moving link and a\n\
             still one. There is no coherent partial bake of a coupled rig."
        );
    }
}
