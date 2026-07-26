//! `PH2D_IMPASTO_SMOKE` — a ready-to-paint Impasto canvas (#16).
//!
//! Spawns a white 1024² paint canvas and sizes the brush for relief (Grain = None; Enio 2026-07-12).
//! The painter opens in **Digital** (the app default) — pick **Impasto** from the Paint Mode dropdown,
//! then drag: the paint comes out thick and lit. A new feature ships with the example that demonstrates
//! it ([[feedback_ready_to_smoke_example]]); the example shows the DEFAULTS, so a bad default has
//! nowhere to hide.
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
//!   PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
//! ```
//!
//! Then: click the white canvas → the **Painter** pill. It opens in **Digital** (the app default); pick
//! **Impasto** from the Paint Mode dropdown (Brush panel) to open its section, and drag — the Lighting
//! card drives the light live.
//!
//! **The RIG (2026-07-12):** the card carries **four lamps** — the chips `1 2 3 4` under Show Impasto. A
//! chip marked `2·` is a lamp that is OFF. Pick lamp 2, tick its **Enable**, and you have a fill: give it an
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
//! ## The SCULPT (`docs/Painter/18…`, Waves 1-3 — 2026-07-13)
//!
//! This same smoke is the sculpt's, and deliberately without an env var of its own: the sculpt needs
//! relief to reshape, and the way you get relief is to paint some. So —
//!
//!   1. Paint two or three thick strokes (as above). Ridges, with brush-marks on them.
//!   2. Click **SCULP** on the left rail (between Deform and Mask).
//!   3. Drag over the ridges. **Eight verbs, one kernel** (`h = pre + k·Δ`):
//!      - **Smooth** knocks them down toward the local average; **Sharpen** runs the same kernel
//!        backwards and brings them back up.
//!      - **Flatten** pulls the paint toward the plane fitted under the brush — and the plane is
//!        **tilted**, so on a slope it takes the marks off and leaves the slope. (Paint a ridge, then
//!        flatten across its FLANK: the flank stays a flank. A level spatula would gouge a trough into it.)
//!      - **Scrape** is that plane, down only — the palette knife taking the high ground off. **Fill** is
//!        its mirror: paint pushed into the hollows, the ridges left standing.
//!      - **Chisel** is Scrape with the knife tipped onto its edge (**Angle**): it spares the flanks and
//!        cuts the axis, so it leaves a groove with a crease at the bottom. At Angle 0 it IS Scrape.
//!      - **Layer** lays exactly **Depth** of paint and stops — pass over the same spot ten times in one
//!        stroke and the coat is still one Depth thick. Nothing else here can do that.
//!      - **Inflate** raises along the surface normal, so the flats come up and the walls do not: a ridge
//!        gets ROUNDED rather than lifted. Negative Depth deflates it.
//!
//! The brush's own **Size** is the spatula's width, its **Strength** is how hard you lean on it, and
//! Spacing / Falloff / Shape / Grain / Symmetry / Tiling all apply — the sculpt rides the same dab list
//! the colour does, so a spatula with a Grain is a *textured* spatula, for free.
//!
//! The card adds only what the brush cannot say, and **it shows the knobs the active verb uses, and no
//! others** — a knob that does nothing to the tool in your hand is a knob that lies:
//!   - Smooth / Sharpen → **Radius** (1-16 px, the scale brush-marks live at).
//!   - Flatten / Scrape / Fill → **Offset**, in paint-loads. It is the spatula's *bite*: at `0` the plane
//!     rests on the surface it was fitted to, so Scrape only takes off what stands above the local slope.
//!     Pull it negative and the knife digs in. Push it positive and Fill mounds paint above the surface.
//!   - Chisel → **Offset** *and* **Angle** (the plane has to be placed before the V is folded about it).
//!   - Layer / Inflate → **Depth**, in paint-loads. Signed: the lower half carves and deflates.
//!
//! **Clay is not a chip, and that is deliberate**: it is Flatten with a positive Offset (the plane sits
//! above the surface, so the hollows fill and the ridges level). Both knobs are already in front of you. A
//! chip that is a preset of another chip cannot tell you which of the two you are holding.
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
//! Note that the MEDIUM is not armed in code — you pick Impasto from the dropdown yourself. That is on
//! purpose. The smoke that arms state under the table skips exactly the seam it was supposed to prove,
//! and this line has the scar twice: `PH2D_IMPASTO_SMOKE` pre-selected the medium, which is how the
//! master switch shipped dead under the mouse and nobody noticed for a week — and it re-appeared on
//! 2026-07-22, opening the painter on Impasto when the app default is Digital (Enio caught it on the Wet
//! Paint side). The medium is the Paint Mode dropdown now, a real control clicked by a real pointer in
//! `seam_paint_media.rs`; the smoke gives you the canvas and gets out of the way.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_core::Vec2;
use ph2d_ecs::SimWorld;
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;

