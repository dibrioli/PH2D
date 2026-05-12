//! M14.7 A — sprite hit-testing in world space.
//!
//! Two operations the editor needs once the user starts manipulating
//! sprites directly on the canvas:
//!
//! - [`pick_sprite_at_world`] — given a cursor mapped through
//!   [`Camera2d::screen_to_world`](crate::camera::Camera2d::screen_to_world)
//!   to world coordinates, return the topmost sprite whose
//!   axis-aligned bbox contains the point.
//! - [`selection_bbox_world`] — given the sim-entity bits the editor
//!   stored as its current selection, recover the world-space rect
//!   the gizmo painter (M14.7 B) draws handles on.
//!
//! Both functions operate on **PresentWorld** because that's where
//! `GlobalTransform` lives (per ADR-0021). The host already mirrors
//! `SimRef` from sim → present every frame; that back-pointer is how
//! we surface a stable `entity_bits` to the editor without exposing a
//! `bevy_ecs::Entity` across the ADR-0021 / HR-8 boundary.
//!
//! ## Rotation handling
//!
//! v1 treats every sprite as axis-aligned in world space. The
//! `Transform.rotation` field exists but is ignored at picking time —
//! same simplification the renderer uses today (the quad shader does
//! not apply rotation either). M14.7 D will add rotation support
//! when the gizmo lands; until then any rotated sprite gets picked
//! against the un-rotated bbox.
//!
//! ## Top-most resolution
//!
//! Multiple sprites can overlap the same world point. With no Z field
//! on `Sprite` today we approximate "top-most" by **last hit in
//! iteration order** — bevy_ecs walks each archetype in insertion
//! order, so within a single archetype the latest spawn wins. Cross-
//! archetype the order is implementation-defined, but the editor's
//! demo content tends to share one archetype (Transform + Sprite +
//! optional Name), so this is good enough until a real Z/layer field
//! lands. Once `Sprite` carries Z the tiebreak switches to that.

use crate::sprite::RenderInstance;
use bevy_ecs::world::World;
use ph2d_ecs::{GlobalTransform, SimRef};

/// World-space axis-aligned bounding box. Min/max in meters.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WorldBbox {
    pub min: [f32; 2],
    pub max: [f32; 2],
}

impl WorldBbox {
    /// `(center_x, center_y, half_width, half_height)` derived from
    /// min/max. Used by the gizmo painter to position handles.
    pub fn center_half(&self) -> ([f32; 2], [f32; 2]) {
        let cx = (self.min[0] + self.max[0]) * 0.5;
        let cy = (self.min[1] + self.max[1]) * 0.5;
        let hw = (self.max[0] - self.min[0]) * 0.5;
        let hh = (self.max[1] - self.min[1]) * 0.5;
        ([cx, cy], [hw, hh])
    }

    /// True when `point` lies inside (or on the boundary of) the box.
    pub fn contains(&self, point: [f32; 2]) -> bool {
        point[0] >= self.min[0]
            && point[0] <= self.max[0]
            && point[1] >= self.min[1]
            && point[1] <= self.max[1]
    }
}

