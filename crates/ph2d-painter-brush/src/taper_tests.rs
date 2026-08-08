//! Gates for the **Taper** (Procreate *Touch Taper*): [`crate::taper`] + the stroke engine's two
//! doors into it (emission when the far end is known, the tail hold when it is not).
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
/// **Mutation that must bleed:** make `ramp` return `tip` when `len <= 0` — an end with no taper window
/// still applying its tip, which would let a knob that is supposed to be inert reshape every dab.
///
/// ⚠️ Deliberately NOT "make `is_active` return `true`": that guard is a COST short-circuit, not the
/// correctness. With both lengths zero the arithmetic already returns exactly `1.0` and `scale_dab`
/// early-returns, so the mutation is semantically neutral and survives — measured, not assumed.
#[test]
fn an_off_taper_does_not_move_a_single_dab() {
    let base = freehand(brush(), 400.0, 40);
    let mut spec = brush();
    // Every knob EXCEPT the two lengths, which are what arms the feature: tip / link / opacity on their
    // own must be provably inert, or the artist finds a control that does nothing until another does.
    spec.taper = Taper {
        start: 0.0,
        end: 0.0,
        tip_start: 0.75,
        tip_end: 0.25,
        link_tips: false,
        opacity: 1.0,
    };
    let with = freehand(spec, 400.0, 40);
    assert_eq!(base.len(), with.len(), "the dab COUNT moved");
    for (i, (a, b)) in base.iter().zip(&with).enumerate() {
        assert_eq!(
            (a.radius_px, a.coverage, a.center, a.arc_len),
            (b.radius_px, b.coverage, b.center, b.arc_len),
            "dab {i} moved under a taper whose lengths are both zero"
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

/// **An open path whose geometry is already known tapers at BOTH ends, live, with no hold.**
///
/// This is the Line / Arc / Free Hand door: the spine exists before the first dab, so the engine can
/// measure it and the end taper is exact while the artist is still reshaping the curve.
///
/// **Mutation that must bleed:** drop the `taper_span = Open(polyline_len(spine))` line — the end stays
/// at full width because `to_end` is `INFINITY`.
#[test]
fn a_known_path_tapers_at_both_ends() {
    let mut spec = brush();
    spec.stroke_method = StrokeMethod::Line;
    spec.taper = Taper {
        start: 2.0,
        end: 2.0,
        ..Taper::default()
    };
    let dabs = polyline(spec, 400.0);
    let r = radii(&dabs);
    assert!(r.len() > 20, "fixture: too few dabs ({})", r.len());
    assert!(r[0] <= 1.0, "the head is not a point ({:.2})", r[0]);
    assert!(
        *r.last().unwrap() <= 2.0,
        "the tail is not a point ({:.2})",
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
/// An ellipse has no ends — the only place a taper could land is the arbitrary point where the fill
/// happened to start, which would put a notch in a circle. The oracle is that every dab is full width.
///
/// **Mutation that must bleed:** drop the `taper_span = Closed` line in the ellipse fill.
#[test]
fn a_closed_loop_has_no_ends_to_taper() {
    let mut spec = brush();
    spec.stroke_method = StrokeMethod::Ellipse;
    spec.taper = Taper {
        start: 3.0,
        end: 3.0,
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

/// **The freehand end taper lands, and it lands on the dabs that are really at the end.**
///
/// The end is unknown while the stroke is open, so the tail is held and released shaped at pen-up. Two
/// halves, and both matter: the tail IS tapered, and the body — which passed through the same buffer —
/// is NOT (a hold that tapered everything it touched would thin the whole stroke).
///
/// **Mutation that must bleed:** make `holds_tail` return `false` — the tail is never held, `to_end` is
/// `INFINITY` for every dab, and the stroke ends bluntly at full width.
#[test]
fn a_freehand_stroke_is_tapered_at_the_end_it_turned_out_to_have() {
    let mut spec = brush();
    spec.taper = Taper {
        start: 0.0,
        end: 3.0, // 60 px
        ..Taper::default()
    };
    let dabs = freehand(spec, 400.0, 40);
    let r = radii(&dabs);
    assert!(r.len() > 20, "fixture: too few dabs ({})", r.len());
    assert!(
        (r[0] - 10.0).abs() < 1e-4,
        "the HEAD was tapered by an end-only taper ({:.4})",
        r[0]
    );
    assert!(
        *r.last().unwrap() <= 1.0,
        "the stroke does not end in a point ({:.2})",
        r.last().unwrap()
    );
    // Everything more than one window from the end is full width.
    for d in dabs.iter().filter(|d| d.arc_len < 320.0) {
        assert!(
            (d.radius_px - 10.0).abs() < 1e-4,
            "a dab at arc {:.1} — far from either end — was thinned to {:.4}",
            d.arc_len,
            d.radius_px
        );
    }
}

/// **The held dabs come out in the order they were laid.**
///
/// The tail hold is the only thing in this engine that can reorder a stroke, and the deposits that
/// would be destroyed by a reorder — the fluid, the read-modify tools — are exactly the ones no gate on
/// WIDTH can see. So `arc_len` is asserted non-decreasing across the whole released list.
///
/// **Mutation that must bleed:** append the released prefix AFTER the fresh batch in `tail_gate`.
#[test]
fn the_tail_hold_never_reorders_the_stroke() {
    let mut spec = brush();
    spec.taper = Taper {
        start: 1.0,
        end: 3.0,
        ..Taper::default()
    };
    let dabs = freehand(spec, 400.0, 40);
    for w in dabs.windows(2) {
        assert!(
            w[1].arc_len >= w[0].arc_len - 1e-4,
            "the stroke came out of order: arc {:.3} after {:.3}",
            w[1].arc_len,
            w[0].arc_len
        );
    }
}

/// **Turning the end taper off mid-stroke releases what was held — in order, and only once.**
///
/// `set_spec` is live: the artist drags the End slider while the stroke is open. The dabs already held
/// must come out (stranding them would silently delete part of the stroke), they must come out BEFORE
/// the batch being laid right now, and they must not be tapered a second time.
///
/// ⚠️ This gate exists because the reorder mutation **survived** without it: with the hold ON the fresh
/// batch has already been drained into the buffer, so `fresh` is empty and the order branch is
/// unobservable. The only way to reach it is to switch the hold off with dabs still held — the fixture
/// has to contain the phenomenon, or the branch is guarded by nothing.
///
/// **Mutation that must bleed:** append the released prefix AFTER the fresh batch in `tail_gate`.
#[test]
fn dropping_the_end_taper_mid_stroke_releases_the_held_dabs_in_order() {
    let mut spec = brush();
    spec.taper = Taper {
        start: 0.0,
        end: 3.0,
        ..Taper::default()
    };
    let mut st = Stroke::new(spec, Dynamics::default(), 7);
    let (mut out, mut all) = (Vec::new(), Vec::new());
    st.begin(
        StrokePoint {
            pos: [0.0, 0.0],
            pressure: 1.0,
        },
        &mut out,
    );
    all.append(&mut out);
    for i in 1..=40 {
        if i == 20 {
            // The artist drags End to zero with a taper window's worth of dabs still held.
            let mut off = spec;
            off.taper.end = 0.0;
            st.set_spec(off);
        }
        st.extend(
            StrokePoint {
                pos: [10.0 * i as f32, 0.0],
                pressure: 1.0,
            },
            &mut out,
        );
        all.append(&mut out);
    }
    st.finish(&mut out);
    all.append(&mut out);

    // ⚠️ **Not one dab may be LOST.** This half exists because the reorder mutation survived the order
    // assertion alone: dropping the fresh batch on the release path leaves what remains perfectly
    // ordered and perfectly full-width, and the stroke is simply missing a piece. A hold that silently
    // deletes part of a stroke is worse than one that reorders it — and only a COUNT can see it.
    let mut off = spec;
    off.taper.end = 0.0;
    let control = freehand(off, 400.0, 40);
    assert_eq!(
        all.len(),
        control.len(),
        "the hold swallowed {} dab(s) when the end taper was switched off mid-stroke",
        control.len() as i64 - all.len() as i64
    );
    for w in all.windows(2) {
        assert!(
            w[1].arc_len >= w[0].arc_len - 1e-4,
            "the released tail came out AFTER the batch that followed it: arc {:.3} then {:.3}",
            w[0].arc_len,
            w[1].arc_len
        );
    }
    // Nothing was tapered: the start length was zero all along and the end length was zero before any
    // dab could reach the end of the stroke.
    for d in &all {
        assert!(
            (d.radius_px - 10.0).abs() < 1e-4,
            "a dab at arc {:.1} was thinned after the taper was switched off ({:.4})",
            d.arc_len,
            d.radius_px
        );
    }
}

/// **A stroke shorter than the two taper windows is a lens, not a vanishing act.**
///
/// The two ends overlap, and this is the whole reason the law combines them with `min` instead of a
/// product: a product thins the middle TWICE, so a short flick nearly disappears and reads as the brush
/// being broken. The oracle is the widest dab in the stroke — under `min` it still reaches a real
/// fraction of the brush.
///
/// **Mutation that must bleed:** combine by `a * b` in [`Taper::width`].
#[test]
fn a_short_stroke_is_a_lens_not_a_vanishing_act() {
    let mut spec = brush();
    spec.taper = Taper {
        start: 4.0, // 80 px each — a 100 px stroke is inside both windows everywhere
        end: 4.0,
        ..Taper::default()
    };
    let dabs = freehand(spec, 100.0, 20);
    let widest = radii(&dabs).into_iter().fold(0.0f32, f32::max);
    // `min` at the midpoint of a 100 px stroke: both ends see 50 of 80 px ⇒ smoothstep(0.625) ≈ 0.68.
    assert!(
        widest > 6.0,
        "a short stroke collapsed (widest dab {widest:.3} of brush radius 10) — the two ends are \
         being multiplied instead of `min`ed"
    );
}

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

/// **Link tip sizes is one number driving two ends.**
///
/// Linked, the end wears the start's tip; unlinked, it wears its own. Asserted through the ENGINE (the
/// released tail dab) rather than by calling [`Taper::effective_tip_end`] — that would be the accessor
/// checking itself.
///
/// **Mutation that must bleed:** always return `tip_end` from [`Taper::effective_tip_end`].
#[test]
fn linking_the_tips_makes_the_end_wear_the_starts_tip() {
    let tail_radius = |link: bool| {
        let mut spec = brush();
        spec.taper = Taper {
            start: 2.0,
            end: 2.0,
            tip_start: 0.8, // blunt
            tip_end: 0.0,   // sharp
            link_tips: link,
            opacity: 0.0,
        };
        freehand(spec, 400.0, 40).last().unwrap().radius_px
    };
    let linked = tail_radius(true);
    let free = tail_radius(false);
    assert!(
        linked > 6.0,
        "linked: the end ignored the start's BLUNT tip (radius {linked:.3})"
    );
    assert!(
        free <= 1.0,
        "unlinked: the end ignored its own SHARP tip (radius {free:.3})"
    );
}

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

/// **The price of the freehand end taper, in a number: the wet end trails the cursor by the taper
/// length, and by nothing more.**
///
/// The tail hold is the one thing in this feature the artist can FEEL rather than see, so it is
/// measured rather than described. Two halves: it must trail at least the window (or the end taper is
/// not actually being resolved at the end) and at most the window plus one dab spacing (or the hold is
/// keeping dabs it could have released, and the lag is larger than the control the artist set).
///
/// Measured at the shipped default brush (radius 25, diameter 50): an end taper of **1 diameter trails
/// 50 px**, **2 diameters 100 px**, **3 diameters 150 px** — and on a radius-60 brush 2 diameters is
/// **240 px**, because the length is brush-relative.
///
/// **Mutation that must bleed:** release on `>= end_px * 2.0` — the artist sets one number and the
/// stroke trails by twice it.
#[test]
fn the_wet_end_trails_the_cursor_by_exactly_the_taper_the_artist_set() {
    for (radius, end_d) in [(25.0f32, 1.0f32), (25.0, 2.0), (25.0, 3.0), (60.0, 2.0)] {
        let mut spec = BrushSpec {
            radius_px: radius,
            ..brush()
        };
        spec.taper = Taper {
            end: end_d,
            ..Taper::default()
        };
        let diameter = 2.0 * radius;
        let window = end_d * diameter;
        let mut st = Stroke::new(spec, Dynamics::default(), 1);
        let mut out = Vec::new();
        st.begin(
            StrokePoint {
                pos: [0.0, 0.0],
                pressure: 1.0,
            },
            &mut out,
        );
        // 1 px steps: the pointer's own granularity must not be mistaken for the hold's.
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
            lag >= window - spacing,
            "radius {radius}, end {end_d} diameters: the tail trailed only {lag:.1} px of a {window:.1} \
             px window — the end taper is not being resolved at the end"
        );
        assert!(
            lag <= window + 2.0 * spacing,
            "radius {radius}, end {end_d} diameters: the tail trailed {lag:.1} px for a {window:.1} px \
             window — the hold is keeping dabs it could have released"
        );
    }
}
