//! Behavioural tests for the stroke engine ([`super`]). Kept in a sibling `tests.rs` so
//! `stroke.rs` stays under the workspace LOC cap (the gate excludes `*/tests.rs`).

use super::*;
use crate::falloff::Falloff;

fn straight_spec(radius: f32, spacing: f32) -> BrushSpec {
    BrushSpec {
        radius_px: radius,
        spacing,
        falloff: Falloff::Constant,
        // Tests below assert dab *positions/counts*; keep attenuation off so coverage stays 1
        // unless a test opts in.
        space_attenuation: false,
        // Raw by default (no stabilizer) so geometry tests are predictable; the stabilizer tests
        // opt in with their own value.
        stabilizer: 0.0,
        ..Default::default()
    }
}

fn no_dynamics() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

fn pt(x: f32, y: f32, p: f32) -> StrokePoint {
    StrokePoint {
        pos: [x, y],
        pressure: p,
    }
}

/// Run a whole stroke (down → moves → up) and return every dab, the way the tool does
/// (`begin` + `extend`× + `finish` to flush the freehand smoother's tail).
fn collect_stroke(spec: BrushSpec, dynamics: Dynamics, points: &[StrokePoint]) -> Vec<Dab> {
    let mut s = Stroke::new(spec, dynamics, 1);
    let mut all = Vec::new();
    let mut out = Vec::new();
    s.begin(points[0], &mut out);
    all.extend_from_slice(&out);
    for &p in &points[1..] {
        s.extend(p, &mut out);
        all.extend_from_slice(&out);
    }
    s.finish(&mut out);
    all.extend_from_slice(&out);
    all
}

#[test]
fn begin_emits_one_dab_at_down() {
    let mut s = Stroke::new(straight_spec(10.0, 0.5), no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(5.0, 5.0, 1.0), &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].center, [5.0, 5.0]);
}

#[test]
fn symmetry_disabled_paints_byte_identically() {
    // A populated-but-disabled symmetry block must not perturb a single dab (the no-regression
    // guarantee: turning the feature off is exactly the old engine).
    let spec = straight_spec(10.0, 0.5);
    assert!(!spec.symmetry.enabled);
    let pts = [pt(0.0, 0.0, 1.0), pt(100.0, 0.0, 1.0)];
    let baseline = collect_stroke(spec, no_dynamics(), &pts);
    let mut spec2 = spec;
    spec2.symmetry.center = [50.0, 50.0];
    spec2.symmetry.circular = true;
    spec2.symmetry.radial_segments = 8; // all ignored while `enabled == false`
    assert_eq!(
        baseline,
        collect_stroke(spec2, no_dynamics(), &pts),
        "disabled symmetry must be byte-identical to no symmetry"
    );
}

#[test]
fn mirror_x_doubles_each_dab_and_preserves_the_base() {
    let mut spec = straight_spec(10.0, 0.5);
    let pts = [pt(0.0, 0.0, 1.0), pt(100.0, 0.0, 1.0)];
    let baseline = collect_stroke(spec, no_dynamics(), &pts);
    assert!(!baseline.is_empty());
    spec.symmetry = crate::symmetry::SymmetrySettings {
        enabled: true,
        axis: crate::symmetry::MirrorAxis::X,
        center: [50.0, 0.0],
        ..Default::default()
    };
    let mirrored = collect_stroke(spec, no_dynamics(), &pts);
    assert_eq!(
        mirrored.len(),
        baseline.len() * 2,
        "one mirror copy per base dab"
    );
    // Emission is interleaved base, mirror, base, mirror, … — the base dabs are untouched and the
    // mirror reflects x across the line at 50 (x' = 100 − x), y unchanged.
    for (i, b) in baseline.iter().enumerate() {
        assert_eq!(&mirrored[i * 2], b, "base dab {i} preserved verbatim");
        let m = &mirrored[i * 2 + 1];
        assert!(
            (m.center[0] - (100.0 - b.center[0])).abs() < 1e-3,
            "dab {i} mirror x"
        );
        assert!(
            (m.center[1] - b.center[1]).abs() < 1e-3,
            "dab {i} mirror y unchanged"
        );
    }
}

#[test]
fn radial_multiplies_dab_count_by_segments() {
    for n in [3u32, 5, 8, 12] {
        let mut spec = straight_spec(10.0, 0.5);
        let pts = [pt(0.0, 0.0, 1.0), pt(100.0, 0.0, 1.0)];
        let baseline = collect_stroke(spec, no_dynamics(), &pts);
        spec.symmetry = crate::symmetry::SymmetrySettings {
            enabled: true,
            circular: true,
            radial_segments: n,
            center: [50.0, 50.0],
            ..Default::default()
        };
        let radial = collect_stroke(spec, no_dynamics(), &pts);
        assert_eq!(
            radial.len() as u32,
            baseline.len() as u32 * n,
            "n={n}: n copies per dab"
        );
    }
}

#[test]
fn space_method_emits_at_arc_length_intervals() {
    // radius 10 → diameter 20; spacing 0.5 → step 10 px. A straight 0→100 drag lays dabs every
    // 10 px ON the x axis — the smoother collapses to a straight line for collinear input.
    let dabs = collect_stroke(
        straight_spec(10.0, 0.5),
        no_dynamics(),
        &[pt(0.0, 0.0, 1.0), pt(100.0, 0.0, 1.0)],
    );
    let xs: Vec<f32> = dabs.iter().map(|d| d.center[0]).collect();
    assert!(dabs.len() >= 10, "got {xs:?}");
    for d in &dabs {
        assert!(
            d.center[1].abs() < 1e-3,
            "stayed on the x axis: {:?}",
            d.center
        );
    }
    assert!(dabs[0].center[0].abs() < 1e-3, "starts at 0");
    assert!(
        (dabs.last().unwrap().center[0] - 100.0).abs() < 1e-2,
        "reaches 100: {xs:?}"
    );
    for w in dabs.windows(2) {
        let dx = w[1].center[0] - w[0].center[0];
        assert!(
            dx > 0.0 && dx <= 10.0 + 1e-3,
            "≈10px monotonic spacing, got {dx}"
        );
    }
}

#[test]
fn jitter_spacing_scatters_the_gaps_and_replays() {
    // radius 10 → diameter 20; spacing 0.5 → base step 10px. Jitter Spacing scales each gap by 1 ±
    // amount, so a straight drag's gaps scatter instead of the even 10px above.
    let mut spec = straight_spec(10.0, 0.5);
    spec.jitter_spacing = 0.8;
    let pts = [pt(0.0, 0.0, 1.0), pt(200.0, 0.0, 1.0)];
    let dabs = collect_stroke(spec, no_dynamics(), &pts);
    let gaps: Vec<f32> = dabs
        .windows(2)
        .map(|w| w[1].center[0] - w[0].center[0])
        .collect();
    assert!(gaps.len() >= 8, "lays a run of dabs, got {gaps:?}");
    let spread = gaps.iter().cloned().fold(f32::MIN, f32::max)
        - gaps.iter().cloned().fold(f32::MAX, f32::min);
    assert!(
        spread > 4.0,
        "Jitter Spacing scatters the gaps, spread={spread} {gaps:?}"
    );

    // Deterministic (HR-5): the same seed replays identical dab centres.
    let again = collect_stroke(spec, no_dynamics(), &pts);
    let a: Vec<[f32; 2]> = dabs.iter().map(|d| d.center).collect();
    let b: Vec<[f32; 2]> = again.iter().map(|d| d.center).collect();
    assert_eq!(a, b, "same seed replays identically");

    // Off (amount 0) → the baseline even ≤10px spacing is untouched.
    let even = collect_stroke(straight_spec(10.0, 0.5), no_dynamics(), &pts);
    for w in even.windows(2) {
        let dx = w[1].center[0] - w[0].center[0];
        assert!(
            dx > 0.0 && dx <= 10.0 + 1e-3,
            "even spacing when off, got {dx}"
        );
    }
}