/// Return the sim-entity bits of the topmost sprite whose axis-aligned
/// bbox contains `world_pos`. Returns `None` when no sprite covers the
/// point (e.g. the user clicked empty canvas) — the host treats this
/// as a deselect.
///
/// Walks every `(SimRef, GlobalTransform, RenderInstance)` triple in
/// `present`. RenderInstance carries the already-extracted size and
/// matches exactly what the renderer painted on the screen, so a
/// click that visually lands on the sprite is guaranteed to pick.
pub fn pick_sprite_at_world(present: &mut World, world_pos: [f32; 2]) -> Option<u64> {
    let mut best: Option<u64> = None;
    let mut q = present.query::<(&SimRef, &GlobalTransform, &RenderInstance)>();
    for (sim_ref, gt, ri) in q.iter(present) {
        let pos = gt.translation();
        let half_w = ri.size[0] * 0.5;
        let half_h = ri.size[1] * 0.5;
        let dx = world_pos[0] - pos.x;
        let dy = world_pos[1] - pos.y;
        // Rotate the cursor delta into the sprite's local frame so the
        // axis-aligned bbox test matches what the user sees on screen.
        // Without the inverse rotation, the picking AABB tracks the
        // unrotated rect — a click on the visible corner of a rotated
        // sprite would miss. ri.rotation comes from the M14.7 polish
        // extract path; zero for legacy entities means this collapses
        // to the original axis-aligned test.
        let cos_r = ri.rotation.cos();
        let sin_r = ri.rotation.sin();
        let local_dx = dx * cos_r + dy * sin_r;
        let local_dy = -dx * sin_r + dy * cos_r;
        if local_dx.abs() <= half_w && local_dy.abs() <= half_h {
            // Last hit wins — within an archetype bevy_ecs walks in
            // insertion order, so the most recently spawned sprite
            // overrides earlier ones (intuitive "top of the pile").
            best = Some(sim_ref.0.to_bits());
        }
    }
    best
}

