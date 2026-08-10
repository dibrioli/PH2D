//! Gates for the **Taper** (Procreate *Touch Taper*): [`crate::taper`] + the single door the stroke
//! engine applies it through, at emission, for every dab it ever makes.
//!
//! The oracle throughout is **the dab list the artist's stroke actually produces** — never the law
//! re-implemented here. A gate that recomputed `tip + (1-tip)*smoothstep` and compared it to itself
//! would be a mirror of the line above it and would stay green through any wiring defect.

use crate::Dab;
use crate::dynamics::Dynamics;
use crate::spec::BrushSpec;
use crate::stroke::{Stroke, StrokePoint};
use crate::stroke_method::StrokeMethod;
use crate::taper::Taper;

/// A brush whose dabs are easy to read: radius 10 (diameter 20) and tight spacing so a taper window of
/// a couple of diameters holds many dabs.
fn brush() -> BrushSpec {
    let mut b = BrushSpec {
        radius_px: 10.0,
        spacing: 0.10,
        ..BrushSpec::default()
    };
    b.stroke_method = StrokeMethod::Space;
    b
}

/// Drive a straight freehand stroke of `len` px along +x, in `steps` pointer moves, and return every
/// dab the engine emitted — including whatever `finish` releases.
fn freehand(spec: BrushSpec, len: f32, steps: u32) -> Vec<Dab> {
    let mut st = Stroke::new(spec, Dynamics::default(), 7);
    let mut out = Vec::new();
    let mut all = Vec::new();
    st.begin(
        StrokePoint {
            pos: [0.0, 0.0],
            pressure: 1.0,
        },
        &mut out,
    );
    all.append(&mut out);
    for i in 1..=steps {
        let x = len * i as f32 / steps as f32;
        st.extend(
            StrokePoint {
                pos: [x, 0.0],
                pressure: 1.0,
            },
            &mut out,
        );
        all.append(&mut out);
    }
    st.finish(&mut out);
    all.append(&mut out);
    all
}

/// Lay a straight open path of `len` px through the whole-path fill (the Line / Arc / Free Hand door).
fn polyline(spec: BrushSpec, len: f32) -> Vec<Dab> {
    let mut st = Stroke::new(spec, Dynamics::default(), 7);
    let mut out = Vec::new();
    let spine: Vec<[f32; 2]> = (0..=64).map(|i| [len * i as f32 / 64.0, 0.0]).collect();
    st.fill_polyline_preview(&spine, &mut out);
    out
}

fn radii(dabs: &[Dab]) -> Vec<f32> {
    dabs.iter().map(|d| d.radius_px).collect()
}

/// **An off taper is the brush that shipped yesterday, to the bit.**
///
/// The default is off, so this is the regression that protects every other Painter claim: a stroke with
/// `Taper::default()` must be byte-identical to one built by a `BrushSpec` that never heard of a taper.
/// Compared field by field on the whole dab list, not just the radius — the taper also touches coverage.
///
/// **Mutation that must bleed:** make `ramp` return `tip` when `len <= 0` — a head with no taper window
/// still applying its tip, which would let a knob that is supposed to be inert reshape every dab.
///
/// ⚠️ Deliberately NOT "make `is_active` return `true`": that guard is a COST short-circuit, not the
/// correctness. With the length zero the arithmetic already returns exactly `1.0` and `scale_dab`
/// early-returns, so the mutation is semantically neutral and survives — measured, not assumed.
#[test]
fn an_off_taper_does_not_move_a_single_dab() {
    let base = freehand(brush(), 400.0, 40);
    let mut spec = brush();
    // Every knob EXCEPT the length, which is what arms the feature: tip / opacity on their own must be
    // provably inert, or the artist finds a control that does nothing until another does.
    spec.taper = Taper {
        start: 0.0,
        tip_start: 0.75,
        opacity: 1.0,
    };
    let with = freehand(spec, 400.0, 40);
    assert_eq!(base.len(), with.len(), "the dab COUNT moved");
    for (i, (a, b)) in base.iter().zip(&with).enumerate() {
        assert_eq!(
            (a.radius_px, a.coverage, a.center, a.arc_len),
            (b.radius_px, b.coverage, b.center, b.arc_len),
            "dab {i} moved under a taper whose length is zero"
        );
    }
}

