//! Vector-as-scene-object glue (ADR-0076, Rank 10).
//!
//! Each committed `Ph2dVectorAsset` (side-channel `App::committed_vector_pen_paths`)
//! is mirrored to ONE SimWorld entity carrying `(Transform, Name, VectorSceneRef)`:
//!
//! - **`Transform`** is a PLACEMENT overlay — the vertices stay in their frozen
//!   rest-pose world coords; the transform is applied **about the rest centroid**
//!   (§2.4) so the gizmo pivot lands on the vector instead of the world origin.
//!   `IDENTITY` ⇒ the vector renders exactly where it was authored (zero regression).
//! - **`Name`** makes it appear in the scene hierarchy (snapshot queries
//!   `With<Transform>`).
//! - **`VectorSceneRef`** carries the rest-pose AABB so the gizmo-view builder
//!   (`snapshots.rs`) can size its handle box without re-reading the asset.
//!
//! The entity↔asset link is **positional**: `entities[i]` ↔ `assets[i]`. The
//! commit bridges (pen/pencil/shape) only ever append to `assets`, and Esc clears
//! it to empty — both append-or-truncate, so the index is stable within a frame.
//! [`reconcile`] re-syncs the entity vec to match each frame (O(delta), usually a
//! no-op). The gizmo's drag write already targets any entity's SimWorld
//! `Transform` (generic — `gizmo_drag.rs`), so moving a vector needs no new code
//! there. See ADR-0076 for the full rationale + the (deferred) reparent/persist
//! increments.

use ph2d_ecs::{Component, Entity, Name, SimWorld, Transform};
use ph2d_vector::Ph2dVectorAsset;

/// Links a scene entity to the same-index `Ph2dVectorAsset`, plus its rest-pose
/// AABB (world coords, pre-placement). Editor-runtime glue — NOT persisted; it is
/// recomputed from `committed_vector_pen_paths` every session.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub(crate) struct VectorSceneRef {
    pub bbox_min: [f32; 2],
    pub bbox_max: [f32; 2],
}

impl VectorSceneRef {
    pub fn centroid(&self) -> [f32; 2] {
        [
            (self.bbox_min[0] + self.bbox_max[0]) * 0.5,
            (self.bbox_min[1] + self.bbox_max[1]) * 0.5,
        ]
    }
    /// Half-extents of the rest-pose AABB (pre-scale).
    pub fn half(&self) -> [f32; 2] {
        [
            (self.bbox_max[0] - self.bbox_min[0]) * 0.5,
            (self.bbox_max[1] - self.bbox_min[1]) * 0.5,
        ]
    }
}

fn asset_bbox(asset: &Ph2dVectorAsset) -> ([f32; 2], [f32; 2]) {
    match asset.network.bounding_box() {
        Some((mn, mx)) => ([mn.x, mn.y], [mx.x, mx.y]),
        None => ([0.0, 0.0], [0.0, 0.0]),
    }
}

// ───────────────────────── pure placement math (tested) ─────────────────────

/// Placement affine `[xx, xy, yx, yy, zx, zy]` (column-major, == `Transform::affine`
/// / `kurbo::Affine::new` order) mapping a rest-pose point `p` →
/// `c + t + R(r)·(s ⊙ (p − c))`. At `t=0, r=0, s=1` this is the identity, so an
/// un-moved vector composes to exactly `world_to_screen`.
pub(crate) fn placement_affine(t: [f32; 2], r: f32, s: [f32; 2], c: [f32; 2]) -> [f32; 6] {
    let (sin, cos) = libm::sincosf(r); // cross-OS bit-identical (mirror of gizmo math)
    // linear M = R(r)·diag(s) — columns x_axis, y_axis.
    let xx = cos * s[0];
    let xy = sin * s[0];
    let yx = -sin * s[1];
    let yy = cos * s[1];
    // translation = (c + t) − M·c
    let mcx = xx * c[0] + yx * c[1];
    let mcy = xy * c[0] + yy * c[1];
    [xx, xy, yx, yy, c[0] + t[0] - mcx, c[1] + t[1] - mcy]
}

/// Inverse of [`placement_affine`]: world point → rest-pose point. `None` if a
/// scale axis is ~0 (non-invertible).
pub(crate) fn world_to_rest(
    t: [f32; 2],
    r: f32,
    s: [f32; 2],
    c: [f32; 2],
    w: [f32; 2],
) -> Option<[f32; 2]> {
    if s[0].abs() < 1e-6 || s[1].abs() < 1e-6 {
        return None;
    }
    let (sin, cos) = libm::sincosf(r);
    let dx = w[0] - c[0] - t[0];
    let dy = w[1] - c[1] - t[1];
    // R(−r)·delta, then ÷ scale.
    let rx = cos * dx + sin * dy;
    let ry = -sin * dx + cos * dy;
    Some([c[0] + rx / s[0], c[1] + ry / s[1]])
}

/// World-space gizmo box for a vector entity: `(center, half_scaled, rotation)`.
/// The box is centered on the pivot (`c + t`); the painter rotates it by
/// `rotation` around that center (same convention as the sprite path).
pub(crate) fn gizmo_box(
    t: [f32; 2],
    r: f32,
    s: [f32; 2],
    vref: &VectorSceneRef,
) -> ([f32; 2], [f32; 2], f32) {
    let c = vref.centroid();
    let h = vref.half();
    (
        [c[0] + t[0], c[1] + t[1]],
        [s[0].abs() * h[0], s[1].abs() * h[1]],
        r,
    )
}

