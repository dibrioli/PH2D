//! Integration glue: M6 (asset) + M7 (script) + M12 (editor data layer).
//!
//! Lives in the desktop shell because integration is shell concern,
//! not engine. Each helper here pulls one engine subsystem into the
//! shell's render loop.
//!
//! Out of scope for this PR (defer to follow-up):
//! - **M11 Vello text overlay** — needs Vello sharing the wgpu
//!   Surface with the M5 sprite pipeline (multi-pass coordination).
//! - **M12 widget paint** — same Vello-on-surface dependency.
//! - **M8 ph2d-input wiring into ScriptHost** — gilrs adapter is
//!   already in shells/desktop (M8 PR #13 in cascade); this PR adds
//!   the ScriptHost but doesn't yet route input → ph2d.input snapshot.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_script::ScriptHost;
use std::path::{Path, PathBuf};

pub const ATLAS_GRID: u32 = 4;
pub const ATLAS_SPRITE_PX: u32 = 64;
pub const ATLAS_PX: u32 = ATLAS_GRID * ATLAS_SPRITE_PX; // 256

/// Where the demo PNGs live, relative to the workspace root. The
/// shell creates them on first launch if missing — keeps the
/// repo lean (no committed binary fixtures).
pub fn demo_assets_dir() -> PathBuf {
    let workspace_root = std::env::var("CARGO_MANIFEST_DIR")
        .map(|p| {
            // CARGO_MANIFEST_DIR points at shells/desktop; go up two.
            Path::new(&p)
                .ancestors()
                .nth(2)
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("."))
        })
        .unwrap_or_else(|_| PathBuf::from("."));
    workspace_root.join("assets/sprites")
}

/// Generate the 16-PNG demo fixture in `dir` if any of the canonical
/// files are missing. Each PNG is a 64×64 solid color cycled by
/// golden-angle hue (same algorithm the dummy atlas used, but now
/// actually persisted to disk so M6 has files to consume).
pub fn ensure_demo_assets_exist(dir: &Path) -> std::io::Result<u32> {
    std::fs::create_dir_all(dir)?;
    let mut created = 0u32;
    for i in 0..16u32 {
        let path = dir.join(format!("sprite_{i:02}.png"));
        if path.exists() {
            continue;
        }
        let hue = (i as f32 * 0.618_034) % 1.0;
        let (r, g, b) = hsv_to_rgb(hue, 0.85, 0.95);
        let mut img = image::RgbaImage::new(ATLAS_SPRITE_PX, ATLAS_SPRITE_PX);
        for px in img.pixels_mut() {
            *px = image::Rgba([r, g, b, 255]);
        }
        image::DynamicImage::ImageRgba8(img)
            .save_with_format(&path, image::ImageFormat::Png)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        created += 1;
    }
    Ok(created)
}

/// Load the 16 demo PNGs through `AssetDb::load_png_path`, returning
/// the AssetIds in canonical order (sprite_00..sprite_15). Any
/// failure aborts and propagates — fixture is mandatory for the demo.
pub fn load_demo_assets(asset_db: &AssetDb, dir: &Path) -> Result<Vec<AssetId>, String> {
    let mut ids = Vec::with_capacity(16);
    for i in 0..16u32 {
        let path = dir.join(format!("sprite_{i:02}.png"));
        let id = asset_db
            .load_png_path(&path)
            .map_err(|e| format!("load {}: {e}", path.display()))?;
        ids.push(id);
    }
    Ok(ids)
}

/// Compose a 256×256 RGBA8 atlas from a 4×4 grid of 64×64 sprite
/// images loaded by [`load_demo_assets`]. Layout is row-major to
/// match `TextureAtlas::dummy_uv`.
pub fn compose_atlas_rgba(asset_db: &AssetDb, ids: &[AssetId]) -> Result<Vec<u8>, String> {
    if ids.len() != (ATLAS_GRID * ATLAS_GRID) as usize {
        return Err(format!(
            "compose_atlas_rgba expects {} ids, got {}",
            ATLAS_GRID * ATLAS_GRID,
            ids.len()
        ));
    }
    let mut out = vec![0u8; (ATLAS_PX * ATLAS_PX * 4) as usize];
    for (i, id) in ids.iter().enumerate() {
        let asset = asset_db
            .get(id)
            .ok_or_else(|| format!("asset id missing: {id}"))?;
        let ph2d_asset::Asset::ImageRgba8 {
            width,
            height,
            pixels,
        } = &*asset
        else {
            return Err("only ImageRgba8 supported in atlas compose".into());
        };
        if *width != ATLAS_SPRITE_PX || *height != ATLAS_SPRITE_PX {
            return Err(format!(
                "sprite {i}: expected {ATLAS_SPRITE_PX}×{ATLAS_SPRITE_PX}, got {width}×{height}"
            ));
        }
        let row = (i as u32) / ATLAS_GRID;
        let col = (i as u32) % ATLAS_GRID;
        let dest_x = col * ATLAS_SPRITE_PX;
        let dest_y = row * ATLAS_SPRITE_PX;
        for sy in 0..ATLAS_SPRITE_PX {
            for sx in 0..ATLAS_SPRITE_PX {
                let src_off = ((sy * ATLAS_SPRITE_PX + sx) * 4) as usize;
                let dest_off = (((dest_y + sy) * ATLAS_PX + (dest_x + sx)) * 4) as usize;
                out[dest_off..dest_off + 4].copy_from_slice(&pixels[src_off..src_off + 4]);
            }
        }
    }
    Ok(out)
}

/// Build a ScriptHost with a minimal placeholder script that proves
/// the M7 pipeline is wired. Just prints once on load — full
/// sim-driving via ph2d.set comes when the editor exposes a
/// "scripts" panel (M12+ follow-up).
pub fn init_script_host() -> Result<ScriptHost, String> {
    let mut host = ScriptHost::new().map_err(|e| format!("ScriptHost::new: {e}"))?;
    let placeholder = r#"
        -- Integration demo script (M7 wired into shells/desktop).
        -- Real script-driven gameplay goes here once the editor's
        -- script panel exists (M12+ follow-up).
        print("[lua] ph2d-host-desktop integration demo: ScriptHost OK")
    "#;
    host.load_script(placeholder)
        .map_err(|e| format!("load_script: {e}"))?;
    Ok(host)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let i = (h * 6.0).floor() as i32 % 6;
    let f = h * 6.0 - h * 6.0_f32.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match i {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}
