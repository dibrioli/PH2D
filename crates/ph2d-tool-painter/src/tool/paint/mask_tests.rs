//! Gates for the **Mask coverage law** (the rewrite of 2026-07-25, doc 25 §13.9).
//!
//! Each one states a property of the *look* — the transition band, the valley between neighbours —
//! not a restatement of the implementation, because the defect they exist to catch lived for months
//! under a green suite whose fixtures could not grow a feather at all (`hardness = 1`, a hard disk).
//! The oracle helpers are the probe's ([`super::mask_probe`]), so the numbers here mean exactly what
//! the measurements that motivated them meant.
//!
//! Every bar below is a number the measurement GAVE, with the old law's value next to it:
//!
//! | property | old (product build-up) | new (Krita-Wash envelope) |
//! |---|---|---|
//! | one pass, transition band | 3.53 px | **6.21 px** (analytic feather 5.40) |
//! | 15 passes, transition band | 1.38 px | **2.10 px** |
//! | scrub in one pen-down, body Δ | 118 levels | **2 levels** |
//! | scrub in one pen-down, band | 3.53 → 1.88 px | **6.21 → 6.18 px** |
//! | two neighbours, valley | fills (hardened) | **fills (sums)** |

use super::mask_probe::{band_px, coverage, cp, cross_x, mask_tool, vstroke};
use crate::tool::PainterTool;
use crate::tool::paint::PaintMode;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};
use ph2d_painter_brush::StrokeCoverLaw;

const S: u32 = 256;

