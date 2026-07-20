//! Gates for **Inflate's MATTER and appearance** — split from [`super::inflate`] for the file-LOC cap,
//! along the seam that was already there: the sibling pins the engine's arithmetic (ramp / dome /
//! reach); this file pins what the ARTIST sees — the paint gets wider, the paint (not just the height
//! buffer) travels, the new rim wears the paint's colour, and exactly one verb moves matter at all.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::{BrushSpec, Falloff};

const LAYER: u8 = 6;
const INFLATE: u8 = 7;

/// The light's height→pixel gain, re-read from the product so this file cannot drift from it.
const UNIT: f32 = super::super::impasto_light::DEPTH_UNIT_PX;

/// Depth `+0.5` loads on the `0..1` track (`depth = (t − 0.5) · 2`).
const DEPTH_UP: f32 = 0.75;
/// Depth `−0.5` loads.
const DEPTH_DOWN: f32 = 0.25;

// ── The appearance oracle: the paint the DEPOSIT actually lays ───────────────────────────────────────

/// Lay a real impasto stroke — the product's own dab path, its own deposit, its own settle — and hand back
/// the tool with the relief that stroke made.
///
/// Not a synthetic ridge. The whole failure this file exists to prevent was a gate that measured a canvas the
/// product cannot produce, and the number that killed it (`n_z = 1.000` at the median painted texel) is a
/// fact about *this* fixture and no other.
fn deposited_stroke(size: u32) -> (PainterTool, crate::tool::RtLayerId, Vec<f32>) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 16.0,
        hardness: 1.0,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.1, 0.2, 0.3],
        space_attenuation: false,
        impasto: true,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("brush");
    t.set_brush_impasto_depth(1.0);
    let layer = t.layers.active().expect("a layer");

    // A straight horizontal stroke across the middle.
    let y = f32::from(u8::try_from(size / 2).unwrap_or(64));
    t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
    let mut x = 44.0;
    while x <= f32::from(u16::try_from(size).unwrap_or(200)) - 40.0 {
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        x += 4.0;
    }
    t.on_canvas_pointer(cp([x, y], PointerPhase::Up));

    let relief = heights_of(&t, layer);
    assert!(
        relief.iter().any(|v| *v > 0.5),
        "fixture: the deposit laid no relief"
    );
    (t, layer, relief)
}

/// The centre of the stroke, and the half-width of the window every measurement is taken in.
///
/// The window is **inside the sculpt brush's footprint** (radius 40), and that is not tidiness — it is the
/// whole correctness of the oracle. An offset moves the GROUND too: dilating raises bare canvas by exactly
/// Depth, eroding lowers it by exactly Depth (a flat, offset along its normal, is a translation). So a
/// column measured across the whole canvas mixes offset ground with untouched ground, its minimum comes from
/// wherever the brush did not reach, and the "half" level it derives is a level nothing means. The first
/// draft of this file did exactly that and reported an *erosion* as having made the paint **142 texels
/// wider** than it was. The oracle was broken, not the kernel — but it took the same shape as a real report,
/// which is what makes this class of mistake expensive.
const MID: u32 = 100;
const BAND: u32 = 32;

/// How wide the paint is, in texels, on the column `x` — the rows (within [`BAND`] of the stroke) whose
/// relief clears **half of the column's own peak above its own floor**.
///
/// Relative to the column's own floor, and that is the fairness of it: `Layer` adds a constant inside the
/// footprint, so floor and peak rise together, the half level rises with them, and the crossing points do not
/// move — Layer *cannot* change this number, which is what the sibling gate asserts before trusting it. An
/// absolute threshold would have reported Layer as "wider" for having done nothing to the shape at all.
fn width_at_half_max(h: &[f32], size: u32, x: u32) -> usize {
    let col: Vec<f32> = (MID - BAND..=MID + BAND)
        .map(|y| h[(y * size + x) as usize])
        .collect();
    let peak = col.iter().copied().fold(f32::MIN, f32::max);
    let floor = col.iter().copied().fold(f32::MAX, f32::min);
    let half = floor + 0.5 * (peak - floor);
    col.iter().filter(|v| **v >= half).count()
}

