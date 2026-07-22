//! `PH2D_WETPAINT_SMOKE` — a ready-to-paint **Wet Paint** canvas (ADR-0134, W1).
//!
//! Spawns a white 1024² paint canvas and arms the Painter's brush IN THE WET
//! PAINT MODE with a wet blue. Run it, pick the Painter pill, drag: the paint
//! goes down as FLUID — pigment suspended in water that keeps moving after
//! pen-up (levels, bleeds, dries). A new feature ships with the example that
//! demonstrates it ([[feedback_ready_to_smoke_example]]).
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
//!   PH2D_WETPAINT_SMOKE=1 cargo run --release -p ph2d-host-desktop
//! ```
//!
//! What W1 shows (and what it does not):
//! - Drag: the stroke deposits pigment + water through the engine's own SPEC
//!   §9 mapping, driven by the painter's REAL stroke engine — pressure,
//!   spacing, stabilizer, Symmetry and Tiling all feed the fluid for free.
//! - Release and WAIT: the 40 Hz sim keeps running — the wash levels, edges
//!   darken as it dries. The session spans strokes (wet-on-wet works).
//! - Undo / switching tools ends the session cleanly (the look stays; the
//!   water stops). That is the bake — there is no separate commit.
//! - NOT yet here: the panel (knobs are engine defaults — W3), Paper presets,
//!   Shape/Grain in the wet deposit, per-dab Randomize colour (W2 seams).
//!
//! ⚠️ The MODE is armed in code, unlike the impasto smoke's rule. That was
//! unavoidable when written (nothing selected Wet Paint), and since
//! 2026-07-22 it is a convenience instead: the **Paint Mode** dropdown at
//! the head of the Brush panel's appearance half selects it, and a real
//! pointer clicking that chip is gate-covered in `seam_paint_media.rs`. The
//! arm stays so the smoke opens ON the feature — but check the chip reads
//! *Wet Paint*, and that picking *Digital* takes you out.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// Whether the smoke is armed. Cheap enough to call per frame.
pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_WETPAINT_SMOKE").is_some()
}

/// Spawn the blank paint canvas when `PH2D_WETPAINT_SMOKE=1`. Returns the
/// entity bits so the caller can seat the selection on it.
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
        2, // opaque white — a watercolor ground
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    ) {
        Ok((label, bits)) => {
            println!(
                "PH2D_WETPAINT_SMOKE: canvas '{label}' ready — pick the Painter tool and drag. \
                 The brush is armed in Wet Paint mode; release the pen and watch the water move."
            );
            Some(bits)
        }
        Err(e) => {
            eprintln!("PH2D_WETPAINT_SMOKE: could not spawn the canvas: {e}");
            None
        }
    }
}

/// Arm the brush the first time the Painter binds a document under the smoke.
/// Idempotent (one-shot), so the artist's own edits are never overwritten.
pub(crate) fn arm_brush_once(painter: &mut ph2d_tool_painter::PainterTool) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static ARMED: AtomicBool = AtomicBool::new(false);
    if !enabled() || ARMED.swap(true, Ordering::Relaxed) {
        return;
    }
    // Arm through the PRODUCT door — the Paint Mode dropdown's setter, so the smoke
    // exercises what the artist has: the chip reads *Wet Paint*, and tool round-trips
    // return to the fluid.
    // Then a size in the engine's native dab range and a pigment you can see on
    // white. Everything else is the shipped default — if a default is bad, the
    // smoke says so instead of hiding it.
    painter.set_paint_media(ph2d_tool_painter::PaintMedia::WetPaint);
    painter.set_brush_size_px(24.0);
    painter.set_brush_color_srgb8([30, 90, 200]); // a wet ultramarine
    println!("PH2D_WETPAINT_SMOKE: brush armed (Wet Paint mode) — drag on the canvas.");
}
