//! Vector-as-scene-object glue (ADR-0076, Rank 10).
//!
//! Each committed `Ph2dVectorAsset` (side-channel `App::committed_vector_pen_paths`)
//! is mirrored to ONE SimWorld entity carrying `(Transform, Name, VectorSceneRef)`:
//!
//! - **`Transform`** is a PLACEMENT overlay — the vertices stay in their frozen
//!   rest-pose world coords; `Transform.translation` is the shape's ABSOLUTE world
//!   centre (spawned = the rest centroid) and the geometry is local to that centroid,
//!   so the gizmo's pivot / scale math (which treats translation as the object centre)
//!   stays consistent and the shape renders exactly where authored at spawn.
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
use ph2d_vector_doc::VectorSelection;

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
/// `translation + R(r)·(s ⊙ (p − c))`, where `translation` is the entity's ABSOLUTE
/// `Transform.translation` (spawned = the centroid `c`) and the geometry is local to
/// the centroid. At `translation = c, r = 0, s = 1` this maps `p → p` (rest pose), so
/// an un-moved vector composes to exactly `world_to_screen`.
///
/// Translation is modeled ABSOLUTE (not a delta from `c`) so the gizmo's pivot /
/// scale math — which treats `Transform.translation` as the object's world centre —
/// stays consistent with the render + pick. A delta model put the gizmo pivot at the
/// world origin → scale drifted.
pub(crate) fn placement_affine(
    translation: [f32; 2],
    r: f32,
    s: [f32; 2],
    c: [f32; 2],
) -> [f32; 6] {
    let (sin, cos) = libm::sincosf(r); // cross-OS bit-identical (mirror of gizmo math)
    // linear M = R(r)·diag(s) — columns x_axis, y_axis.
    let xx = cos * s[0];
    let xy = sin * s[0];
    let yx = -sin * s[1];
    let yy = cos * s[1];
    // world(rest) = translation + M·(rest − c) ⇒ affine translation = translation − M·c
    let mcx = xx * c[0] + yx * c[1];
    let mcy = xy * c[0] + yy * c[1];
    [xx, xy, yx, yy, translation[0] - mcx, translation[1] - mcy]
}

/// Inverse of [`placement_affine`]: world point → rest-pose point. `None` if a
/// scale axis is ~0 (non-invertible). `translation` is the absolute
/// `Transform.translation` (see [`placement_affine`]).
pub(crate) fn world_to_rest(
    translation: [f32; 2],
    r: f32,
    s: [f32; 2],
    c: [f32; 2],
    w: [f32; 2],
) -> Option<[f32; 2]> {
    if s[0].abs() < 1e-6 || s[1].abs() < 1e-6 {
        return None;
    }
    let (sin, cos) = libm::sincosf(r);
    // rest = c + (R(−r)·(w − translation)) ÷ s
    let dx = w[0] - translation[0];
    let dy = w[1] - translation[1];
    let rx = cos * dx + sin * dy;
    let ry = -sin * dx + cos * dy;
    Some([c[0] + rx / s[0], c[1] + ry / s[1]])
}

/// World-space gizmo box for a vector entity: `(center, half_scaled, rotation)`.
/// The box is centered on the absolute `translation` (== the object centre, so it
/// matches the gizmo's own pivot model); the painter rotates it by `rotation` around
/// that center (same convention as the sprite path). `half` is the rest-pose AABB
/// half-extent scaled by `s`.
pub(crate) fn gizmo_box(
    translation: [f32; 2],
    r: f32,
    s: [f32; 2],
    vref: &VectorSceneRef,
) -> ([f32; 2], [f32; 2], f32) {
    let h = vref.half();
    (translation, [s[0].abs() * h[0], s[1].abs() * h[1]], r)
}

// ───────────────────────── ECS glue ─────────────────────────

