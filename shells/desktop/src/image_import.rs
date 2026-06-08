//! Full pipeline for importing image files into the running demo
//! (drag-drop / file-dialog path).
//!
//! Self-contained orchestrator — bytes → AssetDb decode → SpriteRenderer
//! atlas pack → SimWorld spawn `(Transform, Sprite, Name)`. A batch of
//! files is laid out in a tidy near-square grid (see
//! [`import_images_grid`]) so a multi-file drop spreads out instead of
//! stacking every sprite on one point.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::{Sprite, SpriteRenderer};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// Re-export SimWorld type alias used in the signatures (defined in the
// main module). Avoids dragging the heavier bevy_ecs::World import
// surface into this file.
use crate::SimWorld;

/// World-space gap between adjacent grid cells in a multi-image import,
/// expressed as a fraction of the largest sprite's max dimension. Keeps
/// the spacing proportional to the imported sprites' scale so a grid of
/// 4K photos and a grid of 64px icons both read as a tidy grid rather
/// than "touching" or "lost in whitespace". Not a UI token (HR-15) —
/// this is world-space scene layout, same class as `MIN_SPRITE_SIZE`.
const IMPORT_GRID_GAP_FRAC: f32 = 0.08;

/// Outcome of importing one file in a batch, returned in input order so
/// the host can surface a per-file toast and seat the selection.
pub enum ImportItemResult {
    /// File decoded, packed and spawned. `bits` is the sim-entity bits
    /// (so the host can select it); `label` is the assigned `Name`.
    Ok { label: String, bits: u64 },
    /// File failed before spawning. `name` is the file name (for the
    /// toast); `error` is the human-readable cause.
    Err { name: String, error: String },
}

