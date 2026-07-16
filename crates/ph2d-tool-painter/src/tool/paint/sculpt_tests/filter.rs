//! Gates for the **whole-layer filter** (W5b — `super::super::sculpt_filter`).
//!
//! The claim the whole feature rests on: *the filter is the stroke's engine driven by a uniform `amount`*.
//! So the gates are about the seams that claim buys — it reaches where no brush went, it obeys the
//! Selection, Strength means the same thing it means to a stroke, and it refuses the verbs whose target
//! needs a footprint. Each names the mutation that reddens it.

use super::super::sculpt_filter::FilterScope;
use super::super::sculpt_tests::{arm_sculpt, sculpt_canvas};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::RasterEditTool; // `set_source` on the blank-layer gate
use std::sync::Arc;

/// How much the relief moved, and the farthest texel from the canvas centre that moved at all.
fn moved(
    t: &PainterTool,
    layer: crate::tool::RtLayerId,
    before: &[f32],
    size: u32,
) -> (usize, f32) {
    let now = t.heights.get(&layer).expect("relief");
    let mut count = 0usize;
    let mut peak = 0.0f32;
    for (i, (&a, &b)) in now.iter().zip(before.iter()).enumerate() {
        let d = (a - b).abs();
        if d > 1e-4 {
            count += 1;
            peak = peak.max(d);
        }
        let _ = i;
        let _ = size;
    }
    (count, peak)
}

/// **The filter reaches the whole layer — including where no brush ever went.**
///
/// This is the feature in one sentence (plan §8, W5's second half): the artist smooths the WHOLE relief
/// without painting it. A stroke can only change what it touched; the filter changes everything the
/// Selection allows.
///
/// **Mutation that must bleed:** render only the session's bbox instead of the full canvas rect (or leave
/// `amount` zero-filled) — the count collapses to a fraction of the layer.
#[test]
fn the_filter_reaches_the_whole_layer_not_only_where_a_brush_went() {
    const SIZE: u32 = 200;
    let (mut t, layer, before) = sculpt_canvas(SIZE);
    arm_sculpt(&mut t, 0, 0.5, 1.0); // Smooth
    assert!(t.filter_sculpt_layer(FilterScope::Layer), "the filter ran");
    let (count, peak) = moved(&t, layer, &before, SIZE);
    // The fixture's saw-tooth covers the whole canvas, so a whole-layer Smooth must move a LOT of it.
    // (Not every texel: the saw is every third, and the blur leaves flats flat — which is the point.)
    assert!(
        count > (SIZE * SIZE) as usize / 4,
        "the filter moved only {count} texels of {} — it is not reaching the whole layer",
        SIZE * SIZE
    );
    assert!(
        peak > 0.02,
        "the filter barely moved anything (peak {peak})"
    );
    // …and it reached the CORNERS, which no brush in this fixture ever visited.
    let now = t.heights.get(&layer).expect("relief");
    let corner = (SIZE as usize) * 3 + 3;
    assert!(
        (now[corner] - before[corner]).abs() > 1e-4,
        "the far corner is untouched — the filter is still thinking in brush footprints"
    );
}

/// **The filter refuses every verb that does not RESHAPE** — and refusing is not a no-op, it is the honest
/// answer (`sculpt_filter::filters_layer`).
///
/// Two exclusions, two reasons. Flatten / Scrape / Fill / Chisel pull toward a plane least-squares fitted to
/// the dab: a whole layer has no dab, so "the layer's plane" is a DIFFERENT operation (flatten the art to
/// its mean plane) and a verb to design, not a flag. The Chisel is refused twice — its V folds around the
/// stroke's axis. And **Layer** (Enio, 2026-07-16): its target is the constant `pre + Depth`, so filtering a
/// layer with it lifts every texel by the same `k·Depth` — and **the light reads `∇h`**, which a constant
/// does not change. It would not move one pixel. I shipped it reasoning it "fell out for free"; that is how
/// dead knobs get shipped.
///
/// **Mutation that must bleed:** make `filters_layer` return `true` unconditionally (or re-admit
/// `SculptMode::Layer`) — the refusal reports success.
#[test]
fn the_filter_refuses_the_verbs_that_do_not_reshape() {
    for mode in [2u8, 3, 4, 5, 6] {
        // Flatten · Scrape · Fill · Chisel (targets fitted to the footprint) · Layer (a translation:
        // the light reads the gradient, so a uniform lift is invisible — a dead knob)
        const SIZE: u32 = 200;
        let (mut t, layer, before) = sculpt_canvas(SIZE);
        arm_sculpt(&mut t, mode, 0.5, 1.0);
        assert!(
            !t.filter_sculpt_layer(FilterScope::Layer),
            "mode {mode}: a footprint-fitted verb must refuse the whole-layer filter"
        );
        let now = t.heights.get(&layer).expect("relief");
        assert!(
            now.iter().zip(before.iter()).all(|(a, b)| a == b),
            "mode {mode}: the refusal still moved the relief — a refusal must change NOTHING"
        );
    }
    // …and the ones it accepts really do run, so the gate above is not vacuously green.
    for mode in [0u8, 1, 7] {
        // Smooth · Sharpen · Inflate — the three that RESHAPE
        const SIZE: u32 = 200;
        let (mut t, _l, _b) = sculpt_canvas(SIZE);
        arm_sculpt(&mut t, mode, 0.5, 1.0);
        assert!(
            t.filter_sculpt_layer(FilterScope::Layer),
            "mode {mode}: this verb's target is a function of `pre` — it must filter"
        );
    }
}

