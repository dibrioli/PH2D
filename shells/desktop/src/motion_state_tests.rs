//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC
//! shell cap; declared there as a `#[path]` sibling, so `super` is
//! `motion_state`). Cook the default document — now the two ghost-copy FX scenes (a
//! shadowed grid and an aberrated ring) — through the REAL registry, tick by tick, so
//! the whole chain is exercised and not just the node in isolation.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// The shadow scene's grid: 4×4, so the FX must emit 32 rows (a shadow + an element).
const GRID: usize = 16;
/// The split scene's ring: 5×5 — an ODD square, so element 12 sits exactly on the
/// centroid, which is where the radial aberration must be zero.
const RING: usize = 25;
const RING_CENTRE: usize = 12;
/// The aberration coefficient the demo dials in (`fx.rgb_split`'s `strength`).
const STRENGTH: f32 = 0.14;
/// The demo's shadow colour: black at 45 %.
const SHADOW_A: f32 = 0.45;
/// The demo's element colour (a cyan-ish body, so the fringes can only carry what it has).
const BODY: [f32; 4] = [0.15, 0.75, 0.95, 1.0];

/// One cooked tick of a scene: what is drawn, in draw order.
struct Frame {
    pos: Vec<[f32; 2]>,
    tint: Vec<[f32; 4]>,
}

/// Drive `ticks` fixed steps of the real cook (advancing the `pre` edges, exactly as the
/// shell's pump does) and return every tick's rows for `sink`. Both demo scenes animate,
/// so cooking one frame in isolation would prove nothing about whether the FX TRACKS its
/// source.
fn run_scene(state: &MotionState, sink: NodeId, ticks: usize) -> Vec<Frame> {
    let mut cook = Cook::new();
    let mut frames = Vec::with_capacity(ticks);
    for k in 0..ticks {
        let ph = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, sink, ph)
            .unwrap();
        let stream = out[0].as_stream();
        frames.push(Frame {
            pos: match stream.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => Vec::new(),
            },
            tint: match stream.get("tint") {
                Some(Column::Vec4(v)) => v.clone(),
                _ => Vec::new(),
            },
        });
        cook.advance_tick(&state.doc.graph, &state.registry, ph)
            .unwrap();
    }
    frames
}

fn mean_x(pos: &[[f32; 2]]) -> f32 {
    pos.iter().map(|p| p[0]).sum::<f32>() / pos.len() as f32
}

fn centroid(pos: &[[f32; 2]]) -> [f32; 2] {
    let s = pos
        .iter()
        .fold([0.0f32; 2], |a, p| [a[0] + p[0], a[1] + p[1]]);
    [s[0] / pos.len() as f32, s[1] / pos.len() as f32]
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    (dx * dx + dy * dy).sqrt()
}

#[test]
fn new_builds_the_well_typed_fx_document() {
    let state = MotionState::new();
    assert_eq!(state.sinks.len(), 2, "two scenes -> two sinks");
    for sink in &state.sinks {
        assert_eq!(
            state.doc.graph.node(*sink).unwrap().type_name,
            "motion.output"
        );
    }
    // 11 nodes: {grid, oscillator, drop_shadow, move, output}
    // + {grid, orbit, tint, rgb_split, move, output}. The newest nodes (doc 38) —
    // `fx.drop_shadow` and `fx.rgb_split`.
    assert_eq!(state.doc.graph.nodes().len(), 11);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
}