#[test]
fn accumulates_across_short_segments() {
    // step = 10. Small 6-px moves accumulate; over the whole drag + finish a dab lands at the
    // 10 px crossing, all on the x axis (collinear input stays straight through the smoother).
    let dabs = collect_stroke(
        straight_spec(10.0, 0.5),
        no_dynamics(),
        &[pt(0.0, 0.0, 1.0), pt(6.0, 0.0, 1.0), pt(12.0, 0.0, 1.0)],
    );
    for d in &dabs {
        assert!(d.center[1].abs() < 1e-3, "on the x axis");
    }
    assert!(
        dabs.iter().any(|d| (d.center[0] - 10.0).abs() < 1.5),
        "a dab near the 10px crossing: {:?}",
        dabs.iter().map(|d| d.center[0]).collect::<Vec<_>>()
    );
    assert!(
        dabs.last().unwrap().center[0] <= 12.0 + 1e-3,
        "never overshoots the drag"
    );
}

#[test]
fn zero_length_move_emits_nothing() {
    let mut s = Stroke::new(straight_spec(10.0, 0.5), no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(3.0, 3.0, 1.0), &mut out);
    s.extend(pt(3.0, 3.0, 1.0), &mut out);
    assert_eq!(out.len(), 0);
}

#[test]
fn pressure_interpolates_along_segment() {
    let dyn_ = Dynamics {
        size_pressure: true,
        size_min: 0.0,
        ..Default::default()
    };
    let mut s = Stroke::new(straight_spec(10.0, 0.5), dyn_, 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 0.0), &mut out);
    s.extend(pt(100.0, 0.0, 1.0), &mut out);
    assert!(out.len() >= 2);
    assert!(
        out[0].radius_px < out[out.len() - 1].radius_px,
        "radius rises with pressure"
    );
}

#[test]
fn jitter_is_deterministic_for_a_seed() {
    let spec = BrushSpec {
        jitter: 0.5,
        ..straight_spec(10.0, 0.5)
    };
    let run = || {
        let mut s = Stroke::new(spec, no_dynamics(), 42);
        let mut out = Vec::new();
        s.begin(pt(50.0, 50.0, 1.0), &mut out);
        s.extend(pt(150.0, 50.0, 1.0), &mut out);
        out
    };
    assert_eq!(run(), run(), "same seed ⟹ identical jittered dabs");
}

// ── new Stroke-section behaviour (Blender parity) ────────────────────────────────

#[test]
fn dots_emits_one_dab_per_event_ignoring_spacing() {
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Dots,
        ..straight_spec(10.0, 0.5) // step would be 10px under Space
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    assert_eq!(out.len(), 1, "Dots emits the down dab");
    // A single 100px move ⟹ exactly ONE dab at the end (no resampling), unlike Space (10).
    s.extend(pt(100.0, 0.0, 1.0), &mut out);
    assert_eq!(out.len(), 1);
    assert!((out[0].center[0] - 100.0).abs() < 1e-3);
}

#[test]
fn drag_dot_forces_full_pressure_and_no_jitter() {
    let spec = BrushSpec {
        stroke_method: StrokeMethod::DragDot,
        jitter: 1.0, // would scatter under Space/Dots
        ..straight_spec(10.0, 0.5)
    };
    let dyn_ = Dynamics {
        size_pressure: true,
        size_min: 0.0,
        ..Default::default()
    };
    let mut s = Stroke::new(spec, dyn_, 7);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 0.0), &mut out); // pressure 0
    s.extend(pt(40.0, 0.0, 0.0), &mut out);
    assert_eq!(out.len(), 1);
    // Pressure forced to 1 ⇒ full radius despite pressure-0 input; centre un-jittered.
    assert!(
        (out[0].radius_px - 10.0).abs() < 1e-4,
        "full pressure radius"
    );
    assert_eq!(out[0].center, [40.0, 0.0], "no jitter for DragDot");
}

