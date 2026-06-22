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
fn line_fills_a_straight_segment_with_spaced_dabs() {
    // Line: a straight line from the anchor (press point) to the cursor, filled with spaced dabs.
    // radius 10 → diameter 20, spacing 0.5 → 10 px step; a 0→100 drag lays ~11 dabs on the x axis.
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Line,
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out);
    assert!(out.is_empty(), "Line does not stamp on begin (interactive)");

    s.extend(pt(100.0, 0.0, 1.0), &mut out);
    assert!(
        out.len() >= 10,
        "filled the line with spaced dabs, got {}",
        out.len()
    );
    for d in &out {
        assert!(
            d.center[1].abs() < 1e-3,
            "dab on the line (x axis): {:?}",
            d.center
        );
    }
    assert!(out[0].center[0].abs() < 1e-3, "first dab at the anchor");
    assert!(
        (out.last().unwrap().center[0] - 100.0).abs() < 1e-2,
        "reaches the cursor"
    );
}

#[test]
fn line_preview_is_deterministic_across_moves() {
    // The live preview re-stamps each move, so the SAME anchor→cursor must yield IDENTICAL dabs
    // (the dash + jitter + spacing state is snapshot/restored each fill) — no shimmer as it grows.
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Line,
        jitter: 0.5, // jitter ON, to prove the RNG is restored between fills
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 7);
    s.begin(pt(0.0, 0.0, 1.0), &mut Vec::new());
    let mut a = Vec::new();
    let mut b = Vec::new();
    s.extend(pt(80.0, 20.0, 1.0), &mut a);
    s.extend(pt(80.0, 20.0, 1.0), &mut b);
    assert!(!a.is_empty());
    assert_eq!(
        a, b,
        "re-stamping the same line is identical (deterministic preview)"
    );
}

#[test]
fn line_pivots_on_the_anchor() {
    // Every fill starts at the anchor (press point) — the line pivots on it as the cursor moves.
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Line,
        jitter: 0.0,
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(50.0, 50.0, 1.0), &mut out);
    s.extend(pt(50.0, 90.0, 1.0), &mut out); // drag down
    assert_eq!(out[0].center, [50.0, 50.0], "first dab at the anchor");
    s.extend(pt(90.0, 50.0, 1.0), &mut out); // drag right — still from the anchor
    assert_eq!(out[0].center, [50.0, 50.0], "anchor unchanged across moves");
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
        stroke_method: StrokeMethod::Curve,
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
        stroke_method: StrokeMethod::Curve,
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
fn curve_fill_needs_two_points() {
    let spec = BrushSpec {
        stroke_method: StrokeMethod::Curve,
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = vec![Dab {
        center: [9.0, 9.0],
        radius_px: 1.0,
        coverage: 1.0,
    }];
    s.fill_curve_preview(&[[5.0, 5.0]], &mut out);
    assert!(
        out.is_empty(),
        "a single control point fills nothing (and clears the buffer)"
    );
}
