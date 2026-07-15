//! Gates for **Inflate** — rewritten after Enio's smoke of 2026-07-14 (*"parece fazer a mesma coisa de
//! Layer"*), because the ones that were here **pinned the bug**.
//!
//! ## What went wrong, so that the shape of these gates makes sense
//!
//! Inflate shipped as `toward = pre + Depth · n_z` — "raise along the normal, and a height field can only
//! move in `z`". Two things were wrong with it, and only the second one is interesting.
//!
//! 1. **It was upside down.** Re-project the offset graph back into a height field and the raise is
//!    `Depth · S` where `S = √(1+|∇h|²) = 1/n_z` — the *secant*, not the cosine. Steep places must move
//!    **more**, which is how a wall shifts sideways and a form gets *fatter*. It moved them **less**, which
//!    rounds a crest off. Inflate was a worse Smooth. See [`super::super::sculpt_offset`].
//!
//! 2. **Fixing the sign would not have fixed the tool.** Over the relief the *deposit actually lays*, the
//!    median painted texel has `n_z = 1.000` — a stroke's interior is dead flat — so `Depth·n_z`,
//!    `Depth/n_z` and plain `Depth` are the same number, and Inflate is Layer to the bit over every texel
//!    the artist is looking at. A per-texel formula **cannot** inflate: `h + d·S` is one Euler step of the
//!    offset PDE, and one step cannot move material *sideways*, which is the whole word.
//!
//! ## And the gate was green
//!
//! The old gate was `inflate_rounds_the_crest_instead_of_translating_it`, run on a synthetic mesa whose wall
//! fell a full load in 6 px. Two failures, both in the fixture, both mine:
//!
//! * **The name asserted the wrong intention.** "Rounds the crest off" *is* the bug. A gate cannot save you
//!   from a wrong idea; it can only hold you to it.
//! * **The fixture could not occur.** The deposit cannot make a wall that steep — the settle blurs it. So the
//!   gate proved a true thing about a canvas the product never produces
//!   ([[feedback_a_gate_only_proves_what_its_fixture_contains]]). I even *noticed* the gentle ridge made the
//!   tool look like a uniform raise, and concluded the ridge was hiding the bug. It was reporting it.
//!
//! So the load-bearing gate here — [`the_inflate_fattens_the_form_where_the_layer_only_raises_it`] — runs on
//! a stroke laid by the **real deposit**, and its oracle is the thing Enio was looking at: *how wide is the
//! paint*. The analytic one beside it pins the arithmetic exactly, on a ramp, where the offset of a plane has
//! a closed form.

use super::*;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};
use ph2d_painter_brush::{BrushSpec, Falloff};
use std::sync::Arc;

const LAYER: u8 = 6;
const INFLATE: u8 = 7;

/// The light's height→pixel gain — the constant that makes the two axes of a height field comparable, and
/// therefore the one every geometric quantity over `h` has to cross. Re-read from the product so this file
/// cannot drift from it.
const UNIT: f32 = super::super::impasto_light::DEPTH_UNIT_PX;

/// Depth `+0.5` loads on the `0..1` track (`depth = (t − 0.5) · 2`).
const DEPTH_UP: f32 = 0.75;
/// Depth `−0.5` loads.
const DEPTH_DOWN: f32 = 0.25;

// ── The analytic oracle: a ramp, where the offset of a plane is a closed form ────────────────────────