/// **The Selection is the filter's edge** — outside it the relief is byte-identical.
///
/// The brush hands the Selection to the kernel so it attenuates each dab as it lands; the filter is one
/// "dab" covering everything, so it multiplies the mask in once. Same law, one application. A filter that
/// ignored the Selection would be the only tool in the Painter that does.
///
/// **Mutation that must bleed:** drop the mask branch (`amount.fill(strength)` unconditionally) — the
/// unselected half moves.
#[test]
fn the_filter_honours_the_selection_and_leaves_the_rest_byte_identical() {
    const SIZE: u32 = 200;
    let (mut t, layer, before) = sculpt_canvas(SIZE);
    arm_sculpt(&mut t, 0, 0.5, 1.0); // Smooth
    // Select the left half, hard-edged.
    let n = (SIZE * SIZE) as usize;
    let mask: Vec<u8> = (0..n)
        .map(|i| if (i as u32 % SIZE) < SIZE / 2 { 255 } else { 0 })
        .collect();
    t.paint.selection_mask = Arc::new(mask);
    t.paint.selection_active = true;
    assert!(
        t.selection_restricts_paint(),
        "fixture: the Selection must actually restrict, else this gate is vacuous"
    );
    assert!(t.filter_sculpt_layer(FilterScope::Layer), "the filter ran");
    let now = t.heights.get(&layer).expect("relief");
    let (mut inside, mut outside) = (0usize, 0usize);
    for i in 0..n {
        if (now[i] - before[i]).abs() > 1e-4 {
            if (i as u32 % SIZE) < SIZE / 2 {
                inside += 1;
            } else {
                outside += 1;
            }
        }
    }
    assert!(
        inside > 1000,
        "the selected half was not filtered ({inside})"
    );
    assert_eq!(
        outside, 0,
        "{outside} texels OUTSIDE the selection moved — the Selection is the tool's edge, not a hint"
    );
}

/// **Strength means to the filter what it means to a stroke** — how far along the travel it goes.
///
/// The render's `k = amount.clamp(0, 1)`; the filter writes Strength straight into `amount`, so half
/// Strength must land half the travel. That is what makes the filter need no knob of its own: the card
/// already shows this number.
///
/// **Mutation that must bleed:** fill `amount` with `1.0` instead of the Strength — the two runs become
/// identical and the ratio collapses to 1.
#[test]
fn the_filters_strength_is_how_far_along_the_travel_it_goes() {
    const SIZE: u32 = 200;
    let (mut full, layer, before) = sculpt_canvas(SIZE);
    arm_sculpt(&mut full, 0, 0.5, 1.0);
    assert!(full.filter_sculpt_layer(FilterScope::Layer));
    let (mut half, _l, _b) = sculpt_canvas(SIZE);
    arm_sculpt(&mut half, 0, 0.5, 0.5);
    assert!(half.filter_sculpt_layer(FilterScope::Layer));

    let (hf, hh) = (
        full.heights.get(&layer).expect("relief"),
        half.heights.get(&layer).expect("relief"),
    );
    // Where the full filter moved the relief, the half filter moved it about half as far — the travel is
    // linear in `k`, so the ratio is exact up to the texels the blur left alone.
    let (mut n, mut sum) = (0usize, 0.0f64);
    for i in 0..(SIZE * SIZE) as usize {
        let d_full = hf[i] - before[i];
        if d_full.abs() < 0.02 {
            continue; // no travel here to halve
        }
        let d_half = hh[i] - before[i];
        sum += f64::from(d_half / d_full);
        n += 1;
    }
    assert!(n > 500, "fixture: not enough moved texels to measure ({n})");
    let ratio = sum / n as f64;
    assert!(
        (ratio - 0.5).abs() < 0.02,
        "half Strength moved {ratio:.3} of the full travel, not 0.5 — Strength is not driving `k`"
    );
}