// ───────────────────────── ECS glue ─────────────────────────

/// Re-sync the entity vec to match `assets` exactly (positional). Despawns the
/// tail when assets shrank (Esc-clear), spawns `(Transform::IDENTITY, Name,
/// VectorSceneRef)` for new appends. Idempotent / O(delta) — call once per frame
/// after the commit bridges have drained.
pub(crate) fn reconcile(
    sim: &mut SimWorld,
    entities: &mut Vec<Entity>,
    assets: &[Ph2dVectorAsset],
) {
    while entities.len() > assets.len() {
        if let Some(e) = entities.pop() {
            let _ = sim.world_mut().despawn(e);
        }
    }
    while entities.len() < assets.len() {
        let i = entities.len();
        let (bbox_min, bbox_max) = asset_bbox(&assets[i]);
        let e = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                Name::new(format!("Vector {}", i + 1)),
                VectorSceneRef { bbox_min, bbox_max },
            ))
            .id();
        entities.push(e);
    }
}

/// Per-asset placement affine (parallel to `entities` / `assets`), for the render
/// bridge to compose `world_to_screen * placement`. Missing/identity entities
/// yield the identity affine.
pub(crate) fn placements(sim: &SimWorld, entities: &[Entity]) -> Vec<[f32; 6]> {
    entities
        .iter()
        .map(|&e| {
            match (
                sim.world().get::<Transform>(e),
                sim.world().get::<VectorSceneRef>(e),
            ) {
                (Some(t), Some(v)) => placement_affine(
                    [t.translation.x, t.translation.y],
                    t.rotation,
                    [t.scale.x, t.scale.y],
                    v.centroid(),
                ),
                _ => [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            }
        })
        .collect()
}

/// Topmost vector under `world_pos`, as sim entity bits, or `None`. Inverts each
/// entity's placement before the point-in-region test (the vertices are rest-pose;
/// the `Transform` moved the visual). Iterates top-first to match the sprite pick.
pub(crate) fn pick(
    sim: &SimWorld,
    entities: &[Entity],
    assets: &[Ph2dVectorAsset],
    world_pos: [f32; 2],
) -> Option<u64> {
    for (&e, asset) in entities.iter().zip(assets).rev() {
        let (Some(t), Some(v)) = (
            sim.world().get::<Transform>(e),
            sim.world().get::<VectorSceneRef>(e),
        ) else {
            continue;
        };
        let Some(rest) = world_to_rest(
            [t.translation.x, t.translation.y],
            t.rotation,
            [t.scale.x, t.scale.y],
            v.centroid(),
            world_pos,
        ) else {
            continue;
        };
        let p = ph2d_core::Vec2::new(rest[0], rest[1]);
        if asset
            .network
            .regions
            .iter()
            .any(|region| asset.network.region_contains_point(region, p))
        {
            return Some(e.to_bits());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;
    fn close(a: [f32; 2], b: [f32; 2]) -> bool {
        (a[0] - b[0]).abs() < EPS && (a[1] - b[1]).abs() < EPS
    }
    /// Apply an `[xx,xy,yx,yy,zx,zy]` affine to a point.
    fn apply(m: [f32; 6], p: [f32; 2]) -> [f32; 2] {
        [
            m[0] * p[0] + m[2] * p[1] + m[4],
            m[1] * p[0] + m[3] * p[1] + m[5],
        ]
    }

    #[test]
    fn identity_placement_is_no_op() {
        let c = [500.0, 300.0]; // rest centroid far from origin
        let m = placement_affine([0.0, 0.0], 0.0, [1.0, 1.0], c);
        // Any rest point maps to itself.
        assert!(close(apply(m, [510.0, 280.0]), [510.0, 280.0]));
        assert!(close(apply(m, c), c));
    }

    #[test]
    fn translation_moves_geometry_rigidly() {
        let c = [500.0, 300.0];
        let m = placement_affine([20.0, -10.0], 0.0, [1.0, 1.0], c);
        assert!(close(apply(m, [510.0, 280.0]), [530.0, 270.0]));
    }

    #[test]
    fn rotation_is_about_centroid_not_origin() {
        let c = [100.0, 0.0];
        // 90° about the centroid: a point at centroid+(10,0) → centroid+(0,10).
        let m = placement_affine([0.0, 0.0], std::f32::consts::FRAC_PI_2, [1.0, 1.0], c);
        assert!(close(apply(m, [110.0, 0.0]), [100.0, 10.0]));
        // The centroid itself is the fixed point.
        assert!(close(apply(m, c), c));
    }

    #[test]
    fn world_to_rest_inverts_placement() {
        let (t, r, s, c) = ([12.0, -7.0], 0.7, [1.5, 0.8], [320.0, 240.0]);
        let m = placement_affine(t, r, s, c);
        let rest = [331.0, 233.0];
        let world = apply(m, rest);
        let back = world_to_rest(t, r, s, c, world).expect("invertible");
        assert!(close(back, rest), "round-trip {back:?} != {rest:?}");
    }

    #[test]
    fn gizmo_box_centers_on_pivot() {
        let v = VectorSceneRef {
            bbox_min: [400.0, 200.0],
            bbox_max: [600.0, 300.0],
        };
        // centroid = (500,250), half = (100,50).
        let (center, half, rot) = gizmo_box([10.0, 5.0], 0.3, [2.0, 1.0], &v);
        assert!(close(center, [510.0, 255.0])); // c + t
        assert!(close(half, [200.0, 50.0])); // scale ⊙ half
        assert!((rot - 0.3).abs() < EPS);
    }
}