/// **Inflate raises a slope by the SECANT of it, not the cosine.**
///
/// Offsetting a *plane* by a ball of radius `ρ` is exact and has no discretisation to argue about: the
/// supporting sphere touches at `u = ρG/S` and the plane comes up by
///
/// ```text
/// max over |u| ≤ ρ of [ G·u_x + √(ρ² − |u|²) ] = ρ·√(1 + G²) = ρ·S        (G = the slope, in PIXELS/pixel)
/// ```
///
/// which, back in paint-loads, is `Depth · S`. This gate builds the 3-4-5 triangle — `G = 0.75`, so
/// `S = 1.25` exactly — and asks for it.
///
/// The shipped formula computed `Depth / S`. At this slope that is `0.4` where the truth is `0.625`: the two
/// answers differ by 56%, and no tolerance can hide the difference between a number and its reciprocal.
///
/// **Mutations that must bleed** (both checked): flip the cap's sign in `ball_offset_into` (`v − cap` for the
/// dilation); and drop the `/ DEPTH_UNIT_PX` from the cap, which makes the ball sixteen times too tall.
#[test]
fn the_inflate_raises_a_ramp_by_the_secant_of_its_slope() {
    let size = 96u32;
    let (mut t, layer, _) = sculpt_canvas(size);

    // A ramp of slope G = 0.75 as the LIGHT sees it, so S = √(1 + 0.75²) = 1.25 exactly.
    let g_px = 0.75 / UNIT; // loads per texel
    let c = 48.0f32;
    let n = (size * size) as usize;
    let before: Vec<f32> = (0..n)
        .map(|i| ((i as u32 % size) as f32 - c) * g_px)
        .collect();
    t.heights.insert(layer, Arc::new(before.clone()));
    t.covers.insert(layer, Arc::new(vec![255u8; n]));
    t.sync_relief_flags();

    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    t.set_sculpt_depth(DEPTH_UP); // +0.5 loads ⇒ a ball of 8 px
    drag(&mut t, &[[c, 40.0], [c, 48.0], [c, 56.0]]);

    let after = heights_of(&t, layer);
    let at = |x: u32, y: u32| -> f32 {
        let i = (y * size + x) as usize;
        after[i] - before[i]
    };

    let expected = 0.5 * 1.25; // Depth · S
    let got = at(48, 48);
    assert!(
        (got - expected).abs() < 0.02,
        "Inflate raised a slope-0.75 ramp by {got:.4} loads; the offset of a plane by a ball of Depth 0.5 \
         raises it by Depth·S = {expected:.4}, exactly. {}",
        if got < 0.5 {
            "It came out BELOW the Depth, which means the normal is being applied upside down (Depth/S, the \
             cosine) — the steep places are moving LESS, which rounds crests off instead of pushing the \
             form outward. That is a Smooth, and we have Smooth."
        } else {
            "The ball's cap is in the wrong unit, or the kernel is not a ball at all."
        }
    );
}

/// **On a flat, Inflate makes a ROUNDED DOME — not a flat Layer raise.** (Enio's 3rd smoke, 2026-07-14:
/// *"parece uma mistura de inflate com layer … estude o Blob do Blender."*)
///
/// The old gate here asserted the OPPOSITE — that Inflate equals Layer on a flat — and it was the behaviour
/// Enio rejected. Inflate is the **Blob** now: a ball whose radius follows the falloff, so on a flat it lifts
/// a smooth spherical mound — a rounded peak at the centre, tapering to nothing at the brush edge — instead
/// of the flat, falloff-scaled raise a Layer gives. That rounded centre is the whole of the fix.
///
/// The gate reads the profile down a radius: it must rise ≈ Depth at the centre, fall **monotonically and
/// convexly** (no flat plateau — the flat top is precisely the *"mistura com layer"* complaint), and reach
/// zero by the brush edge (*"na borda nada se move"*).
///
/// **Mutation that must bleed:** make `render_inflate`'s ball radius constant (`depth.abs()·unit` instead of
/// `·amount·unit`) — the dome flat-tops and the plateau check fails.
#[test]
fn on_a_flat_the_inflate_is_a_rounded_dome_not_a_layer_raise() {
    let size = 160u32;
    let n = (size * size) as usize;
    let (mut t, layer, _) = sculpt_canvas(size);
    t.heights.insert(layer, Arc::new(vec![0.5f32; n])); // dead-flat paint, half a load thick
    t.covers.insert(layer, Arc::new(vec![255u8; n]));
    t.sync_relief_flags();

    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    let mut b = t.paint.brush;
    b.radius_px = 40.0;
    b.hardness = 0.0;
    b.falloff = Falloff::Smooth; // a soft brush: the falloff is the ball's size profile
    t.paint.brush = b;
    t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
    t.set_sculpt_depth(DEPTH_UP); // +0.5 loads
    let c = 80.0f32;
    drag(&mut t, &[[c, c]]);

    let h = heights_of(&t, layer);
    let rise = |r: u32| -> f32 { h[(80 * size + 80 + r) as usize] - 0.5 };

    // Rises ≈ Depth at the centre.
    assert!(
        (rise(0) - 0.5).abs() < 0.08,
        "the blob's centre rose {:.3}, not ≈ 0.5 (Depth)",
        rise(0)
    );
    // Tapers to nothing at the brush edge — *"na borda nada se move"*.
    assert!(
        rise(40).abs() < 0.02,
        "the blob is still {:.3} loads high at the brush edge (40 px); it must taper to zero",
        rise(40)
    );
    // Monotonically decreasing AND rounded (no flat top): the drop from centre to quarter-radius is real,
    // which a flat Layer plateau would not have. This is the assertion the old "agree with Layer" gate had
    // exactly backwards.
    let (r0, r8, r16) = (rise(0), rise(8), rise(16));
    assert!(
        r0 > r8 && r8 > r16 && r16 > rise(24),
        "the blob's profile is not a monotone dome (centre {r0:.3}, 8px {r8:.3}, 16px {r16:.3})"
    );
    assert!(
        (r0 - r8) > 0.01,
        "the blob's top is FLAT (centre {r0:.3} vs 8 px {r8:.3}) — a flat top is the *mistura com layer* \
         Enio rejected; the ball radius must follow the falloff so the centre is a rounded peak"
    );
}

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

