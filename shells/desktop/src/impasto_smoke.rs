//! `PH2D_IMPASTO_SMOKE` — a ready-to-paint Impasto canvas (#16).
//!
//! Spawns a white 1024² paint canvas AND pre-arms the brush with a thick, bristly impasto (the Grain
//! depth source over a noise grain, so the relief carries brush-marks). Run it, pick the Painter, drag:
//! the paint comes out thick and lit. No knob hunting — a new feature ships with the example that
//! demonstrates it, it does not ask the artist to assemble one
//! ([[feedback_ready_to_smoke_example]]).
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
//!   PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
//! ```
//!
//! Then: click the white canvas → the **Painter** pill → drag. The Impasto section (Brush panel) is
//! already open with Enable ticked; the Lighting card drives the light live (Angle / Elevation /
//! Amount / Shine), and **Depth** goes negative to carve instead of lift.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// Whether the smoke is armed. Cheap enough to call per frame.
pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_IMPASTO_SMOKE").is_some()
}

/// Spawn the blank paint canvas when `PH2D_IMPASTO_SMOKE=1`. Returns the entity bits so the caller can
/// seat the selection on it (so the artist lands ON the canvas, not hunting for it).
pub(crate) fn spawn_if_enabled(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    cell_idx: u32,
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> Option<u64> {
    if !enabled() {
        return None;
    }
    match crate::image_import::spawn_blank_canvas(
        sim,
        renderer,
        asset_db,
        cell_idx,
        1024,
        2, // opaque white — relief reads clearest on a light ground
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    ) {
        Ok((label, bits)) => {
            println!(
                "PH2D_IMPASTO_SMOKE: canvas '{label}' ready — pick the Painter tool and drag. \
                 The brush is armed with thick impasto (Depth 0.7, Grain source)."
            );
            Some(bits)
        }
        Err(e) => {
            eprintln!("PH2D_IMPASTO_SMOKE: could not spawn the canvas: {e}");
            None
        }
    }
}

/// Arm the brush the first time the Painter binds a document under the smoke. Idempotent (the flag
/// makes it a one-shot), so the artist's own edits are never overwritten afterwards.
pub(crate) fn arm_brush_once(painter: &mut ph2d_tool_painter::PainterTool) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static ARMED: AtomicBool = AtomicBool::new(false);
    if !enabled() || ARMED.swap(true, Ordering::Relaxed) {
        return;
    }
    // A loaded bristle brush: thick paint, the grain's striations carried into the relief.
    painter.set_brush_size_px(40.0);
    painter.set_brush_texture_kind(ph2d_tool_painter::TextureKind::Noise.to_u8());
    painter.toggle_brush_impasto();
    painter.set_brush_impasto_depth(0.7);
    painter.set_brush_impasto_source(ph2d_tool_painter::DepthSource::Grain.to_u8());
    painter.set_brush_impasto_smoothing(0.15);
    println!("PH2D_IMPASTO_SMOKE: brush armed — drag on the canvas.");
}
