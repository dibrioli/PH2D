//! **Which entity gets a point handle** — the joint-anchor gizmo's publish rule.
//!
//! Extracted from `snapshots::publish` so the rule ("a selected joint, and
//! nothing else") is gated HEADLESS: the publish itself needs a camera, a
//! present world and a live `HeroScreen`, none of which a unit test has, but the
//! decision is a pure function of the selection and the sim world.
//!
//! The point gizmo is `ph2d_editor::PointGizmoView` (editor-core): a grabbable
//! dot at a world anchor, for a selected entity that is a POINT rather than a box.
//! A physics joint is exactly that — it carries a `Transform` (its authored
//! anchor) but no drawable geometry, so `snapshots::build_view` returns `None`
//! for it and it would otherwise get no canvas handle at all.

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_host::WindowSize;
use ph2d_render::Camera2d;

/// The point gizmo for the current selection, or `None`.
///
/// `Some` only for a physics **joint**, and only **at rest**:
///
/// - Its `Transform.translation` is the A anchor (the same value the Inspector's
///   Position fields edit), so the filled dot is drawn there.
/// - `anchor_b` is the B anchor, resolved by the caller through the bridge's
///   anchor door; `None` when there is no body B to attach to, and then no
///   second handle is offered (W-J2).
/// - A sprite, a group, or an empty selection gets `None` — a sprite already has
///   a box gizmo, and the others have no anchor to author.
///
/// ⚠️ **Rest-only, and this closes a gap rather than opening one:**
/// `sync_joint_pivots` already declares in its own doc that "during play the dot
/// is not shown and the overlay draws the live solver anchors" — and that claim
/// was false, because nothing here asked. During play the anchors the overlay
/// draws are the solver's, which is what the artist should be reading; the
/// authored handles are a rest-time thing, and a handle that took a drag against
/// a swinging body would author against a pose nobody chose.
///
/// Joints are root entities, so `translation` is the world anchor with no parent
/// compose needed.
#[must_use]
pub(super) fn build_point_view(
    sim: &SimWorld,
    selection: Option<u64>,
    camera: &Camera2d,
    window_size: WindowSize,
    at_rest: bool,
    anchor_b: Option<[f32; 2]>,
    snap: Option<[f32; 2]>,
) -> Option<ph2d_editor::PointGizmoView> {
    if !at_rest {
        return None;
    }
    let entity = Entity::from_bits(selection?);
    sim.world().get::<ph2d_physics_ecs::PhysicsJoint>(entity)?;
    let t = sim.world().get::<Transform>(entity)?;
    Some(ph2d_editor::PointGizmoView {
        anchor_world: [t.translation.x, t.translation.y],
        anchor_b_world: anchor_b,
        snap_world: snap,
        camera_center: camera.center,
        camera_height_world: camera.height_world,
        window_w: window_size.width as f32,
        window_h: window_size.height as f32,
        canvas: ph2d_editor::zones::Rect::new(
            0.0,
            0.0,
            window_size.width as f32,
            window_size.height as f32,
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Vec2;
    use ph2d_ecs::Name;
    use ph2d_physics_ecs::{JointKind, PhysicsJoint};
    use ph2d_render::Sprite;

    fn camera() -> Camera2d {
        Camera2d {
            center: [0.0, 0.0],
            height_world: 10.0,
            ..Camera2d::default()
        }
    }

    fn window() -> WindowSize {
        WindowSize {
            width: 1000,
            height: 1000,
        }
    }

    /// **A selected JOINT gets a point handle at its authored anchor.**
    ///
    /// The whole publish rule: the joint's `Transform.translation` is the anchor,
    /// and the dot must sit exactly there so a drag moves the pivot the artist
    /// sees. Mutation-tested: dropping the `PhysicsJoint` guard would publish a
    /// point handle for every selected entity (a sprite would get a dot on top of
    /// its box), and the sprite case below goes red.
    #[test]
    fn a_selected_joint_gets_a_point_handle_at_its_anchor() {
        let mut sim = SimWorld::new();
        let joint = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(1.5, -2.0)),
                Name::new("Pivot".to_string()),
                PhysicsJoint {
                    kind: JointKind::Pin,
                    ..PhysicsJoint::default()
                },
            ))
            .id();

        let v = build_point_view(
            &sim,
            Some(joint.to_bits()),
            &camera(),
            window(),
            true,
            None,
            None,
        )
        .expect("a selected joint publishes a point handle");
        assert_eq!(
            v.anchor_world,
            [1.5, -2.0],
            "the dot must sit at the joint's authored anchor (its Transform.translation)"
        );
    }

    /// **A selected sprite gets NO point handle** — it already has a box gizmo,
    /// and a dot on top of it would be a second, conflicting move handle. This is
    /// the guard the mutation above breaks.
    #[test]
    fn a_selected_sprite_gets_no_point_handle() {
        let mut sim = SimWorld::new();
        let sprite = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                Sprite::atlas(
                    ph2d_render::WHITE_TILE_KEY,
                    [1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                ),
                Name::new("Box".to_string()),
            ))
            .id();
        assert!(
            build_point_view(
                &sim,
                Some(sprite.to_bits()),
                &camera(),
                window(),
                true,
                None,
                None
            )
            .is_none(),
            "a sprite is not a joint — it already has a box gizmo, so it must not \
             also grow a point handle"
        );
    }

    /// **No selection, no handle.**
    #[test]
    fn nothing_selected_publishes_nothing() {
        let sim = SimWorld::new();
        assert!(build_point_view(&sim, None, &camera(), window(), true, None, None).is_none());
    }

    /// **The handles are rest-only.** During play the overlay draws the SOLVER's
    /// anchors — the live ones — and a grabbable dot authoring against a swinging
    /// pose would write an anchor nobody chose. `sync_joint_pivots` has claimed
    /// this in its own doc since W-AnchorFollow; nothing enforced it until now.
    ///
    /// Mutation-tested: dropping the `at_rest` guard makes this return `Some`.
    #[test]
    fn the_handles_are_not_offered_while_the_clock_runs() {
        let mut sim = SimWorld::new();
        let joint = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(1.5, -2.0)),
                Name::new("Pivot".to_string()),
                PhysicsJoint::default(),
            ))
            .id();
        assert!(
            build_point_view(
                &sim,
                Some(joint.to_bits()),
                &camera(),
                window(),
                false,
                Some([0.0, 0.0]),
                None
            )
            .is_none(),
            "no anchor handles while the simulation is running"
        );
    }

    /// **The B anchor and the snap mark are carried verbatim** — this function
    /// decides *whether* there are handles, never *where* the B one is. That
    /// answer comes from the bridge's anchor door, and re-deriving it here would
    /// be the second opinion that drifts.
    #[test]
    fn the_b_anchor_and_snap_are_passed_through() {
        let mut sim = SimWorld::new();
        let joint = sim
            .world_mut()
            .spawn((
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                Name::new("Pivot".to_string()),
                PhysicsJoint::default(),
            ))
            .id();
        let v = build_point_view(
            &sim,
            Some(joint.to_bits()),
            &camera(),
            window(),
            true,
            Some([4.0, -1.0]),
            Some([2.0, 2.0]),
        )
        .expect("a selected joint publishes handles");
        assert_eq!(v.anchor_b_world, Some([4.0, -1.0]));
        assert_eq!(v.snap_world, Some([2.0, 2.0]));
    }
}