/// The mask brush's default falloff (`Smooth`, `w = t²(3 − 2t)` with `t = 1 − d/r`) crosses 0.9 at
/// `d = 0.736·r` and 0.1 at `d = 0.804·… ` — solved numerically below rather than pinned, so the bar
/// tracks the BRUSH instead of encoding one fixture's radius.
fn analytic_feather_px(radius: f32) -> f32 {
    let d_at = |target: f32| {
        // Bisection on w(t) = t²(3−2t), monotone on [0,1].
        let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
        for _ in 0..40 {
            let mid = 0.5 * (lo + hi);
            if mid * mid * (3.0 - 2.0 * mid) < target {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        radius * (1.0 - 0.5 * (lo + hi)) // t → distance
    };
    d_at(0.1) - d_at(0.9)
}

/// **THE law.** Scrubbing back and forth WITHOUT lifting the pen must not move the feather: the
/// stroke's coverage is the ENVELOPE of its dabs (Krita's Wash / Alpha Darken), so a dab that re-lays
/// what is already there adds nothing. This is the half of the reported "muitas passadas" defect that
/// happens inside a single gesture, and it is where the old law was worst — it drove the body 118
/// levels darker and collapsed the band from 3.53 px to 1.88 px in ONE pen-down.
///
/// The oracle deliberately ignores the stroke's END CAPS: the scrub finishes where it started, so its
/// last dab lands at the other end of the segment and the two strokes genuinely differ there (measured
/// 172 levels at `y = 202`, which is geometry, not the law). The BODY is where the law lives.
///
/// Mutation that must bleed: `stroke_cover_law` returning `BuildUp` for Mask mode.
#[test]
fn a_scrub_within_one_mask_stroke_cannot_move_the_feather() {
    let single = {
        let mut t = mask_tool(S);
        vstroke(&mut t, 128.0, 60.0, 200.0, 25);
        coverage(&t, S)
    };
    let scrubbed = {
        let mut t = mask_tool(S);
        t.on_canvas_pointer(cp([128.0, 60.0], PointerPhase::Down));
        for leg in 0..4u32 {
            let (a, b) = if leg % 2 == 0 {
                (60.0, 200.0)
            } else {
                (200.0, 60.0)
            };
            for i in 1..=25u32 {
                let y = a + (b - a) * (i as f32) / 25.0;
                t.on_canvas_pointer(cp([128.0, y], PointerPhase::Move));
            }
        }
        t.on_canvas_pointer(cp([128.0, 60.0], PointerPhase::Up));
        let _ = t.take_preview_arc();
        coverage(&t, S)
    };
    let body = (90..170)
        .flat_map(|y| (100..160).map(move |x| y * S as usize + x))
        .map(|i| (scrubbed[i] - single[i]).abs())
        .fold(0.0_f32, f32::max);
    let levels = (body * 255.0).round();
    assert!(
        levels <= 4.0,
        "a scrub must be inert in the stroke's body (measured 2 of 255 under the envelope, \
         118 under the old build-up); got {levels} levels"
    );
    let (b1, b4) = (band_px(&single, S, 130), band_px(&scrubbed, S, 130));
    assert!(
        (b4 - b1).abs() / b1 < 0.05,
        "scrubbing must not narrow the feather (single {b1:.2} px, scrubbed {b4:.2} px; \
         the old law went 3.53 -> 1.88)"
    );
}

/// **One pass shows the brush you chose.** The transition band a single mask stroke leaves must be at
/// least the brush's own analytic feather — the old law pre-hardened it (3.53 px against an analytic
/// 5.40 px at r = 10), which is why even ONE pass read harder than the cursor promised.
///
/// Mutation that must bleed: `BuildUp` for Mask mode (3.53 px).
#[test]
fn one_mask_pass_lays_the_brushs_own_feather() {
    let mut t = mask_tool(S);
    let r = t.paint.brush.radius_px;
    vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    let band = band_px(&coverage(&t, S), S, 130);
    let want = analytic_feather_px(r);
    assert!(
        band >= want * 0.9,
        "one pass must lay the brush's own feather: got {band:.2} px, brush's analytic feather is \
         {want:.2} px (r = {r}); the old product build-up gave 3.53 px"
    );
}

/// **The reported picture.** After the pass count a human actually does, the edge must still be a
/// RAMP — a boundary thinner than a texel is what stair-steps on a curve, and that is the "serrilhado".
///
/// ⚠️ This is NOT invariance, and the doc says so: cross-stroke build-up narrows the ramp as
/// `N^(−1/2)` (measured 6.21 / 2.10 / 1.94 / 1.55 px at 1 / 15 / 30 / 45 passes), and the only law
/// that would freeze it is the cross-stroke union — which cannot fill the valley between two strokes
/// and was rejected for exactly that (doc 25 §13.8). What the envelope buys is ~1.5× the ramp at every
/// pass count, plus a scrub that costs nothing.
///
/// Mutation that must bleed: `BuildUp` for Mask mode (1.38 px at fifteen passes).
#[test]
fn the_mask_edge_still_has_an_anti_aliased_ramp_after_fifteen_passes() {
    let mut t = mask_tool(S);
    for _ in 0..15 {
        vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    }
    let band = band_px(&coverage(&t, S), S, 130);
    assert!(
        band >= 1.8,
        "fifteen passes must leave a ramp at least ~2 texels wide (measured 2.10 px under the \
         envelope, 1.38 px under the old build-up); got {band:.2} px"
    );
}

/// **No seams.** Two overlapping strokes must SUM where they meet, strictly darker than either alone.
/// The rejected §13.6 attempt folded strokes by `min` (a union), which left the valley at the *larger*
/// of the two tails — lighter than its surroundings, i.e. the white lines Enio reported.
///
/// Mutation that must bleed: keeping the coverage buffer across strokes (never clearing it in
/// `paint_begin`), which turns the per-stroke envelope into a global union — the valley then reads the
/// same as a single stroke.
#[test]
fn neighbour_mask_strokes_sum_where_they_overlap() {
    let lone = {
        let mut t = mask_tool(S);
        let r = t.paint.brush.radius_px;
        vstroke(&mut t, 128.0 - r * 0.5, 60.0, 200.0, 25);
        coverage(&t, S)
    };
    let mut t = mask_tool(S);
    let r = t.paint.brush.radius_px;
    for x in [128.0 - r * 0.5, 128.0 + r * 0.5] {
        vstroke(&mut t, x, 60.0, 200.0, 25);
    }
    let both = coverage(&t, S);
    let mid = 130usize * S as usize + 128;
    assert!(
        both[mid] > lone[mid] + 0.1,
        "the second stroke must ADD in the valley (one stroke {:.3}, two {:.3}) — a union would \
         leave it at the larger tail and read as a light seam",
        lone[mid],
        both[mid]
    );
    // …and the valley must not be the darkest-lightest outlier of the pair: with two strokes half a
    // radius apart the midline reads at least three quarters of the lane centres.
    let centre = 130usize * S as usize + (128.0 - r * 0.5) as usize;
    assert!(
        both[mid] >= both[centre] * 0.75,
        "valley {:.3} vs lane centre {:.3}: the overlap of two half-radius-apart strokes must not \
         dip below three quarters of the lanes",
        both[mid],
        both[centre]
    );
}

/// **The pigment path never asks for the coverage channel's law.** The whole byte-identity claim for
/// normal paint rests on this door answering `BuildUp` (or nothing) for every mode that is not Mask —
/// the epoch-projection attempt of §13.7 leaked into the brush precisely because a mask-shaped
/// decision reached a pigment stroke.
#[test]
fn only_the_mask_asks_for_the_coverage_channels_law() {
    let brush = ph2d_painter_brush::BrushSpec::default();
    for mode in [
        PaintMode::Paint,
        PaintMode::Smear,
        PaintMode::Knife,
        PaintMode::Blur,
        PaintMode::Clone,
        PaintMode::Sculpt,
        PaintMode::Inpaint,
        PaintMode::Selection,
        PaintMode::WetPaint,
        PaintMode::Mask,
    ] {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (S * S * 4) as usize], S, S);
        t.paint.paint_mode = mode;
        let law = t.stroke_cover_law(&brush);
        if matches!(mode, PaintMode::Mask) {
            assert_eq!(
                law,
                Some(StrokeCoverLaw::Envelope),
                "the Mask brush paints a coverage channel and must get the envelope"
            );
        } else {
            assert!(
                law != Some(StrokeCoverLaw::Envelope),
                "{mode:?} is pigment: the envelope must never reach it (got {law:?})"
            );
        }
    }
    // And the pigment cap still arms exactly where it used to: Accumulate off + Strength < 1.
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (S * S * 4) as usize], S, S);
    let capped = ph2d_painter_brush::BrushSpec {
        accumulate: false,
        strength: 0.5,
        ..brush
    };
    assert_eq!(
        t.stroke_cover_law(&capped),
        Some(StrokeCoverLaw::BuildUp),
        "Accumulate-OFF under full Strength must still get the pigment cap"
    );
    assert_eq!(
        t.stroke_cover_law(&ph2d_painter_brush::BrushSpec {
            accumulate: true,
            ..capped
        }),
        None,
        "Accumulate ON tracks no coverage at all (the plain per-dab build-up)"
    );
}