/// **THE gate Enio's smoke was missing: Inflate makes the paint WIDER. Layer only makes it taller.**
///
/// This is the difference, stated as the artist sees it. A ball rolled over the relief pushes the form's rim
/// **outward** — that is what dilation *is*, and it is the only reason to reach for this verb rather than
/// Layer. Layer adds a constant inside the footprint: a rigid lift, and the cross-section it lifts is the
/// cross-section it found.
///
/// The oracle is the paint's **width at half its own height**, which a rigid lift cannot change and an offset
/// must. Measured on a stroke from the **real deposit** — because on the relief the deposit actually makes,
/// the old kernel and Layer were the same number, and no gate written on a synthetic cliff could ever say so.
///
/// **Mutation that must bleed:** restore the per-texel formula (`p + depth * n_z`, in either direction) — the
/// widths collapse onto Layer's, which is the report. Also: make the ball's radius `0`.
#[test]
fn the_inflate_fattens_the_form_where_the_layer_only_raises_it() {
    let size = 200u32;
    let probe_x = 100u32; // mid-stroke: away from the ends, where the footprint is not the issue

    let run = |mode: Option<u8>| -> (Vec<f32>, usize) {
        let (mut t, layer, _) = deposited_stroke(size);
        if let Some(m) = mode {
            arm_sculpt(&mut t, m, 0.5, 1.0);
            let mut b = t.paint.brush;
            b.radius_px = 40.0; // wide enough to cover the stroke's whole cross-section
            b.falloff = Falloff::Constant;
            t.paint.brush = b;
            t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
            t.set_sculpt_depth(DEPTH_UP); // +0.5 loads ⇒ a ball of 8 px
            drag(&mut t, &[[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]]);
        }
        let h = heights_of(&t, layer);
        let w = width_at_half_max(&h, size, probe_x);
        (h, w)
    };

    let (_, bare) = run(None);
    let (_, layered) = run(Some(LAYER));
    let (inf, inflated) = run(Some(INFLATE));

    assert_eq!(
        layered, bare,
        "Layer changed the paint's width at half-max from {bare} to {layered} texels. It must not: it adds a \
         constant inside the footprint, which lifts the cross-section rigidly and leaves its shape alone. If \
         this moved, the oracle is measuring the lift and not the shape, and the gate below means nothing."
    );
    assert!(
        inflated >= bare + 8,
        "Inflate left the paint {inflated} texels wide at half-max; before the stroke it was {bare}, and \
         Layer (which is NOT supposed to change it) left it {layered}. A Depth of 0.5 loads is a ball of \
         {:.0} px, so the form's rim should be pushed out by something of that order on each side. \
         Inflate is not offsetting the surface — it is raising it, which is Layer, and we have Layer. \
         (This is the report from Enio's smoke, 2026-07-14.)",
        0.5 * UNIT
    );
    // And it really is the PAINT that got wider, not noise below the paint: the new rim carries real height.
    let peak = inf.iter().copied().fold(f32::MIN, f32::max);
    assert!(
        peak > 0.5,
        "fixture: the inflated relief peaks at only {peak:.3} loads"
    );
}

/// **A negative Depth ERODES: the form gets thinner, not merely lower.**
///
/// The mirror of the gate above, and it is not a formality — erosion is the half of the tool that has no
/// counterpart anywhere else in the verb list. Scrape and Chisel take material off *down to a plane*; only
/// this eats a form's **sides** in, and only this can make a thin ridge disappear.
///
/// A `min`-plus that had been written as a `max`-plus with a negated cap would still *lower* the paint (so a
/// "did it go down?" gate would pass) while making it **wider** — the shape is where the sign lives.
///
/// **Mutation that must bleed:** take `r_px.abs()` for the branch (`dilate = true` always), so a negative
/// Depth dilates with a shrinking ball.
#[test]
fn a_negative_depth_erodes_the_form_instead_of_just_lowering_it() {
    let size = 200u32;
    let probe_x = 100u32;

    let (mut t, layer, before) = deposited_stroke(size);
    let bare = width_at_half_max(&before, size, probe_x);

    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    let mut b = t.paint.brush;
    b.radius_px = 40.0;
    b.falloff = Falloff::Constant;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
    t.set_sculpt_depth(DEPTH_DOWN); // −0.5 loads ⇒ erode by a ball of 8 px
    drag(&mut t, &[[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]]);

    let after = heights_of(&t, layer);
    let eroded = width_at_half_max(&after, size, probe_x);

    assert!(
        eroded + 4 <= bare,
        "a negative Depth left the paint {eroded} texels wide at half-max; it was {bare}. Erosion has to eat \
         the form's SIDES in — a kernel that only lowers the surface is dilating with a negative sign, and \
         the difference does not show in the height, only in the shape."
    );
    let peak_before = before.iter().copied().fold(f32::MIN, f32::max);
    let peak_after = after.iter().copied().fold(f32::MIN, f32::max);
    assert!(
        peak_after < peak_before,
        "…and it did not go down at all ({peak_before:.3} → {peak_after:.3})"
    );
}

