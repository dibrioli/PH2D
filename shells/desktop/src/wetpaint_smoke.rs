//! `PH2D_WETPAINT_SMOKE` — a ready-to-paint **Wet Paint** canvas (ADR-0134, W1).
//!
//! Spawns a white 1024² paint canvas. The painter opens in **Digital** — the app's initial default
//! (Enio, 2026-07-22: *"o modo que aparece ao abrir o painter deve ser o digital"*) — so **pick Wet
//! Paint from the Paint Mode dropdown** at the head of the Brush panel, then drag: the paint goes down
//! as FLUID — pigment suspended in water that keeps moving after pen-up (levels, bleeds, dries). A new
//! feature ships with the example that demonstrates it ([[feedback_ready_to_smoke_example]]).
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
//! ⚠️ **The mode is NOT armed in code any more** (Enio, 2026-07-22: *"quando abro o painter o Wet
//! paint ainda é o que aparece primeiro mas deveria ser o digital"*). It used to be, because until W3
//! nothing else selected Wet Paint; now the **Paint Mode** dropdown does, a real control gate-covered
//! in `seam_paint_media.rs`. Arming under the table made the painter open on a medium the app default
//! says it should not — the exact scar the impasto smoke has been warning about — and it skipped the
//! very seam the smoke exists to prove. So the smoke gives you the canvas and gets out of the way: the
//! dropdown is the door.

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
                "PH2D_WETPAINT_SMOKE: canvas '{label}' ready — pick the Painter tool. It opens in \
                 DIGITAL; choose Wet Paint from the Paint Mode dropdown, then drag and watch the \
                 water move."
            );
            Some(bits)
        }
        Err(e) => {
            eprintln!("PH2D_WETPAINT_SMOKE: could not spawn the canvas: {e}");
            None
        }
    }
}

/// Pre-configure the brush COLOUR the first time the Painter binds a document under the smoke.
/// Idempotent (one-shot), so the artist's own edits are never overwritten.
///
/// ⚠️ **It no longer selects the medium** — the painter opens in Digital and the artist picks Wet Paint
/// from the dropdown (Enio, 2026-07-22). Only the colour is set, and that reaches the wet slot for free:
/// the paint colour is synced across every mode's `BrushSpec` on purpose (Brush = Fill = picker, one
/// colour). The SIZE is per-slot, so it is left at the wet default — which the smoke should SHOW rather
/// than hide, by its own rule ("if a default is bad, the smoke says so").
pub(crate) fn arm_brush_once(painter: &mut ph2d_tool_painter::PainterTool) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static ARMED: AtomicBool = AtomicBool::new(false);
    if !enabled() || ARMED.swap(true, Ordering::Relaxed) {
        return;
    }
    painter.set_brush_color_srgb8([30, 90, 200]); // a wet ultramarine (synced to the wet slot)
    println!(
        "PH2D_WETPAINT_SMOKE: canvas ready in DIGITAL — pick Wet Paint from the Paint Mode dropdown, \
         then drag."
    );
}
