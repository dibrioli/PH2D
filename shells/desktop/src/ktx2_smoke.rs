//! `PH2D_KTX2_SMOKE` — KTX2 Fase 2 / W2.T4 end-to-end cooked-texture smoke.
//!
//! When the env var is set, this cooks a recognizable RGBA8 image into a
//! `Constrained`-tier KTX2 **in memory** (no ISPC, no committed `.ktx2`
//! fixture), registers it in the `AssetDb` + `LogicalTextureMap`, and spawns a
//! `SpriteSource::CookedTexture` sprite at the canvas origin. The full loader
//! path then runs every frame:
//!
//! `cooked_texture_bridge` (resolve logical+tier → blob → upload) → extract
//! (stamp the cooked `texture_id`) → renderer (bind the cooked store).
//!
//! Because the artifact is the universal RGBA8 `Constrained` tier, the device
//! fallback ladder descends to it on **every** GPU (BC desktop / ASTC Apple
//! Silicon / bare WebGPU), so the smoke renders the same everywhere — it
//! exercises resolve + tier-fallback + decode + upload + bind + draw without
//! needing a per-platform compressed cook. (A real compressed BC7/ASTC artifact
//! comes from the offline `cooker` CLI; this harness proves the runtime path.)

use ph2d_asset::{AssetDb, LogicalTextureId, LogicalTextureMap, TierIndex};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_render::Sprite;

/// Spawn the W2.T4 cooked-texture smoke sprite when `PH2D_KTX2_SMOKE=1`.
/// No-op otherwise (zero cost on the normal boot path).
pub(crate) fn spawn_if_enabled(
    sim: &mut SimWorld,
    asset_db: &AssetDb,
    logical_map: &mut LogicalTextureMap,
) {
    if std::env::var_os("PH2D_KTX2_SMOKE").is_none() {
        return;
    }

    let (w, h) = (64u32, 64u32);
    let rgba = checker_rgba(w, h);
    // `srgb = true`: the canonical sprite encoding — the cooked-texture
    // pipeline binds it as `Rgba8UnormSrgb`, so the GPU sampler decodes
    // sRGB→linear like every other sprite (the checkerboard reads at its
    // authored brightness, no gamma surprise).
    let blob = match ph2d_asset_ktx2::encode_uncompressed_rgba8(w, h, true, &rgba) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("PH2D_KTX2_SMOKE: KTX2 encode failed: {e}");
            return;
        }
    };

    // Register the cooked artifact for the Constrained (RGBA8) tier and map a
    // logical id (hashed from the *source* pixels — stable across tiers) to it.
    let asset_id = asset_db.insert_ktx2_bytes(TierIndex::CONSTRAINED, &blob);
    let logical = LogicalTextureId::from_source_bytes(&rgba);
    logical_map.insert(logical, TierIndex::CONSTRAINED, asset_id);

    // Spawn the cooked-texture sprite at the origin (0.6 m square), visible in
    // the canvas centre alongside the demo sprites.
    sim.world_mut().spawn((
        Transform::from_translation(Vec2::new(0.0, 0.0)),
        Sprite::cooked_texture(logical, [0.6, 0.6], [1.0, 1.0, 1.0, 1.0]),
        Name::new("ktx2_smoke"),
    ));

    eprintln!(
        "PH2D_KTX2_SMOKE: spawned a cooked-texture sprite ({w}×{h} RGBA8 KTX2, \
         Constrained tier, logical {logical}, {} blob bytes)",
        blob.len()
    );
}

/// An 8×8-cell red/blue checkerboard — high-contrast so the cooked sprite is
/// unmistakable on canvas, and any UV-flip / gamma / channel-swap regression
/// is visually obvious.
fn checker_rgba(w: u32, h: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity((w as usize) * (h as usize) * 4);
    for y in 0..h {
        for x in 0..w {
            let cell_even = ((x / 8) + (y / 8)) % 2 == 0;
            let (r, g, b) = if cell_even {
                (235u8, 64, 52) // red
            } else {
                (52u8, 119, 235) // blue
            };
            out.extend_from_slice(&[r, g, b, 255]);
        }
    }
    out
}