/// **One filter, one undo step — and it puts the relief back exactly.**
///
/// The filter is an operation, like a stroke, so it commits like one (`snapshot_model` →
/// `commit_structural_edit`). The relief is the thing it changed, so the relief is the thing undo owes.
///
/// **Mutation that must bleed:** drop the `commit_structural_edit` (or the `snapshot_model` before it) —
/// the undo either does nothing or takes the wrong step.
#[test]
fn the_filter_is_one_undo_step_and_the_relief_comes_back() {
    const SIZE: u32 = 200;
    let (mut t, layer, before) = sculpt_canvas(SIZE);
    arm_sculpt(&mut t, 0, 0.5, 1.0);
    assert!(t.filter_sculpt_layer(FilterScope::Layer));
    assert!(
        t.heights
            .get(&layer)
            .expect("relief")
            .iter()
            .zip(before.iter())
            .any(|(a, b)| (a - b).abs() > 1e-4),
        "fixture: the filter changed something, else the undo below is vacuous"
    );
    assert!(t.undo_last(), "one undo step");
    let now = t.heights.get(&layer).expect("relief");
    assert!(
        now.iter().zip(before.iter()).all(|(a, b)| a == b),
        "one undo did not put the relief back — the filter's step is not the whole operation"
    );
}

/// **A layer with no relief is nothing to filter — and the tool says so instead of inventing some.**
///
/// The sculpt session refuses to open on a layer with no height plane (§5: *"a sculpt brush creates no
/// paint"*), and the filter inherits that honesty rather than allocating a plane of zeros and lighting it.
///
/// **Mutation that must bleed:** have the filter allocate `pre` when it is missing — it starts reporting
/// success on a blank layer, and a blank layer grows a relief nobody painted.
#[test]
fn a_layer_with_no_relief_has_nothing_to_filter() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 64 * 64 * 4], 64, 64);
    arm_sculpt(&mut t, 0, 0.5, 1.0);
    let layer = t.layers.active().expect("a layer");
    assert!(
        !t.filter_sculpt_layer(FilterScope::Layer),
        "a layer with no relief must refuse — there is nothing to smooth"
    );
    assert!(
        t.heights
            .get(&layer)
            .is_none_or(|h| h.iter().all(|&v| v == 0.0)),
        "the refusal invented a relief"
    );
}

