//! `PH2D_TAPER_SMOKE` — the **Taper** scene (Procreate *Touch Taper*; Enio 2026-08-08).
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
//!   PH2D_TAPER_SMOKE=1 cargo run --release -p ph2d-host-desktop
//! ```
//!
//! ⚠️ **Nothing is armed in code beyond the canvas** — not the medium, not the brush, not the taper.
//! The painter opens in **Digital** with the shipped default brush, and the taper opens **off**, which
//! is the whole first question this scene asks: *does an untouched build paint exactly what it painted
//! yesterday?* A smoke that pre-armed the feature would skip the seam it exists to prove, and would
//! hide a bad default (the scar the Impasto and Wet Paint smokes each left).
//!
//! ## The script
//!
//! 1. Click the canvas → the **Painter** pill. Paint one ordinary stroke.
//!    → It must look exactly as it always has. The taper is off; nothing may have moved.
//! 2. Scroll the brush panel to **Taper**, directly under the Falloff. Drag the **left handle** in.
//!    → Strokes now open from a point. The head must reach full width smoothly, with no step.
//! 3. Drag the **right handle** in (it comes from the right edge — dragging it left LENGTHENS the end
//!    taper). Now paint again, watching the tip **while you drag** and then **at the moment you lift**.
//!    → ⚠️ **While you drag, the ink must stay under the cursor: no lag, no catching up, no smoothing.**
//!    An earlier cut held the tail back to learn where the stroke would end and was rejected on sight.
//!    → **The tail narrows the instant you lift.** The stroke is put back and laid again, tapered — one
//!    stroke, one undo step. Press Ctrl+Z once and the whole thing goes.
//! 4. **Tip** (sharp ↔ blunt) and **Opacity** (how much the taper fades as well as narrows). Untick
//!    **Link tip sizes** to give the two ends different points — the end tip is seeded from the start's,
//!    so nothing jumps at the instant you untick it.
//! 5. **The four media.** Set a taper, then switch the Paint Mode dropdown through Digital →
//!    Watercolor → Impasto → Wet Paint → back to Digital, painting one stroke in each.
//!    → The taper must be the SAME in all four, and must still be there when you come back.
//! 6. **The shape editors — this is where the END taper lives.** Pick Method = **Line** (or Curve, or
//!    **Free Hand**) and drag one out.
//!    → Both ends taper, **live and exact**, with no lag at all: the whole path exists before the first
//!    dab, so the engine measures it instead of guessing. Reshape the curve and watch both ends follow.
//! 7. **Ellipse / Polygon.**
//!    → They must come out with **no taper at all**, uniform all the way round.
//!
//! ## What is EXPECTED, and must not be "fixed"
//!
//! - **A closed loop is never tapered** (step 7). It has no ends: the only place a taper could land is
//!   the arbitrary point the fill happened to start at, and a circle with a notch in it is a defect.
//! - **A stroke shorter than the two windows is a lens**, thinner throughout, never a vanishing act —
//!   the two ends combine by `min`, not by product.
//! - **A plain drag's tail narrows at LIFT, not while you draw** (step 3). While the stroke is open its
//!   far end has not happened yet; withholding dabs to find out is the delay that was rejected, so the
//!   ink is laid full width and the last window is re-laid at pen-up.
//! - **Watercolor and Wet Paint keep a blunt tail on a plain drag, deliberately.** The wash rebuilds
//!   itself from its own accumulators and the fluid cannot be rewound, so putting their pixels back
//!   would not put their STATE back. Their shape editors (step 6) still taper both ends.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// Whether the smoke is armed. Cheap enough to call per frame.
pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_TAPER_SMOKE").is_some()
}

/// Spawn the paint canvas when `PH2D_TAPER_SMOKE=1`, returning its entity bits so the caller can seat
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
        2, // opaque white — a tapered tip is a thin dark mark, and it reads clearest on a light ground
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    ) {
        Ok((label, bits)) => {
            println!(
                "PH2D_TAPER_SMOKE: canvas '{label}' staged (1024x1024, opaque white). NOTHING else is \
                 armed — the painter opens in DIGITAL, with the shipped default brush and the taper \
                 OFF.\n\
                 PH2D_TAPER_SMOKE: 1) paint one stroke: it must look exactly as it always has  \
                 2) brush panel -> TAPER (under the Falloff): drag the LEFT handle in — strokes open \
                 from a point  \
                 3) drag the RIGHT handle in, then paint: THE INK MUST STAY UNDER THE CURSOR while you \
                 drag (no lag, no smoothing), and THE TAIL NARROWS WHEN YOU LIFT -- one undo step  \
                 4) Tip (sharp<->blunt), Opacity, and untick 'Link tip sizes' for two different points  \
                 5) switch the Paint Mode through all FOUR media — the taper must survive every switch \
                 and still be there when you come back  \
                 6) Method = Line / Curve / Free Hand: THIS is where the END taper lives -- both ends, \
                 live and exact  \
                 7) Ellipse / Polygon: NO taper at all — a closed loop has no ends."
            );
            Some(bits)
        }
        Err(e) => {
            eprintln!("PH2D_TAPER_SMOKE: could not spawn the canvas: {e}");
            None
        }
    }
}
