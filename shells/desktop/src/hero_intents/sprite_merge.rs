//! Merge Sprites — composite ≥ 2 selected sprites' images into a
//! single new Individual-texture sprite at the union bounding box,
//! then despawn the originals. Triggered by the Hierarchy row's
//! right-click → "Merge Sprites" entry (Enio 2026-05-27).
//!
//! Pipeline (single one-shot CPU pass):
//!   1. For each selected entity, read source RGBA via the
//!      `texture_edit::read_sprite_source` chokepoint (carries the
//!      sprite's `AlphaMode`; we normalise to straight here so the
//!      compositor is uniform). Cache the transform parameters +
//!      pre-compute a tight axis-aligned world-bbox of the rotated
//!      / scaled / anchored quad.
//!   2. Union the per-source world bboxes → output rect in world
//!      meters. Quantise to `project.pixels_per_meter` for the
//!      output image dims (consistent with import-time density).
//!   3. Backward warp: for each output pixel, project the pixel
//!      centre to world coords; for each source (in selection
//!      order), skip when outside its world bbox, else inverse-
//!      transform to source image px, bilinear-sample (straight),
//!      premultiply, alpha-over into the output (premultiplied
//!      accumulator). The output buffer ends up premultiplied —
//!      matches the Apply path bake (`Sprite.premultiplied = true`).
//!   4. Upload the output as a new `IndividualTextureStore` slot,
//!      spawn one fresh sprite at the union-bbox centre (parenting
//!      under the right-clicked row's parent so the Hierarchy slot
//!      stays in place), and despawn every source entity.
//!
//! Undo: deferred. Multi-sprite snapshot (full Transform + Sprite +
//! Name + ChildOf + source-pixels of N entities) is heavier than the
//! existing per-sprite `ImageEditTransaction` shape; revisit when
//! the user asks (Enio explicitly skipped undo for v1).

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::{Sprite, SpriteRenderer};

use crate::EPS_PIXELS_PER_METER;
use crate::hero_intents::texture_edit;

/// Per-source record gathered in the read pass and reused twice (for
/// bbox union + per-pixel inverse transform during the warp).
struct SrcRecord {
    /// Entity_bits — recorded so the despawn pass at the end targets
    /// ONLY entities that actually contributed pixels (audit A1: the
    /// previous blanket despawn killed entities that failed to read,
    /// silently losing their data).
    bits: u64,
    rgba: Vec<u8>,
    w: u32,
    h: u32,
    // Forward-transform parameters (snapshotted once; world borrow
    // released before the despawn pass below):
    tx: f32,
    ty: f32,
    rot: f32,
    // Cached `rot.cos()` / `rot.sin()` — the per-pixel inner loop
    // calls `world_to_image` for every (out_x, out_y, src), which
    // would otherwise pay one cos+sin per call (≈ 60-cycle
    // transcendentals × N pixels × M sources ≈ tens of millions of
    // ops on a worst-case 4k² merge). Hoisting cuts that to one
    // pair per source — measured ~2× speedup on the dense loop.
    cos_t: f32,
    sin_t: f32,
    scale_x: f32,
    scale_y: f32,
    /// `1.0 / scale_x` — same hoisting reasoning as `cos_t`; lets
    /// `world_to_image` use a multiply where it used to divide.
    inv_scale_x: f32,
    inv_scale_y: f32,
    anchor_x: f32,
    anchor_y: f32,
    size_w: f32,
    size_h: f32,
    /// `1.0 / size_w` — hoisted reciprocal (perf, see `cos_t`).
    inv_size_w: f32,
    inv_size_h: f32,
    // Pre-computed world AABB of the rotated quad — lets the per-
    // pixel inner loop skip sources whose footprint can't possibly
    // cover the current output pixel.
    world_min_x: f32,
    world_max_x: f32,
    world_min_y: f32,
    world_max_y: f32,
}

