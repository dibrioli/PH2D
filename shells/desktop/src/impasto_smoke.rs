//! `PH2D_IMPASTO_SMOKE` — a ready-to-paint Impasto canvas (#16).
//!
//! Spawns a white 1024² paint canvas AND pre-arms the brush with impasto ON — **and nothing else**
//! (Grain = None; Enio 2026-07-12). Run it, pick the Painter, drag: the paint comes out thick and lit.
//! No knob hunting — a new feature ships with the example that demonstrates it, it does not ask the artist
//! to assemble one ([[feedback_ready_to_smoke_example]]). And the example shows the DEFAULTS, so a bad
//! default has nowhere to hide.
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
//!   PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
//! ```
//!
//! Then: click the white canvas → the **Painter** pill → drag. The Impasto section (Brush panel) is
//! already open with Enable ticked; the Lighting card drives the light live.
//!
//! **The RIG (2026-07-12):** the card carries **four lamps** — the chips `1 2 3 4` under Show Impasto. A
//! chip marked `2·` is a lamp that is OFF. Pick lamp 2, tick **Enable**, and you have a fill: give it an
//! Angle across from the key, a low Intensity, and a colour. **A coloured lamp tints the paint only where
//! it TILTS** — flat paint stays exactly as you mixed it, because the shading is relative *per channel*.
//! (One lamp of any colour therefore only brightens and darkens; the hue needs two lamps disagreeing.)
//! **Shine is global** — it is how wet the OIL is, not a property of a lamp.
//!
//! The old rows are still there and now belong to the SELECTED lamp (Angle / Elevation — there is no
//! Amount any more: thickness is the brush's Depth, the slope is geometry),
//! **Depth** goes negative to carve instead of lift, and **Body** dials the cross-section: 1 = a
//! level film with a wall (default), 0 = the relief obeys the falloff — the perfectly rounded ridge.
//!
//! **Every knob edits the stroke you already painted**: lay a stroke, then drag Depth / Body / Depth
//! Source / Smoothing and watch the LAST one re-sculpt live (the stroke stores the paint it laid, and
//! the relief is derived from it). Only `Draw To` is authoring-only — it routes channels, and the
//! pigment it already laid cannot be un-laid.
//!
//! ## The SCULPT (`docs/Painter/18…`, Wave 1 — 2026-07-13)
//!
//! This same smoke is the sculpt's, and deliberately without an env var of its own: the sculpt needs
//! relief to reshape, and the way you get relief is to paint some. So —
//!
//!   1. Paint two or three thick strokes (as above). Ridges, with brush-marks on them.
//!   2. Click **SCULP** on the left rail (between Deform and Mask).
//!   3. Drag over the ridges. **Smooth** knocks them down toward the local average; the **Sharpen**
//!      chip runs the same kernel backwards and brings them back up.
//!
//! The brush's own **Size** is the spatula's width, its **Strength** is how hard you lean on it, and
//! Spacing / Falloff / Shape / Grain / Symmetry / Tiling all apply — the sculpt rides the same dab list
//! the colour does, so a spatula with a Grain is a *textured* spatula, for free. The Sculpt card adds
//! only what the brush cannot say: which verb, and the kernel's own **Radius** (1-16 px — the scale
//! brush-marks live at; smoothing at a large scale is Flatten, which is Wave 2 and a different kernel).
//!
//! **The card arms the NEXT stroke — it does not rewrite the last one.** Pick Sharpen and the Smooth
//! behind you stays a Smooth; to change a mark you have made, undo it. (It behaved the other way for one
//! afternoon, riding the Body card's "Adjust Last Stroke", and Enio's smoke killed it: reaching for the
//! other verb, in order to use it *somewhere else*, inverted the work already on the canvas. Paint is a
//! substance whose properties you can keep tuning; a smoothing is a verb that already happened.)
//!
//! The exception proves the rule: inside an **open shape editor** (Line / Curve / Ellipse / Polygon) the
//! knobs DO re-render, because a shape is a preview with an Apply button — it is not canvas yet. Draw a
//! Line in Sculpt, turn Radius before applying, and watch it re-render; then Apply, turn it again, and
//! watch nothing happen.
//!
//! Note that nothing here is armed in code — you click the rail chip yourself. That is on purpose. The
//! smoke that arms state under the table skips exactly the seam it was supposed to prove, and this line
//! has the scar: `PH2D_IMPASTO_SMOKE` pre-ticked Enable, which is how the master switch shipped dead
//! under the mouse and nobody noticed for a week.

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
                 The brush is armed with impasto ON and Grain = None; everything else is the default."
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
    // A brush with BODY, and nothing else: Grain stays at **None** (Enio, 2026-07-12). The smoke used to
    // arm a Noise grain + the Grain depth source, and that was a demo lying about a default — it showed a
    // bristly relief no fresh brush produces. Everything below the size is now the shipped default, so
    // what the smoke shows IS what the artist gets on a clean install: if the default is bad, the smoke
    // says so instead of hiding it.
    //
    // (The mapping note the old code carried is preserved because it will bite whoever turns a grain ON:
    // a ViewPlane grain is DAB-relative — every dab stamps the identical noise in its own frame, and at
    // 10% spacing the dabs overlap tenfold, so the relief comes out corrugated at exactly the dab pitch.
    // Anchor it to the canvas — `TextureMapping::Tiled` — and the marks read as bristle streaks along the
    // path instead of ribs across it. The first smoke shipped with the default ViewPlane and Enio saw the
    // corduroy immediately.)
    painter.set_brush_size_px(40.0);
    painter.toggle_brush_impasto();
    println!("PH2D_IMPASTO_SMOKE: brush armed (impasto ON, Grain = None) — drag on the canvas.");
}
