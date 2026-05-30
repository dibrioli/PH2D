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
//! ## Rotation / scale / skew handling
//!
//! Picking honours the full `RenderInstance.basis` (the 2×2 world
//! linear map, ADR-0070-amendment-4): [`pick_sprite_at_world`] inverts
//! the basis to bring the cursor into the sprite's local frame and
//! tests the local quad, so rotated / scaled / skewed sprites pick
//! exactly where they render. The rect / gizmo-box paths use the exact
//! parallelogram AABB. A degenerate basis (det ≈ 0) is unpickable.
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

// ─── basis geometry helpers (ADR-0070-amendment-4) ──────────────────
//
// `RenderInstance.basis` is the 2x2 world linear map (rotation + scale +
// skew), column-major `[col0.x, col0.y, col1.x, col1.y]`, i.e. matrix
// `M = [[b0, b2], [b1, b3]]`. `size`/`anchor` are now LOCAL, so picking
// inverts `M` to test the world cursor against the local quad, and uses
// the exact parallelogram AABB for the rubber-band / gizmo box.

/// Map a WORLD-space delta (cursor − pivot) into the sprite's LOCAL
/// frame by inverting `basis`. `None` when the basis is degenerate.
#[inline]
fn world_delta_to_local(basis: [f32; 4], dx: f32, dy: f32) -> Option<(f32, f32)> {
    let [b0, b1, b2, b3] = basis;
    let det = b0 * b3 - b1 * b2;
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    Some(((b3 * dx - b2 * dy) * inv, (-b1 * dx + b0 * dy) * inv))
}

/// Map a LOCAL point through `basis` to a WORLD delta from the pivot.
#[inline]
fn basis_apply(basis: [f32; 4], x: f32, y: f32) -> (f32, f32) {
    let [b0, b1, b2, b3] = basis;
    (b0 * x + b2 * y, b1 * x + b3 * y)
}

