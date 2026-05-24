//! Drain Equalize Sizes Apply over the multi-selection — cross-sprite
//! bake + optional Arrange-on-Grid layout.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::SpriteRenderer;

use crate::hero_intents::texture_edit;
use crate::{EPS_PIXELS_PER_METER, ImageEditSnapshot};

/// Per-entity gather record for `drain_equalize_sizes`: the source-read
/// metadata (`old_*` + alpha mode) paired with the entity_bits so the
/// commit phase can rebuild the `SpriteImage`, restore the undo
/// snapshot, and (in `compute_arrange_positions`) look up the current
/// world translation by entity. Lifted out of the drain so it can be
/// the parameter type for the arrange helper.
struct EqsEntry {
    bits: u64,
    src: texture_edit::SourceRead,
    input_alpha: ph2d_render::AlphaMode,
}

/// Compute the per-entity arrange-on-grid layout positions (port of
/// the legacy `EqualizeModal.arrangeSprites` from
/// `Game-Engine-Legada/apps/editor/src/tools/EqualizeModal.ts:296`):
/// sort the selection by world `(y, x)`, lay them out in `cols ×
/// rows` (cols = `ceil(sqrt(N))`), anchored at the selection's
/// top-left snapped down to the grid origin. Cell center is at
/// `origin + (col + 0.5) * cell` (X) / `origin - (row + 0.5) * cell`
/// (Y; Y grows UP in PH2D world space).
fn compute_arrange_positions(
    entries: &[EqsEntry],
    sim: &SimWorld,
    cell_m: f32,
    grid_origin: [f32; 2],
) -> Vec<Option<[f32; 2]>> {
    let n = entries.len();
    if n == 0 || cell_m <= 0.0 {
        return vec![None; n];
    }
    let mut posed: Vec<(usize, [f32; 2])> = Vec::with_capacity(n);
    for (i, e) in entries.iter().enumerate() {
        let entity = ph2d_ecs::Entity::from_bits(e.bits);
        if let Some(t) = sim.world().get::<ph2d_ecs::Transform>(entity) {
            posed.push((i, [t.translation.x, t.translation.y]));
        }
    }
    // Sort by (y desc, x asc): the legacy modal used
    // `a.y - b.y || a.x - b.x` in DOM coords (Y grows DOWN). PH2D Y
    // grows UP, so reverse the Y sort to keep the same visual reading
    // order (top → bottom, left → right).
    posed.sort_by(|a, b| {
        b.1[1]
            .partial_cmp(&a.1[1])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.1[0]
                    .partial_cmp(&b.1[0])
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    let min_x = posed.iter().map(|p| p.1[0]).fold(f32::INFINITY, f32::min);
    let max_y = posed
        .iter()
        .map(|p| p.1[1])
        .fold(f32::NEG_INFINITY, f32::max);
    // Snap the layout's top-left corner DOWN to the grid origin lattice
    // (so cells align with the live Grid Snap overlay).
    let start_x = ((min_x - grid_origin[0]) / cell_m).floor() * cell_m + grid_origin[0];
    let start_y_top = ((max_y - grid_origin[1]) / cell_m).ceil() * cell_m + grid_origin[1];
    let cols = (posed.len() as f32).sqrt().ceil().max(1.0) as usize;

    let mut result: Vec<Option<[f32; 2]>> = vec![None; n];
    for (i, (entry_idx, _)) in posed.iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let cx = start_x + (col as f32 + 0.5) * cell_m;
        let cy = start_y_top - (row as f32 + 0.5) * cell_m;
        result[*entry_idx] = Some([cx, cy]);
    }
    result
}

/// Drain the Equalize Sizes Apply latch over the multi-selection.
///
/// Cross-sprite shape: collects per-entity `SpriteInput`s
/// (`{ rgba, w, h, scale_x, scale_y }`) from `bits_list`, runs
/// [`ph2d_tool_equalize_sizes::EqualizeSizesTool::run_full_resolution_multi`]
/// in one batch (Max mode needs the global max, so per-sprite broadcast
/// doesn't work), then iterates the outputs back into each entity.
///
/// When `arrange_on_grid && target_mode == GridUnit`, AFTER the size
/// bake we lay sprites 1-per-cell sorted by world `(y, x)`, anchored
/// at the selection's top-left snapped to the grid origin.
///
/// Single-level undo captures the LAST successful sprite (consistent
/// with `super::color_equalization::drain_color_equalization`).
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_equalize_sizes(
    bits_list: &[u64],
    project_pixels_per_meter: f32,
    grid_origin: [f32; 2],
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    pending_undo_entries: &mut Vec<ImageEditSnapshot>,
    eqs: &mut ph2d_tool_equalize_sizes::EqualizeSizesTool,
) -> bool {
    if bits_list.is_empty() {
        return false;
    }
    let px_per_m = project_pixels_per_meter.max(EPS_PIXELS_PER_METER);
    let max_dim = renderer.max_texture_dimension_2d();

    // Phase 1: gather per-entity inputs in the original `bits_list`
    // order. We pair each `SpriteInput` with the source-read metadata
    // (`old_*` + alpha mode) so the commit phase can rebuild the
    // `SpriteImage` + restore the undo snapshot without re-reading.
    let mut entries: Vec<EqsEntry> = Vec::with_capacity(bits_list.len());
    let mut inputs: Vec<ph2d_tool_equalize_sizes::SpriteInput> =
        Vec::with_capacity(bits_list.len());
    let mut skipped = 0usize;
    for &bits in bits_list {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let Some((scale_x, scale_y)) = sim
            .world()
            .get::<ph2d_ecs::Transform>(entity)
            .map(|t| (t.scale.x, t.scale.y))
        else {
            skipped += 1;
            continue;
        };
        let Some(src) =
            texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
        else {
            skipped += 1;
            continue;
        };
        // Convert to straight-alpha for the resample kernels: Lanczos3 /
        // Mitchell-Netravali interpolate RGBA per-channel; resampling a
        // premultiplied buffer would attenuate edges where alpha drops.
        // We round-trip back to the source alpha mode at commit (below).
        let source_alpha = src.image.alpha;
        let straight = src.image.into_straight();
        let (w, h) = (straight.width, straight.height);
        let input = ph2d_tool_equalize_sizes::SpriteInput {
            rgba: straight.pixels,
            width: w,
            height: h,
            scale_x,
            scale_y,
        };
        // Re-wrap the straight buffer back into a `SourceRead` shell so
        // the commit phase still has the `old_*` fields — but we keep a
        // separate handle to the original alpha mode (the round-trip
        // target) since `src.image` was moved.
        let src_shell = texture_edit::SourceRead {
            image: ph2d_render::SpriteImage::from_bytes(
                w,
                h,
                Vec::new(),
                ph2d_render::AlphaMode::Straight,
            ),
            old_size_world: src.old_size_world,
            old_translation: src.old_translation,
            old_source: src.old_source,
            old_premultiplied: src.old_premultiplied,
            old_anchor: src.old_anchor,
        };
        entries.push(EqsEntry {
            bits,
            src: src_shell,
            input_alpha: source_alpha,
        });
        inputs.push(input);
    }
    if entries.is_empty() {
        toasts.push(Toast::error(
            "Equalize Sizes: no eligible sprites in selection",
        ));
        return true;
    }

    // Phase 2: cross-sprite bake.
    let params_snapshot = *eqs.params();
    let outputs = eqs.run_full_resolution_multi(&inputs);
    debug_assert_eq!(outputs.len(), entries.len());

    // Pre-compute arrange-on-grid layout (legacy `EqualizeModal`
    // semantics): when `arrange_on_grid && target_mode == GridUnit`,
    // lay sprites 1-per-cell sorted by world `(y, x)`, anchored at the
    // selection's top-left snapped down to the grid origin.
    let arrange_positions = params_snapshot.arrange_on_grid
        && params_snapshot.target_mode == ph2d_tool_equalize_sizes::TargetMode::GridUnit;
    let cell_m = (params_snapshot.grid_unit as f32 / px_per_m).max(1.0e-3);
    let new_translations: Vec<Option<[f32; 2]>> = if arrange_positions {
        compute_arrange_positions(&entries, sim, cell_m, grid_origin)
    } else {
        vec![None; entries.len()]
    };

    // Phase 3: commit per-entity. Each successful sprite pushes an
    // entry into the caller's `pending_undo_entries` so the cross-sprite
    // Apply restores ALL sprites on Cmd+Z (the old single-slot design
    // only restored the last one — §1 audit CRITICAL data-loss bug).
    let mut applied = 0usize;
    for ((entry, out), maybe_pos) in entries.into_iter().zip(outputs).zip(new_translations) {
        // Skip only when the bake produced NO change AND there's no
        // arrange-on-grid layout move to apply — otherwise the sprite
        // still needs its translation rewritten.
        if !out.changed && maybe_pos.is_none() {
            continue;
        }
        let entity = ph2d_ecs::Entity::from_bits(entry.bits);
        let buffer_changed = out.changed
            && (out.width, out.height) != (entry.src.image.width, entry.src.image.height);
        let mut produced_individual: Option<u32> = None;
        if buffer_changed {
            if out.width > max_dim || out.height > max_dim {
                toasts.push(Toast::error(format!(
                    "Equalize Sizes: target exceeds GPU texture limit ({} px max, would need {} × {} px)",
                    max_dim, out.width, out.height
                )));
                continue;
            }
            // Round-trip back to source alpha mode at the chokepoint —
            // `commit_edited_texture` derives `Sprite.premultiplied`
            // from `img.alpha`, so we hand the straight-from-resample
            // buffer in straight if the source was straight, or
            // re-premultiplied otherwise.
            let edited_straight = ph2d_render::SpriteImage::from_bytes(
                out.width,
                out.height,
                out.rgba,
                ph2d_render::AlphaMode::Straight,
            );
            let edited = if entry.input_alpha.is_premultiplied() {
                edited_straight.into_premultiplied()
            } else {
                edited_straight
            };
            let new_size_world = [out.width as f32 / px_per_m, out.height as f32 / px_per_m];
            match texture_edit::commit_edited_texture(
                entity,
                sim,
                renderer,
                &edited,
                new_size_world,
            ) {
                Err(err) => {
                    toasts.push(Toast::error(format!("Equalize Sizes failed: {err}")));
                    continue;
                }
                Ok(texture_id) => {
                    produced_individual = Some(texture_id);
                }
            }
        }
        if let Some(mut t) = sim.world_mut().get_mut::<ph2d_ecs::Transform>(entity) {
            if out.changed {
                t.scale.x = out.new_scale_x;
                t.scale.y = out.new_scale_y;
            }
            if let Some([nx, ny]) = maybe_pos {
                t.translation.x = nx;
                t.translation.y = ny;
            }
        }
        applied += 1;
        // Append THIS sprite to the transaction. `produced_individual`
        // None ⇒ fit-by-scale path (no texture acquired); we record `0`
        // as the sentinel the undo drainer already tolerates.
        pending_undo_entries.push(ImageEditSnapshot {
            entity_bits: entry.bits,
            pre_source: entry.src.old_source,
            pre_size: entry.src.old_size_world,
            pre_translation: entry.src.old_translation,
            pre_premultiplied: entry.src.old_premultiplied,
            pre_anchor: entry.src.old_anchor,
            post_individual_id: produced_individual.unwrap_or(0),
            label: "Equalize Sizes",
        });
    }

    if applied == 0 {
        toasts.push(Toast::info("Equalize Sizes: nothing to change"));
        return true;
    }

    if skipped > 0 {
        toasts.push(Toast::success(format!(
            "Equalize Sizes · {} sprite(s), {} skipped · Cmd+Z to undo",
            applied, skipped
        )));
    } else if applied == 1 {
        toasts.push(Toast::success("Equalize Sizes applied · Cmd+Z to undo"));
    } else {
        toasts.push(Toast::success(format!(
            "Equalize Sizes · {} sprites · Cmd+Z to undo",
            applied
        )));
    }
    true
}
