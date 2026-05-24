//! Drain one `OneShotImageOp { tool_id: "color_equalization" }` per
//! sprite — the cross-sprite list iterates in the caller.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::SpriteRenderer;

use crate::ImageEditSnapshot;
use crate::hero_intents::texture_edit;

/// Drain one `EditorAction::OneShotImageOp { tool_id:
/// "color_equalization", entity_bits }`: push the sprite's source
/// RGBA into the active `ColorEqualizationTool` and run the full-res
/// pipeline (CLAHE + brightness/contrast/saturation + auto-WB). The
/// caller iterates a `Vec<u64>` from `color_equalization_bridge` (one
/// drain per selected sprite) so multi-select Apply works.
///
/// Same shape as `super::bgremoval::drain_bgremoval` but with PRESERVE-
/// alpha output: Color EQ only edits the RGB channels, never crops or
/// scales, so dimensions match the source and the alpha mode round-trips
/// straight → straight.
///
/// Returns `true` if a toast was pushed (caller marks title dirty).
#[allow(clippy::too_many_arguments)]
pub(crate) fn drain_color_equalization(
    entity_bits: u64,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    pending_undo_entries: &mut Vec<ImageEditSnapshot>,
    ceq: &mut ph2d_tool_color_equalization::ColorEqualizationTool,
) -> bool {
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let Some(src) =
        texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
    else {
        toasts.push(Toast::error(
            "Color EQ: source unavailable (Atlas key missing or readback failed)",
        ));
        return true;
    };
    let old_size_world = src.old_size_world;
    let old_source = src.old_source;
    let old_translation = src.old_translation;
    let old_premultiplied = src.old_premultiplied;
    let old_anchor = src.old_anchor;
    // Color EQ reasons about TRUE colors in straight-alpha sRGB →
    // un-premultiply if the sprite was baked premultiplied (mirror of
    // bgremoval). Dimensions are preserved, alpha is preserved.
    let straight = src.image.into_straight();
    let (w, h) = (straight.width, straight.height);
    let mut out: Vec<u8> = Vec::new();
    // Wave 11 migration (ADR-0042 §6 #2): typed `Vec<SrgbRgba>` input.
    ceq.set_source_snapshot(bytemuck::allocation::cast_vec(straight.pixels), w, h);
    let (out_w, out_h) = ceq.run_full_resolution(&mut out);
    debug_assert_eq!((out_w, out_h), (w, h), "Color EQ must preserve dimensions");
    // Keep the OUTPUT in straight-alpha — Color EQ leaves alpha
    // untouched, so we round-trip exactly. The chokepoint flips
    // `Sprite.premultiplied` to match.
    let edited =
        ph2d_render::SpriteImage::from_bytes(out_w, out_h, out, ph2d_render::AlphaMode::Straight);
    match texture_edit::commit_edited_texture(entity, sim, renderer, &edited, old_size_world) {
        Err(err) => {
            toasts.push(Toast::error(format!("Color EQ failed: {err}")));
            true
        }
        Ok(texture_id) => {
            pending_undo_entries.push(ImageEditSnapshot {
                entity_bits,
                pre_source: old_source,
                pre_size: old_size_world,
                pre_translation: old_translation,
                pre_premultiplied: old_premultiplied,
                pre_anchor: old_anchor,
                post_individual_id: texture_id,
                label: "Color EQ",
            });
            toasts.push(Toast::success("Color EQ applied · Cmd+Z to undo"));
            true
        }
    }
}