#[test]
fn dots_use_the_stabilizer_like_space() {
    // Blender enables smooth-stroke for Dots too: with the stabilizer up, the dab is placed at the
    // lazy-mouse-filtered position (lagged), not the raw cursor (still one dab per event though).
    let spec = BrushSpec {
        stabilizer: 1.0,
        stroke_method: StrokeMethod::Dots,
        ..straight_spec(4.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    s.extend(pt(80.0, 0.0, 1.0), &mut out);
    assert_eq!(out.len(), 1, "Dots still emits exactly one dab per move");
    assert!(
        out[0].center[0] < 40.0,
        "with a heavy stabilizer the Dots dab should lag well behind the cursor (80); got {}",
        out[0].center[0]
    );
}

#[test]
fn drag_dot_ignores_the_stabilizer_and_sits_at_the_cursor() {
    // Drag Dot places its dab exactly at the cursor even with a heavy stabilizer — careful
    // positioning (Blender disables smooth-stroke for it). One dab per move at the raw cursor; the
    // tool turns the per-move dabs into a single moving dot.
    let spec = BrushSpec {
        stabilizer: 1.0,
        stroke_method: StrokeMethod::DragDot,
        ..straight_spec(4.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    s.extend(pt(80.0, 0.0, 1.0), &mut out);
    assert_eq!(out.len(), 1, "Drag Dot emits exactly one dab per move");
    assert!(
        (out[0].center[0] - 80.0).abs() < 1e-3,
        "Drag Dot dab must sit at the cursor (80), not lagged by the stabilizer; got {}",
        out[0].center[0]
    );
}

#[test]
fn input_samples_average_smooths_position() {
    // window 2: a dab's centre is the mean of the last two raw samples.
    let spec = BrushSpec {
        input_samples: 2,
        stroke_method: StrokeMethod::Dots,
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out); // window = [(0,0)]
    s.extend(pt(10.0, 0.0, 1.0), &mut out); // window = [(0,0),(10,0)] → mean (5,0)
    assert_eq!(out.len(), 1);
    assert!(
        (out[0].center[0] - 5.0).abs() < 1e-3,
        "got {}",
        out[0].center[0]
    );
}

#[test]
fn dash_gates_dabs_off() {
    // A straight drag: dashing (4 slots, ratio 0.2 → ~1/4 of slots painted) lays clearly fewer
    // dabs than solid (ratio 1.0). Per-dab gating is `slot % dash_samples / dash_samples <= ratio`.
    let drag = [pt(0.0, 0.0, 1.0), pt(200.0, 0.0, 1.0)];
    let solid = collect_stroke(
        BrushSpec {
            dash_samples: 20,
            dash_ratio: 1.0,
            ..straight_spec(10.0, 0.5)
        },
        no_dynamics(),
        &drag,
    );
    let dashed = collect_stroke(
        BrushSpec {
            dash_samples: 4,
            dash_ratio: 0.2,
            ..straight_spec(10.0, 0.5)
        },
        no_dynamics(),
        &drag,
    );
    assert!(!dashed.is_empty());
    assert!(
        dashed.len() < solid.len(),
        "dash gated dabs off: dashed {} vs solid {}",
        dashed.len(),
        solid.len()
    );
    // ~1/4 of slots painted (idx 0 of every 4) — well under half the solid count.
    assert!(
        (dashed.len() as f32) <= (solid.len() as f32) * 0.5,
        "dashed ≈ 1/4: dashed {} vs solid {}",
        dashed.len(),
        solid.len()
    );
}

#[test]
fn space_paints_up_to_the_cursor_each_event() {
    // Real-time: a SINGLE extend paints all the way to the new point — there is no half-segment
    // held back for the next event / pointer-up (the old quadratic-midpoint smoother stopped at
    // mid(prev,cur), so the stroke trailed the cursor). Checked on the first move (no tangent yet)
    // and a second collinear move (tangent set).
    let mut s = Stroke::new(straight_spec(4.0, 0.25), no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    s.extend(pt(40.0, 0.0, 1.0), &mut out);
    assert!(
        (out.last().unwrap().center[0] - 40.0).abs() < 1e-2,
        "first move reaches the cursor on the same event, got {:?}",
        out.last().unwrap().center
    );
    s.extend(pt(80.0, 0.0, 1.0), &mut out);
    assert!(
        (out.last().unwrap().center[0] - 80.0).abs() < 1e-2,
        "second move reaches the cursor too, got {:?}",
        out.last().unwrap().center
    );
}

#[test]
fn stabilizer_zero_keeps_the_raw_path() {
    // At intensity 0 there is no smoothing: the painted dabs sit on the raw input polyline, so the
    // apex sample (20,20) is reached exactly (within one dab spacing).
    let spec = BrushSpec {
        stabilizer: 0.0,
        ..straight_spec(2.0, 0.5)
    };
    let dabs = collect_stroke(
        spec,
        no_dynamics(),
        &[pt(0.0, 0.0, 1.0), pt(20.0, 20.0, 1.0), pt(40.0, 0.0, 1.0)],
    );
    assert!(
        dabs.iter()
            .any(|d| (d.center[0] - 20.0).abs() < 2.0 && (d.center[1] - 20.0).abs() < 2.0),
        "raw path should pass through the apex (20,20): {:?}",
        dabs.iter().map(|d| d.center).collect::<Vec<_>>()
    );
}

#[test]
fn stabilizer_regularizes_a_jittery_line() {
    // A horizontal line drawn with ±6 px vertical hand tremor. The stabilizer must flatten it:
    // at full intensity the painted path stays much closer to the centre line than the raw path.
    let zig = [
        pt(0.0, 0.0, 1.0),
        pt(10.0, 6.0, 1.0),
        pt(20.0, -6.0, 1.0),
        pt(30.0, 6.0, 1.0),
        pt(40.0, -6.0, 1.0),
        pt(50.0, 0.0, 1.0),
    ];
    let max_abs_y = |stab: f32| {
        let spec = BrushSpec {
            stabilizer: stab,
            ..straight_spec(2.0, 0.5)
        };
        collect_stroke(spec, no_dynamics(), &zig)
            .iter()
            .map(|d| d.center[1].abs())
            .fold(0.0_f32, f32::max)
    };
    let raw = max_abs_y(0.0);
    let smooth = max_abs_y(0.95);
    assert!(raw > 4.0, "raw path should follow the ±6 tremor, got {raw}");
    assert!(
        smooth < raw * 0.6,
        "stabilizer did not regularise the line: raw amplitude {raw:.1} vs stabilized {smooth:.1}"
    );
}

#[test]
fn settle_catches_the_stroke_up_to_the_cursor_on_a_pause() {
    // High stabilizer: one move leaves the painted point lagging far behind the cursor. Repeated
    // settle ticks (pointer parked, NO pointer-up) must walk the stroke up to the cursor.
    let spec = BrushSpec {
        stabilizer: 1.0,
        ..straight_spec(2.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    let mut all = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    s.extend(pt(80.0, 0.0, 1.0), &mut out);
    all.extend_from_slice(&out);
    let after_move = all.last().map(|d| d.center[0]).unwrap_or(0.0);
    assert!(
        after_move < 40.0,
        "heavy stabilizer should lag far behind the cursor on a single move, got {after_move}"
    );
    // Park the pointer and tick: the stroke should reach the cursor without a pointer-up.
    for _ in 0..120 {
        s.settle(&mut out);
        all.extend_from_slice(&out);
    }
    let last = all.last().unwrap().center[0];
    assert!(
        (last - 80.0).abs() < 2.0,
        "settle did not catch the stroke up to the parked cursor (80), got {last}"
    );
}

#[test]
fn settle_is_a_noop_without_lag() {
    // Stabilizer 0 (no lag) ⇒ settle emits nothing (there is nothing to catch up).
    let spec = BrushSpec {
        stabilizer: 0.0,
        ..straight_spec(2.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    s.extend(pt(40.0, 0.0, 1.0), &mut out);
    s.settle(&mut out);
    assert!(
        out.is_empty(),
        "settle should be a no-op with the stabilizer off"
    );
}

#[test]
fn stabilizer_catches_up_to_release_on_finish() {
    // A heavy stabilizer lags the painted point far behind the cursor, but pointer-up must flush
    // the lag so the stroke ends exactly at the release point (no truncated stroke).
    let spec = BrushSpec {
        stabilizer: 1.0,
        ..straight_spec(2.0, 0.5)
    };
    let dabs = collect_stroke(
        spec,
        no_dynamics(),
        &[pt(0.0, 0.0, 1.0), pt(60.0, 0.0, 1.0)],
    );
    let last_x = dabs.last().unwrap().center[0];
    assert!(
        (last_x - 60.0).abs() < 2.0,
        "stabilized stroke did not reach the release point, last dab x = {last_x}"
    );
}

#[test]
fn space_attenuation_reduces_coverage_below_full_spacing() {
    // spacing 0.1 (10%) with attenuation on ⇒ overlap factor < 1 ⇒ coverage < strength.
    let spec = BrushSpec {
        radius_px: 10.0,
        spacing: 0.1,
        strength: 1.0,
        falloff: Falloff::Smooth,
        space_attenuation: true,
        ..Default::default()
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    assert!(
        out[0].coverage < 1.0,
        "dense spacing attenuates: {}",
        out[0].coverage
    );
    assert!(out[0].coverage > 0.0);
    // With attenuation off, the same dab is full strength.
    let mut s2 = Stroke::new(
        BrushSpec {
            space_attenuation: false,
            ..spec
        },
        no_dynamics(),
        1,
    );
    let mut out2 = Vec::new();
    s2.begin(pt(0.0, 0.0, 1.0), &mut out2);
    assert!((out2[0].coverage - 1.0).abs() < 1e-4);
}

#[test]
fn fill_segment_lays_a_spaced_line() {
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Line,
        ..straight_spec(10.0, 0.5) // step 10px
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    assert_eq!(out.len(), 0, "Line does not paint on begin");
    s.fill_segment([0.0, 0.0], [50.0, 0.0], 1.0, &mut out);
    // start dab + dabs at 10,20,30,40,50 ⟹ 6.
    assert_eq!(
        out.len(),
        6,
        "got {:?}",
        out.iter().map(|d| d.center[0]).collect::<Vec<_>>()
    );
    assert_eq!(out[0].center, [0.0, 0.0]);
    assert!((out[5].center[0] - 50.0).abs() < 1e-3);
}

#[test]
fn airbrush_tick_emits_by_time() {
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Airbrush,
        airbrush_rate_s: 0.1,
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(5.0, 5.0, 1.0), &mut out); // down dab
    assert_eq!(out.len(), 1);
    // 0.25s parked at rate 0.1s ⇒ 2 dabs (0.1, 0.2), 0.05 left over.
    s.tick(0.25, &mut out);
    assert_eq!(out.len(), 2);
    assert!(
        out.iter().all(|d| d.center == [5.0, 5.0]),
        "parked at the cursor"
    );
}

#[test]
fn airbrush_does_not_emit_on_motion_only_tracks_the_cursor() {
    // Blender airbrush deposits dabs ONLY on the timer (TIMER events), never on motion. A move
    // updates the cursor the next tick deposits at; it must NOT lay a dab itself (that would be
    // Dots). Regression: `extend` used to share the Dots arm and emit one dab per move.
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Airbrush,
        airbrush_rate_s: 0.1,
        stabilizer: 0.0, // raw position so the tick lands exactly where the cursor moved
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out); // one down dab
    assert_eq!(out.len(), 1, "down emits the first dab");

    // Several moves with NO time elapsed (no tick): airbrush must emit nothing.
    s.extend(pt(20.0, 0.0, 1.0), &mut out);
    assert!(
        out.is_empty(),
        "move 1 emitted a dab — airbrush must not paint on motion"
    );
    s.extend(pt(40.0, 0.0, 1.0), &mut out);
    assert!(
        out.is_empty(),
        "move 2 emitted a dab — airbrush must not paint on motion"
    );

    // Now a tick fires the timer: one dab at the LAST moved-to position (40, 0), not the down point.
    s.tick(0.1, &mut out);
    assert_eq!(out.len(), 1, "one rate period ⇒ one timed dab");
    assert!(
        (out[0].center[0] - 40.0).abs() < 1e-3 && out[0].center[1].abs() < 1e-3,
        "the timed dab lands where the cursor was tracked to, got {:?}",
        out[0].center
    );
}

#[test]
fn anchored_radius_is_the_drag_distance_centred_on_the_anchor() {
    // Blender Anchored: a single stamp pinned at the press point; its radius = the distance dragged
    // from it (the Size slider is overridden). No dab on begin (it's interactive).
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Anchored,
        edge_to_edge: false,
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(20.0, 20.0, 1.0), &mut out);
    assert!(
        out.is_empty(),
        "anchored does not stamp on begin (interactive)"
    );

    // Drag to (50, 60): displacement (30, 40) → distance 50, centred on the anchor.
    s.extend(pt(50.0, 60.0, 1.0), &mut out);
    assert_eq!(out.len(), 1, "one anchored dab per move");
    assert_eq!(
        out[0].center,
        [20.0, 20.0],
        "centred on the anchor (press point)"
    );
    assert!(
        (out[0].radius_px - 50.0).abs() < 1e-3,
        "radius = drag distance, got {}",
        out[0].radius_px
    );

    // Dragging further GROWS the radius; the anchor stays put (no trail — the tool re-stamps).
    s.extend(pt(20.0, 120.0, 1.0), &mut out); // 100 px straight down from the anchor
    assert_eq!(out.len(), 1);
    assert_eq!(
        out[0].center,
        [20.0, 20.0],
        "anchor stays fixed across moves"
    );
    assert!((out[0].radius_px - 100.0).abs() < 1e-3);
}

#[test]
fn anchored_edge_to_edge_spans_anchor_to_cursor() {
    // Edge to Edge: the stamp is centred on the anchor↔cursor midpoint with HALF the radius, so it
    // spans exactly from the press point to the cursor (Blender `BRUSH_EDGE_TO_EDGE`).
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Anchored,
        edge_to_edge: true,
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    s.extend(pt(100.0, 0.0, 1.0), &mut out); // drag 100 px along +x
    assert_eq!(out.len(), 1);
    assert_eq!(
        out[0].center,
        [50.0, 0.0],
        "centred on the anchor↔cursor midpoint"
    );
    assert!(
        (out[0].radius_px - 50.0).abs() < 1e-3,
        "radius = half the drag distance (spans 0→100), got {}",
        out[0].radius_px
    );
}

#[test]
fn fill_polyline_preview_lays_spaced_dabs_along_a_bent_path() {
    // The Line method fills a multi-segment POLYLINE (the tool drives `fill_polyline_preview`): dabs are
    // laid at the brush spacing continuously across the corner; the first dab is at the start, and the
    // path reaches each vertex. radius 10 / spacing 0.5 → 10px step.
    let spec = straight_spec(10.0, 0.5);
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    // An L-shaped path: (0,0) → (100,0) → (100,100).
    s.fill_polyline_preview(&[[0.0, 0.0], [100.0, 0.0], [100.0, 100.0]], &mut out);
    assert!(
        out.len() >= 18,
        "both legs filled with spaced dabs, got {}",
        out.len()
    );
    assert_eq!(out[0].center, [0.0, 0.0], "first dab at the start");
    assert!(
        out.iter()
            .any(|d| (d.center[0] - 100.0).abs() < 1.0 && (d.center[1] - 100.0).abs() < 2.0),
        "the fill reaches the last vertex"
    );
}

#[test]
fn fill_polyline_preview_is_deterministic() {
    // Fresh-per-fill: identical params + seed ⇒ identical dabs (the tool builds a fresh `Stroke` each
    // re-fill, so the growing/edited polyline never shimmers). Jitter ON to exercise the RNG.
    let spec = BrushSpec {
        jitter: 0.5,
        ..straight_spec(10.0, 0.5)
    };
    let fill = || {
        let mut s = Stroke::new(spec, no_dynamics(), 7);
        let mut o = Vec::new();
        s.fill_polyline_preview(&[[0.0, 0.0], [80.0, 20.0], [120.0, 0.0]], &mut o);
        o
    };
    assert_eq!(fill(), fill(), "identical params → identical polyline fill");
}

#[test]
fn flatten_catmull_rom_keeps_collinear_points_on_the_line() {
    // The initial 3-point curve is collinear (start, midpoint, end) — auto-smoothing must leave it a
    // straight line, so it reads as a line until a point is dragged off it.
    let pts = [[0.0, 0.0], [50.0, 0.0], [100.0, 0.0]];
    let mut spine = Vec::new();
    flatten_catmull_rom(&pts, &mut spine);
    assert!(spine.len() > 3, "subdivided into a dense polyline");
    assert_eq!(
        spine.first(),
        Some(&[0.0, 0.0]),
        "starts at the first point"
    );
    assert_eq!(spine.last(), Some(&[100.0, 0.0]), "ends at the last point");
    for p in &spine {
        assert!(p[1].abs() < 1e-3, "stays on the x axis (collinear): {p:?}");
    }
}

#[test]
fn flatten_catmull_rom_bends_off_a_moved_midpoint() {
    // Drag the midpoint off the line → the spline must bow toward it (some interior point leaves y=0).
    let pts = [[0.0, 0.0], [50.0, 40.0], [100.0, 0.0]];
    let mut spine = Vec::new();
    flatten_catmull_rom(&pts, &mut spine);
    assert!(
        spine.iter().any(|p| p[1] > 1.0),
        "the curve bows toward the moved midpoint"
    );
}

#[test]
fn curve_fills_spaced_dabs_along_the_spline() {
    // Curve: author 3 points → the engine auto-smooths + lays spaced dabs along the whole spline.
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Arc,
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    let pts = [[0.0, 0.0], [50.0, 40.0], [100.0, 0.0]];
    s.fill_curve_preview(&pts, &mut out);
    assert!(
        out.len() >= 10,
        "filled the curve with spaced dabs, got {}",
        out.len()
    );
    assert!(
        out[0].center[0].abs() < 1e-3,
        "first dab at the first control point"
    );
    assert!(
        (out.last().unwrap().center[0] - 100.0).abs() < 2.0,
        "reaches the last control point"
    );
    // The spline bows up, so some dab must sit clearly above the chord.
    assert!(
        out.iter().any(|d| d.center[1] > 1.0),
        "dabs follow the bowed curve, not the straight chord"
    );
}

#[test]
fn curve_preview_is_deterministic_per_fill() {
    // A fresh Stroke per re-fill with the SAME points + seed yields IDENTICAL dabs (no shimmer as the
    // curve is reshaped), even with jitter on — the engine fill is a pure function of (points, spec).
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Arc,
        jitter: 0.5,
        ..straight_spec(10.0, 0.5)
    };
    let pts = [[0.0, 0.0], [40.0, 30.0], [90.0, 10.0]];
    let mut a = Vec::new();
    let mut b = Vec::new();
    Stroke::new(spec, no_dynamics(), 9).fill_curve_preview(&pts, &mut a);
    Stroke::new(spec, no_dynamics(), 9).fill_curve_preview(&pts, &mut b);
    assert!(!a.is_empty());
    assert_eq!(a, b, "same points + seed ⇒ identical dabs");
}

#[test]
fn ellipse_perimeter_is_round_for_a_circle() {
    // rx == ry → every perimeter point sits at distance ~R from the centre (a circle), with no
    // transcendentals. Check the chord deviation is sub-pixel.
    let mut out = Vec::new();
    let (cx, cy, r) = (100.0_f32, 100.0_f32, 40.0_f32);
    ellipse_perimeter([cx, cy], [1.0, 0.0], r, r, &mut out);
    assert!(
        out.len() >= 32,
        "subdivided into a dense polyline, got {}",
        out.len()
    );
    for p in &out {
        let d = ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt();
        assert!((d - r).abs() < 0.2, "point off the circle: dist {d} vs {r}");
    }
    // Spans the full circle (has points on both sides of the centre, both axes).
    assert!(out.iter().any(|p| p[0] > cx + r * 0.9));
    assert!(out.iter().any(|p| p[0] < cx - r * 0.9));
    assert!(out.iter().any(|p| p[1] > cy + r * 0.9));
    assert!(out.iter().any(|p| p[1] < cy - r * 0.9));
}

#[test]
fn ellipse_perimeter_respects_axes_and_orientation() {
    // An axis-aligned ellipse rx=60, ry=20: extents are ±60 in x, ±20 in y.
    let mut out = Vec::new();
    ellipse_perimeter([0.0, 0.0], [1.0, 0.0], 60.0, 20.0, &mut out);
    let max_x = out.iter().fold(0.0_f32, |m, p| m.max(p[0].abs()));
    let max_y = out.iter().fold(0.0_f32, |m, p| m.max(p[1].abs()));
    assert!((max_x - 60.0).abs() < 0.5, "x extent ~rx: {max_x}");
    assert!((max_y - 20.0).abs() < 0.5, "y extent ~ry: {max_y}");
    // Rotate the SAME ellipse 90° (u = +y) → the long axis is now vertical.
    let mut rot = Vec::new();
    ellipse_perimeter([0.0, 0.0], [0.0, 1.0], 60.0, 20.0, &mut rot);
    let rmax_x = rot.iter().fold(0.0_f32, |m, p| m.max(p[0].abs()));
    let rmax_y = rot.iter().fold(0.0_f32, |m, p| m.max(p[1].abs()));
    assert!(
        (rmax_x - 20.0).abs() < 0.5,
        "rotated: x extent ~ry: {rmax_x}"
    );
    assert!(
        (rmax_y - 60.0).abs() < 0.5,
        "rotated: y extent ~rx: {rmax_y}"
    );
}

#[test]
fn circle_fills_spaced_dabs_around_the_perimeter() {
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Ellipse,
        ..straight_spec(6.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    let (cx, cy, r) = (100.0_f32, 100.0_f32, 40.0_f32);
    s.fill_ellipse_preview([cx, cy], [1.0, 0.0], r, r, &mut out);
    assert!(
        out.len() >= 16,
        "filled the perimeter with spaced dabs, got {}",
        out.len()
    );
    // Every dab sits on the ring (radius ~r, within a dab radius of tolerance).
    for d in &out {
        let dist = ((d.center[0] - cx).powi(2) + (d.center[1] - cy).powi(2)).sqrt();
        assert!(
            (dist - r).abs() < 8.0,
            "dab off the ring: dist {dist} vs {r}"
        );
    }
    // The loop is closed: a dab near the top AND near the bottom of the ring.
    assert!(
        out.iter().any(|d| d.center[1] > cy + r * 0.8),
        "covers the top"
    );
    assert!(
        out.iter().any(|d| d.center[1] < cy - r * 0.8),
        "covers the bottom"
    );
}

#[test]
fn circle_preview_is_deterministic_per_fill() {
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Ellipse,
        jitter: 0.5,
        ..straight_spec(6.0, 0.5)
    };
    let mut a = Vec::new();
    let mut b = Vec::new();
    Stroke::new(spec, no_dynamics(), 5).fill_ellipse_preview(
        [50.0, 50.0],
        [1.0, 0.0],
        30.0,
        18.0,
        &mut a,
    );
    Stroke::new(spec, no_dynamics(), 5).fill_ellipse_preview(
        [50.0, 50.0],
        [1.0, 0.0],
        30.0,
        18.0,
        &mut b,
    );
    assert!(!a.is_empty());
    assert_eq!(a, b, "same params + seed ⇒ identical dabs");
}

#[test]
fn circle_degenerate_axis_fills_nothing() {
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Ellipse,
        ..straight_spec(6.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = vec![Dab {
        center: [9.0, 9.0],
        radius_px: 1.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
    }];
    s.fill_ellipse_preview([10.0, 10.0], [1.0, 0.0], 30.0, 0.2, &mut out);
    assert!(
        out.is_empty(),
        "a near-zero axis fills nothing (and clears the buffer)"
    );
}

#[test]
fn polygon_perimeter_has_n_vertices_on_the_ellipse() {
    // A regular polygon inscribed in a circle (rx == ry): exactly `n` vertices, each at radius R.
    let (cx, cy, r) = (100.0_f32, 100.0_f32, 40.0_f32);
    for n in 3u32..=12 {
        let mut out = Vec::new();
        polygon_perimeter([cx, cy], [1.0, 0.0], r, r, n, &mut out);
        assert_eq!(out.len(), n as usize, "{n}-gon has {n} vertices");
        for p in &out {
            let d = ((p[0] - cx).powi(2) + (p[1] - cy).powi(2)).sqrt();
            assert!(
                (d - r).abs() < 0.02,
                "{n}-gon vertex off the circle: {d} vs {r}"
            );
        }
    }
    // First vertex sits at the top (+y): center + (0, r).
    let mut tri = Vec::new();
    polygon_perimeter([cx, cy], [1.0, 0.0], r, r, 3, &mut tri);
    assert!(
        (tri[0][0] - cx).abs() < 0.01 && (tri[0][1] - (cy + r)).abs() < 0.01,
        "first vertex at the top: {:?}",
        tri[0]
    );
}

#[test]
fn polygon_side_count_clamps() {
    let mut out = Vec::new();
    polygon_perimeter([0.0, 0.0], [1.0, 0.0], 20.0, 20.0, 2, &mut out); // below min
    assert_eq!(out.len(), 3, "clamps up to 3 sides");
    polygon_perimeter([0.0, 0.0], [1.0, 0.0], 20.0, 20.0, 99, &mut out); // above max
    assert_eq!(out.len(), 12, "clamps down to 12 sides");
}

#[test]
fn polygon_fills_spaced_dabs_around_the_perimeter() {
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Polygon,
        ..straight_spec(6.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    let (cx, cy, r) = (100.0_f32, 100.0_f32, 40.0_f32);
    s.fill_polygon_preview([cx, cy], [1.0, 0.0], r, r, 5, &mut out);
    assert!(
        out.len() >= 12,
        "filled the pentagon with spaced dabs, got {}",
        out.len()
    );
    // Every dab lies inside the bounding circle (within a dab radius of the rim).
    for d in &out {
        let dist = ((d.center[0] - cx).powi(2) + (d.center[1] - cy).powi(2)).sqrt();
        assert!(dist <= r + 8.0, "dab outside the polygon: {dist} vs {r}");
    }
    // The first vertex (top) is covered, and so is the bottom region (closed loop).
    assert!(
        out.iter().any(|d| d.center[1] > cy + r * 0.8),
        "covers the top vertex"
    );
    assert!(
        out.iter().any(|d| d.center[1] < cy - r * 0.3),
        "covers the bottom edge"
    );
}

#[test]
fn polygon_preview_is_deterministic_per_fill() {
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Polygon,
        jitter: 0.5,
        ..straight_spec(6.0, 0.5)
    };
    let mut a = Vec::new();
    let mut b = Vec::new();
    Stroke::new(spec, no_dynamics(), 5).fill_polygon_preview(
        [50.0, 50.0],
        [1.0, 0.0],
        30.0,
        18.0,
        7,
        &mut a,
    );
    Stroke::new(spec, no_dynamics(), 5).fill_polygon_preview(
        [50.0, 50.0],
        [1.0, 0.0],
        30.0,
        18.0,
        7,
        &mut b,
    );
    assert!(!a.is_empty());
    assert_eq!(a, b, "same params + seed ⇒ identical dabs");
}

#[test]
fn curve_fill_needs_two_points() {
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Arc,
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = vec![Dab {
        center: [9.0, 9.0],
        radius_px: 1.0,
        coverage: 1.0,
        color: [0.0, 0.0, 0.0],
        rotation: [1.0, 0.0],
        dir: [0.0, 0.0],
    }];
    s.fill_curve_preview(&[[5.0, 5.0]], &mut out);
    assert!(
        out.is_empty(),
        "a single control point fills nothing (and clears the buffer)"
    );
}

// ── Per-dab jitter (Jitter Scale / Rotate / Randomize Color) ─────────────────────────────────

/// A long straight, constant-pressure stroke with attenuation off — every dab is identical unless a
/// per-dab jitter perturbs it, so it isolates the jitter under test.
fn jitter_probe(extra: impl FnOnce(&mut BrushSpec)) -> Vec<Dab> {
    let mut spec = straight_spec(10.0, 0.5);
    extra(&mut spec);
    collect_stroke(
        spec,
        no_dynamics(),
        &[pt(0.0, 0.0, 1.0), pt(200.0, 0.0, 1.0)],
    )
}

#[test]
fn no_jitter_dabs_are_uniform_baseline() {
    let dabs = jitter_probe(|_| {});
    assert!(dabs.len() > 3);
    for d in &dabs {
        assert_eq!(d.radius_px, 10.0, "radius constant with no Jitter Scale");
        assert_eq!(
            d.color,
            [0.0, 0.0, 0.0],
            "colour constant with Randomize off"
        );
        assert_eq!(
            d.rotation,
            [1.0, 0.0],
            "rotation identity with no Jitter Rotate"
        );
    }
}

#[test]
fn jitter_scale_varies_radius_deterministically() {
    let a = jitter_probe(|s| s.jitter_scale = 0.5);
    let b = jitter_probe(|s| s.jitter_scale = 0.5);
    // Same seed → identical (replayable).
    assert_eq!(
        a.iter().map(|d| d.radius_px).collect::<Vec<_>>(),
        b.iter().map(|d| d.radius_px).collect::<Vec<_>>(),
    );
    // Radii actually scatter, and never collapse below the floor.
    let distinct = a.iter().any(|d| (d.radius_px - 10.0).abs() > 1e-4);
    assert!(distinct, "Jitter Scale must vary the radius");
    assert!(a.iter().all(|d| d.radius_px >= 0.5));
}

#[test]
fn randomize_color_varies_colour_deterministically() {
    let cfg = |s: &mut BrushSpec| {
        s.color = [0.5, 0.5, 0.5];
        s.color_jitter_enabled = true;
        s.color_jitter_hue = 0.5;
        s.color_jitter_sat = 0.3;
        s.color_jitter_val = 0.3;
    };
    let a = jitter_probe(cfg);
    let b = jitter_probe(cfg);
    assert_eq!(
        a.iter().map(|d| d.color).collect::<Vec<_>>(),
        b.iter().map(|d| d.color).collect::<Vec<_>>(),
    );
    assert!(
        a.iter().any(|d| d.color != [0.5, 0.5, 0.5]),
        "Randomize Color must vary the colour"
    );
    // Enabled but every amount zero ⇒ no visible change (and the base colour is preserved).
    let none = jitter_probe(|s| {
        s.color = [0.2, 0.4, 0.8];
        s.color_jitter_enabled = true;
    });
    assert!(none.iter().all(|d| d.color == [0.2, 0.4, 0.8]));
}

#[test]
fn jitter_rotate_varies_rotation_deterministically() {
    let a = jitter_probe(|s| s.jitter_rotate = 1.0);
    let b = jitter_probe(|s| s.jitter_rotate = 1.0);
    assert_eq!(
        a.iter().map(|d| d.rotation).collect::<Vec<_>>(),
        b.iter().map(|d| d.rotation).collect::<Vec<_>>(),
    );
    assert!(
        a.iter().any(|d| d.rotation != [1.0, 0.0]),
        "Jitter Rotate must produce non-identity rotations"
    );
    // Rotation vectors stay unit-length (transcendental-free rotation can't drift far).
    for d in &a {
        let len2 = d.rotation[0] * d.rotation[0] + d.rotation[1] * d.rotation[1];
        assert!((len2 - 1.0).abs() < 1e-3, "rotation must stay unit length");
    }
}

#[test]
fn dragdot_opts_out_of_per_dab_jitter() {
    // Drag Dot disables jitter (like position jitter); colour/rotation/scale all stay at base.
    let mut spec = straight_spec(10.0, 0.5);
    spec.stroke_method = StrokeMethod::DragDot;
    spec.jitter_scale = 0.8;
    spec.jitter_rotate = 1.0;
    spec.color = [0.5, 0.5, 0.5];
    spec.color_jitter_enabled = true;
    spec.color_jitter_hue = 0.5;
    let dabs = collect_stroke(
        spec,
        no_dynamics(),
        &[pt(0.0, 0.0, 1.0), pt(40.0, 0.0, 1.0), pt(80.0, 0.0, 1.0)],
    );
    assert!(!dabs.is_empty());
    for d in &dabs {
        assert_eq!(d.radius_px, 10.0);
        assert_eq!(d.color, [0.5, 0.5, 0.5]);
        assert_eq!(d.rotation, [1.0, 0.0]);
    }
}

// ── Rake heading (Dab::dir) — the proof that "rotation follows the stroke" actually works ──────────
//
// These are the tests the old `advance_rake` (reconstructed from dab centres) could not pass: a stable
// per-dab heading that tracks the path tangent on a CURVE without oscillating. `f32::sin/cos` here only
// GENERATE the arc input — the engine's heading filter ([`crate::heading`]) is transcendental-free.

/// 2D cross product sign (`+` = left turn / counter-clockwise from `a` to `b`).
fn cross(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

// ── Rake warm-up (deferred start) — the stroke opens already at the settled angle ──────────────────

/// A `straight_spec` with the texture **Rake** on, so the warm-up engages (it gates on the rake flag).
fn rake_spec(radius: f32, spacing: f32) -> BrushSpec {
    let mut spec = straight_spec(radius, spacing);
    spec.texture.rake = true;
    spec
}

#[test]
fn dots_rake_advances_the_heading_and_releases_in_real_time() {
    // The Dots+Rake bug: Dots emits per event (no spline walk) and so never advanced the heading —
    // `Dab::dir` stayed [0,0] (Rake fell back to Angle) AND the warm-up never released (its gate needs a
    // heading), so the whole stroke only appeared on pointer-up. Now Dots advances the heading from the
    // inter-dab travel: the dabs follow the stroke AND appear DURING it.
    let mut spec = rake_spec(10.0, 0.5); // Rake on → warm-up engages; diameter 20 → warm-up 3px
    spec.stroke_method = StrokeMethod::Dots;
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out); // down dab (held during warm-up)
    let mut during = out.len();
    for k in 1..=8 {
        s.extend(pt(k as f32 * 12.0, 0.0, 1.0), &mut out); // straight +x run
        during += out.len();
    }
    assert!(
        during >= 5,
        "Dots dabs appear DURING the stroke, not only on finish (got {during})"
    );
    // The last (post-warm-up) batch's dab follows the +x stroke direction (Rake), not the rest Angle.
    let d = out
        .iter()
        .find(|d| d.dir != [0.0, 0.0])
        .expect("a Dots dab carries the Rake heading");
    assert!(
        (d.dir[0] - 1.0).abs() < 0.2 && d.dir[1].abs() < 0.2,
        "Dots Rake heading follows +x: {:?}",
        d.dir
    );
}

#[test]
fn rake_warmup_holds_the_opening_dabs_then_releases_them_at_the_settled_angle() {
    // Press + a tiny move (< warm-up length) must emit NOTHING — the opening is held back until the
    // stroke direction is known. Then a move past the warm-up length releases the whole held batch,
    // and EVERY released dab — including the very first one at the press point — carries the stroke
    // heading (+x), not the rest [0,0]. That is the fix: the stroke starts already at the right angle.
    let mut s = Stroke::new(rake_spec(10.0, 0.5), no_dynamics(), 1); // diameter 20 → warm-up 3px
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    assert!(out.is_empty(), "the down dab is held during warm-up");
    s.extend(pt(2.0, 0.0, 1.0), &mut out); // 2px: below the warm-up length (and the dab spacing) → holds
    assert!(out.is_empty(), "a sub-threshold move keeps holding");
    s.extend(pt(60.0, 0.0, 1.0), &mut out); // crosses the warm-up length → release
    assert!(
        out.len() >= 2,
        "the held opening is released once the angle is known"
    );
    assert!(
        out[0].center[0].abs() < 1e-3,
        "first released dab is the press-point dab, back-filled: {:?}",
        out[0].center
    );
    for d in &out {
        assert!(
            (d.dir[0] - 1.0).abs() < 1e-2 && d.dir[1].abs() < 1e-2,
            "every opening dab opens at the stroke angle +x, not [0,0]: {:?}",
            d.dir
        );
    }
}

#[test]
fn rake_warmup_releases_a_short_tap_on_finish_at_the_rest_angle() {
    // A tap (press + release, no travel) never defines a heading. `finish` must still flush the held
    // down dab — at the rest angle [0,0] (→ the bare Angle), since a directionless tap has no heading.
    let mut s = Stroke::new(rake_spec(10.0, 0.5), no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(5.0, 5.0, 1.0), &mut out);
    assert!(out.is_empty(), "held during warm-up");
    s.finish(&mut out);
    assert_eq!(out.len(), 1, "the tap's single dab is flushed on finish");
    assert_eq!(
        out[0].dir,
        [0.0, 0.0],
        "no travel ⇒ rest angle (bare Angle)"
    );
}

#[test]
fn non_rake_brush_is_unaffected_by_the_warmup() {
    // The warm-up gates on the Rake flag: a plain brush emits the down dab immediately, exactly as
    // before — no start latency for the common (non-Rake) case.
    let mut s = Stroke::new(straight_spec(10.0, 0.5), no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    assert_eq!(
        out.len(),
        1,
        "non-Rake down dab emits immediately (no warm-up)"
    );
}

#[test]
fn straight_stroke_gives_a_constant_heading() {
    // A straight drag: every dab past the first must carry essentially the SAME heading (+x), with no
    // dab-to-dab wobble. (The first dab is at the down point, before any travel → heading [0,0].)
    let dabs = collect_stroke(
        straight_spec(10.0, 0.5),
        no_dynamics(),
        &[pt(0.0, 0.0, 1.0), pt(60.0, 0.0, 1.0), pt(120.0, 0.0, 1.0)],
    );
    let moving: Vec<[f32; 2]> = dabs
        .iter()
        .map(|d| d.dir)
        .filter(|d| *d != [0.0, 0.0])
        .collect();
    assert!(moving.len() >= 5, "enough dabs to judge stability");
    for d in &moving {
        assert!(
            (d[0] - 1.0).abs() < 1e-3 && d[1].abs() < 1e-3,
            "heading is +x: {d:?}"
        );
    }
}

#[test]
fn arc_stroke_heading_tracks_the_tangent_and_rotates_monotonically() {
    // Paint a quarter-circle arc (centre origin, radius R) from angle 0 to 90°, WITH the stabilizer on
    // (the realistic case — the spline + lazy-mouse smoothing is exactly what made the old chord-based
    // rake oscillate). Assert each dab's heading (a) points roughly along the local arc tangent and
    // (b) rotates monotonically counter-clockwise — the texture frame turns WITH the stroke, smoothly.
    const R: f32 = 200.0;
    let mut spec = straight_spec(8.0, 0.4);
    spec.stabilizer = 0.5; // the default-ish smoothing that corrupted the old reconstructed direction
    let mut pts = Vec::new();
    let steps = 60;
    for i in 0..=steps {
        let th = (i as f32 / steps as f32) * std::f32::consts::FRAC_PI_2;
        pts.push(pt(R * th.cos(), R * th.sin(), 1.0));
    }
    let dabs = collect_stroke(spec, no_dynamics(), &pts);
    let moving: Vec<&Dab> = dabs.iter().filter(|d| d.dir != [0.0, 0.0]).collect();
    assert!(moving.len() >= 10, "got {} moving dabs", moving.len());

    // (a) Each heading is close to the IDEAL tangent at its position. For a point on the circle the
    //     travel tangent (increasing θ) is perpendicular to the radius, turned counter-clockwise:
    //     tangent = normalize(perp(centre→point)) = (-y, x)/R. Allow a modest lag tolerance (the EMA
    //     trails the instantaneous tangent slightly — that is the whole point of smoothing).
    for d in &moving {
        let r = d.center; // centre is origin
        let rl = (r[0] * r[0] + r[1] * r[1]).sqrt();
        let tangent = [-r[1] / rl, r[0] / rl];
        let dot = d.dir[0] * tangent[0] + d.dir[1] * tangent[1];
        assert!(
            dot > 0.9,
            "heading aligns with the arc tangent (dot={dot:.3}) at {:?}",
            d.center
        );
    }

    // (b) Monotonic rotation: consecutive headings turn the SAME way (CCW here) and never jitter back.
    //     A single sign flip would be the old "anarchy". Tiny negative slack absorbs float noise.
    let mut reversals = 0;
    for w in moving.windows(2) {
        let turn = cross(w[0].dir, w[1].dir);
        if turn < -1e-4 {
            reversals += 1;
        }
    }
    assert_eq!(
        reversals, 0,
        "heading rotates monotonically, no dab-to-dab oscillation"
    );

    // And it actually swept ~90° overall: first heading ≈ +y, last ≈ -x (a quarter turn CCW).
    let first = moving.first().unwrap().dir;
    let last = moving.last().unwrap().dir;
    assert!(first[1] > 0.85, "starts heading ~+y: {first:?}");
    assert!(last[0] < -0.85, "ends heading ~-x: {last:?}");
}

#[test]
fn the_rake_heading_does_not_lag_the_stroke_on_a_large_brush() {
    // The sibling above checks radius 8 with a 25° tolerance — and STAYED GREEN over the reported bug,
    // because the old Rake heading (the length-weighted EMA) trailed the arc tangent by a lag that
    // scaled with the brush: measured 5.9° at radius 8 but **51.7° at radius 60** — half a right angle,
    // read by the artist as *"Rake não consegue rotacionar o brush"* (Enio 2026-07-19). `Dab::dir` is
    // now the direction between consecutive dab CENTRES (exact path samples), whose lag is ~half a dab
    // spacing REGARDLESS of the brush (measured 6.9° at radius 60, matching the Sculpt Chisel's cure).
    // Assert the worst lag stays under 15° on the big brush the EMA failed. Red-first: reverting
    // `dab_at` to `dir: self.heading` puts the worst lag back to 51.7° and fails this.
    const R: f32 = 200.0;
    let mut spec = straight_spec(60.0, 0.4); // the brush size where the old EMA lag was worst
    spec.stabilizer = 0.5; // the realistic smoothing (spline + lazy-mouse), as the sibling uses
    let mut pts = Vec::new();
    for i in 0..=60 {
        let th = (i as f32 / 60.0) * std::f32::consts::FRAC_PI_2;
        pts.push(pt(R * th.cos(), R * th.sin(), 1.0));
    }
    let dabs = collect_stroke(spec, no_dynamics(), &pts);
    let moving: Vec<&Dab> = dabs.iter().filter(|d| d.dir != [0.0, 0.0]).collect();
    assert!(moving.len() >= 5, "got {} moving dabs", moving.len());
    let mut worst = 1.0f32;
    for d in &moving {
        let r = d.center; // centre at the origin
        let rl = (r[0] * r[0] + r[1] * r[1]).sqrt();
        let tangent = [-r[1] / rl, r[0] / rl]; // CCW travel tangent at this arc position
        let dot = (d.dir[0] * tangent[0] + d.dir[1] * tangent[1]).clamp(-1.0, 1.0);
        worst = worst.min(dot);
    }
    // 15° (dot 0.966) cleanly separates the fix (6.9°) from the bug (51.7°).
    assert!(
        worst > 0.966,
        "the raked heading tracks the arc tangent within 15° on a large brush (worst {:.1}° lag)",
        worst.acos().to_degrees()
    );
}

#[test]
fn heading_is_independent_of_dab_spacing() {
    // Length-weighting guarantee: the SAME physical arc gives ~the same heading at a given arc position
    // whether dabs are dense or sparse — because the EMA blend is driven by DISTANCE travelled, not by
    // dab count. They are not bit-identical (a coarser spacing quantises the same continuous filter into
    // bigger steps), but they stay within ~15° (dot > 0.96), versus the old chord-direction scheme where
    // the heading was a function of inter-dab spacing and swung wildly. This is the property that made
    // the old rake unusable at different brush sizes / spacings.
    const R: f32 = 200.0;
    let arc = |spacing: f32| -> Vec<Dab> {
        let mut spec = straight_spec(8.0, spacing);
        spec.stabilizer = 0.5;
        let mut pts = Vec::new();
        for i in 0..=60 {
            let th = (i as f32 / 60.0) * std::f32::consts::FRAC_PI_2;
            pts.push(pt(R * th.cos(), R * th.sin(), 1.0));
        }
        collect_stroke(spec, no_dynamics(), &pts)
    };
    // Heading at the dab nearest 45° (centre of the quarter arc) for each spacing.
    let at_mid = |dabs: &[Dab]| -> [f32; 2] {
        let mid = [
            R * std::f32::consts::FRAC_1_SQRT_2,
            R * std::f32::consts::FRAC_1_SQRT_2,
        ];
        dabs.iter()
            .filter(|d| d.dir != [0.0, 0.0])
            .min_by(|a, b| {
                let da = (a.center[0] - mid[0]).hypot(a.center[1] - mid[1]);
                let db = (b.center[0] - mid[0]).hypot(b.center[1] - mid[1]);
                da.total_cmp(&db)
            })
            .unwrap()
            .dir
    };
    let dense = at_mid(&arc(0.2));
    let sparse = at_mid(&arc(0.8));
    let dot = dense[0] * sparse[0] + dense[1] * sparse[1];
    assert!(
        dot > 0.96,
        "dense vs sparse heading agree (dot={dot:.4}): {dense:?} vs {sparse:?}"
    );
}

// ── The heading warm-up serves every reader of `Dab::dir`, not just the two texture slots ────────────

/// A stroke whose consumer reads [`Dab::dir`] — but which rakes neither texture slot. The Sculpt **Chisel**.
fn chisel_like_spec() -> BrushSpec {
    BrushSpec {
        needs_heading: true,
        ..straight_spec(10.0, 0.5)
    }
}

/// **A stroke that says it needs a heading gets one on EVERY dab — including the first.**
///
/// The warm-up holds the opening dabs until enough travel has settled a direction, then flushes them at it
/// ([`crate::heading`]). It was gated on `texture.rake || shape.rake` — an enumeration of the readers of
/// `Dab::dir` written when the two texture slots were the only ones there were.
///
/// The Sculpt **Chisel** is a third reader: it carves a V about the stroke's axis, taken from `dir`. With the
/// warm-up off, the dab emitted at pen-down carries `dir = [0, 0]`, so `perp = [0, 0]`, so the tilt term is
/// zero — the V collapses and that dab cuts a **plain Scrape**. The groove starts blunt, on every stroke, and
/// the only way an artist could fix it was to tick a checkbox about a *silhouette image*: two doors to one
/// question, and they had already diverged.
///
/// **Mutation that must bleed:** drop `|| self.spec.needs_heading` from the `warming` gate in
/// [`super::Stroke::begin`] — the first dab comes back with `[0, 0]` and this fails. (Checked: it does.)
#[test]
fn a_stroke_that_needs_a_heading_gets_one_on_its_very_first_dab() {
    let dabs = collect_stroke(
        chisel_like_spec(),
        no_dynamics(),
        &[
            pt(20.0, 20.0, 1.0),
            pt(40.0, 20.0, 1.0),
            pt(60.0, 20.0, 1.0),
        ],
    );
    assert!(!dabs.is_empty(), "fixture: the stroke emitted no dabs");

    let blind = dabs.iter().filter(|d| d.dir == [0.0, 0.0]).count();
    assert_eq!(
        blind,
        0,
        "{blind} of {} dabs came out with no heading. Anything that reads `Dab::dir` gets NOTHING from those \
         — for the Chisel that means the V degenerates and the dab cuts a flat scrape instead.",
        dabs.len()
    );
    // …and the heading it settled on is the direction the stroke actually went (+x), not merely non-zero.
    assert!(
        dabs[0].dir[0] > 0.9 && dabs[0].dir[1].abs() < 0.1,
        "the first dab's heading is {:?}, not the stroke's direction (+x). A warm-up that flushes at a \
         heading nobody travelled is worse than none: the groove would start pointing somewhere the artist \
         never went.",
        dabs[0].dir
    );
}

/// The **presence sibling** of the gate above: without the flag, the opening dab really is blind.
///
/// An absence gate ("no dab lacks a heading") is green on a stroke that emits no dabs, and green on an engine
/// that hands every dab a heading for free — in which case `needs_heading` is dead code and the Chisel's bug
/// was somewhere else entirely. This is what proves the flag is the thing doing the work
/// ([[feedback_absence_gate_needs_a_presence_sibling]]).
#[test]
fn without_the_flag_the_opening_dab_is_blind() {
    let dabs = collect_stroke(
        straight_spec(10.0, 0.5), // the same brush, `needs_heading: false`
        no_dynamics(),
        &[
            pt(20.0, 20.0, 1.0),
            pt(40.0, 20.0, 1.0),
            pt(60.0, 20.0, 1.0),
        ],
    );
    assert!(
        dabs.iter().any(|d| d.dir == [0.0, 0.0]),
        "every dab of a NON-raking stroke already carries a heading, so the warm-up gate is not what was \
         starving the Chisel — and the gate above proves nothing. Go and find the real reader."
    );
}
