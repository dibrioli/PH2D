//! Gates for the **whole-layer filter** (W5b — `super::super::sculpt_filter`).
//!
//! The claim the whole feature rests on: *the filter is the stroke's engine driven by a uniform `amount`*.
//! So the gates are about the seams that claim buys — it reaches where no brush went, it obeys the
//! Selection, Strength means the same thing it means to a stroke, and it refuses the verbs whose target
//! needs a footprint. Each names the mutation that reddens it.

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
    assert!(t.filter_sculpt_layer(), "the filter ran");
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

/// **The filter refuses the verbs whose target is fitted to the FOOTPRINT** — and refusing is not a
/// no-op, it is the honest answer (`sculpt_filter::filters_layer`).
///
/// Flatten / Scrape / Fill / Chisel pull toward a plane least-squares fitted to the dab. A whole layer has
/// no dab, so "the whole layer's plane" is a DIFFERENT operation (flatten the art to its mean plane) and a
/// verb to design — not a flag. The Chisel is refused twice: its V folds around the stroke's axis.
///
/// **Mutation that must bleed:** make `filters_layer` return `true` unconditionally — the Plane verbs then
/// render against an unfitted `plane_sum` and the relief moves (or the call reports success).
#[test]
fn the_filter_refuses_the_verbs_whose_target_is_a_footprint() {
    for mode in [2u8, 3, 4, 5] {
        // Flatten · Scrape · Fill · Chisel
        const SIZE: u32 = 200;
        let (mut t, layer, before) = sculpt_canvas(SIZE);
        arm_sculpt(&mut t, mode, 0.5, 1.0);
        assert!(
            !t.filter_sculpt_layer(),
            "mode {mode}: a footprint-fitted verb must refuse the whole-layer filter"
        );
        let now = t.heights.get(&layer).expect("relief");
        assert!(
            now.iter().zip(before.iter()).all(|(a, b)| a == b),
            "mode {mode}: the refusal still moved the relief — a refusal must change NOTHING"
        );
    }
    // …and the ones it accepts really do run, so the gate above is not vacuously green.
    for mode in [0u8, 1, 6, 7] {
        // Smooth · Sharpen · Layer · Inflate
        const SIZE: u32 = 200;
        let (mut t, _l, _b) = sculpt_canvas(SIZE);
        arm_sculpt(&mut t, mode, 0.5, 1.0);
        assert!(
            t.filter_sculpt_layer(),
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
    assert!(t.filter_sculpt_layer(), "the filter ran");
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
    assert!(full.filter_sculpt_layer());
    let (mut half, _l, _b) = sculpt_canvas(SIZE);
    arm_sculpt(&mut half, 0, 0.5, 0.5);
    assert!(half.filter_sculpt_layer());

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
    assert!(t.filter_sculpt_layer());
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
        !t.filter_sculpt_layer(),
        "a layer with no relief must refuse — there is nothing to smooth"
    );
    assert!(
        t.heights
            .get(&layer)
            .is_none_or(|h| h.iter().all(|&v| v == 0.0)),
        "the refusal invented a relief"
    );
}
