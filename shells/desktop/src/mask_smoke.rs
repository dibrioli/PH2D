//! `PH2D_MASK_SMOKE` — a ready-to-mask canvas for the **coverage-law rewrite** (doc 25 §13.9).
//!
//! The defect this scene exists to judge only appears when you **scrub**: one pass always looked fine,
//! and that is why it survived so long ("VC não percebe porque dá poucas passadas", Enio). So the
//! script below asks for the gesture, not for a stroke.
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
//! 1. Click the canvas → the **Painter** pill → paint a few ordinary strokes, in a strong colour. This
//!    is the art the mask will FREEZE, and you need it to see the mask do its job.
//! 2. Click **MASK** on the left rail. Take the **Size** up (~60) so the edge is big enough to judge,
//!    and zoom in.
//! 3. **Scrub.** Hold the pen down and go back and forth over the same curve, ten or twenty times.
//!    → The edge must stay a soft ramp. It must NOT tighten into a hard, stair-stepped boundary as
//!    you keep going. (This half is now completely inert: measured, scrubbing four legs in one
//!    pen-down moves the coverage by **2 levels of 255**, where the old law moved it by **118** and
//!    collapsed the ramp from 3.53 px to 1.88 px *within that one gesture*.)
//! 4. Lift, and paint **neighbouring** mask strokes side by side, overlapping.
//!    → The valleys where they meet must FILL and get darker, never leave a light line between the
//!    lanes (the seam of the reverted attempt — doc 25 §13.8). A second sweep over the block must
//!    close what the first left (measured 0.75 → 0.94 → 0.98 at one-radius spacing).
//! 5. Now the point of the whole thing: switch back to the **Brush** and paint across the masked
//!    region. The protected art must resist while the paint lands freely around it.
//! 6. **Ctrl+Z** — one undo per mask stroke, and the next stroke must deposit normally afterwards.
//!
//! ## What is EXPECTED to differ from the old build (say so, do not "fix" it in the smoke)
//!
//! - **One pass is SOFTER than it used to be** — it now lays the brush's own feather (6.21 px against
//!   the analytic 5.40 px at r = 10) instead of a pre-hardened 3.53 px. If you want a crisper mask, that
//!   is the brush's Hardness / falloff, which is where it belongs.
//! - **One pass leaves the core at 0.984, not 1.000** (a soft brush's dab centres never land dead on a
//!   texel centre). The second pass reaches exactly 1.000.
//! - **Many passes still tighten the ramp, slowly** — as `N^(−1/2)`: 6.21 / 2.10 / 1.94 / 1.55 px at
//!   1 / 15 / 30 / 45 passes, against the old law's 3.53 / 1.38. The only law that freezes the edge
//!   outright is the cross-stroke union, and it cannot fill the valley in step 4 — that trade is the
//!   one Enio already rejected, so this is the honest half of it, not an oversight.

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
                 3) SCRUB one pen-down back and forth 10-20x: the edge must stay a soft ramp  \
                 4) neighbouring strokes: the valleys must FILL, no light lines  \
                 5) back to Brush: the masked art must resist  6) Ctrl+Z once per stroke."
            );
            Some(bits)
        }
        Err(e) => {
            eprintln!("PH2D_MASK_SMOKE: could not spawn the canvas: {e}");
            None
        }
    }
}