// ── The APPEARANCE: what the artist sees is the COVERAGE, not the height buffer ──────────────────────

/// The paint's width, in texels, on column `x` — measured on the **coverage**, which is what the light
/// multiplies by (`impasto_light::paint_body(cover) = cover`).
///
/// This is the oracle the sibling above should have had. `width_at_half_max` reads `heights`, and `heights`
/// grew: the gate was green while the screen was unchanged, because relief standing over zero coverage is
/// **invisible**. The buffer is the implementation; the coverage is the appearance
/// ([[feedback_oracle_must_model_appearance_not_implementation]] — for the third time on this line, and the
/// first two were mine as well).
fn paint_width(cov: &[u8], size: u32, x: u32) -> usize {
    (MID - BAND..=MID + BAND)
        .filter(|y| cov[((y * size) + x) as usize] > 128)
        .count()
}

fn covers_of(t: &PainterTool, layer: crate::tool::RtLayerId) -> Vec<u8> {
    t.covers
        .get(&layer)
        .map(|c| (**c).clone())
        .unwrap_or_default()
}

/// **THE gate for Enio's second smoke: the PAINT gets wider, not merely the height field.**
///
/// *"inflate não engorda"* — it did not, and every gate said it did. They measured `heights`, which really
/// does spread; but the light shades by the **coverage**, so the new rim of relief stood on bare canvas and
/// rendered nothing at all. The form grew in a buffer nobody looks at.
///
/// So Inflate moves the **matter**: what arrives at a texel came from the ball's argmax, and it brings that
/// texel's coverage, material and colour with it. Paint is a substance; a substance that moves takes its
/// colour with it. (That sentence was already written down — for W4, the advective family. It arrived early,
/// forced by the one verb that grows.)
///
/// **Mutations that must bleed** (both checked): delete the `advect_matter` call from `render_sculpt` — the
/// coverage stops moving and this reads exactly the shipped bug; and return `false` from
/// `SculptMode::moves_matter`.
#[test]
fn the_inflate_fattens_the_paint_and_not_just_the_height_buffer() {
    let size = 200u32;
    let probe_x = 100u32;

    let run = |mode: Option<u8>| -> (Vec<u8>, usize) {
        let (mut t, layer, _) = deposited_stroke(size);
        if let Some(m) = mode {
            arm_sculpt(&mut t, m, 0.5, 1.0);
            let mut b = t.paint.brush;
            b.radius_px = 40.0;
            b.falloff = Falloff::Constant;
            t.paint.brush = b;
            t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
            t.set_sculpt_depth(DEPTH_UP); // +0.5 loads ⇒ a ball of 8 px
            drag(&mut t, &[[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]]);
        }
        let cov = covers_of(&t, layer);
        let w = paint_width(&cov, size, probe_x);
        (cov, w)
    };

    let (_, bare) = run(None);
    let (_, layered) = run(Some(LAYER));
    let (_, inflated) = run(Some(INFLATE));

    assert!(bare > 8, "fixture: the deposit laid no measurable paint");
    assert_eq!(
        layered, bare,
        "Layer changed how WIDE the paint is ({bare} → {layered} texels). It must not: it lays a coat of \
         height inside the footprint and moves no matter at all. If this moved, the oracle is measuring \
         something other than the paint's silhouette and the assert below means nothing."
    );
    assert!(
        inflated >= bare + 8,
        "Inflate left the PAINT {inflated} texels wide; it was {bare}, and Layer left it {layered}. A Depth \
         of 0.5 loads is a ball of {:.0} px, so the form's rim should be pushed out by something of that \
         order. \n\nThis is Enio's smoke of 2026-07-14 (*\"inflate não engorda\"*), and note WHERE it \
         reads: the COVERAGE. The height buffer fattened all along — the light multiplies by the coverage, \
         so relief on bare canvas is invisible, and the form grew somewhere nobody can see.",
        0.5 * UNIT
    );
}