/// **The cost is the FOOTPRINT's, not the canvas's.** The envelope threads a canvas-sized byte buffer
/// the mask stroke did not use before, so the thing to prove is that no pass started walking the whole
/// plane: quadrupling the canvas must not move the per-move cost. A RATIO first (immune to machine
/// drift), then a generous wall-clock kill — measured 0.9 ms mean / 2.5 ms worst at BOTH sizes, in
/// debug and in release, against a 16.7 ms frame.
#[test]
fn the_mask_stroke_cost_does_not_follow_the_canvas() {
    let cost = |size: u32| -> f64 {
        let mut t = mask_tool(size);
        let c = size as f32 * 0.5;
        t.set_brush_size_px(120.0);
        t.on_canvas_pointer(cp([c - 100.0, c], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let n = 20;
        let t0 = std::time::Instant::now();
        for i in 1..=n {
            t.on_canvas_pointer(cp([c - 100.0 + i as f32 * 10.0, c], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        let dt = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(n);
        t.on_canvas_pointer(cp([c + 100.0, c], PointerPhase::Up));
        dt
    };
    let small = cost(1024);
    let big = cost(2048);
    assert!(
        big < small * 1.6 + 0.15,
        "a mask move must be footprint-bound: {small:.2} ms @1024² vs {big:.2} ms @2048² \
         (measured ~1.0× — quadrupling the canvas moved nothing; a pass that walked the plane \
         would show 4×)"
    );
    assert!(
        big < 8.0,
        "a mask move must fit a frame with room to spare: {big:.2} ms @2048² (kill 8 ms)"
    );
}

/// The overlay is unchanged by the rewrite, and the coverage it reads is the same buffer — a
/// regression guard for the one thing a coverage-law change could silently break: full protection must
/// still be reachable (two passes measured exactly 1.000, one pass 0.984 — the honest peak of a soft
/// brush whose dab centres never land dead on a texel centre).
#[test]
fn two_mask_passes_reach_full_protection() {
    let mut t = mask_tool(S);
    vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    let one = coverage(&t, S)[130 * S as usize + 128];
    vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    let two = coverage(&t, S)[130 * S as usize + 128];
    assert!(
        one > 0.97,
        "one pass must protect the core almost fully (measured 0.984), got {one:.3}"
    );
    assert!(
        two >= 1.0,
        "two passes must protect it FULLY (measured 1.000), got {two:.3}"
    );
    assert!(
        cross_x(&coverage(&t, S), S, 130, 0.5).is_some(),
        "the coverage must still have an edge to cross"
    );
}

/// **The per-stroke buffer is TRANSIENT, and undo proves it.** The envelope lives in the stroke's own
/// coverage buffer, not in the document: a mask stroke is one undo step back to the coverage that was
/// there before it, and the NEXT stroke must not inherit the undone stroke's envelope (which would
/// silently make it a no-op — the buffer says "already laid"). This is the `mats`/impasto lesson from
/// the other direction: a new plane either enters the snapshot in the same commit, or it must be
/// provably per-stroke. `stroke_mask` is the latter, and `paint_begin` is where that is established.
#[test]
fn a_mask_stroke_is_one_undo_step_and_the_next_stroke_starts_fresh() {
    let mut t = mask_tool(S);
    let blank = coverage(&t, S)[130 * S as usize + 128];
    vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    let painted = coverage(&t, S)[130 * S as usize + 128];
    assert!(
        painted > 0.9 && blank < 0.01,
        "fixture: the stroke must protect the core (blank {blank:.3} -> painted {painted:.3})"
    );
    assert!(t.undo_last(), "a mask stroke is an undo step");
    let undone = coverage(&t, S)[130 * S as usize + 128];
    assert!(
        undone < 0.01,
        "undo must restore the pre-stroke coverage, got {undone:.3}"
    );
    // The re-paint must land the same coverage as the first time — if the envelope had survived the
    // undo, every texel would report "already laid" and the stroke would deposit NOTHING.
    vstroke(&mut t, 128.0, 60.0, 200.0, 25);
    let again = coverage(&t, S)[130 * S as usize + 128];
    assert!(
        (again - painted).abs() < 0.01,
        "re-painting after an undo must deposit the same coverage ({painted:.3} then {again:.3})"
    );
}

/// **The route that escaped the door** (found auditing this wave, lens 1 — "does the law reach EVERY
/// mask stroke?"). The per-layer-COLOUR path (layers-as-brush) is decided BEFORE the ramp/cache branches
/// and never consulted [`super::PainterTool::stroke_cover_law`], so a mask stroke painted with it left
/// the coverage law behind: **measured, scrubbing one pen-down darkened the body by 16 levels of 255
/// with the route open, and by 0 with it closed.** It is also wrong in kind — that path takes its
/// colours from the LAYER STACK, while `stamp_dabs_mask` forces black/white + Mix precisely to keep the
/// scratch a pure COVERAGE channel.
///
/// Pigment's routing is untouched (per-layer colour + Accumulate-OFF keeps its long-standing
/// behaviour) — moving the paint path is not this wave's to do.
///
/// ⚠️ **Two oracles were tried and only the third is honest.** The FEATHER cannot answer: arming layers
/// also installs them as the Shape silhouette, and a Shape image stamps a hard border by design (`t = 1`,
/// the bow-wave anchor note), so the band is ~0.8 px either way. The COLOUR cannot answer either: the
/// coloured route still resolves to grayscale here, so that assertion passed with the bug in place —
/// a gate that could not fail. The SCRUB answers, because it is the very property the law is about.
///
/// Mutation that must bleed: dropping the `!Mask` guard in `stamp_dabs_inner`'s per-layer-colour branch.
#[test]
fn per_layer_colour_never_hijacks_a_mask_stroke() {
    let arm = |t: &mut PainterTool| {
        let red: Vec<u8> = (0..64).flat_map(|_| [220u8, 20, 20, 255]).collect();
        let blue: Vec<u8> = (0..64).flat_map(|_| [20u8, 20, 220, 255]).collect();
        t.set_brush_shape_layers(vec![(red, 8, 8), (blue, 8, 8)]);
        t.toggle_brush_shape_per_layer_color();
    };
    let single = {
        let mut t = mask_tool(S);
        arm(&mut t);
        assert!(
            t.brush_settings().shape_per_layer_color && t.brush_settings().shape_layer_count == 2,
            "fixture: per-layer colour must be armed for this gate to mean anything"
        );
        vstroke(&mut t, 128.0, 60.0, 200.0, 25);
        coverage(&t, S)
    };
    let scrubbed = {
        let mut t = mask_tool(S);
        arm(&mut t);
        t.on_canvas_pointer(cp([128.0, 60.0], PointerPhase::Down));
        for leg in 0..4u32 {
            let (a, b) = if leg % 2 == 0 {
                (60.0, 200.0)
            } else {
                (200.0, 60.0)
            };
            for i in 1..=25u32 {
                let y = a + (b - a) * (i as f32) / 25.0;
                t.on_canvas_pointer(cp([128.0, y], PointerPhase::Move));
            }
        }
        t.on_canvas_pointer(cp([128.0, 60.0], PointerPhase::Up));
        let _ = t.take_preview_arc();
        coverage(&t, S)
    };
    let body = (90..170)
        .flat_map(|y| (100..160).map(move |x| y * S as usize + x))
        .map(|i| (scrubbed[i] - single[i]).abs())
        .fold(0.0_f32, f32::max);
    let levels = (body * 255.0).round();
    assert!(
        levels <= 2.0,
        "a mask stroke with Per-Layer Color armed must still obey the coverage law: the scrub moved \
         it {levels} levels (measured 0 with the route closed, 16 with it open)"
    );
}
