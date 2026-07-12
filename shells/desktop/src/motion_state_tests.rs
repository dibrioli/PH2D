//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now two simulation scenes (a pinned
//! curtain and a slit-scanned bob) — through the REAL registry, driving the `pre` loops
//! tick by tick so the whole chain is exercised, not just the node in isolation.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// The pin scene's curtain: 8×8, one row (8 elements) nailed. `motion.grid` is row-major
/// from the LOWEST y up, so the curtain's top row — the one it hangs from — is the LAST
/// 8 elements, and the free ones are the 56 below it.
const CURTAIN: usize = 64;
const CURTAIN_COLS: usize = 8;
const PINNED: std::ops::Range<usize> = (CURTAIN - CURTAIN_COLS)..CURTAIN;
const FREE: std::ops::Range<usize> = 0..(CURTAIN - CURTAIN_COLS);
/// The slit-scan scene's strip: 6×12.
const STRIP: usize = 72;

/// Drive `ticks` fixed steps of the real cook (advancing the `pre` edges, exactly as the
/// shell's pump does) and return every tick's positions for `sink`. The simulation scenes
/// only exist over time — cooking one frame in isolation would show the seed pose and
/// prove nothing.
fn run_scene(state: &MotionState, sink: NodeId, ticks: usize) -> Vec<Vec<[f32; 2]>> {
    let mut cook = Cook::new();
    let mut frames = Vec::with_capacity(ticks);
    for k in 0..ticks {
        let ph = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, sink, ph)
            .unwrap();
        frames.push(match out[0].as_stream().get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        });
        cook.advance_tick(&state.doc.graph, &state.registry, ph)
            .unwrap();
    }
    frames
}

fn mean_x(pos: &[[f32; 2]]) -> f32 {
    pos.iter().map(|p| p[0]).sum::<f32>() / pos.len() as f32
}

/// How far element `i` travelled from the first frame to the last.
fn travel(frames: &[Vec<[f32; 2]>], i: usize) -> f32 {
    let (a, b) = (frames[0][i], frames[frames.len() - 1][i]);
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    (dx * dx + dy * dy).sqrt()
}

/// The mean distance of `idx` to the point `c` — the attractor's target, in world space.
fn mean_dist_to(pos: &[[f32; 2]], idx: std::ops::Range<usize>, c: [f32; 2]) -> f32 {
    let n = idx.len() as f32;
    idx.map(|i| {
        let (dx, dy) = (pos[i][0] - c[0], pos[i][1] - c[1]);
        (dx * dx + dy * dy).sqrt()
    })
    .sum::<f32>()
        / n
}

#[test]
fn new_builds_the_well_typed_simulation_document() {
    let state = MotionState::new();
    assert_eq!(state.sinks.len(), 2, "two scenes -> two sinks");
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 13 nodes: {grid, pin_constraint, integrate, attractor, drag, collide, move, output}
    // + {grid, oscillator, slit_scan, move, output}. The newest nodes (doc 34) —
    // `motion.pin_constraint` and `motion.slit_scan`.
    assert_eq!(state.doc.graph.nodes().len(), 13);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
}

/// **The pin, through the whole chain** (doc 34): `motion.pin_constraint` writes
/// `inv_mass = 0` on the curtain's top row, and BOTH downstream consumers must honour it
/// — `motion.integrate` (no force, no displacement) and `motion.collide` (the falling
/// elements pack around it and may not shove it). So the pinned row is bit-for-bit
/// motionless while its 56 free neighbours are dragged into the attractor.
///
/// FALSIFIED four ways: an ignored `inv_mass` in the integrator (the pinned row falls in
/// with the rest) · an ignored `inv_mass` in the collision (the row is nudged off its
/// anchor when the pile lands on it — the "fixed obstacle drifts" bug) · a pin that also
/// froze the free elements (nothing moves at all) · **the wrong row pinned** (the curtain
/// must hang from its TOP row, which in the grid's bottom-up row-major order is the LAST
/// one — the demo pinned the bottom row until the smoke caught it).
#[test]
fn the_pinned_row_holds_while_the_rest_falls_into_the_attractor() {
    let state = MotionState::new();
    let sink = state.sinks[0]; // the pin scene (added first)
    let frames = run_scene(&state, sink, 120);
    assert_eq!(frames[0].len(), CURTAIN, "the 8×8 curtain");

    for i in PINNED {
        assert_eq!(
            frames[119][i], frames[0][i],
            "pinned element {i} never moved (neither force nor contact could move it)"
        );
    }
    // The pinned row is the curtain's TOP: every pinned element sits above every free one.
    // Asserted on the GEOMETRY, not the index, so pinning the wrong row goes red.
    let lowest_pinned = PINNED.map(|i| frames[0][i][1]).fold(f32::MAX, f32::min);
    let highest_free = FREE.map(|i| frames[0][i][1]).fold(f32::MIN, f32::max);
    assert!(
        lowest_pinned > highest_free,
        "the curtain hangs from its TOP row (pinned y {lowest_pinned} vs free y {highest_free})"
    );

    let free_travel: f32 = FREE.map(|i| travel(&frames, i)).sum::<f32>() / FREE.len() as f32;
    assert!(
        free_travel > 0.3,
        "the free elements were dragged in (mean travel {free_travel})"
    );

    // And they were dragged TOWARD the attractor: the scene is shifted by move(−7), so
    // its target sits at (−7, 0) in world space.
    let centre = [-7.0, 0.0];
    let before = mean_dist_to(&frames[0], FREE, centre);
    let after = mean_dist_to(&frames[119], FREE, centre);
    assert!(
        after < before * 0.8,
        "the free elements closed on the attractor ({before} -> {after})"
    );
    assert!(mean_x(&frames[0]) < -3.0, "the curtain sits on the left");
}