/// **The stroke starts thin and reaches full width — and it does so while the artist is still drawing.**
///
/// The start taper needs nothing but the arc already travelled, so it is exact from the first dab. The
/// oracle is the SHAPE of the radius sequence: sharp at the head, monotonically opening, full body after
/// the window. `stroke_radius_px` is checked too: it is the Flow phase divisor and is stroke-constant by
/// contract, so the taper must not have touched it.
///
/// **Mutation that must bleed:** drop the `scale_dab` call in `Stroke::dab_at`.
#[test]
fn the_stroke_opens_from_a_point_to_full_width() {
    let mut spec = brush();
    spec.taper = Taper {
        start: 3.0, // 3 diameters = 60 px
        ..Taper::default()
    };
    let dabs = freehand(spec, 400.0, 40);
    let r = radii(&dabs);
    assert!(
        r.len() > 20,
        "fixture: too few dabs to read a ramp ({})",
        r.len()
    );
    assert!(
        r[0] <= 1.0,
        "the first dab is not a POINT (radius {:.2}, brush radius 10)",
        r[0]
    );
    // Monotone through the window, then flat at the brush radius.
    for w in dabs.windows(2) {
        if w[0].arc_len < 60.0 {
            assert!(
                w[1].radius_px >= w[0].radius_px - 1e-4,
                "the taper is not monotone at arc {:.1}: {:.3} -> {:.3}",
                w[0].arc_len,
                w[0].radius_px,
                w[1].radius_px
            );
        }
    }
    let body: Vec<f32> = dabs
        .iter()
        .filter(|d| d.arc_len > 80.0)
        .map(|d| d.radius_px)
        .collect();
    assert!(
        !body.is_empty(),
        "fixture: the stroke never left the taper window"
    );
    for r in &body {
        assert!(
            (r - 10.0).abs() < 1e-4,
            "the body of the stroke is not full width ({r:.4})"
        );
    }
    for d in &dabs {
        assert!(
            (d.stroke_radius_px - 10.0).abs() < 1e-6,
            "the taper moved `stroke_radius_px`, which the Shape Flow divides by"
        );
    }
}

/// **An open path tapers at its HEAD, live, with no hold — and its tail comes out full width.**
///
/// This is the Line / Arc / Free Hand door. It used to taper BOTH ends here (the spine exists before the
/// first dab, so the far end was exact); the end went with the rest on 2026-08-10, and the tail half of
/// this gate is what says so. Keeping both halves in one test is deliberate: *the head still works* and
/// *the tail no longer does* are the two things a reader needs, and split across two tests one of them
/// eventually gets deleted for looking redundant.
///
/// **Mutation that must bleed:** make `Stroke::taper_width_at` ignore `self.taper_head` — a whole-path
/// fill still tapers, so only a CLOSED loop would notice, and this gate would stay green while the
/// ellipse gate went red. The half that bleeds here instead is dropping `scale_dab` (the head goes
/// blunt).
#[test]
fn a_known_path_tapers_at_its_head_and_not_at_its_tail() {
    let mut spec = brush();
    spec.stroke_method = StrokeMethod::Line;
    spec.taper = Taper {
        start: 2.0,
        ..Taper::default()
    };
    let dabs = polyline(spec, 400.0);
    let r = radii(&dabs);
    assert!(r.len() > 20, "fixture: too few dabs ({})", r.len());
    assert!(r[0] <= 1.0, "the head is not a point ({:.2})", r[0]);
    assert!(
        (r.last().unwrap() - 10.0).abs() < 1e-4,
        "the tail of a known path was tapered ({:.4}) — the far end is gone, so something is measuring \
         a distance-to-the-end again",
        r.last().unwrap()
    );
    let mid = r[r.len() / 2];
    assert!(
        (mid - 10.0).abs() < 1e-4,
        "the middle of the path is not full width ({mid:.4})"
    );
}

/// **A closed loop is NOT tapered.**
///
/// An ellipse has no head — the only place a taper could land is the arbitrary point where the fill
/// happened to start, which would put a notch in a circle. The oracle is that every dab is full width.
///
/// **Mutation that must bleed:** drop the `taper_head = false` line in the ellipse fill.
#[test]
fn a_closed_loop_has_no_head_to_taper() {
    let mut spec = brush();
    spec.stroke_method = StrokeMethod::Ellipse;
    spec.taper = Taper {
        start: 3.0,
        ..Taper::default()
    };
    let mut st = Stroke::new(spec, Dynamics::default(), 7);
    let mut out = Vec::new();
    st.fill_ellipse_preview([0.0, 0.0], [1.0, 0.0], 120.0, 90.0, &mut out);
    assert!(out.len() > 30, "fixture: too few dabs ({})", out.len());
    for (i, d) in out.iter().enumerate() {
        assert!(
            (d.radius_px - 10.0).abs() < 1e-4,
            "dab {i} of a CLOSED loop was tapered ({:.4}) — that is a notch in a circle",
            d.radius_px
        );
    }
}

