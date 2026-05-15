//! Full pipeline for importing one image file into the running demo
//! (drag-drop / file-dialog path).
//!
//! Extracted from [`main`] (Track B5). Self-contained orchestrator
//! — bytes → AssetDb decode → SpriteRenderer atlas pack → SimWorld
//! spawn `(Transform, Sprite, Name)`.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::{Camera2d, Sprite, SpriteRenderer};
use std::collections::BTreeMap;

// Re-export SimWorld type alias used in the signature (defined in the
// main module). Avoids dragging the heavier bevy_ecs::World import
// surface into this file.
use crate::SimWorld;

/// M14.4c+M14.4d retrofit: full pipeline for importing one image
/// file into the running demo.
///
/// 1. Read bytes from disk.
/// 2. `AssetDb::insert_image_bytes` (auto-detects PNG/WEBP/JPEG,
///    hashes blake3 per HR-6).
/// 3. `SpriteRenderer::insert_atlas_sprite` packs the native-
///    resolution RGBA into the atlas via the Skyline rect packer
///    — no resize, no aspect-ratio squash.
/// 4. Spawn `(Transform at camera center, Sprite { atlas_index:
///    cell_idx, size: source_pixels / pixels_per_meter }, Name)`.
///
/// Returns the human-readable label that was assigned to the
/// spawned entity (so the host can show it in a toast).
// Helper has 8 args because the import path is a top-level fixture
// orchestrator that needs full access to sim/renderer/asset_db plus
// the camera + per-import inputs. Splitting into a struct would just
// move the noise — every call site already destructures `AppGfx`.
#[allow(clippy::too_many_arguments)]
pub fn import_image_at_camera(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    camera: &Camera2d,
    cell_idx: u32,
    path: &std::path::Path,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let asset_id = asset_db
        .insert_image_bytes(&bytes)
        .map_err(|e| format!("decode {}: {e}", path.display()))?;
    let decoded = asset_db
        .get(&asset_id)
        .ok_or_else(|| format!("asset {asset_id} missing after insert"))?;
    let (width, height, pixels) = match &*decoded {
        ph2d_asset::Asset::ImageRgba8 {
            width,
            height,
            pixels,
        } => (*width, *height, pixels.clone()),
        _ => return Err(format!("{asset_id} is not ImageRgba8 after decode")),
    };
    // M14.7 polish: pack via the regrow path so a 2nd / 3rd 4K
    // import doubles the atlas instead of failing with AtlasFull.
    // The closure recovers each existing region's source bytes from
    // `asset_db` via `atlas_asset_map[key] → AssetId`. Track this
    // import's mapping BEFORE the insert so a regrow triggered by
    // it sees the new key.
    atlas_asset_map.insert(cell_idx, asset_id);
    let fetch_pixels = |key: u32| -> Option<Vec<u8>> {
        let aid = atlas_asset_map.get(&key)?;
        let asset = asset_db.get(aid)?;
        match &*asset {
            // `pixels` is `Arc<[u8]>`; the regrow callback wants an
            // owned `Vec<u8>` because the underlying packer may
            // outlive the asset borrow.
            ph2d_asset::Asset::ImageRgba8 { pixels, .. } => Some(pixels.to_vec()),
            _ => None,
        }
    };
    if let Err(e) =
        renderer.insert_atlas_sprite_with_regrow(cell_idx, width, height, &pixels, fetch_pixels)
    {
        // On failure, roll back the map insert so the next import
        // doesn't see a dangling key → nonexistent atlas region.
        atlas_asset_map.remove(&cell_idx);
        return Err(format!("atlas insert {}: {e}", path.display()));
    }

    let base_label = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| format!("imported_{cell_idx}"));
    // M14.7 polish: enforce unique entity names. Without this, the
    // same image imported N times produced N rows all sharing the
    // same `Name`, and the M14.6 D selection-by-label sync would
    // highlight every row when one of the duplicates was canvas-
    // picked. We scan the SimWorld for an existing match and append
    // `_{cell_idx}` (monotonic, never repeats) on collision.
    let label = {
        let world = sim.world_mut();
        let mut q = world.query::<&Name>();
        let collides = q
            .iter(world)
            .any(|n: &Name| n.as_str() == base_label.as_str());
        if collides {
            format!("{base_label}_{cell_idx}")
        } else {
            base_label
        }
    };
    let spawn_pos = Vec2::new(camera.center[0], camera.center[1]);
    // Sprite world size = source pixels / pixels_per_meter. With the
    // Skyline atlas the source bytes are stored at full resolution,
    // so the world quad's aspect ratio matches the file exactly
    // (a 256×128 PNG renders as a 2:1 rect in world space).
    let safe_px_per_m = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let world_w = (width as f32 / safe_px_per_m).max(crate::MIN_SPRITE_SIZE);
    let world_h = (height as f32 / safe_px_per_m).max(crate::MIN_SPRITE_SIZE);
    sim.world_mut().spawn((
        Transform::from_translation(spawn_pos),
        Sprite::atlas(cell_idx, [world_w, world_h], [1.0, 1.0, 1.0, 1.0]),
        Name::new(label.clone()),
    ));
    Ok(label)
}