/// A canvas with real impasto relief laid by a real stroke in ONE band, so "the last stroke" is a fact of
/// the product and the rest of the layer is a control surface. Returns the tool, the layer, and the relief
/// as the stroke left it.
fn canvas_with_one_stroke(size: u32) -> (PainterTool, crate::tool::RtLayerId, Vec<f32>) {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    t.set_brush_size_px(10.0);
    t.toggle_brush_impasto();
    let cp = |p: [f32; 2], phase| CanvasPointer {
        pos: p,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(cp([20.0, 30.0], PointerPhase::Down));
    for i in 1..=8u8 {
        t.on_canvas_pointer(cp([20.0 + 8.0 * f32::from(i), 30.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([84.0, 30.0], PointerPhase::Up));
    let layer = t.layers.active().expect("a layer");
    let relief = t
        .heights
        .get(&layer)
        .expect("the stroke laid relief")
        .to_vec();
    (t, layer, relief)
}

/// **Filter Stroke touches the last stroke and NOTHING else** (Enio, 2026-07-16).
///
/// The whole point of the second button: smooth the stroke you just made without touching the art around
/// it. The mask is that stroke's own paint envelope (`relief.live_paint`) — the record the Body card
/// already re-derives it from — so it is the honest answer to *"where did the last stroke go"*, and it
/// carries the stroke's **soft falloff edge**, so the filter feathers exactly where the paint did.
///
/// **Mutation that must bleed:** ignore the scope (fill `amount` uniformly) — the far half of the layer
/// moves and `untouched` stops being zero.
#[test]
fn filter_stroke_touches_the_last_stroke_and_nothing_else() {
    const SIZE: u32 = 120;
    let (mut t, layer, before) = canvas_with_one_stroke(SIZE);
    // Relief far from the stroke, so "it left the rest alone" is a claim with something to be wrong about.
    {
        let h = Arc::make_mut(t.heights.get_mut(&layer).expect("relief"));
        for y in 80..110u32 {
            for x in 10..110u32 {
                h[(y * SIZE + x) as usize] = if (x + y).is_multiple_of(3) { 0.6 } else { 0.2 };
            }
        }
    }
    let before_far: Vec<f32> = t.heights.get(&layer).expect("relief").to_vec();
    let _ = before;

    arm_sculpt(&mut t, 0, 0.5, 1.0); // Smooth
    assert!(
        t.can_filter_last_stroke(),
        "fixture: a stroke was made on this layer, so the button must be on offer"
    );
    assert!(
        t.filter_sculpt_layer(FilterScope::LastStroke),
        "the filter ran"
    );

    let now = t.heights.get(&layer).expect("relief");
    let (mut on_stroke, mut far) = (0usize, 0usize);
    for i in 0..(SIZE * SIZE) as usize {
        if (now[i] - before_far[i]).abs() > 1e-4 {
            let y = i as u32 / SIZE;
            if y < 60 {
                on_stroke += 1;
            } else {
                far += 1;
            }
        }
    }
    assert!(
        on_stroke > 100,
        "the last stroke was not filtered ({on_stroke} texels moved)"
    );
    assert_eq!(
        far, 0,
        "{far} texels far from the last stroke moved — Filter Stroke is filtering the LAYER"
    );
}

/// **Filter Stroke is strictly narrower than Filter Layer** — the two buttons are not the same button.
///
/// A gate that only checked "the stroke moved" would pass for a Filter Layer wired to the wrong id. This
/// pins the DIFFERENCE: the layer scope reaches the relief the stroke never went near, and the stroke scope
/// does not.
///
/// **Mutation that must bleed:** route `PAINTER_SCULPT_FILTER_STROKE` to `FilterScope::Layer` — the two
/// counts converge.
#[test]
fn filter_stroke_is_strictly_narrower_than_filter_layer() {
    const SIZE: u32 = 120;
    let count_moved = |scope: FilterScope| -> usize {
        let (mut t, layer, _) = canvas_with_one_stroke(SIZE);
        {
            let h = Arc::make_mut(t.heights.get_mut(&layer).expect("relief"));
            for y in 80..110u32 {
                for x in 10..110u32 {
                    h[(y * SIZE + x) as usize] = if (x + y).is_multiple_of(3) { 0.6 } else { 0.2 };
                }
            }
        }
        let before: Vec<f32> = t.heights.get(&layer).expect("relief").to_vec();
        arm_sculpt(&mut t, 0, 0.5, 1.0);
        assert!(t.filter_sculpt_layer(scope));
        let now = t.heights.get(&layer).expect("relief");
        now.iter()
            .zip(before.iter())
            .filter(|(a, b)| (*a - *b).abs() > 1e-4)
            .count()
    };
    let layer_scope = count_moved(FilterScope::Layer);
    let stroke_scope = count_moved(FilterScope::LastStroke);
    assert!(
        stroke_scope > 0 && layer_scope > stroke_scope * 2,
        "Filter Layer moved {layer_scope} texels and Filter Stroke {stroke_scope} — the stroke scope is \
         not narrower, so the two buttons are doing the same thing"
    );
}

/// **With no last stroke there is no stroke to filter — and the button is never offered.**
///
/// `live_paint` is empty until a stroke commits, and it belongs to the layer that stroke was on. Filtering
/// layer B by the envelope of a stroke made on layer A would mask B's relief with the silhouette of a
/// stroke that was never there — a filter shaped like a ghost.
///
/// **Mutation that must bleed:** drop the `live_relief_layer != active` check (or the emptiness check) in
/// `live_stroke_envelope` — the refusals become successes.
#[test]
fn with_no_last_stroke_on_this_layer_there_is_nothing_to_filter() {
    const SIZE: u32 = 200;
    // A layer with relief but NO stroke behind it (the fixture plants the field directly).
    let (mut t, _layer, _before) = sculpt_canvas(SIZE);
    arm_sculpt(&mut t, 0, 0.5, 1.0);
    assert!(
        !t.can_filter_last_stroke(),
        "there is no last stroke, so the button must not be on offer"
    );
    assert!(
        !t.filter_sculpt_layer(FilterScope::LastStroke),
        "a filter scoped to a stroke that does not exist must refuse"
    );
    // …and the layer scope, which needs no stroke, still runs — so the refusal above is about the SCOPE.
    assert!(
        t.filter_sculpt_layer(FilterScope::Layer),
        "the layer scope needs no last stroke"
    );
}
