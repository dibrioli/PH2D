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
//! 2. Scroll the brush panel to **Taper**, directly under the Falloff. Drag the **handle** in.
//!    → Strokes now open from a point. The head must reach full width smoothly, with no step.
//!    → ⚠️ **Look at the widget itself first:** the stroke shape and the dot must sit INSIDE the panel,
//!    with nothing crossing the right border. That edge is where a full-width disc used to hang over.
//! 3. Paint again, watching the ink **while you drag** and **at the moment you lift**.
//!    → ⚠️ **The ink must stay under the cursor: no lag, no catching up, no smoothing.**
//!    → ⚠️ **Nothing may change at the instant you lift.** The tail is full width while you draw and it
//!    stays full width when you let go — a stroke that visibly re-draws itself on pen-up is the resolve
//!    coming back from the dead.
//! 4. **Tip** (sharp ↔ blunt) and **Opacity** (how much the taper fades as well as narrows).
//! 5. **The four media.** Set a taper, then switch the Paint Mode dropdown through Digital →
//!    Watercolor → Impasto → Wet Paint → back to Digital, painting one stroke in each.
//!    → The taper must be the SAME in all four, must still be there when you come back, and every one
//!    of them must taper the HEAD — this is the half that used to be Digital-only.
//! 6. **The shape editors.** Pick Method = **Line** (or Curve, or **Free Hand**) and drag one out.
//!    → The head tapers, live and exact, and reshaping the path keeps it there. The far end comes out
//!    **blunt** — it used to taper here, and that went with the rest.
//! 7. **Ellipse / Polygon.**
//!    → They must come out with **no taper at all**, uniform all the way round.
//!
//! ## What is EXPECTED, and must not be "fixed"
//!
//! - **The far end is never tapered, on any method or any medium** (steps 3, 5, 6). The tail control,
//!   the *Link tip sizes* toggle and the second Tip row are gone with it — Enio 2026-08-10, *"quanto à
//!   cauda do taper vamos desativar para todos os modos de pintura; deixe o ajuste apenas para o início
//!   do traço, como já funciona perfeitamente"*.
//! - **A closed loop is never tapered** (step 7). It has no head: the only place a taper could land is
//!   the arbitrary point the fill happened to start at, and a circle with a notch in it is a defect.
//! - **Nothing happens at pen-up.** The pen-up used to put the stroke back and lay it again to shape the
//!   tail; that is gone, so a lift costs exactly what it cost before the taper existed.

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
                 2) brush panel -> TAPER (under the Falloff): the shape and the dot must sit INSIDE the \
                 panel (nothing crossing the right border), then drag the handle in — strokes open from \
                 a point  \
                 3) paint again: THE INK MUST STAY UNDER THE CURSOR while you drag (no lag, no \
                 smoothing), and NOTHING MAY CHANGE WHEN YOU LIFT -- the tail is blunt all along  \
                 4) Tip (sharp<->blunt) and Opacity  \
                 5) switch the Paint Mode through all FOUR media — the taper must survive every switch, \
                 still be there when you come back, and taper the HEAD in every one of them  \
                 6) Method = Line / Curve / Free Hand: the head tapers live and exact; the far end is \
                 BLUNT (it used to taper here — that went with the tail)  \
                 7) Ellipse / Polygon: NO taper at all — a closed loop has no head."
            );
            Some(bits)
        }
        Err(e) => {
            eprintln!("PH2D_TAPER_SMOKE: could not spawn the canvas: {e}");
            None
        }
    }
}