/// **NO DAB IS EVER WITHHELD: every pointer move lays what it reached, in order.**
///
/// This is the law the taper lives under, and it is structural rather than a setting. Two halves, and
/// they are the two ways a hold shows: the arcs never go BACKWARDS across the whole stroke (nothing was
/// released out of order), and after every single pointer move the ink has reached that move (nothing
/// was parked waiting to learn its distance-to-the-end).
///
/// ⛔ The first cut of this feature broke exactly this. It held the tail back until the cursor had
/// travelled past the end-taper window so it could learn each dab's distance-to-the-end, which is exact
/// and is **wrong as a product**: the mark stops following the hand. Enio, 2026-08-08 — *"o algoritmo que
/// vc usou para o taper e ruim, tem um super delay e um stabilize ruim. O traço não pode ter nenhum
/// delay e nenhum stabilize"*. The two complaints are one mechanism: a stroke that lags and then catches
/// up in a lump is what a heavy stabilizer feels like.
///
/// ⚠️ The oracle is per-MOVE reach and order, never "the same dabs as an untapered stroke" — the taper
/// legitimately changes how MANY dabs there are (it tightens the spacing at the tips so the point does
/// not come out as a row of dots), and a gate written against the count would have to be weakened the
/// moment that landed. Reach and order are what a hold actually breaks.
///
/// **Mutation that must bleed:** hold anything back — buffer the trailing dabs and release them later.
#[test]
fn no_dab_is_ever_withheld_or_moved() {
    let mut spec = brush();
    spec.taper = Taper {
        start: 2.0,
        ..Taper::default()
    };
    let mut st = Stroke::new(spec, Dynamics::default(), 7);
    let (mut out, mut arcs) = (Vec::new(), Vec::new());
    st.begin(
        StrokePoint {
            pos: [0.0, 0.0],
            pressure: 1.0,
        },
        &mut out,
    );
    arcs.extend(out.iter().map(|d| d.arc_len));
    let mut reached = 0.0f32;
    // 80 moves of 5 px: past the head window, deep into the body, and long enough that a tail hold
    // would still be sitting on dabs at every one of them.
    for i in 1..=80 {
        let x = 5.0 * i as f32;
        st.extend(
            StrokePoint {
                pos: [x, 0.0],
                pressure: 1.0,
            },
            &mut out,
        );
        if let Some(d) = out.last() {
            reached = d.arc_len;
        }
        arcs.extend(out.iter().map(|d| d.arc_len));
        // The stabilizer lags the painted path by design (it is not the taper's), so the bar is the
        // brush's own spacing plus that settled lag — what must NOT happen is the ink parking a taper
        // window behind, which at 3 diameters would be 60 px on this brush.
        assert!(
            x - reached < 20.0,
            "after the move to {x:.0} px the ink had only reached {reached:.1} px — something is \
             holding dabs back"
        );
    }
    for w in arcs.windows(2) {
        assert!(
            w[1] >= w[0] - 1e-4,
            "the stroke came out of order: arc {:.2} was laid after {:.2}",
            w[1],
            w[0]
        );
    }
}

/// **The far end is left at FULL WIDTH — on the freehand drag AND on a path that knows its own length.**
///
/// This is the gate for the removal itself (Enio 2026-08-10: *"quanto à cauda do taper vamos desativar
/// para todos os modos de pintura; deixe o ajuste apenas para o início do traço"*). Both doors are here
/// on purpose, because they used to answer DIFFERENTLY: a drag left its tail blunt because the end had
/// not happened yet, while a whole-path fill tapered it exactly. Only one of those is a claim about the
/// feature being gone; testing the drag alone would stay green with the end term fully alive.
///
/// ⛔ It also stands where the two rejected designs stood, so neither comes back by accident: withholding
/// dabs until the end is known (the delay Enio rejected on 2026-08-08) and re-stamping the tail from a
/// restored base at pen-up (the resolve that shipped and went with the end — it took the artist's paint
/// under impasto and could not run at all in watercolor or Wet Paint).
///
/// **Mutation that must bleed:** give [`Taper::width`] a second term measured from any far end.
#[test]
fn the_far_end_is_left_at_full_width_on_every_door() {
    let mut spec = brush();
    spec.taper = Taper {
        start: 2.0,
        ..Taper::default()
    };
    let full = spec.clamped_radius();
    for (door, dabs) in [
        ("freehand drag", freehand(spec, 400.0, 40)),
        ("whole-path fill", {
            let mut s = spec;
            s.stroke_method = StrokeMethod::Line;
            polyline(s, 400.0)
        }),
    ] {
        let first = dabs.first().expect("the stroke laid dabs");
        assert!(
            first.radius_px < full * 0.2,
            "{door}: the head did not taper ({:.3} of {full:.3}) — the half that IS live is broken",
            first.radius_px
        );
        let last = dabs.last().expect("the stroke laid dabs");
        assert!(
            (last.radius_px - full).abs() < 1e-3,
            "{door}: the last dab came out at {:.3} instead of the full {full:.3} — something is \
             tapering a far end again",
            last.radius_px
        );
    }
}

