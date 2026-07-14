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

/// **On a flat, Inflate IS Layer — and that is geometry, not the bug.**
///
/// Offsetting a *plane* along its normal is a *translation*: every offset operator, by any algorithm, raises
/// flat ground by exactly the offset. (Blender's Inflate is likewise Draw on a flat plane.) So the two verbs
/// agreeing over the flat interior of a stroke is **correct**, and it is precisely the observation that looks
/// like the bug — it is what Enio saw. What separates them is what they do to the *shape*, which is the
/// sibling gate below.
///
/// This one exists so that the next person to read that report does not "fix" the agreement back into an
/// inversion. The `−` in `Depth · n_z` is one keystroke away, it makes this gate greener than green, and it
/// is exactly wrong.
///
/// **Mutation that must bleed:** scale the target by any function of the local slope that is not `1` at zero
/// slope — e.g. restore `p + depth * inflate_nz(…)` and *also* dent the flat.
#[test]
fn on_a_flat_the_inflate_and_the_layer_agree_and_that_is_geometry() {
    let size = 128u32;
    let run = |mode: u8| -> Vec<f32> {
        let (mut t, layer, _) = sculpt_canvas(size);
        let n = (size * size) as usize;
        // Dead-flat paint, one load thick. No structure at all: the one surface on which the two verbs are
        // required to be indistinguishable.
        t.heights.insert(layer, Arc::new(vec![1.0f32; n]));
        t.covers.insert(layer, Arc::new(vec![255u8; n]));
        t.sync_relief_flags();
        arm_sculpt(&mut t, mode, 0.5, 1.0);
        t.set_sculpt_depth(DEPTH_UP);
        drag(&mut t, &[[50.0, 64.0], [64.0, 64.0], [78.0, 64.0]]);
        heights_of(&t, layer)
    };
    let layer = run(LAYER);
    let inflate = run(INFLATE);

    let worst = layer
        .iter()
        .zip(&inflate)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        worst < 1e-3,
        "on dead-flat paint Inflate and Layer differ by up to {worst:.4} loads. They must not: an offset \
         along the normal of a PLANE is a translation. If this fails, the kernel is doing something to a \
         flat that a ball cannot do."
    );
    // …and the fixture is not vacuously equal because neither verb did anything.
    let risen = inflate.iter().filter(|v| **v > 1.4).count();
    assert!(
        risen > 200,
        "fixture: Inflate raised nothing (only {risen} texels above 1.4), so the equality above is the \
         equality of two no-ops"
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

/// **A tile of the PRODUCT's offset memo is bit-for-bit the whole canvas's offset.**
///
/// The sibling of the blur's gate, and the same argument (the read window is grown by `⌈ρ⌉`, and a truncated
/// window's edge IS the canvas's). It is here because the tile memo now serves TWO kernels and takes its
/// window growth from [`MemoKey::reach`](super::super::sculpt::MemoKey::reach). A `reach` that answered for
/// the blur while the offset ran would be wrong ONLY at a tile seam — 64 px apart, in a thin line, in a tool
/// whose whole job is to change the relief. The single hardest artefact in this file to attribute.
///
/// ## Two drafts of this gate were green with the bug in. Both failures were the FIXTURE's
///
/// **Draft 1 walked the tiles itself**, calling `ball_offset_into` with a `reach` it computed on the spot. It
/// was byte-exact, and it survived the mutation that makes `MemoKey::reach` return `0` for the offset —
/// because it never asked `MemoKey::reach` anything. A gate that re-implements the product tests the copy.
///
/// **Draft 2 drove the product, and was still green** — because it ran on the *deposited stroke* used by the
/// gates above, and that stroke cannot see a seam. It is a straight horizontal band: **constant along x**, so
/// a max over a disc of it is achieved at `dx = 0` whatever the disc is truncated to, and truncation in `x`
/// changes nothing; and it happens to fit **entirely inside one 64-px tile row** in `y` (the paint plus the
/// ball spans 76..124), so the horizontal seams never look at it either. A realistic fixture, proving nothing
/// ([[feedback_a_gate_only_proves_what_its_fixture_contains]] — for the third time on this line).
///
/// So it runs on `sculpt_canvas`'s ridge-with-a-sawtooth, which is what the **blur's** memo gate uses and for
/// the same reason: a memo gate needs structure at every seam, in both axes. Realism belongs to the gates that
/// measure the *tool*; this one measures the *tiling*.
///
/// **Mutation that must bleed:** return `0` (or the blur's radius) from `reach`'s `Offset` arm. Checked: it
/// does now — and it did not in either earlier draft.
#[test]
fn the_offset_memo_is_byte_identical_to_a_whole_canvas_offset() {
    use super::super::Region;
    use super::super::sculpt_offset::ball_offset_into;

    let size = 200u32; // deliberately not a multiple of the 64-px tile: the edge tiles are truncated
    let (mut t, _layer, _) = sculpt_canvas(size);

    arm_sculpt(&mut t, INFLATE, 0.5, 1.0);
    t.set_sculpt_depth(DEPTH_UP); // +0.5 loads ⇒ a ball of 8 px
    assert!(
        t.ensure_sculpt_session(),
        "fixture: no session — there is no memo to fill"
    );
    let key = t.paint.sculpt.memo_key();
    let pre = (*t.paint.sculpt.pre).clone();

    // The product's own memo, filled through its own tile loop, over the whole canvas.
    let whole = Region {
        x: 0,
        y: 0,
        w: size,
        h: size,
    };
    t.ensure_memo_tiles(whole, key);
    let memo = t.paint.sculpt.memo.clone();

    // The oracle: the same kernel, once, over the entire canvas — no tiles, no windows, nothing to get wrong.
    let mut oracle = vec![0.0f32; (size * size) as usize];
    ball_offset_into(&pre, size, size, 0.5 * UNIT, whole, &mut oracle);

    let differing = memo
        .iter()
        .zip(&oracle)
        .filter(|(a, b)| a.to_bits() != b.to_bits())
        .count();
    assert_eq!(
        differing, 0,
        "{differing} texels of the tiled memo differ from the whole-canvas offset. The memo is only \
         legitimate if a tile is the canvas's answer, restricted — otherwise every 64-px seam is a place \
         where the relief quietly changes, and nothing on screen says which side is right."
    );
    // The fixture must contain paint the ball actually MOVED, or byte-equality is the equality of two copies
    // of `pre` ([[feedback_zero_valued_fixture_is_a_gate_that_cannot_fail]]).
    let moved = oracle
        .iter()
        .zip(&pre)
        .filter(|(a, b)| (*a - *b).abs() > 1e-4)
        .count();
    assert!(
        moved > 1000,
        "fixture: the offset moved only {moved} texels, so the identity above proves nothing"
    );
}