// ── The memo ────────────────────────────────────────────────────────────────────────────────────────

// ── The APPEARANCE: coverage, and the CLOSING that shapes it ─────────────────────────────────────────
//
// The Blob's footprint is a morphological CLOSING now (`super::super::sculpt_close`): it fills concave
// armpits and leaves convex flanks where the artist drew them (Enio, 2026-07-17). So "the paint gets WIDER"
// is true only at a concavity — a straight stroke's convex sides do not grow — and the gates that measure the
// coverage growth, the colour it carries and the edge it makes were moved to where the concave fixture is, in
// `super::inflate_junction_probes` (`the_convex_flank_is_preserved_while_the_armpit_fills` and its siblings).
// What remains here needs no concavity: the height still dilates (the dome), a negative Depth still erodes,
// and exactly one verb moves the matter at all.

fn covers_of(t: &PainterTool, layer: crate::tool::RtLayerId) -> Vec<u8> {
    t.covers
        .get(&layer)
        .map(|c| (**c).clone())
        .unwrap_or_default()
}

/// **Exactly ONE of the eight verbs moves matter — and the other seven leave the paint byte-identical.**
///
/// §5 of the plan says *the sculpt writes `h` and only `h`*, and it is right about seven verbs. Inflate is
/// the exception, and the exception is the entire point of the verb: it **grows the form**. Every other verb
/// redistributes height inside paint that is already there, and if any of them started shifting pixels the
/// artist would see their colour creep with no name for it.
///
/// The sweep is the gate. A `matches!(mode, Inflate)` written at a call site would be a claim; this is a
/// measurement, over all eight, on the real dab path.
///
/// **Mutation that must bleed:** delete the matter loop in `render_inflate` — the Inflate branch stops moving
/// coverage and the `INFLATE` arm below fails. (Checked in the sibling above.)
///
/// The invariant is held STRUCTURALLY: the matter loop lives inside `render_inflate`, and `render_sculpt`
/// routes only Inflate there — every other verb goes through the memo / plane / height paths, which never
/// touch `covers` or `rgba`. There is no per-verb flag to get wrong; a second verb could move matter only by
/// growing its own advection, which the sweep would catch on that verb's row.
#[test]
fn exactly_one_verb_moves_the_matter() {
    let size = 160u32;
    for mode in 0u8..8 {
        let (mut t, layer, _) = deposited_stroke(size);
        let cov0 = covers_of(&t, layer);
        let rgba0 = (*t.canvas_rgba).clone();

        arm_sculpt(&mut t, mode, 0.5, 1.0);
        let mut b = t.paint.brush;
        b.radius_px = 30.0;
        b.falloff = Falloff::Constant;
        t.paint.brush = b;
        t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
        t.set_sculpt_depth(DEPTH_UP);
        t.set_sculpt_offset(0.25);
        drag(&mut t, &[[50.0, 80.0], [80.0, 80.0], [110.0, 80.0]]);

        let cov1 = covers_of(&t, layer);
        let rgba1 = (*t.canvas_rgba).clone();
        let cov_moved = cov0.iter().zip(&cov1).filter(|(a, b)| a != b).count();
        let rgba_moved = rgba0.iter().zip(&rgba1).filter(|(a, b)| a != b).count();

        if mode == INFLATE {
            assert!(
                cov_moved > 200 && rgba_moved > 200,
                "Inflate moved no matter (coverage: {cov_moved} texels, pixels: {rgba_moved} bytes). It is \
                 the one verb that GROWS the form, and a form that grows onto bare canvas without taking \
                 its paint along does not grow at all — the light multiplies by the coverage."
            );
        } else {
            assert_eq!(
                (cov_moved, rgba_moved),
                (0, 0),
                "verb {mode} moved the paint: {cov_moved} coverage texels and {rgba_moved} pixel bytes. \
                 Only Inflate may — the other seven reshape the relief INSIDE the paint that is there. A \
                 Smooth that shifted colour would look like the artist's hand was smearing, and nothing on \
                 screen would say why."
            );
        }
    }
}