/// World-space AABB half-extents of a centered local quad (half-sizes
/// `half_w`/`half_h`) under `basis` — the exact bbox of the transformed
/// parallelogram (sum of absolute column contributions).
#[inline]
fn world_aabb_half_extents(basis: [f32; 4], half_w: f32, half_h: f32) -> (f32, f32) {
    let [b0, b1, b2, b3] = basis;
    (
        b0.abs() * half_w + b2.abs() * half_h,
        b1.abs() * half_w + b3.abs() * half_h,
    )
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
/// Return every sprite whose oriented bbox contains `world_pos`,
/// ordered top → bottom. Top = last in archetype iteration order
/// (= most recently spawned within an archetype, which matches the
/// "topmost = last hit" heuristic used by [`pick_sprite_at_world`]).
///
/// Used by the alternate-click cycling UX (M14.7 polish 19.3): the
/// host caches this list per click position and walks it on every
/// other click so the user can step down the stack at a single spot
/// without losing their cursor location.
pub fn pick_sprites_at_world(present: &mut World, world_pos: [f32; 2]) -> Vec<u64> {
    let mut hits: Vec<u64> = Vec::new();
    let mut q = present.query::<(&SimRef, &GlobalTransform, &RenderInstance)>();
    for (sim_ref, gt, ri) in q.iter(present) {
        let pos = gt.translation();
        // LOCAL half-extents (basis applies scale); invert the basis to
        // bring the world cursor into the sprite's local frame, so the
        // test honours rotation + scale + skew.
        let half_w = ri.size[0] * 0.5;
        let half_h = ri.size[1] * 0.5;
        let dx = world_pos[0] - pos.x;
        let dy = world_pos[1] - pos.y;
        let Some((local_dx, local_dy)) = world_delta_to_local(ri.basis, dx, dy) else {
            continue;
        };
        // The quad center sits at `anchor` in the local frame — `world_pos`
        // is the pivot, not necessarily the center — so the bbox spans
        // `[anchor - half, anchor + half]`. anchor [0,0] (every legacy
        // sprite) collapses to the original centered test.
        if (local_dx - ri.anchor[0]).abs() <= half_w && (local_dy - ri.anchor[1]).abs() <= half_h {
            hits.push(sim_ref.0.to_bits());
        }
    }
    // Reverse → top-of-pile (last spawned) ends up at index 0.
    hits.reverse();
    hits
}

/// Fase 0f: return every sprite whose world bbox intersects the rect
/// spanning `rect_min`..`rect_max`. Used by the canvas rubber-band
/// box-select gesture. The bbox is the exact AABB of the transformed
/// parallelogram (rotation + scale + skew via the basis). The order of
/// returned bits matches archetype iteration; the caller deduplicates.
pub fn pick_sprites_in_world_rect(
    present: &mut World,
    rect_min: [f32; 2],
    rect_max: [f32; 2],
) -> Vec<u64> {
    let mut hits: Vec<u64> = Vec::new();
    let mut q = present.query::<(&SimRef, &GlobalTransform, &RenderInstance)>();
    for (sim_ref, gt, ri) in q.iter(present) {
        let pos = gt.translation();
        // Exact world AABB of the (possibly rotated/scaled/skewed) quad.
        let (half_w, half_h) =
            world_aabb_half_extents(ri.basis, ri.size[0] * 0.5, ri.size[1] * 0.5);
        let (ax, ay) = basis_apply(ri.basis, ri.anchor[0], ri.anchor[1]);
        let cx = pos.x + ax;
        let cy = pos.y + ay;
        let smin_x = cx - half_w;
        let smax_x = cx + half_w;
        let smin_y = cy - half_h;
        let smax_y = cy + half_h;
        if smax_x >= rect_min[0]
            && smin_x <= rect_max[0]
            && smax_y >= rect_min[1]
            && smin_y <= rect_max[1]
        {
            hits.push(sim_ref.0.to_bits());
        }
    }
    hits
}

pub fn pick_sprite_at_world(present: &mut World, world_pos: [f32; 2]) -> Option<u64> {
    let mut best: Option<u64> = None;
    let mut q = present.query::<(&SimRef, &GlobalTransform, &RenderInstance)>();
    for (sim_ref, gt, ri) in q.iter(present) {
        let pos = gt.translation();
        let half_w = ri.size[0] * 0.5;
        let half_h = ri.size[1] * 0.5;
        let dx = world_pos[0] - pos.x;
        let dy = world_pos[1] - pos.y;
        // Invert the 2x2 basis to bring the cursor delta into the sprite's
        // local frame so the axis-aligned bbox test matches what the user
        // sees — now honouring rotation + scale + skew (ADR-0070-
        // amendment-4). A degenerate basis (det~0) can't be hit.
        let Some((local_dx, local_dy)) = world_delta_to_local(ri.basis, dx, dy) else {
            continue;
        };
        // The quad center sits at `anchor` in the local frame — `world_pos`
        // is the pivot, not necessarily the center — so the bbox spans
        // `[anchor - half, anchor + half]`. anchor [0,0] (every legacy
        // sprite) collapses to the original centered test.
        if (local_dx - ri.anchor[0]).abs() <= half_w && (local_dy - ri.anchor[1]).abs() <= half_h {
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
            // Exact world AABB of the transformed quad (rotation + scale +
            // skew via the basis); centered on the visible quad center
            // (`pivot + basis·anchor`), not the pivot. anchor [0,0]
            // reproduces the original pivot-centered box.
            let (half_w, half_h) =
                world_aabb_half_extents(ri.basis, ri.size[0] * 0.5, ri.size[1] * 0.5);
            let (ax, ay) = basis_apply(ri.basis, ri.anchor[0], ri.anchor[1]);
            let cx = pos.x + ax;
            let cy = pos.y + ay;
            return Some(WorldBbox {
                min: [cx - half_w, cy - half_h],
                max: [cx + half_w, cy + half_h],
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
            basis: RenderInstance::IDENTITY_BASIS,
            texture_id: 0,
            premultiplied: 0.0,
            anchor: [0.0, 0.0],
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: 0,
            z_order: 0,
        };
        present.world_mut().spawn((SimRef(sim_entity), gt, ri));
        sim_entity.to_bits()
    }

    /// Like [`spawn_at`] but with a non-zero pivot offset: the pivot
    /// (transform/world_pos) is at `(x, y)`, while the quad CENTER sits
    /// at `(x + anchor.0, y + anchor.1)`.
    fn spawn_at_with_anchor(
        present: &mut PresentWorld,
        sim_entity: Entity,
        x: f32,
        y: f32,
        size: [f32; 2],
        anchor: [f32; 2],
    ) -> u64 {
        let gt =
            GlobalTransform::from_transform(ph2d_ecs::Transform::from_translation(Vec2::new(x, y)));
        let ri = RenderInstance {
            world_pos: [x, y],
            size,
            atlas_uv: [0.0, 0.0, 1.0, 1.0],
            tint: [1.0, 1.0, 1.0, 1.0],
            basis: RenderInstance::IDENTITY_BASIS,
            texture_id: 0,
            premultiplied: 0.0,
            anchor,
            per_corner_tint: [[1.0; 4]; 4],
            opacity: 1.0,
            flip_uv: 0,
            z_order: 0,
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
    fn anchor_offsets_pick_region_away_from_pivot() {
        // Pivot at origin, 2×2 quad shifted +5 on X by the anchor →
        // quad spans x∈[4,6], y∈[-1,1]. A click on the pivot must MISS
        // (no quad there); a click on the shifted quad center must HIT.
        let mut sim = ph2d_ecs::SimWorld::new();
        let mut present = PresentWorld::new();
        let sim_e = fresh_sim_entity(&mut sim);
        let bits = spawn_at_with_anchor(&mut present, sim_e, 0.0, 0.0, [2.0, 2.0], [5.0, 0.0]);
        assert_eq!(
            pick_sprite_at_world(present.world_mut(), [0.0, 0.0]),
            None,
            "pivot is empty once the quad is anchored away"
        );
        assert_eq!(
            pick_sprite_at_world(present.world_mut(), [5.0, 0.0]),
            Some(bits),
            "the shifted quad center is pickable"
        );
    }

    #[test]
    fn anchor_offsets_selection_bbox_to_quad_center() {
        let mut sim = ph2d_ecs::SimWorld::new();
        let mut present = PresentWorld::new();
        let sim_e = fresh_sim_entity(&mut sim);
        let bits = spawn_at_with_anchor(&mut present, sim_e, 0.0, 0.0, [2.0, 2.0], [5.0, 0.0]);
        let bbox = selection_bbox_world(present.world_mut(), bits).expect("entity present");
        let (center, half) = bbox.center_half();
        assert_eq!(
            center,
            [5.0, 0.0],
            "gizmo box tracks the quad, not the pivot"
        );
        assert_eq!(half, [1.0, 1.0]);
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