/// Whether the smoke is armed. Cheap enough to call per frame.
pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_IMPASTO_SMOKE").is_some()
}

/// The canvas edge this run paints on.
///
/// `PH2D_IMPASTO_SMOKE=2` opens a **4096²** canvas — the tela where the fold regression lived and the
/// only one where its cure is something the artist can FEEL. Measured on the RTX, per pointer move, on
/// the GPU producer that a sculpted document takes:
///
/// | canvas | antes | agora |
/// |---|---|---|
/// | 2048² | 57,1 ms | **1,98 ms** |
/// | 4096² | **225,6 ms** | **2,62 ms** |
///
/// 225 ms a move is ~4 fps: the brush lags a whole hand's width behind the cursor. At 1024² — what `=1`
/// opens — the same defect cost ~11 ms, which reads as "a bit heavy" and is exactly why it survived a
/// smoke. A cure whose scene cannot show the disease is a scene that will approve the disease back.
fn canvas_edge() -> u32 {
    match std::env::var("PH2D_IMPASTO_SMOKE").as_deref() {
        Ok("2") => 4096,
        _ => 1024,
    }
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
        canvas_edge(),
        2, // opaque white — relief reads clearest on a light ground
        Vec2::new(0.0, 0.0),
        pixels_per_meter,
        atlas_asset_map,
    ) {
        Ok((label, bits)) => {
            let edge = canvas_edge();
            println!(
                "PH2D_IMPASTO_SMOKE: canvas '{label}' ({edge}x{edge}) ready — pick the Painter tool. \
                 It opens in DIGITAL; pick Impasto from the Paint Mode dropdown (Grain = None, \
                 size 40), then drag."
            );
            if edge >= 4096 {
                println!(
                    "PH2D_IMPASTO_SMOKE=2: o que julgar e o numero que ele tinha. Pinte um traco \
                     LONGO e continuo, e depois va e volte sobre ele. O pincel tem de acompanhar o \
                     cursor. Medido nesta tela, por movimento: 225,6 ms antes (cerca de 4 fps, o pincel \
                     ficava uma mao atras do cursor) contra 2,62 ms agora. O desenho nao muda -- a \
                     wave e so velocidade -- entao se a tinta parecer diferente da de 1024, isso e um \
                     achado e nao um detalhe."
                );
            }
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
    // ⚠️ **The MEDIUM is NOT selected here** — the module doc above has said so all along ("nothing here
    // is armed in code — you click the rail chip yourself"), and until 2026-07-22 the code contradicted
    // it (`set_paint_media(Impasto)`), re-opening the very scar it warns about: the painter opened on a
    // medium the app default says it should not, and it skipped the dropdown seam. Only the SIZE is set,
    // and it lands on the shared Paint slot (the Impasto Deposit IS `PaintMode::Paint`), so it is ready
    // the moment you pick Impasto from the Paint Mode dropdown.
    painter.set_brush_size_px(40.0);
    println!(
        "PH2D_IMPASTO_SMOKE: canvas ready in DIGITAL — pick Impasto from the Paint Mode dropdown."
    );
}