/// Drain `EditorAction::HierMergeSprites`. Returns `true` if the
/// caller should set `title_dirty` (a toast was pushed).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_merge_sprites(
    entity_bits_list: Vec<u64>,
    primary_bits: u64,
    project_pixels_per_meter: f32,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
) -> bool {
    if entity_bits_list.len() < 2 {
        toasts.push(Toast::warning(
            "Merge Sprites: select 2 or more sprites first",
        ));
        return true;
    }
    let project_pm = project_pixels_per_meter.max(EPS_PIXELS_PER_METER);

    // Dedup + reorder so the right-clicked entity (`primary_bits`)
    // lands at index 0. Audit B2: the grid-snap heuristic at Step 2.5
    // uses `srcs[0]` as the "primary" anchor, and the Hierarchy parent
    // is read from `primary_bits` — those two must agree, otherwise
    // the lossless-snap targets a different sprite than the user
    // right-clicked. Audit A2: dedup defends against accidental
    // duplicates in the selection iter that would over-composite
    // (premul-over isn't idempotent).
    let mut ordered_bits: Vec<u64> = Vec::with_capacity(entity_bits_list.len());
    if entity_bits_list.contains(&primary_bits) {
        ordered_bits.push(primary_bits);
    }
    for &bits in &entity_bits_list {
        if bits != primary_bits && !ordered_bits.contains(&bits) {
            ordered_bits.push(bits);
        }
    }
    let total_requested = ordered_bits.len();

    // Step 1 — read each source + snapshot its transform.
    let mut srcs: Vec<SrcRecord> = Vec::with_capacity(ordered_bits.len());
    for &bits in &ordered_bits {
        let entity = Entity::from_bits(bits);
        let Some(read) =
            texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
        else {
            // Skip — non-sprite entity OR atlas miss / readback fail.
            // Audit A1 + B-H1: the entity is also EXCLUDED from the
            // despawn pass below, so a partial read never destroys
            // the user's sprite without its pixels making it into
            // the merged output.
            continue;
        };
        // Compositing in PREMULTIPLIED space — both for the sampler
        // (avoids the dark-fringe straight-bilinear produces at
        // partial-alpha edges: a half-pixel between opaque red and
        // transparent reads as `R/2, A/2` which composes to
        // half-brightness; premul makes it `R/2, A/2` which composes
        // to full-brightness, half-coverage — the GPU's behaviour for
        // bake-time `Rgba8UnormSrgb` textures) AND for the accumulator
        // (premul "over" is `dst = src + dst*(1-src.a)` with no
        // divide). Atlas sprites are stored straight on disk → premul
        // at read time; Individual-source sprites are already premul
        // after BG-Removal / Trim / etc. bakes → `into_premultiplied`
        // no-ops them.
        let premul = read.image.into_premultiplied();
        let world = sim.world();
        let Some(tr) = world.get::<Transform>(entity) else {
            continue;
        };
        let Some(sprite) = world.get::<Sprite>(entity) else {
            continue;
        };
        let tx = tr.translation.x;
        let ty = tr.translation.y;
        let rot = tr.rotation;
        let scale_x = tr.scale.x;
        let scale_y = tr.scale.y;
        let anchor_x = sprite.anchor[0];
        let anchor_y = sprite.anchor[1];
        let size_w = sprite.size[0];
        let size_h = sprite.size[1];
        // Audit A4 + A5: skip sources whose world footprint OR image
        // dims are degenerate. They contribute zero pixels but the
        // previous code grew the union bbox + ran the inner loop per
        // pixel only to reject every sample → wasted CPU AND output
        // area. `scale` near zero is also caught here — `world_to_image`
        // would divide by zero downstream.
        if size_w.abs() < 1e-6
            || size_h.abs() < 1e-6
            || scale_x.abs() < 1e-6
            || scale_y.abs() < 1e-6
            || premul.width == 0
            || premul.height == 0
        {
            continue;
        }
        // World AABB of the rotated quad — walk the 4 image corners
        // through the same forward chain compose uses
        // (`T * R * Ta * S * P_local`).
        let cos_t = rot.cos();
        let sin_t = rot.sin();
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for &(cx_u, cy_u) in &[(-0.5_f32, -0.5_f32), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)] {
            let lx = cx_u * size_w;
            let ly = cy_u * size_h;
            let sx = lx * scale_x;
            let sy = ly * scale_y;
            let ax = sx + anchor_x;
            let ay = sy + anchor_y;
            let rx = ax * cos_t - ay * sin_t;
            let ry = ax * sin_t + ay * cos_t;
            let wx = rx + tx;
            let wy = ry + ty;
            if wx < min_x {
                min_x = wx;
            }
            if wx > max_x {
                max_x = wx;
            }
            if wy < min_y {
                min_y = wy;
            }
            if wy > max_y {
                max_y = wy;
            }
        }
        srcs.push(SrcRecord {
            bits,
            rgba: premul.pixels,
            w: premul.width,
            h: premul.height,
            tx,
            ty,
            rot,
            cos_t,
            sin_t,
            scale_x,
            scale_y,
            inv_scale_x: 1.0 / scale_x,
            inv_scale_y: 1.0 / scale_y,
            anchor_x,
            anchor_y,
            size_w,
            size_h,
            inv_size_w: 1.0 / size_w,
            inv_size_h: 1.0 / size_h,
            world_min_x: min_x,
            world_max_x: max_x,
            world_min_y: min_y,
            world_max_y: max_y,
        });
    }

    if srcs.len() < 2 {
        toasts.push(Toast::error(
            "Merge Sprites: could not read 2 source images (atlas miss or readback failed)",
        ));
        return true;
    }

    // Step 2 — union bbox in world meters.
    let mut union_min_x = srcs
        .iter()
        .map(|s| s.world_min_x)
        .fold(f32::INFINITY, f32::min);
    let mut union_max_x = srcs
        .iter()
        .map(|s| s.world_max_x)
        .fold(f32::NEG_INFINITY, f32::max);
    let mut union_min_y = srcs
        .iter()
        .map(|s| s.world_min_y)
        .fold(f32::INFINITY, f32::min);
    let mut union_max_y = srcs
        .iter()
        .map(|s| s.world_max_y)
        .fold(f32::NEG_INFINITY, f32::max);
    if union_max_x <= union_min_x || union_max_y <= union_min_y {
        toasts.push(Toast::error("Merge Sprites: degenerate union bounding box"));
        return true;
    }

    // Step 2.5 — pixel-grid alignment (Enio 2026-05-27 "o merge não
    // modificar nada das imagens prévias sobrepostas"). For the common
    // case where the first source is axis-aligned at unit scale, two
    // changes make the output LOSSLESS for that source AND any source
    // sharing its grid:
    //
    //   (a) Output density matches the source's native px/m instead of
    //       the (possibly different) project px/m. No up/down-sampling.
    //   (b) Union bbox snaps to the source's pixel boundaries. Output
    //       pixel `o` then aligns with source pixel `o + k` (integer k)
    //       → `img_x` from the inverse warp is exactly integer →
    //       bilinear at integer reads ONE sample → no half-pixel blur.
    //
    // Rotated / scaled sources still bilinear-resample (unavoidable),
    // but the dark-fringe is fixed by the premul-space sampling above.
    let primary = &srcs[0];
    let primary_axis_aligned = primary.rot.abs() < 1e-4
        && (primary.scale_x - 1.0).abs() < 1e-4
        && (primary.scale_y - 1.0).abs() < 1e-4;
    let out_pm = if primary_axis_aligned && primary.size_w > 0.0 && primary.size_h > 0.0 {
        let px_per_m_w = primary.w as f32 / primary.size_w;
        let px_per_m_h = primary.h as f32 / primary.size_h;
        // Sanity: square pixels for an axis-aligned unit-scale sprite.
        // Average defensively against floating-point asymmetry.
        (px_per_m_w + px_per_m_h) * 0.5
    } else {
        project_pm
    };
    if primary_axis_aligned {
        // Primary's pixel boundaries in world.
        let p_left = primary.tx - primary.size_w * 0.5 + primary.anchor_x;
        let p_top = primary.ty + primary.size_h * 0.5 + primary.anchor_y;
        let snap_to_grid = |coord: f32, origin: f32, ceil: bool| {
            let offset = (coord - origin) * out_pm;
            let snapped = if ceil { offset.ceil() } else { offset.floor() };
            origin + snapped / out_pm
        };
        // Floor for min, ceil for max — grow the union outward so no
        // contribution from any source gets clipped.
        union_min_x = snap_to_grid(union_min_x, p_left, false);
        union_max_x = snap_to_grid(union_max_x, p_left, true);
        union_min_y = snap_to_grid(union_min_y, p_top, false);
        union_max_y = snap_to_grid(union_max_y, p_top, true);
    }
    let union_w_m = union_max_x - union_min_x;
    let union_h_m = union_max_y - union_min_y;

    let out_w = (union_w_m * out_pm).round().max(1.0) as u32;
    let out_h = (union_h_m * out_pm).round().max(1.0) as u32;
    let max_dim = renderer.max_texture_dimension_2d();
    if out_w > max_dim || out_h > max_dim {
        toasts.push(Toast::error(format!(
            "Merge Sprites: output {out_w}×{out_h} px exceeds device limit {max_dim} px"
        )));
        return true;
    }

    // Step 3 — backward-warp + premultiplied "over" composite.
    // Bytes sampled from `src.rgba` are already premultiplied (Step 1
    // normalised every source via `into_premultiplied`), so the
    // per-pixel inner loop just bilerps in premul space and runs the
    // canonical Porter-Duff "over" without re-multiplying by alpha.
    let n_pixels = (out_w as usize) * (out_h as usize);
    let mut out_rgba = vec![0u8; n_pixels * 4];
    for out_y in 0..out_h {
        let wy = union_max_y - (out_y as f32 + 0.5) / out_pm;
        for out_x in 0..out_w {
            let wx = union_min_x + (out_x as f32 + 0.5) / out_pm;
            let mut acc_r = 0.0_f32;
            let mut acc_g = 0.0_f32;
            let mut acc_b = 0.0_f32;
            let mut acc_a = 0.0_f32;
            for src in &srcs {
                if wx < src.world_min_x
                    || wx > src.world_max_x
                    || wy < src.world_min_y
                    || wy > src.world_max_y
                {
                    continue;
                }
                let (img_x, img_y) = world_to_image(wx, wy, src);
                let Some((pr_u8, pg_u8, pb_u8, pa_u8)) =
                    bilinear_sample_premul(&src.rgba, src.w, src.h, img_x, img_y)
                else {
                    continue;
                };
                if pa_u8 == 0 {
                    continue;
                }
                let pa = pa_u8 as f32 * (1.0 / 255.0);
                let pr = pr_u8 as f32 * (1.0 / 255.0);
                let pg = pg_u8 as f32 * (1.0 / 255.0);
                let pb = pb_u8 as f32 * (1.0 / 255.0);
                let inv = 1.0 - pa;
                acc_r = pr + acc_r * inv;
                acc_g = pg + acc_g * inv;
                acc_b = pb + acc_b * inv;
                acc_a = pa + acc_a * inv;
            }
            let idx = ((out_y as usize) * (out_w as usize) + (out_x as usize)) * 4;
            out_rgba[idx] = (acc_r * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            out_rgba[idx + 1] = (acc_g * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            out_rgba[idx + 2] = (acc_b * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            out_rgba[idx + 3] = (acc_a * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
        }
    }

    // Step 4a — upload to a fresh Individual slot BEFORE despawning the
    // originals so a failed acquire bails without losing data.
    let texture_id = match renderer.acquire_individual(out_w, out_h, &out_rgba) {
        Ok(id) => id,
        Err(e) => {
            toasts.push(Toast::error(format!(
                "Merge Sprites: GPU upload failed: {e}"
            )));
            return true;
        }
    };

    // Step 4b — gather the primary's parent (Hierarchy anchor) before
    // despawn invalidates the entity. The dedup at Step 0 guarantees
    // `primary_bits` is in `srcs[0]` whenever it was in the original
    // selection AND read succeeded; otherwise fall back to None
    // (root-level merge).
    let parent_opt = sim
        .world()
        .get::<ph2d_ecs::ChildOf>(Entity::from_bits(primary_bits))
        .map(|c| c.parent());

    // Step 4c — despawn originals. ONLY the entities whose pixels
    // actually contributed to the output (audit A1 + B-H1: blindly
    // despawning `entity_bits_list` would have destroyed sprites that
    // failed readback AND non-sprite entities in the selection like a
    // camera the user happened to multi-select). `ChildOf` cascade
    // despawns descendants too — same convention as `HierDelete`.
    let n_sources = srcs.len();
    let n_requested = total_requested;
    for src in &srcs {
        sim.world_mut().despawn(Entity::from_bits(src.bits));
    }

    // Step 4d — spawn the merged sprite at the union-bbox centre.
    let world_center_x = (union_min_x + union_max_x) * 0.5;
    let world_center_y = (union_min_y + union_max_y) * 0.5;
    let mut transform = Transform::default();
    transform.translation.x = world_center_x;
    transform.translation.y = world_center_y;
    let mut merged_sprite =
        Sprite::individual(texture_id, [union_w_m, union_h_m], [1.0, 1.0, 1.0, 1.0]);
    merged_sprite.premultiplied = true;

    let new_entity = match parent_opt {
        Some(parent) => sim
            .world_mut()
            .spawn((
                transform,
                merged_sprite,
                ph2d_ecs::Name::new("Merged"),
                ph2d_ecs::ChildOf(parent),
            ))
            .id(),
        None => sim
            .world_mut()
            .spawn((transform, merged_sprite, ph2d_ecs::Name::new("Merged")))
            .id(),
    };

    // The new merged entity_bits is communicated upward through a
    // panic-free side channel: the caller in `render_loop/mod.rs`
    // reads it via the returned struct (audit B-H2 — Photoshop /
    // Figma promote the merged layer to the selection so the user's
    // next action operates on it). See `MergeResult` below.
    last_merge_result_set(MergeResult {
        new_entity_bits: new_entity.to_bits(),
    });

    // Audit B-H3: surface skipped entities in the toast so the user
    // notices when a non-sprite / readback-failed entity was in the
    // selection and got SKIPPED (not merged, but also not destroyed).
    let skipped = n_requested.saturating_sub(n_sources);
    if skipped > 0 {
        toasts.push(Toast::success(format!(
            "Merged {n_sources} sprites · skipped {skipped} non-sprite entries"
        )));
    } else {
        toasts.push(Toast::success(format!("Merged {n_sources} sprites")));
    }
    true
}

/// Side channel for the caller to learn the newly-spawned merged
/// entity's bits so it can promote it to the selection (audit B-H2).
/// Threading a return field through the drain signature would change
/// the public surface; a thread-local set-once cell keeps the drain
/// API stable and follows the same pattern `brush_cursor` already
/// uses for input-overlay state.
#[derive(Copy, Clone, Debug)]
pub(crate) struct MergeResult {
    pub new_entity_bits: u64,
}

thread_local! {
    static LAST_MERGE: std::cell::Cell<Option<MergeResult>> = const { std::cell::Cell::new(None) };
}

fn last_merge_result_set(r: MergeResult) {
    LAST_MERGE.with(|c| c.set(Some(r)));
}

/// Drain the most recent `MergeResult` (set by `drain_merge_sprites`
/// on success). Returns `None` outside the immediate frame after a
/// merge. Caller should read this AFTER `drain_merge_sprites` and
/// before the next frame to promote the new entity to the selection.
pub(crate) fn take_last_merge_result() -> Option<MergeResult> {
    LAST_MERGE.with(|c| c.take())
}

/// World-px → source-image-px via inverse of the forward chain
/// `T * R * Ta * S * P_local` (matches `sprite_image_to_screen_affine`
/// minus the screen projection step). Returns `None` on degenerate
/// scale / size that would divide by zero.
///
/// Returns coordinates in PIXEL-INDEX convention — `img_x = i` lands
/// exactly on the centre of source pixel `i` (matches the GPU
/// `textureSample` convention where UV `(0.5/W, 0.5/H)` is the centre
/// of texel `(0, 0)`). The `- 0.5` half-pixel correction converts the
/// forward chain's edge-coordinate output (where `img_x = 0` is the
/// LEFT edge of pixel 0, not its centre) into the centre-aligned
/// coordinate the bilinear sampler expects. Without it, every output
/// pixel landed at fractional `img_x = N + 0.5` → bilinear blended
/// pixel `N` with pixel `N + 1` 50/50 → half-pixel blur everywhere,
/// even when the grid was perfectly snapped (Enio 2026-05-27 "ainda
/// borra imagem").
fn world_to_image(wx: f32, wy: f32, src: &SrcRecord) -> (f32, f32) {
    let dx = wx - src.tx;
    let dy = wy - src.ty;
    // Inverse rotation = transpose for a pure rotation matrix. Uses
    // the pre-cached `cos_t` / `sin_t` (computed once per source at
    // read-time, not per-pixel).
    let post_anchor_x = dx * src.cos_t + dy * src.sin_t;
    let post_anchor_y = -dx * src.sin_t + dy * src.cos_t;
    let post_scale_x = post_anchor_x - src.anchor_x;
    let post_scale_y = post_anchor_y - src.anchor_y;
    // Degenerate scale / size are filtered at the read pass (audit
    // A4/A5), so the cached reciprocals are always finite here.
    let local_x = post_scale_x * src.inv_scale_x;
    let local_y = post_scale_y * src.inv_scale_y;
    let img_x = (local_x * src.inv_size_w + 0.5) * src.w as f32 - 0.5;
    let img_y = (0.5 - local_y * src.inv_size_h) * src.h as f32 - 0.5;
    (img_x, img_y)
}

/// Bilinear sample of an RGBA8 buffer in PREMULTIPLIED space — the
/// caller is responsible for handing in premul bytes (see Step 1's
/// `into_premultiplied`). Returns `None` when `(x, y)` lies outside
/// `[0, w-1] × [0, h-1]` — caller treats that as fully transparent
/// (skips compositing for that source at this output pixel).
///
/// Sampling premul instead of straight at edge pixels avoids the
/// "dark fringe" classical straight-bilinear produces: a half-pixel
/// between an opaque colour and full transparency reads in straight
/// space as half-brightness colour at half coverage, which composes
/// to quarter-brightness; in premul space it reads as half-brightness
/// premul at half coverage, which composes to half-brightness — the
/// physically correct partial coverage of full-brightness pixels.
fn bilinear_sample_premul(rgba: &[u8], w: u32, h: u32, x: f32, y: f32) -> Option<(u8, u8, u8, u8)> {
    if w == 0 || h == 0 {
        return None;
    }
    // Pixel-index convention with half-pixel border on each edge —
    // `(-0.5, w - 0.5)` is the half-pixel "clamp-to-edge" zone
    // (matches GPU `textureSample` with `address_mode = ClampToEdge`).
    // Outside that range, the source's footprint doesn't cover this
    // world position; return `None` so the caller skips it.
    if x < -0.5 || x > (w as f32 - 0.5) || y < -0.5 || y > (h as f32 - 0.5) {
        return None;
    }
    // Clamp into the integer-index range `[0, w-1] × [0, h-1]` before
    // splitting into the bilinear quad. Values in `[-0.5, 0)` fold
    // onto pixel 0 (clamp-to-edge); similarly for the right border.
    let xc = x.clamp(0.0, (w - 1) as f32);
    let yc = y.clamp(0.0, (h - 1) as f32);
    let x0 = xc.floor() as u32;
    let y0 = yc.floor() as u32;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let fx = xc - x0 as f32;
    let fy = yc - y0 as f32;
    let sample = |xx: u32, yy: u32| -> (f32, f32, f32, f32) {
        let base = ((yy as usize) * (w as usize) + (xx as usize)) * 4;
        (
            rgba[base] as f32,
            rgba[base + 1] as f32,
            rgba[base + 2] as f32,
            rgba[base + 3] as f32,
        )
    };
    let s00 = sample(x0, y0);
    let s10 = sample(x1, y0);
    let s01 = sample(x0, y1);
    let s11 = sample(x1, y1);
    let lerp = |a: f32, b: f32, t: f32| a * (1.0 - t) + b * t;
    let r0 = lerp(s00.0, s10.0, fx);
    let g0 = lerp(s00.1, s10.1, fx);
    let b0 = lerp(s00.2, s10.2, fx);
    let a0 = lerp(s00.3, s10.3, fx);
    let r1 = lerp(s01.0, s11.0, fx);
    let g1 = lerp(s01.1, s11.1, fx);
    let b1 = lerp(s01.2, s11.2, fx);
    let a1 = lerp(s01.3, s11.3, fx);
    let r = (r0 * (1.0 - fy) + r1 * fy).clamp(0.0, 255.0);
    let g = (g0 * (1.0 - fy) + g1 * fy).clamp(0.0, 255.0);
    let b = (b0 * (1.0 - fy) + b1 * fy).clamp(0.0, 255.0);
    let a = (a0 * (1.0 - fy) + a1 * fy).clamp(0.0, 255.0);
    Some((r as u8, g as u8, b as u8, a as u8))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build an axis-aligned-at-unit-scale `SrcRecord` for tests.
    /// World-rect: `[tx − size_w/2, tx + size_w/2] × [...]` (Y-up).
    fn unit_src(tx: f32, ty: f32, size_w: f32, size_h: f32, w: u32, h: u32) -> SrcRecord {
        SrcRecord {
            bits: 0,
            rgba: vec![0u8; (w as usize) * (h as usize) * 4],
            w,
            h,
            tx,
            ty,
            rot: 0.0,
            cos_t: 1.0,
            sin_t: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            inv_scale_x: 1.0,
            inv_scale_y: 1.0,
            anchor_x: 0.0,
            anchor_y: 0.0,
            size_w,
            size_h,
            inv_size_w: 1.0 / size_w,
            inv_size_h: 1.0 / size_h,
            world_min_x: tx - size_w * 0.5,
            world_max_x: tx + size_w * 0.5,
            world_min_y: ty - size_h * 0.5,
            world_max_y: ty + size_h * 0.5,
        }
    }

    /// The `-0.5` half-pixel correction in `world_to_image` MUST land
    /// integer `img_x` for an output-pixel world centre that aligns
    /// with the source pixel grid. Audit + Enio 2026-05-27 "ainda
    /// borra imagem": before the correction, `img_x` was `i + 0.5` →
    /// bilinear blended pixel `i` with pixel `i+1` 50/50 → uniform
    /// half-pixel blur everywhere even on perfectly snapped grids.
    /// This test guards the lossless invariant.
    #[test]
    fn axis_aligned_centre_maps_to_integer_pixel_index() {
        // 10×10 m sprite at origin, 100×100 px (10 px/m).
        let src = unit_src(0.0, 0.0, 10.0, 10.0, 100, 100);
        // World-px-centre of source pixel 0 (top-left): `(p_left +
        // 0.5/pm, p_top - 0.5/pm)`.
        let wx = -5.0 + 0.5 / 10.0;
        let wy = 5.0 - 0.5 / 10.0;
        let (img_x, img_y) = world_to_image(wx, wy, &src);
        assert!(
            (img_x - 0.0).abs() < 1e-3,
            "img_x should be exactly 0 (pixel-index of pixel 0 centre), got {img_x}"
        );
        assert!(
            (img_y - 0.0).abs() < 1e-3,
            "img_y should be exactly 0, got {img_y}"
        );

        // World-centre of source pixel 50 along X.
        let wx_50 = -5.0 + (50.5) / 10.0;
        let (img_x_50, _) = world_to_image(wx_50, 0.0, &src);
        assert!(
            (img_x_50 - 50.0).abs() < 1e-3,
            "img_x at pixel 50 centre should be 50, got {img_x_50}"
        );
    }

    /// World-edge (where `img_x` should be exactly `−0.5`, the
    /// clamp-to-edge zone boundary) must NOT exceed the sampler's
    /// reject threshold of `−0.5` (`bilinear_sample_premul` line ~441).
    #[test]
    fn left_edge_lands_at_neg_half() {
        let src = unit_src(0.0, 0.0, 10.0, 10.0, 100, 100);
        // Exact world left edge of the sprite.
        let wx = -5.0;
        let (img_x, _) = world_to_image(wx, 0.0, &src);
        assert!(
            (img_x - (-0.5)).abs() < 1e-3,
            "img_x at world left-edge should be -0.5, got {img_x}"
        );
        // Sampler must accept -0.5 (clamp-to-edge zone). It does — the
        // check is `x < -0.5`, strict less than.
        let s = bilinear_sample_premul(&src.rgba, src.w, src.h, img_x, 0.0);
        assert!(s.is_some(), "sampler must accept -0.5 (clamp-to-edge)");
    }

    /// Negative scale (mirrored sprite) flips the inverse mapping
    /// correctly. Audit hand-trace said this works; assert it.
    #[test]
    fn negative_scale_mirror_inverse_is_correct() {
        // size 100 m, scale -1 → effective world range `[-50, 50]` still
        // (corners swap which side they map from in source).
        let mut src = unit_src(0.0, 0.0, 100.0, 100.0, 100, 100);
        src.scale_x = -1.0;
        src.inv_scale_x = -1.0;
        // World x = 25 (right of centre). For a mirrored sprite, this
        // should sample the LEFT half of the source. local_x = 25 / -1
        // = -25 → img_x = ((-25)/100 + 0.5) * 100 - 0.5 = 25 - 0.5 =
        // 24.5 → pixel ~24 (half between 24 and 25). The OLD code (no
        // -0.5) returned 25, but pixel-index 25 is centre of pixel 25,
        // not pixel 24.5. The new convention is the correct one.
        let (img_x, _) = world_to_image(25.0, 0.0, &src);
        assert!(
            (img_x - 24.5).abs() < 1e-3,
            "mirrored sprite: world x=25 → img_x=24.5, got {img_x}"
        );
    }

    /// Sampler bounds: outside `[-0.5, w-0.5]` should reject, inside
    /// should accept including the half-pixel border. Defensive
    /// regression — audit flagged that the old `[0, w-1]` range
    /// underflows `floor()` on `-0.5`.
    #[test]
    fn sampler_bounds_include_half_pixel_border() {
        let rgba = vec![255u8; 4 * 4];
        // Inside.
        assert!(bilinear_sample_premul(&rgba, 2, 2, 0.0, 0.0).is_some());
        // Left clamp-to-edge border.
        assert!(bilinear_sample_premul(&rgba, 2, 2, -0.4, 0.0).is_some());
        // Right clamp-to-edge border.
        assert!(bilinear_sample_premul(&rgba, 2, 2, 1.4, 0.0).is_some());
        // Strictly outside.
        assert!(bilinear_sample_premul(&rgba, 2, 2, -0.6, 0.0).is_none());
        assert!(bilinear_sample_premul(&rgba, 2, 2, 1.6, 0.0).is_none());
    }
}
