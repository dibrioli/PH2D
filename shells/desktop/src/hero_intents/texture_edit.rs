//! Image-tool texture chokepoint — the SINGLE sanctioned path for
//! reading a sprite's source pixels and committing an edited result.
//!
//! Every Image Tool (Trim, Make Square, BG-Removal, and the many to come)
//! reads a sprite's texture, transforms the pixels, and uploads a new one.
//! The alpha REPRESENTATION (straight vs premultiplied) must survive that
//! round-trip: a bug where Trim / Make-Square un-premultiplied a
//! BG-Removal result and silently dropped the `premultiplied` flag
//! re-introduced the edge fringe. Routing every tool through
//! [`read_sprite_source`] (pixels are a [`SpriteImage`] that CARRIES its
//! [`AlphaMode`]) and [`commit_edited_texture`] (writes
//! `Sprite.premultiplied` FROM the result's `AlphaMode`, never a
//! hand-typed bool) makes that class of bug impossible.
//!
//! An arch gate (`tests/texture_edit_chokepoint.rs`) forbids
//! `readback_individual` anywhere else in the shell, so a future tool
//! cannot obtain individual-texture pixels without the mode-carrying
//! wrapper.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_render::{AlphaMode, Sprite, SpriteImage, SpriteRenderer, SpriteSource};

/// A sprite's source pixels (native alpha mode) plus the pre-edit
/// transform metadata image tools need for undo + recenter.
pub(crate) struct SourceRead {
    pub image: SpriteImage,
    pub old_size_world: [f32; 2],
    pub old_translation: [f32; 2],
    pub old_source: SpriteSource,
    pub old_premultiplied: bool,
    /// `Sprite.anchor` (intrinsic-local pivot offset) before the edit.
    /// Padding's Keep mode rebases it so content + pivot stay world-
    /// fixed; undo restores it. `[0,0]` for any sprite the TOOL_PIVOT
    /// tool hasn't moved the pivot on.
    pub old_anchor: [f32; 2],
}

/// Read `entity`'s sprite source as a [`SpriteImage`] carrying its native
/// [`AlphaMode`] — Atlas resolves to the straight asset pixels; Individual
/// reads the texture back, premultiplied iff `Sprite.premultiplied`.
/// `None` if the sprite is missing, the atlas key/asset is absent, or the
/// readback fails.
///
/// THE single sanctioned `readback_individual` call site (arch-gated).
pub(crate) fn read_sprite_source(
    entity: Entity,
    sim: &SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
) -> Option<SourceRead> {
    let world = sim.world();
    let sprite = world.get::<Sprite>(entity)?;
    let old_size_world = sprite.size;
    let old_source = sprite.source;
    let old_premultiplied = sprite.premultiplied;
    let old_anchor = sprite.anchor;
    let old_translation = world
        .get::<Transform>(entity)
        .map(|t| [t.translation.x, t.translation.y])
        .unwrap_or([0.0, 0.0]);
    let image = match sprite.source {
        SpriteSource::Atlas { key } => {
            let aid = atlas_asset_map.get(&key)?;
            let asset = asset_db.get(aid)?;
            match &*asset {
                ph2d_asset::Asset::ImageRgba8 {
                    width,
                    height,
                    pixels,
                } => SpriteImage::from_bytes(*width, *height, pixels.to_vec(), AlphaMode::Straight),
                _ => return None,
            }
        }
        SpriteSource::Individual { texture_id } => {
            let (w, h, pixels) = renderer.readback_individual(texture_id).ok()?;
            SpriteImage::from_bytes(
                w,
                h,
                pixels,
                AlphaMode::from_premultiplied_flag(old_premultiplied),
            )
        }
        // W2.T2: a tier-cooked KTX2 texture is GPU-compressed (BC/ASTC/
        // ETC2) with no CPU-side RGBA — the raster image tools (Trim,
        // Make Square, Bg Removal) can't read it back to edit. Unsupported.
        SpriteSource::CookedTexture { .. } => return None,
    };
    Some(SourceRead {
        image,
        old_size_world,
        old_translation,
        old_source,
        old_premultiplied,
        old_anchor,
    })
}

/// Upload `img` as a fresh Individual texture and repoint `entity`'s
/// Sprite at it: `source` + `new_size_world` + `premultiplied` derived
/// from `img.alpha`. Returns the new texture id.
///
/// THE single place an image-tool result writes `Sprite.premultiplied`
/// — derived from the bytes' actual [`AlphaMode`], so the flag can never
/// drift from the representation (the original Trim/Make-Square bug).
///
/// ## It is also where the pixels get a DURABLE NAME
///
/// `texture_id` is a GPU allocation id and the store restarts numbering at `1` every process, so
/// a sprite that only carried it came back **invisible** — or showing another sprite's pixels —
/// after save/load. The same bytes therefore also go into the [`AssetDb`] (blake3 content hash,
/// HR-6) and the entity is stamped with [`ph2d_ecs::SpritePixels`], which travels in the
/// `WorldSnapshot` and is what `project_sprite_pixels` writes to (and restores from) the file.
///
/// Doing it HERE is the whole point: an arch gate pins this function as the single door every
/// image tool leaves through, so eight tools are covered by one stamp rather than eight.
pub(crate) fn commit_edited_texture(
    entity: Entity,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    img: &SpriteImage,
    new_size_world: [f32; 2],
) -> Result<u32, String> {
    let texture_id = renderer
        .acquire_individual(img.width, img.height, &img.pixels)
        .map_err(|e| e.to_string())?;
    // The durable name of these exact bytes. `insert_image_rgba8` takes `&self` (interior
    // mutability) and hashes dims + content, so re-committing an identical edit is idempotent
    // and two sprites that end up with the same pixels share one entry in the project file.
    let pixels_id = asset_db.insert_image_rgba8(img.width, img.height, img.pixels.clone());
    if let Some(mut sprite) = sim.world_mut().get_mut::<Sprite>(entity) {
        sprite.source = SpriteSource::Individual { texture_id };
        sprite.size = new_size_world;
        sprite.premultiplied = img.alpha.is_premultiplied();
    }
    // Stamped AFTER the sprite write and unconditionally: an entity whose `Sprite` vanished
    // mid-frame gets no stamp because `insert` on a dead entity is the caller's bug, so guard on
    // the same lookup the write used.
    if sim.world().get::<Sprite>(entity).is_some() {
        sim.world_mut()
            .entity_mut(entity)
            .insert(ph2d_ecs::SpritePixels(pixels_id));
    }
    Ok(texture_id)
}