/// Decode + atlas-pack one file WITHOUT spawning, returning its
/// world-space size `[w, h]`. Split out from the spawn step so a batch
/// can be measured first (for grid layout) and spawned second.
///
/// 1. Read bytes from disk.
/// 2. `AssetDb::insert_image_bytes` (auto-detects PNG/WEBP/JPEG,
///    hashes blake3 per HR-6).
/// 3. `SpriteRenderer::insert_atlas_sprite_with_regrow` packs the
///    native-resolution RGBA into the atlas via the Skyline rect packer
///    — no resize, no aspect-ratio squash. The regrow path lets a 2nd /
///    3rd 4K import double the atlas instead of failing with AtlasFull;
///    the closure recovers each existing region's source bytes from
///    `asset_db` via `atlas_asset_map[key] → AssetId`.
fn pack_image(
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    path: &Path,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Result<[f32; 2], String> {
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
    // Track this import's mapping BEFORE the insert so a regrow
    // triggered by it sees the new key.
    atlas_asset_map.insert(cell_idx, asset_id);
    let fetch_pixels = |key: u32| -> Option<Vec<u8>> {
        let aid = atlas_asset_map.get(&key)?;
        let asset = asset_db.get(aid)?;
        match &*asset {
            // `pixels` is `Arc<[u8]>`; the regrow callback wants an
            // owned `Vec<u8>` because the underlying packer may outlive
            // the asset borrow.
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
    // Sprite world size = source pixels / pixels_per_meter. With the
    // Skyline atlas the source bytes are stored at full resolution, so
    // the world quad's aspect ratio matches the file exactly (a 256×128
    // PNG renders as a 2:1 rect in world space).
    let safe_px_per_m = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let world_w = (width as f32 / safe_px_per_m).max(crate::MIN_SPRITE_SIZE);
    let world_h = (height as f32 / safe_px_per_m).max(crate::MIN_SPRITE_SIZE);
    Ok([world_w, world_h])
}

/// Human-readable base name for an imported file, falling back to a
/// cell-indexed stub for paths with no usable stem.
fn base_label(path: &Path, cell_idx: u32) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_owned())
        .unwrap_or_else(|| format!("imported_{cell_idx}"))
}

/// Spawn one already-packed sprite at `world_center`. The unique-name
/// bump runs HERE (not at pack time) so batch siblings sharing a file
/// stem see each other already in the world and get distinct " (1)" /
/// " (2)" labels — same convention as `HierDuplicate` / `AddChild` /
/// rename (mirrors the duplicate-`Name` bug fixed 2026-05-27).
fn spawn_sprite(
    sim: &mut SimWorld,
    cell_idx: u32,
    world_center: Vec2,
    world_size: [f32; 2],
    base: &str,
) -> (String, u64) {
    let label = crate::name_unique::unique_name(sim, base);
    let entity = sim
        .world_mut()
        .spawn((
            Transform::from_translation(world_center),
            Sprite::atlas(cell_idx, world_size, [1.0, 1.0, 1.0, 1.0]),
            Name::new(label.clone()),
        ))
        .id();
    (label, entity.to_bits())
}

/// One packed-but-not-yet-spawned image, carried from the measure pass
/// into the spawn pass.
struct Packed {
    /// Index into the caller's `paths` slice — used to restore input
    /// order in the returned results (errors are interleaved).
    input_index: usize,
    cell_idx: u32,
    world_size: [f32; 2],
    base_label: String,
}

/// Imports a batch of image files, laying them out in a tidy near-square
/// grid and returning per-file outcomes in input order.
///
/// **Layout.** `cols = ceil(sqrt(N))` (so the footprint stays as close
/// to a square as a row-major grid allows), and cells are *uniform* —
/// pitch = largest imported sprite + a proportional gap — so rows and
/// columns line up even when the files differ in size (each sprite is
/// centered in its cell). The first cell's CENTER sits on
/// `anchor_world`, and the grid grows right (`+x`) and down (`-y`) —
/// "ao lado e abaixo". `N == 1` reduces to a single sprite centered on
/// the anchor, identical to the pre-grid single-import behavior.
///
/// **Two passes.** Files are decoded + atlas-packed first (to measure
/// every world size before placing anything), then spawned at their
/// computed cells. A file that fails to pack is recorded as
/// [`ImportItemResult::Err`] in place and dropped from the layout, so a
/// bad file never leaves a hole in the grid.
///
/// `next_cell` is advanced once per *successful* pack (errors don't
/// consume an atlas cell).
// Helper has 8 args because the import path is a top-level fixture
// orchestrator that needs full access to sim/renderer/asset_db plus the
// batch inputs. Splitting into a struct would just move the noise.
#[allow(clippy::too_many_arguments)]
pub fn import_images_grid(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    anchor_world: [f32; 2],
    next_cell: &mut u32,
    paths: &[PathBuf],
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Vec<ImportItemResult> {
    // Pass 1 — measure. Decode + atlas-pack each file; successful packs
    // feed the layout, failures are slotted straight into the results.
    let mut packed: Vec<Packed> = Vec::new();
    let mut results: Vec<Option<ImportItemResult>> = (0..paths.len()).map(|_| None).collect();
    for (i, path) in paths.iter().enumerate() {
        let cell_idx = *next_cell;
        match pack_image(
            renderer,
            asset_db,
            cell_idx,
            path,
            pixels_per_meter,
            atlas_asset_map,
        ) {
            Ok(world_size) => {
                *next_cell = next_cell.saturating_add(1);
                packed.push(Packed {
                    input_index: i,
                    cell_idx,
                    world_size,
                    base_label: base_label(path, cell_idx),
                });
            }
            Err(error) => {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("(unnamed)")
                    .to_owned();
                results[i] = Some(ImportItemResult::Err { name, error });
            }
        }
    }

    // Pass 2 — lay out + spawn.
    let n = packed.len();
    if n > 0 {
        let cols = (n as f32).sqrt().ceil().max(1.0) as usize;
        let max_w = packed
            .iter()
            .map(|p| p.world_size[0])
            .fold(0.0_f32, f32::max);
        let max_h = packed
            .iter()
            .map(|p| p.world_size[1])
            .fold(0.0_f32, f32::max);
        let gap = IMPORT_GRID_GAP_FRAC * max_w.max(max_h);
        let pitch_x = max_w + gap;
        let pitch_y = max_h + gap;
        for (k, p) in packed.iter().enumerate() {
            let col = (k % cols) as f32;
            let row = (k / cols) as f32;
            // y grows up, so successive rows step DOWN (−y).
            let center = Vec2::new(
                anchor_world[0] + col * pitch_x,
                anchor_world[1] - row * pitch_y,
            );
            let (label, bits) = spawn_sprite(sim, p.cell_idx, center, p.world_size, &p.base_label);
            results[p.input_index] = Some(ImportItemResult::Ok { label, bits });
        }
    }

    // Every index was filled (Err in pass 1 or Ok in pass 2); `flatten`
    // drops the structurally-impossible `None` and preserves order.
    results.into_iter().flatten().collect()
}