/// **The slit scan, through the whole chain** (doc 34): the oscillator's `phase_stagger`
/// is 0, so every element of the strip bobs at the SAME phase — a rigid, lockstep bob.
/// The travelling wave on screen can therefore only come from `motion.slit_scan` sampling
/// each element at its own delay.
///
/// The oracle is the *displacement from the seed pose* (`y(k) − y(0)`), which cancels the
/// grid's own row geometry: in lockstep every element shares one displacement, so the
/// spread across the strip would be EXACTLY zero. FALSIFIED if the scan forwarded the live
/// pose (spread 0 — the "it compiles and shows the input" failure) or delayed everything
/// equally (spread 0 again).
#[test]
fn the_lockstep_bob_becomes_a_travelling_wave() {
    let state = MotionState::new();
    let sink = state.sinks[1]; // the slit-scan scene (added second)
    let frames = run_scene(&state, sink, 90);
    assert_eq!(frames[0].len(), STRIP, "the 6×12 strip");

    // Tick 0: the delay line seeds flat, so the scan opens exactly on the input pose.
    let seed_spread = spread(&y_offsets(&frames, 0));
    assert!(
        seed_spread < 1e-6,
        "the scan opens on the live pose (spread {seed_spread})"
    );

    // Later: each element sees a different phase of the same bob.
    let sheared = (1..90)
        .map(|k| spread(&y_offsets(&frames, k)))
        .fold(0.0, f32::max);
    assert!(
        sheared > 0.5,
        "the lockstep bob was sheared into a wave (peak spread {sheared})"
    );

    // The head of the ramp has no delay: element 0 is always live, so it leads.
    let head_moved = (1..90).any(|k| (frames[k][0][1] - frames[0][0][1]).abs() > 0.1);
    assert!(head_moved, "the undelayed head still bobs");
    assert!(mean_x(&frames[0]) > 3.0, "the strip sits on the right");
}

/// Each element's y displacement from its seed pose at tick `k` (cancels the grid's rows).
fn y_offsets(frames: &[Vec<[f32; 2]>], k: usize) -> Vec<f32> {
    (0..frames[k].len())
        .map(|i| frames[k][i][1] - frames[0][i][1])
        .collect()
}

fn spread(v: &[f32]) -> f32 {
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for x in v {
        lo = lo.min(*x);
        hi = hi.max(*x);
    }
    hi - lo
}

/// The default document replays bit-identically: the whole boot doc is now HR-5
/// arithmetic (no expression / libm on this path), and both scenes are stateful — the
/// simulation loops must return to the same trajectory from a fresh pump, which is what
/// makes a scrub reproducible.
#[test]
fn the_default_document_replays_deterministically() {
    use ph2d_eval_motion::MotionCookPump;
    let run = || {
        let state = MotionState::new();
        let mut pump = MotionCookPump::new();
        let mut frames = Vec::new();
        for k in 0..12u64 {
            pump.pump(
                &state.doc.graph,
                &state.registry,
                &state.sinks,
                k,
                k as f64 / 60.0,
                state.default_uv_rect,
                state.default_size,
            );
            frames.push(
                pump.instances
                    .iter()
                    .map(|i| (i.world_pos, i.tint))
                    .collect::<Vec<_>>(),
            );
        }
        frames
    };
    assert_eq!(run(), run(), "two runs of the same document match exactly");
}

/// The text-param channel (doc 32) still round-trips through a save/load. The boot
/// document no longer carries a formula (the demo moved on to the simulation nodes), so
/// the guard authors its own: a text param bumps the doc to `v2` and comes back
/// byte-for-byte. FALSIFIED by the old data-loss bug (the formula dropped on save).
#[test]
fn a_text_param_survives_a_text_round_trip() {
    let mut doc = ph2d_motion_doc::MotionDoc::default();
    let expr = doc.graph.add_node("motion.expression");
    doc.graph
        .set_text_param(expr, "expr", "cos(f * a + t) * f * 4");

    let text = doc.to_text();
    assert!(text.starts_with("v2"), "a text param bumps the doc to v2");
    assert!(
        text.contains("cos(f * a + t) * f * 4"),
        "the formula is serialized (interior spaces intact)"
    );
    let back = ph2d_motion_doc::MotionDoc::from_text(&text).expect("the v2 doc reloads");
    assert_eq!(
        back.graph.node_text_params(),
        doc.graph.node_text_params(),
        "every formula survives the round trip"
    );
}