/// **The drop shadow, through the whole chain** (doc 38): a 4×4 grid reaches
/// `motion.output` as **32** rows — every element's shadow, as one block BEHIND, then
/// the elements themselves.
///
/// FALSIFIED four ways: the seam never wired (16 rows arrive, the classic "it compiles
/// and cooks the input") · the shadows drawn on top (the block order flipped) · the
/// shadow **baked at tick 0** instead of tracking the bob (the offset would stop being
/// constant once the elements move) · the shadow thrown the wrong way (315° in a y-up
/// world falls down-AND-right, so `dx > 0` and `dy < 0`).
#[test]
fn every_element_casts_a_shadow_that_tracks_it() {
    let state = MotionState::new();
    let sink = state.sinks[0]; // the shadow scene (added first)
    let frames = run_scene(&state, sink, 60);
    assert_eq!(frames[0].pos.len(), 2 * GRID, "a shadow + an element, each");

    // The shadow block leads, and every shadow carries the SAME offset — which is what
    // makes it one layout's shadow rather than 16 unrelated ghosts.
    let off = |f: &Frame, i: usize| {
        [
            f.pos[i][0] - f.pos[GRID + i][0],
            f.pos[i][1] - f.pos[GRID + i][1],
        ]
    };
    let seed = off(&frames[0], 0);
    assert!(
        seed[0] > 0.0 && seed[1] < 0.0,
        "315° falls down-right: {seed:?}"
    );
    // 0.3 at 315° → 0.3/√2 on each axis.
    assert!(
        (dist(seed, [0.0, 0.0]) - 0.3).abs() < 0.01,
        "distance = 0.3"
    );

    // The bob is real (otherwise "the shadow tracks it" is vacuous)…
    let bobbed = (1..60).any(|k| (frames[k].pos[GRID][1] - frames[0].pos[GRID][1]).abs() > 0.1);
    assert!(bobbed, "the elements bob");
    // …and the shadow rides it every tick, for every element.
    for f in &frames {
        for i in 0..GRID {
            let o = off(f, i);
            assert!(
                (o[0] - seed[0]).abs() < 1e-4 && (o[1] - seed[1]).abs() < 1e-4,
                "shadow {i} drifted off its caster: {o:?} vs {seed:?}"
            );
        }
    }

    // A shadow is a COLOUR carrying the element's alpha — not a copy of it.
    for i in 0..GRID {
        assert_eq!(frames[0].tint[i][0..3], [0.0, 0.0, 0.0], "black shadow");
        assert!((frames[0].tint[i][3] - SHADOW_A).abs() < 1e-6);
        assert_eq!(frames[0].tint[GRID + i][3], 1.0, "the element is opaque");
    }
    assert!(mean_x(&frames[0].pos) < -3.0, "the grid sits on the left");
}

/// **The RGB split, through the whole chain** (doc 38): a 5×5 ring reaches
/// `motion.output` as **75** rows — the R ghost, the G+B ghost, then the elements.
///
/// In **Aberration** mode the fringe is ZERO at the layout's optical axis (its centroid)
/// and grows linearly outward: `|ghost − element| = strength × |element − centroid|`.
///
/// FALSIFIED four ways: the seam never wired (25 rows) · the ghosts drawn over the
/// element (block order) · **the uniform Split** wired instead of the radial one (the
/// centre element would be displaced too) · the naive "paint one ghost red" (the R ghost
/// of this cyan-ish body must be nearly BLACK — it has almost no red to throw).
#[test]
fn the_aberration_is_clean_at_the_axis_and_smears_at_the_rim() {
    let state = MotionState::new();
    let sink = state.sinks[1]; // the split scene (added second)
    let frames = run_scene(&state, sink, 30);
    let f = &frames[29]; // mid-orbit, so nothing about this depends on the seed pose
    assert_eq!(f.pos.len(), 3 * RING, "two ghosts + the element, each");

    // The elements themselves come LAST and keep the colour the artist authored.
    for (i, body) in f.tint[2 * RING..].iter().enumerate() {
        for (got, want) in body.iter().zip(&BODY) {
            assert!((got - want).abs() < 1e-6, "body {i}: {got} vs {want}");
        }
    }
    // The ghosts are the element's own channels, split apart — NOT pure red / pure cyan.
    assert!(
        f.tint[0][0] > 0.0 && f.tint[0][0] < 0.2,
        "a faint R ghost, not 1.0"
    );
    assert_eq!(f.tint[0][1..3], [0.0, 0.0], "the R ghost holds no G or B");
    assert!(
        (f.tint[RING][1] - BODY[1]).abs() < 1e-6,
        "the G+B ghost keeps G"
    );
    assert_eq!(f.tint[RING][0], 0.0, "…and none of the red");

    // The fringe grows with the distance from the optical axis.
    let elems: Vec<[f32; 2]> = f.pos[2 * RING..].to_vec();
    let axis = centroid(&elems);
    let fringe = |i: usize| dist(f.pos[i], f.pos[2 * RING + i]);
    assert!(
        fringe(RING_CENTRE) < 0.01,
        "the element ON the axis is clean ({})",
        fringe(RING_CENTRE)
    );
    for (i, e) in elems.iter().enumerate() {
        let expected = STRENGTH * dist(*e, axis);
        assert!(
            (fringe(i) - expected).abs() < 0.02,
            "element {i}: fringe {} vs strength × radius {expected}",
            fringe(i)
        );
    }
    // A corner really does smear (otherwise "grows outward" is satisfied by all-zero).
    let rim = (0..RING).map(fringe).fold(0.0, f32::max);
    assert!(rim > 0.2, "the rim smears (peak fringe {rim})");
    assert!(mean_x(&f.pos) > 3.0, "the ring sits on the right");
}

/// The default document replays bit-identically: the whole boot doc is HR-5 arithmetic
/// (no expression / libm on this path), which is what makes a scrub reproducible.
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
/// document no longer carries a formula (the demo moved on), so the guard authors its
/// own: a text param bumps the doc to `v2` and comes back byte-for-byte. FALSIFIED by the
/// old data-loss bug (the formula dropped on save).
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