/// Look up the world-space bbox of the sprite currently selected by
/// the editor. Returns `None` when the entity no longer exists in
/// PresentWorld (e.g. it was despawned this frame) — callers treat
/// that as "no selection to draw the gizmo over".
pub fn selection_bbox_world(present: &mut World, sim_entity_bits: u64) -> Option<WorldBbox> {
    let mut q = present.query::<(&SimRef, &GlobalTransform, &RenderInstance)>();
    for (sim_ref, gt, ri) in q.iter(present) {
        if sim_ref.0.to_bits() == sim_entity_bits {
            let pos = gt.translation();
            let half_w = ri.size[0] * 0.5;
            let half_h = ri.size[1] * 0.5;
            return Some(WorldBbox {
                min: [pos.x - half_w, pos.y - half_h],
                max: [pos.x + half_w, pos.y + half_h],
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::entity::Entity;
    use ph2d_core::Vec2;
    use ph2d_ecs::PresentWorld;

    /// Spawn a present-side mirror entity for `sim_entity` at `(x,
    /// y)` with `size`. Returns the bits the renderer's picking
    /// surfaces back to callers.
    fn spawn_at(
        present: &mut PresentWorld,
        sim_entity: Entity,
        x: f32,
        y: f32,
        size: [f32; 2],
    ) -> u64 {
        let gt =
            GlobalTransform::from_transform(ph2d_ecs::Transform::from_translation(Vec2::new(x, y)));
        let ri = RenderInstance {
            world_pos: [x, y],
            size,
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            tint: [1.0, 1.0, 1.0, 1.0],
            rotation: 0.0,
            texture_id: 0,
            _pad: [0; 2],
        };
        present.world_mut().spawn((SimRef(sim_entity), gt, ri));
        sim_entity.to_bits()
    }

    /// Allocate a fresh Entity from a SimWorld, then immediately drop
    /// the world to keep the test pure-PresentWorld. Used to get
    /// realistic `Entity::to_bits()` values where index+generation
    /// reflect actual bevy_ecs allocator state.
    fn fresh_sim_entity(sim: &mut ph2d_ecs::SimWorld) -> Entity {
        sim.world_mut().spawn_empty().id()
    }

    #[test]
    fn world_bbox_contains_center() {
        let b = WorldBbox {
            min: [-1.0, -1.0],
            max: [1.0, 1.0],
        };
        assert!(b.contains([0.0, 0.0]));
        assert!(b.contains([1.0, 1.0]));
        assert!(b.contains([-1.0, -1.0]));
        assert!(!b.contains([1.001, 0.0]));
    }

    #[test]
    fn world_bbox_center_half_derives_correctly() {
        let b = WorldBbox {
            min: [2.0, 3.0],
            max: [4.0, 9.0],
        };
        let (c, h) = b.center_half();
        assert_eq!(c, [3.0, 6.0]);
        assert_eq!(h, [1.0, 3.0]);
    }

    #[test]
    fn pick_returns_none_when_no_sprites_overlap() {
        let mut sim = ph2d_ecs::SimWorld::new();
        let mut present = PresentWorld::new();
        let sim_e = fresh_sim_entity(&mut sim);
        spawn_at(&mut present, sim_e, 10.0, 10.0, [1.0, 1.0]);
        let hit = pick_sprite_at_world(present.world_mut(), [0.0, 0.0]);
        assert_eq!(hit, None);
    }

    #[test]
    fn pick_returns_entity_when_point_inside_bbox() {
        let mut sim = ph2d_ecs::SimWorld::new();
        let mut present = PresentWorld::new();
        let sim_e = fresh_sim_entity(&mut sim);
        let bits = spawn_at(&mut present, sim_e, 0.0, 0.0, [2.0, 2.0]);
        // Inside the unit-half bbox.
        let hit = pick_sprite_at_world(present.world_mut(), [0.5, -0.5]);
        assert_eq!(hit, Some(bits));
    }

    #[test]
    fn pick_topmost_when_two_sprites_overlap() {
        let mut sim = ph2d_ecs::SimWorld::new();
        let mut present = PresentWorld::new();
        // Spawn order = present-side iteration order within the
        // shared archetype (Transform + RenderInstance + SimRef).
        // Last spawned wins per the documented "topmost = last hit"
        // heuristic.
        let lower = fresh_sim_entity(&mut sim);
        let upper = fresh_sim_entity(&mut sim);
        spawn_at(&mut present, lower, 0.0, 0.0, [4.0, 4.0]);
        let upper_bits = spawn_at(&mut present, upper, 0.0, 0.0, [2.0, 2.0]);
        let hit = pick_sprite_at_world(present.world_mut(), [0.0, 0.0]);
        assert_eq!(hit, Some(upper_bits));
    }

    #[test]
    fn selection_bbox_recovers_size_from_render_instance() {
        let mut sim = ph2d_ecs::SimWorld::new();
        let mut present = PresentWorld::new();
        let sim_e = fresh_sim_entity(&mut sim);
        let bits = spawn_at(&mut present, sim_e, 5.0, -3.0, [3.0, 2.0]);
        let b = selection_bbox_world(present.world_mut(), bits).unwrap();
        // 3×2 centered on (5, -3): min=(5-1.5, -3-1), max=(5+1.5, -3+1)
        assert!((b.min[0] - 3.5).abs() < 1e-5);
        assert!((b.min[1] - (-4.0)).abs() < 1e-5);
        assert!((b.max[0] - 6.5).abs() < 1e-5);
        assert!((b.max[1] - (-2.0)).abs() < 1e-5);
    }

    #[test]
    fn selection_bbox_none_when_entity_absent() {
        let mut sim = ph2d_ecs::SimWorld::new();
        let mut present = PresentWorld::new();
        let known = fresh_sim_entity(&mut sim);
        spawn_at(&mut present, known, 0.0, 0.0, [1.0, 1.0]);
        let missing = fresh_sim_entity(&mut sim);
        let b = selection_bbox_world(present.world_mut(), missing.to_bits());
        assert!(b.is_none());
    }

    #[test]
    fn pick_respects_size_asymmetry() {
        // A wide-thin sprite — the picking algorithm should honor the
        // size aspect, not just the larger dimension.
        let mut sim = ph2d_ecs::SimWorld::new();
        let mut present = PresentWorld::new();
        let sim_e = fresh_sim_entity(&mut sim);
        let bits = spawn_at(&mut present, sim_e, 0.0, 0.0, [10.0, 1.0]);
        // Inside the wide bbox but well past the thin Y range.
        let outside_y = pick_sprite_at_world(present.world_mut(), [4.0, 2.0]);
        assert_eq!(outside_y, None);
        let inside = pick_sprite_at_world(present.world_mut(), [4.0, 0.3]);
        assert_eq!(inside, Some(bits));
    }
}