/// **The new rim is PAINT — it has the paint's colour, not the canvas's.**
///
/// Coverage without pixels would light **bare paper in relief**: the shade *modulates* the RGBA that is
/// already there (`rgba[i] = light_pixel(albedo, mul, add)`), it does not create any. So the colour has to
/// travel too, and it travels along the same vector the height did — one ball, one answer to *where did this
/// come from*, so the relief and the colour cannot disagree about it.
///
/// **Mutation that must bleed:** stop writing `rgba` in `advect_matter` (keep coverage + material).
#[test]
fn the_inflated_rim_carries_the_paints_colour() {
    let size = 200u32;
    let (mut t, layer, _) = deposited_stroke(size); // a stroke of [0.1, 0.2, 0.3] — a dark blue
    let before = (*t.canvas_rgba).clone();

    let probe = |rgba: &[u8], i: usize| -> [u8; 4] {
        [
            rgba[i * 4],
            rgba[i * 4 + 1],
            rgba[i * 4 + 2],
            rgba[i * 4 + 3],
        ]
    };
    // Find the paint's own colour (a texel at the stroke's core) and a bare texel just past its edge.
    let cov0 = covers_of(&t, layer);
    let core = (0..size)
        .find(|y| cov0[((y * size) + 100) as usize] > 200)
        .map(|y| (y * size + 100) as usize)
        .expect("fixture: a painted texel on the probe column");
    let edge = (0..size)
        .filter(|y| cov0[((y * size) + 100) as usize] == 0)
        .filter(|y| {
            let d = i64::from(*y) - 100;
            // The deposit's brush is 16 px, so the paint reaches |d| ≈ 16. This texel is 4-6 px BEYOND its
            // rim — bare canvas, and inside the reach of the 16-px ball a full Depth rolls over it.
            (20..=22).contains(&d.abs())
        })
        .map(|y| (y * size + 100) as usize)
        .next()
        .expect("fixture: a bare texel just outside the paint");
    let core_rgb = probe(&before, core);
    assert_ne!(
        probe(&before, edge),
        core_rgb,
        "fixture: the bare texel already has the paint's colour, so the assert below cannot fail"
    );

    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    let mut b = t.paint.brush;
    b.radius_px = 40.0;
    b.falloff = Falloff::Constant;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::super::PaintMode::Sculpt.slot()] = b;
    t.set_sculpt_depth(1.0); // +1.0 loads ⇒ a 16-px ball: it reaches
    drag(&mut t, &[[60.0, 100.0], [100.0, 100.0], [140.0, 100.0]]);

    let after = (*t.canvas_rgba).clone();
    let got = probe(&after, edge);
    let near = |a: u8, b: u8| i32::from(a).abs_diff(i32::from(b)) <= 24;
    assert!(
        near(got[0], core_rgb[0]) && near(got[1], core_rgb[1]) && near(got[2], core_rgb[2]),
        "the inflated rim came out {got:?}; the paint it grew from is {core_rgb:?}. Coverage without pixels \
         lights BARE PAPER in relief — the shade modulates the colour that is there, it does not invent \
         any. The matter has to bring its colour."
    );
    assert!(
        got[3] > 128,
        "the inflated rim is transparent (alpha {}), so there is nothing on screen to light",
        got[3]
    );
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
/// **Mutation that must bleed:** delete the `advect_matter` call from `render_sculpt`. (Checked.)
///
/// Adding a second verb to `moves_matter` does NOT bleed it, and that is worth knowing rather than hiding:
/// the advection reads `memo_src`, and only the ball offset ever writes it — a blur leaves it all zeros, and
/// a zero source means *this matter is its own*. Two independent things hold the invariant up. I claimed the
/// opposite in this comment first, mutated it, and watched the gate stay green
/// ([[feedback_a_mutation_that_survives_may_mean_a_missing_gate]] — third cause: the gate is right, it just
/// does not speak about that).
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