/// Re-sync the entity vec to match `assets` exactly (positional). Despawns the
/// tail when assets shrank (Esc-clear), spawns `(Transform::from_translation(
/// centroid), Name, VectorSceneRef)` for new appends. Idempotent / O(delta) —
/// call once per frame after the commit bridges have drained.
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
        // Spawn AT the rest centroid (absolute-translation model — see
        // `placement_affine`): translation == centroid renders the vector exactly
        // at its authored rest pose, and gives the gizmo a pivot ON the shape (a
        // zero translation would put the pivot at the world origin → scale drift).
        let centroid = ph2d_core::Vec2::new(
            (bbox_min[0] + bbox_max[0]) * 0.5,
            (bbox_min[1] + bbox_max[1]) * 0.5,
        );
        let e = sim
            .world_mut()
            .spawn((
                Transform::from_translation(centroid),
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

/// Topmost vector under `world_pos`, as the positional index into `entities` /
/// `assets`, or `None`. Inverts each entity's placement before the point-in-region
/// test (the vertices are rest-pose; the `Transform` moved the visual), so a
/// gizmo-moved shape is hit at its NEW position, not its rest pose. Iterates
/// top-first to match the sprite pick.
pub(crate) fn pick_index(
    sim: &SimWorld,
    entities: &[Entity],
    assets: &[Ph2dVectorAsset],
    world_pos: [f32; 2],
) -> Option<usize> {
    for (i, (&e, asset)) in entities.iter().zip(assets).enumerate().rev() {
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
            return Some(i);
        }
    }
    None
}

/// As [`pick_index`] but yields the sim entity bits (for the gizmo selection).
pub(crate) fn pick(
    sim: &SimWorld,
    entities: &[Entity],
    assets: &[Ph2dVectorAsset],
    world_pos: [f32; 2],
) -> Option<u64> {
    pick_index(sim, entities, assets, world_pos).map(|i| entities[i].to_bits())
}

/// Bridge the two object-level selections so a vector selected anywhere is
/// selected everywhere:
/// - **`gizmo_selection`** (`hero.gizmo.selection`) — set by the hierarchy panel
///   and the canvas gizmo-pick; drives the gizmo box + hierarchy highlight.
/// - **`vector_selection.networks`** — set by the vector Select tool; drives the
///   fill/color apply (`vector_inspector_bridge`).
///
/// Edge-triggered: whichever side CHANGED since last frame propagates to the
/// other (so they never fight). Result: select a shape in the hierarchy →
/// recolor it; select it with the Select tool → the gizmo appears; etc. The
/// vertex-level selection (Direct tool) is orthogonal and untouched.
pub(crate) fn sync_object_selection(
    gizmo_selection: &mut Option<u64>,
    vector_selection: &mut VectorSelection,
    entities: &[Entity],
    last_gizmo: &mut Option<u64>,
    last_networks: &mut Vec<usize>,
) {
    let entity_index = |bits: u64| entities.iter().position(|&e| e.to_bits() == bits);
    let is_vector = |bits: u64| entities.iter().any(|&e| e.to_bits() == bits);

    if *gizmo_selection != *last_gizmo {
        // Gizmo / hierarchy changed the selection → derive the vector network so
        // the fill picker recolors the right shape.
        match gizmo_selection.and_then(entity_index) {
            Some(idx) => vector_selection.select_only_network(idx),
            // Selected a sprite / nothing → drop the object-level network pick
            // (any vertex selection is a different tool's concern, left as-is).
            None => vector_selection.networks.clear(),
        }
    } else if vector_selection.networks != *last_networks {
        // A vector tool changed the network selection → arm the gizmo so the box
        // appears + the hierarchy highlights, even though the pick came from the
        // Select tool rather than a canvas gizmo-pick.
        match vector_selection
            .networks
            .first()
            .and_then(|&i| entities.get(i))
        {
            Some(&e) => *gizmo_selection = Some(e.to_bits()),
            None => {
                if gizmo_selection.is_some_and(is_vector) {
                    *gizmo_selection = None;
                }
            }
        }
    }
    *last_gizmo = *gizmo_selection;
    last_networks.clone_from(&vector_selection.networks);
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
    fn at_centroid_is_rest_pose() {
        // translation == centroid, identity rot/scale ⇒ every rest point maps to
        // itself (the un-moved vector renders exactly where authored).
        let c = [500.0, 300.0];
        let m = placement_affine(c, 0.0, [1.0, 1.0], c);
        assert!(close(apply(m, [510.0, 280.0]), [510.0, 280.0]));
        assert!(close(apply(m, c), c));
    }

    #[test]
    fn translation_moves_geometry_rigidly() {
        // Move the centre to c+(20,-10) ⇒ every point shifts by (20,-10).
        let c = [500.0, 300.0];
        let m = placement_affine([520.0, 290.0], 0.0, [1.0, 1.0], c);
        assert!(close(apply(m, [510.0, 280.0]), [530.0, 270.0]));
        assert!(close(apply(m, c), [520.0, 290.0]));
    }

    #[test]
    fn rotation_is_about_the_centre() {
        // Centre held at c, 90°: a point at c+(10,0) → c+(0,10).
        let c = [100.0, 0.0];
        let m = placement_affine(c, std::f32::consts::FRAC_PI_2, [1.0, 1.0], c);
        assert!(close(apply(m, [110.0, 0.0]), [100.0, 10.0]));
        assert!(close(apply(m, c), c)); // the centre is the fixed point
    }

    #[test]
    fn scale_about_the_centre_does_not_drift() {
        // Pure 2× scale with the centre held at c: the centre stays put (no drift),
        // extents double. This is the regression the absolute-translation model fixes.
        let c = [200.0, 100.0];
        let m = placement_affine(c, 0.0, [2.0, 2.0], c);
        assert!(close(apply(m, c), c));
        assert!(close(apply(m, [210.0, 100.0]), [220.0, 100.0])); // +10 from centre → +20
    }

    #[test]
    fn world_to_rest_inverts_placement() {
        let (translation, r, s, c) = ([312.0, 233.0], 0.7, [1.5, 0.8], [320.0, 240.0]);
        let m = placement_affine(translation, r, s, c);
        let rest = [331.0, 233.0];
        let world = apply(m, rest);
        let back = world_to_rest(translation, r, s, c, world).expect("invertible");
        assert!(close(back, rest), "round-trip {back:?} != {rest:?}");
    }

    #[test]
    fn gizmo_box_centers_on_translation() {
        let v = VectorSceneRef {
            bbox_min: [400.0, 200.0],
            bbox_max: [600.0, 300.0],
        };
        // half = (100,50); the box centre is the ABSOLUTE translation (== the gizmo's
        // own pivot model), not the rest centroid.
        let (center, half, rot) = gizmo_box([510.0, 255.0], 0.3, [2.0, 1.0], &v);
        assert!(close(center, [510.0, 255.0]));
        assert!(close(half, [200.0, 50.0])); // scale ⊙ half
        assert!((rot - 0.3).abs() < EPS);
    }
}
