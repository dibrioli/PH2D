//! `PH2D_MASK_SMOKE` — a ready-to-mask canvas for the mask/protection axis (doc 25 §13.10 + **§13.12**).
//!
//! Both defects this scene judges only appear when you **repeat**: one pass always looked fine, and that
//! is why they survived so long ("VC não percebe porque dá poucas passadas", Enio). So the script below
//! asks for the GESTURE, not for a stroke — and for the SPEED, because the crackle it now judges was a
//! fact about the mouse's report rate.
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
//!   PH2D_MASK_SMOKE=1 cargo run --release -p ph2d-host-desktop
//! ```
//!
//! ⚠️ **Nothing is armed in code** beyond the canvas — not the mode, not the brush (the same rule the
//! Impasto and Wet Paint smokes learned the hard way: a smoke that arms state under the table skips
//! exactly the seam it was supposed to prove, and hides a bad default). What you see IS the shipped
//! default mask brush.
//!
//! ## The script
//!
//! 1. Click the canvas -> the **Painter** pill -> paint a few ordinary strokes, in a strong colour. This
//!    is the art the mask will FREEZE, and you need it to see the mask do its job.
//! 2. Click **MASK** on the left rail. Take the **Size** up (~60) so the edge is big enough to judge,
//!    and zoom in.
//! 3. **Scrub.** Hold the pen down and go back and forth over the same curve, ten or twenty times.
//!    -> The mask must lay the SAME field the plain brush lays — solid core, no beads along the shoulder,
//!    no light lines where lanes meet.
//! 4. **Now the star of this pass** (the crackle report, doc 25 §13.11 -> §13.12): switch back to the
//!    **Brush**, pick a strong colour, and paint ACROSS the masked region — many strokes, repeatedly,
//!    over the same place. Change speed: drag slowly, then whip across fast.
//!    -> The frozen art must resist while the paint lands freely around it, and the boundary where the
//!    paint dies must be a SMOOTH ramp that looks the same however fast you moved. It must NOT come out
//!    crackled / stair-stepped, and the paint must not creep further in when you move the mouse slowly.
//! 5. **Ctrl+Z** — one undo per stroke, and the next stroke must deposit normally afterwards.
//!
//! ## What is EXPECTED, and must not be "fixed" in the smoke
//!
//! - **Repeated strokes across the feather still BUILD UP** — passing again over a half-protected texel
//!   deepens it, exactly as the plain digital brush does. That is the law, not a leak: a protection that
//!   converged instead would be the cross-stroke ceiling that got doc 25 §13.7 reverted.
//! - **The mask edge still tightens under very many passes** (3.53 px -> 1.38 px at fifteen). That is the
//!   OTHER defect and it is still open (§13.10.4): both accumulation laws have been tried and each has
//!   its artifact, so the cure is not the coverage law. Do not judge this pass on it.
//! - **Smear / Blur / Clone dragged over a protected zone now read the UNRESTRICTED paint** (layer-mask
//!   semantics) instead of the masked view. The old behaviour read the view, but WHAT it read depended on
//!   the mouse's report rate, so it was never a stable reference. If this reads wrong, say so — it is a
//!   product decision, and it is named in §13.12.
//! - **The first frame of a protected stroke is heavier** (7.4 ms at 2048^2 against 3.0 ungated; 24.5 vs
//!   11.3 at 4096^2): it allocates the free plane once per stroke. Measured, named, and deliberately not
//!   optimised in a correctness wave (§13.12.5).

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// Whether the smoke is armed. Cheap enough to call per frame.
pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_MASK_SMOKE").is_some()
}

/// Spawn the paint canvas when `PH2D_MASK_SMOKE=1`, returning its entity bits so the caller can seat
/// the selection on it. Prints WHAT it staged — a scene that does not say what it built cannot be
/// trusted when the rest of the smoke looks wrong (the Flip colorize lesson).
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
        2, // opaque white — the mask overlay tint reads clearest over a light ground
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    ) {
        Ok((label, bits)) => {
            println!(
                "PH2D_MASK_SMOKE: canvas '{label}' staged (1024x1024, opaque white). NOTHING else is \
                 armed — the painter opens in DIGITAL with the shipped default brush.\n\
                 PH2D_MASK_SMOKE: 1) paint some art  2) rail chip MASK, Size ~60, zoom in  \
                 3) SCRUB one pen-down back and forth 10-20x: the mask must look like the plain brush  \
                 4) THE STAR: back to Brush, paint ACROSS the masked zone many times, SLOW then FAST — \
                 the boundary must be a smooth ramp and must look the SAME at both speeds  \
                 5) Ctrl+Z once per stroke."
            );
            Some(bits)
        }
        Err(e) => {
            eprintln!("PH2D_MASK_SMOKE: could not spawn the canvas: {e}");
            None
        }
    }
}