/// **The separable parabolic dilation equals the brute-force `O(N²)` one — the fast path is the true one.**
///
/// The Blob's engine is Felzenszwalb's `O(N)` lower-envelope sweep, twice (x then y). A subtle sign or
/// intersection error would give a plausible-but-wrong dome — the first cut inverted the lift sign and
/// turned a flat top into a peak (the fatten gates caught it, but only on a shaped fixture). This pins the
/// algorithm itself: for a random field, the swept result must match the naïve `max over all p of
/// [f(p) − a·|q−p|²]`, to the visible bit.
///
/// **Mutation that must bleed:** flip the `out_val` lift sign in `ParabolaScratch::transform` (`sign` →
/// `−sign`); the dome inverts and this diverges everywhere.
#[test]
fn the_parabolic_blob_matches_the_brute_force_dilation() {
    use super::super::sculpt_offset::{blob_dilate, unpack_src};
    let (w, h) = (24u32, 18u32);
    let (wu, hu) = (w as usize, h as usize);
    // A deterministic, bumpy field (HR-5: no RNG in a gate).
    let g: Vec<f32> = (0..wu * hu)
        .map(|i| {
            let x = (i % wu) as f32;
            let y = (i / wu) as f32;
            0.3 * (x * 0.7).fract() + 0.5 * (y * 0.4 + x * 0.1).fract()
        })
        .collect();
    let a = 1.0 / (2.0 * 0.5 * 16.0 * 16.0); // the Blob's curvature at Depth 0.5

    let (fast, src) = blob_dilate(&g, w, h, a, true);

    // The naïve dilation: for each output, the max over EVERY source of the parabola, and its argmax.
    let mut worst = 0.0f32;
    let mut src_mismatch = 0u32;
    for qy in 0..hu {
        for qx in 0..wu {
            let mut best = f32::NEG_INFINITY;
            for py in 0..hu {
                for px in 0..wu {
                    let d2 = ((qx as f32 - px as f32).powi(2)) + ((qy as f32 - py as f32).powi(2));
                    best = best.max(g[py * wu + px] - a * d2);
                }
            }
            let o = qy * wu + qx;
            worst = worst.max((fast[o] - best).abs());
            // The argmax can tie; only count a mismatch when the fast pick's VALUE is worse than the
            // brute pick's (a real error), not when two equal-value sources were chosen differently.
            let (dx, dy) = unpack_src(src[o]);
            let (sx, sy) = (qx as i64 + dx, qy as i64 + dy);
            if sx >= 0 && sy >= 0 && (sx as usize) < wu && (sy as usize) < hu {
                let picked = g[(sy as usize) * wu + (sx as usize)]
                    - a * ((qx as f32 - sx as f32).powi(2) + (qy as f32 - sy as f32).powi(2));
                if picked < best - 1e-4 {
                    src_mismatch += 1;
                }
            }
        }
    }
    assert!(
        worst < 1e-4,
        "the separable parabolic dome differs from the brute-force one by {worst:e} — the O(N) sweep is not \
         the O(N²) truth, so the fast Blob is a different (wrong) shape"
    );
    assert_eq!(
        src_mismatch, 0,
        "the composed argmax pointed at a source the brute force beats — the paint would follow the wrong \
         texel and the fattening would carry the wrong colour"
    );
}
