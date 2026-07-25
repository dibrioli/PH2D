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
             PAUSED. This walks the whole create loop:\n  \
               1. Select Hook AND Plank (click Hook, then Ctrl-click Plank).\n  \
               2. Inspector > Physics Body grows a 'Join As' selector\n     \
                  (Pin/Spring/Rope/Weld) + 'Join Selected Bodies'. Pick a TYPE --\n     \
                  say Rope -- then click Join. A ROPE appears, ALREADY SELECTED,\n     \
                  so the 'Physics Joint' section (12) is right there (no hunting the\n     \
                  Hierarchy). Leave it Pin and it is one click.\n  \
               3. In section 12: switch the Kind if you like (Spring/Rope re-anchor\n     \
                  body B at its centre); turn Limits or a Motor on and tune.\n  \
               4. Press B: an amber DOT sits at the pivot -- DRAG it to move the anchor.\n  \
               5. RE-PICK a body: the section shows 'Body A' / 'Body B' with the\n     \
                  current name + an EYEDROPPER each. Click Body A's (it lights up),\n     \
                  then click 'Post' on the canvas. The joint re-binds to POST, no\n     \
                  delete-and-remake, no other object selected first.\n  \
               6. Play: the body swings/hangs from wherever you left the anchor.\n\
             Ctrl+Z undoes each step; Ctrl+S / Ctrl+O: the joint survives a round trip."
        );
    }

    /// **Scene 41 (gold-standard anchor: it FOLLOWS the body).** A pendulum
    /// pinned at the plank's LEFT end, PAUSED. The scene exists to show the fix
    /// for Enio's report: moving a body used to SLIDE the pin along it (the
    /// anchor was a world point re-derived every frame); now the anchor is
    /// body-local, so a body carries its pin with it.
    ///
    /// The two things to try, and what each proves:
    ///   - Move the PLANK (body B): the pin stays glued to its left end (before
    ///     this wave it slid toward the centre / off the end). Press B and Play
    ///     to confirm the plank swings from its END, not its middle.
    ///   - Move the HOOK (body A): the amber DOT follows the hook — the display
    ///     pivot tracks body A.
    pub(crate) fn physics_smoke_anchor_follows(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        // A static hook, a dynamic plank pinned at the plank's LEFT end (which
        // sits exactly under the hook). The plank's centre is 0.7 m to the right
        // of the pin, so "where on the plank is the pin" has teeth: a slide is
        // visible as the plank hanging from its middle instead of its end.
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
            "[physics-smoke 41] A pendulum pinned at the plank's LEFT end, PAUSED.\n\
             The fix: an anchor is BODY-LOCAL, so moving a body carries its pin.\n  \
               1. Press B to see the amber joint link (it starts at the plank's LEFT end).\n  \
               2. Select 'Plank' -> drag it around. The pin STAYS glued to its left end\n     \
                  (before this fix it slid toward the plank's centre). Play: it swings\n     \
                  from its END, not its middle.\n  \
               3. Select 'Hook' (body A) -> move it. The amber DOT follows the hook --\n     \
                  the display pivot tracks body A.\n\
             Ctrl+Z undoes each move; the relationship is preserved either way."
        );
    }

    /// **Scene 42 (W-JointParams).** A ball hanging on a SOFT spring, PLAYING —
    /// the scene exists to smoke Enio's report *"os parâmetros de Spring não
    /// mudam em nada o comportamento da mola"*. Until this wave, editing a joint
    /// parameter while the clock ran did nothing (the re-describe was gated on
    /// the clock being at rest), so tuning a spring while watching it bounce —
    /// the whole point of a spring — was inert until a Reset.
    ///
    /// PLAYS on purpose (not in the pause list): the demonstration IS that the
    /// edit lands live. The ball is heavy and the spring soft, so the sag is big
    /// and cranking the stiffness visibly pulls the ball up.
    pub(crate) fn physics_smoke_live_tune(&mut self) {
        let gfx = self.gfx.as_mut().expect("gfx");
        spawn_floor(gfx.sim.world_mut());
        let world = gfx.sim.world_mut();

        // A static hook up top, a heavy ball hanging a metre below on a spring.
        world.spawn((
            Transform::from_translation(Vec2::new(0.0, 5.6)),
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
            Transform::from_translation(Vec2::new(0.0, 4.6)),
            Sprite::atlas(WHITE_TILE_KEY, [0.7, 0.7], [0.35, 0.6, 0.95, 1.0]),
            Name::new("Ball".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.35 },
                ..Collider::default()
            },
        ));
        // A SOFT spring (stiffness 10, under-damped) so the sag is dramatic and
        // the bounce is visible — the artist stiffens it and the ball rises.
        world.spawn((
            Transform::from_translation(Vec2::new(0.0, 5.6)),
            Name::new("Spring".to_string()),
            PhysicsJoint {
                body_a: stable_name_id("Hook"),
                body_b: stable_name_id("Ball"),
                kind: ph2d_physics_ecs::JointKind::Spring,
                rest_length: 1.0,
                stiffness: 10.0,
                damping: 0.3,
                ..PhysicsJoint::default()
            },
        ));

        eprintln!(
            "[physics-smoke 42] A heavy ball on a SOFT spring, PLAYING.\n\
             The bug: tuning a joint parameter while it ran did NOTHING until a Reset.\n  \
               1. Watch the ball settle -- it sags ~0.4 m below the rest length (soft spring).\n  \
               2. Select 'Spring' in the Hierarchy -> Inspector section 12 (Physics Joint).\n  \
               3. DRAG the Stiffness slider UP (10 -> ~100) WHILE it plays. The ball must\n     \
                  RISE as the spring tightens, LIVE -- before this wave it sat there.\n  \
               4. Drag Damping down -> it bounces more; up -> it stops bouncing. Also live.\n  \
               5. (Motor/limits on a Pin tune live too -- same fix, any parameter.)\n\
             If the ball ignores the sliders until you press Reset, the fix regressed."
        );
    }
}
