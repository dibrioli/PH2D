//! **The LOD partition** (the 160k freeze fix, ADR-0154 follow-up) — sibling of
//! `motion_bridge_objects` via `#[path]`, split off to keep the parent under the shell
//! LOC cap. `use super::*` reaches the parent's `RenderInstance`/`VectorInstance`
//! imports; the parent re-exports [`apply_object_lod`]/[`LOD_COUNT`] so
//! `motion_bridge::dispatch` and the object-bake gates resolve them unchanged.
//!
//! Part 1 made a stamped `source.object` vector render CRISP (one Vello `fill` per
//! instance). That is right at low counts and a per-frame FREEZE at high counts
//! (160k fills). The pre-Part-1 answer — rasterize once, GPU-instance the tile — scaled
//! to millions; this partition brings it back as the LOD floor, keeping crispness where
//! it shows (few / large copies) and swapping to tiles where each stamp is tiny.

use super::*;

/// **The count threshold above which a live vector renders as a GPU-instanced TILE
/// instead of a crisp per-instance vector** — the LOD knee (the freeze-killer).
/// MEASURED (`ph2d-vec-render`'s `the_live_vector_scale_of_shared_instances` sonda,
/// the smoke star with fill+stroke): the crisp vector pass emits ~one Vello `fill`/
/// `stroke` per instance per frame, **18,5 ms/frame CPU at 160k** through the product
/// door [`ph2d_vec_render::draw_shared_instances`] (which now caches the tessellated
/// `BezPath` per geometry — 1,7× the pre-cache 31,9 ms), + GPU — still a freeze at
/// 160k. A tile is one GPU-instanced quad. That cost is LINEAR (≈4× per 4× the count),
/// crossing ~4 ms CPU at **~34k**. The knee is set at **16k** — half of that, since the
/// ~4 ms is CPU-only (the per-instance Vello scene-build) and the GPU rasterizes those
/// N fills on top; halving leaves headroom so the crisp path never stutters. (The cache
/// raised it from 10k: the crisp path got 1,7× cheaper, so the same 4 ms budget affords
/// 1,7× more crisp copies before falling to tiles.) Below it, crispness at any zoom is
/// worth the per-instance cost (a large or few-copy object — a handful to ~sixteen
/// thousand); above it, each stamp is tiny on screen and the fixed-DPI tile is
/// indistinguishable. Tunable; the smoke (`PH2D_MOTION_OBJ_SMOKE=6`) validates 160k →
/// tiles (no freeze) and 16 → crisp — raise it if a few-thousand-copy crisp grid stays
/// smooth for you.
pub(crate) const LOD_COUNT: usize = 16_000;

/// **LOD partition — the freeze fix.** After the cook, `vector_instances` holds one
/// crisp [`VectorInstance`] per stamped live vector; 160k of them is a per-frame
/// freeze. This scans them per `geometry_id`, and any geometry stamped more than
/// `threshold` times **that has a baked LOD tile** has its instances MOVED to
/// `instances` as GPU-instanced tile quads (the pre-Part-1 path, which scaled to
/// millions), leaving only the below-threshold geometries crisp.
///
/// ⚠️ **Per-geometry, not per-object:** the SAME star duplicated 160k times is ONE
/// `geometry_id`, so the count that drives the freeze is per shared geometry. A mixed
/// group with three distinct shapes stamped 60k times each sees three over-threshold
/// geometries; each swaps independently. Covers both a single duplicated object AND a
/// duplicated group's vector children (both ride `geometry_id`).
///
/// ⚠️ **The swap is FAITHFUL:** the tile keeps the instance's `world_pos`/`size`/
/// `basis`/`tint`, only replacing `geometry_id` with the tile `texture_id` + the
/// individual-texture unit UV — so a tile lands exactly where the crisp vector would.
pub(crate) fn apply_object_lod(
    instances: &mut Vec<RenderInstance>,
    vector_instances: &mut Vec<VectorInstance>,
    object_bake: &crate::motion_object_bake::ObjectBake,
    threshold: usize,
) {
    if vector_instances.is_empty() {
        return;
    }
    // Count per geometry_id (a handful of distinct geometries — a small map).
    let mut counts: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
    for vi in vector_instances.iter() {
        *counts.entry(vi.geometry_id).or_insert(0) += 1;
    }
    // Resolve the tile ONCE per over-threshold geometry (not per instance): a geometry
    // past the knee WITH a baked tile becomes a tile; without a tile it stays crisp
    // (correctness before speed — a missing tile must not blank the shape).
    let tile_for: std::collections::BTreeMap<u32, u32> = counts
        .iter()
        .filter(|entry| *entry.1 > threshold)
        .filter_map(|(&gid, _)| Some((gid, object_bake.tile_texture_for_gid(gid)?)))
        .collect();
    if tile_for.is_empty() {
        return; // nothing over threshold (or no tile baked) — all stay crisp
    }
    // Partition: over-threshold instances become tiles appended to `instances`; the
    // rest are retained crisp. `retain` walks once, order-preserving.
    vector_instances.retain(|vi| match tile_for.get(&vi.geometry_id) {
        Some(&texture_id) => {
            instances.push(vector_instance_as_tile(vi, texture_id));
            false // moved to a tile
        }
        None => true, // stays crisp
    });
}

/// Build the GPU-instanced tile quad for a live-vector instance the LOD moved to a
/// raster tile — the SAME field-map [`ph2d_eval_motion::lower_to_instances`]'s `make`
/// builds, so a converted tile is byte-identical to one the membrane would have lowered
/// for the same pose. Keeps `world_pos`/`size`/`basis`/`tint`; adds the tile
/// `texture_id` and the individual-texture unit UV `[0,0,1,1]` (the
/// [`SpriteSource::Individual`] branch); every other field takes its identity value (a
/// Motion instance has no per-corner / opacity / flip / clip authoring surface).
pub(crate) fn vector_instance_as_tile(vi: &VectorInstance, texture_id: u32) -> RenderInstance {
    RenderInstance {
        world_pos: vi.world_pos,
        size: vi.size,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        tint: vi.tint,
        basis: vi.basis,
        premultiplied: 0.0,
        anchor: [0.0, 0.0],
        per_corner_tint: [[1.0; 4]; 4],
        opacity: 1.0,
        flip_uv: 0,
        texture_id,
        z_order: 0,
        sampling: 0,
        uv_xform: RenderInstance::IDENTITY_UV_XFORM,
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        clip_meta: 0,
    }
}