// ⛔ `a_short_stroke_is_a_lens_not_a_vanishing_act` lived here and died with the far end. It pinned that
// the two windows combined by `min` and not by a product — on a stroke shorter than both, a product
// thins the middle TWICE and a short flick nearly vanishes. With one window there is nothing to combine,
// so the gate has no fact left to assert; the reasoning is kept in `crate::taper` in case a second
// window is ever authored again.

/// **Opacity fades the tip only when the artist asks it to.**
///
/// Procreate's taper Opacity is a separate control from the width for a reason: a thin-but-opaque tip
/// and a thin-and-fading tip are different marks. Two halves in one gate — at `0` the tip is as opaque
/// as the body, at `1` it fades with the width.
///
/// **Mutation that must bleed:** ignore `opacity` in [`Taper::coverage_mult`] (return `1.0`).
#[test]
fn the_taper_fades_the_tip_only_when_opacity_asks() {
    let mut solid = brush();
    solid.taper = Taper {
        start: 3.0,
        opacity: 0.0,
        ..Taper::default()
    };
    let body_cov = freehand(solid, 400.0, 40)[0].coverage;
    let mut fading = brush();
    fading.taper = Taper {
        start: 3.0,
        opacity: 1.0,
        ..Taper::default()
    };
    let tip_cov = freehand(fading, 400.0, 40)[0].coverage;
    assert!(
        body_cov > 0.9,
        "opacity 0 faded the tip anyway (coverage {body_cov:.4})"
    );
    assert!(
        tip_cov < 0.1,
        "opacity 1 did not fade the tip (coverage {tip_cov:.4})"
    );
}

// ⛔ `linking_the_tips_makes_the_end_wear_the_starts_tip` lived here and died with the far end: with one
// end there is one tip, so *Link tip sizes* had nothing left to link.

/// **The taper length is read in DIAMETERS, so one number means one thing at every brush size.**
///
/// A taper measured in pixels is a flourish on a liner and invisible on a wash. The oracle is the arc at
/// which the stroke reaches full width: doubling the brush must double it.
///
/// **Mutation that must bleed:** drop the `* diameter_px` in [`Taper::start_px`].
#[test]
fn the_taper_length_scales_with_the_brush() {
    let full_width_at = |radius: f32| {
        let mut spec = brush();
        spec.radius_px = radius;
        spec.taper = Taper {
            start: 2.0,
            ..Taper::default()
        };
        freehand(spec, 900.0, 90)
            .into_iter()
            .find(|d| d.radius_px >= radius - 1e-3)
            .map(|d| d.arc_len)
            .expect("the stroke never reached full width")
    };
    let small = full_width_at(10.0);
    let big = full_width_at(20.0);
    let ratio = big / small.max(1e-6);
    assert!(
        (1.7..=2.3).contains(&ratio),
        "the taper is not brush-relative: reaching full width took {small:.1} px on a radius-10 brush \
         and {big:.1} px on a radius-20 one (ratio {ratio:.2}, expected ~2)"
    );
}

