//! M6 atlas composition from real PNG files.
//!
//! Wave 3.1 stage B — extracted from `main.rs::App::try_load_real_atlas`
//! as a free function under the shell. Behavior-preserving lift.
//!
//! Composes the demo atlas by walking 16 PNG fixtures under
//! `assets/sprites/`. Generates the fixtures on first launch
//! (idempotent); subsequent launches reuse them. Any failure bubbles
//! a `String` — caller falls back to the dummy procedural atlas.

use ph2d_asset::AssetDb;
use ph2d_gpu::GpuContext;
use ph2d_render::TextureAtlas;

use crate::integration;

/// Compose the M6 atlas from real PNG files. Generates fixtures on
/// first launch; subsequent launches reuse the on-disk files. Any
/// failure bubbles a `String` — caller falls back to the dummy atlas.
pub(crate) fn load_atlas(
    gpu: &GpuContext,
    asset_db: &AssetDb,
    dir: &std::path::Path,
) -> Result<TextureAtlas, String> {
    let created = integration::ensure_demo_assets_exist(dir)
        .map_err(|e| format!("ensure_demo_assets_exist({}): {e}", dir.display()))?;
    if created > 0 {
        println!(
            "M6: generated {created} demo PNG fixtures in {}",
            dir.display()
        );
    }
    let ids = integration::load_demo_assets(asset_db, dir)?;
    let composed = integration::compose_atlas_rgba(asset_db, &ids)?;
    // M14.4d retrofit: the demo atlas used to be a single 256×256
    // texture mirroring the `compose_atlas_rgba` layout. With the
    // Skyline packer it's a dynamic atlas seeded by 16 inserts of
    // the per-tile slices — keys 0..16 reproduce the same
    // (col, row) ordering the dummy HSV path uses, so the rest of
    // the demo (which addresses tiles by `i % 16`) needs no
    // change.
    let mut atlas = TextureAtlas::new(gpu, ph2d_render::ATLAS_DEFAULT_SIZE_PX);
    let tile_px = ph2d_render::DEMO_TILE_PX;
    let composed_px = integration::ATLAS_PX;
    for i in 0..ph2d_render::DEMO_TILE_COUNT {
        let col = i % integration::ATLAS_GRID;
        let row = i / integration::ATLAS_GRID;
        let mut tile = Vec::with_capacity((tile_px * tile_px * 4) as usize);
        for ty in 0..tile_px {
            let src_y = row * tile_px + ty;
            let src_x = col * tile_px;
            let row_start = ((src_y * composed_px + src_x) * 4) as usize;
            let row_end = row_start + (tile_px * 4) as usize;
            tile.extend_from_slice(&composed[row_start..row_end]);
        }
        atlas
            .insert(gpu, i, tile_px, tile_px, &tile)
            .map_err(|e| format!("demo tile {i}: {e}"))?;
    }
    Ok(atlas)
}
