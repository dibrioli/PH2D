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
fn stabilize_dead_zone_then_lags_toward_anchor() {
    let spec = BrushSpec {
        smooth_stroke: true,
        smooth_radius_px: 50.0,
        smooth_factor: 0.5,
        stroke_method: StrokeMethod::Dots, // one dab per event, easy to inspect
        ..straight_spec(10.0, 0.5)
    };
    let mut s = Stroke::new(spec, no_dynamics(), 1);
    let mut out = Vec::new();
    s.begin(pt(0.0, 0.0, 1.0), &mut out); // anchor at 0
    // Move 40px (< 50 dead-zone) ⇒ no dab.
    s.extend(pt(40.0, 0.0, 1.0), &mut out);
    assert_eq!(out.len(), 0, "inside dead-zone, no dab");
    // Move to 100px (> 50 from anchor) ⇒ a dab lagged halfway (factor 0.5) toward the anchor.
    s.extend(pt(100.0, 0.0, 1.0), &mut out);
    assert_eq!(out.len(), 1);
    // lerp(input=100, anchor=0, u=0.5) = 50.
    assert!(
        (out[0].center[0] - 50.0).abs() < 1e-3,
        "got {}",
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
fn smoother_rounds_a_corner_instead_of_facets() {
    // A 90° corner (P0→P1→P2). Straight chords would keep every dab exactly on the two axes (a
    // sharp vertex). The quadratic smoother rounds the corner → at least one dab lands in the
    // interior of the bounding L, proving the path curves instead of faceting.
    let dabs = collect_stroke(
        straight_spec(4.0, 0.25), // small brush + fine spacing → dabs land in the corner region
        no_dynamics(),
        &[pt(0.0, 0.0, 1.0), pt(40.0, 0.0, 1.0), pt(40.0, 40.0, 1.0)],
    );
    let rounded = dabs.iter().any(|d| {
        d.center[0] > 1.0 && d.center[0] < 39.0 && d.center[1] > 1.0 && d.center[1] < 39.0
    });
    assert!(
        rounded,
        "corner not rounded (still faceted) — centres {:?}",
        dabs.iter().map(|d| d.center).collect::<Vec<_>>()
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