/// **The stroke reaches the cursor the instant the pointer does.**
///
/// The consequence of the law above, measured where the artist feels it: after a long drag with an end
/// taper armed, the last dab the engine has emitted sits within one dab spacing of the pointer. Not
/// within a taper window — within a SPACING, which is the granularity the brush has anyway.
///
/// The four cases sweep brush radius and taper length together, because the withheld-tail design that
/// this replaces lagged by `end × diameter`: it trailed 50 px at one diameter on the shipped brush,
/// 100 at two, 150 at three, and 240 px on a radius-60 brush. Every one of those is a failure here.
///
/// **Mutation that must bleed:** buffer the trailing dabs again — the lag jumps to the taper window.
#[test]
fn the_stroke_reaches_the_cursor_the_instant_the_pointer_does() {
    for (radius, len_d) in [(25.0f32, 1.0f32), (25.0, 2.0), (25.0, 3.0), (60.0, 2.0)] {
        let mut spec = BrushSpec {
            radius_px: radius,
            ..brush()
        };
        spec.taper = Taper {
            start: len_d,
            ..Taper::default()
        };
        let mut st = Stroke::new(spec, Dynamics::default(), 1);
        let mut out = Vec::new();
        st.begin(
            StrokePoint {
                pos: [0.0, 0.0],
                pressure: 1.0,
            },
            &mut out,
        );
        // 1 px steps: the pointer's own granularity must not be mistaken for the engine's.
        let mut last = 0.0f32;
        let mut cursor = 0.0f32;
        for i in 1..=1200 {
            cursor = i as f32;
            st.extend(
                StrokePoint {
                    pos: [cursor, 0.0],
                    pressure: 1.0,
                },
                &mut out,
            );
            if let Some(d) = out.last() {
                last = d.center[0];
            }
        }
        let lag = cursor - last;
        let spacing = spec.dab_spacing_px();
        assert!(
            lag <= spacing + 1.0,
            "radius {radius}, taper {len_d} diameters: the ink trailed the cursor by {lag:.1} px with a \
             dab spacing of {spacing:.1} px — something is holding dabs back"
        );
    }
}

/// **The taper tightens the SPACING as it thins the dab, so the point is a point and not a row of dots.**
///
/// Spacing in this engine is a RATIO — *how much of a dab's width until the next one* — and it was being
/// paid out against the brush's NOMINAL diameter. Hold the gap fixed while the taper shrinks the dab and
/// the ratio blows up: the tip comes out as separate discs with clean paper between them. Enio reported
/// it with the picture on 2026-08-08.
///
/// The oracle is the worst gap-to-diameter ratio anywhere along a tapered path, over the dabs big enough
/// for it to be visible. **Measured on the fixture below: 1.600 with the nominal step — a gap of more
/// than a whole dab, so the discs do not even touch — and 0.284 once the step follows the live width**,
/// which is the brush's own spacing. The bar is `0.5`: at `1.0` the discs are exactly tangent, so half
/// of that is comfortably inside real overlap, and it sits between the two measurements with room.
///
/// ⚠️ The fixture is the one that PRODUCES the picture — a big brush (radius 25) with a loose spacing
/// (0.25). The first version of this probe used radius 10 and spacing 0.10, where the same defect
/// measures 0.463 and would have needed a bar so tight it read as noise: the gap is fixed, so the ratio
/// grows as `1/r`, and a tight-spaced small brush barely shows it.
///
/// **Mutation that must bleed:** step by `base_step` instead of `base_step * w`.
#[test]
fn the_taper_tightens_the_spacing_so_the_point_is_not_a_row_of_dots() {
    let mut spec = brush();
    spec.stroke_method = StrokeMethod::Line;
    spec.radius_px = 25.0;
    spec.spacing = 0.25;
    spec.taper = Taper {
        start: 3.0,
        ..Taper::default()
    };
    let dabs = polyline(spec, 400.0);
    let mut worst = 0.0f32;
    let mut worst_r = 0.0f32;
    for w in dabs.windows(2) {
        let gap = ((w[1].center[0] - w[0].center[0]).powi(2)
            + (w[1].center[1] - w[0].center[1]).powi(2))
        .sqrt();
        let rmin = w[0].radius_px.min(w[1].radius_px);
        // Below ~2 px a dab is sub-pixel and the gap between two of them is not a thing anyone sees;
        // the engine's own 1 px floor on the walk lives down there.
        if rmin < 2.0 {
            continue;
        }
        let ratio = gap / (2.0 * rmin);
        if ratio > worst {
            worst = ratio;
            worst_r = rmin;
        }
    }
    assert!(
        worst > 0.0,
        "fixture: the path laid no measurable pair of dabs"
    );
    assert!(
        worst <= 0.5,
        "the dabs stopped overlapping in the taper: worst gap {worst:.3} of a diameter at radius \
         {worst_r:.2} — the tip is a row of dots"
    );
}
