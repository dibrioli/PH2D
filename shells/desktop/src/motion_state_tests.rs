//! Headless demo/cook tests for `motion_state` (split for the HR-18 600-LOC shell cap; declared
//! there as a `#[path]` sibling, so `super` is `motion_state`).
//!
//! The default document is the **snow** — a whole particle system (`motion_demo_strobe`). It is
//! cooked here through the REAL registry, tick by tick, exactly as the shell's pump does: a
//! simulation only exists over time, and cooking one frame in isolation would show the empty
//! world it starts from and prove nothing.
//!
//! The rig scenes' guards went with the scenes (Enio: *"deixe só o grafo da chuva"*). Every node
//! they exercised — `rig.rubber_hose`, `rig.fabrik`, `rig.skin_deformer`, `rig.skeleton` — keeps
//! its own tests in its own crate; what died here was the boot document's copy of them, not the
//! coverage.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

#[test]
fn new_builds_the_well_typed_snow_document() {
    let state = MotionState::new();
    assert_eq!(state.sinks.len(), 1, "one scene: the snow");
    assert_eq!(
        state.doc.graph.node(state.sinks[0]).unwrap().type_name,
        "motion.output"
    );
    // 19 nodes: the zone's interior {sim.zone, combine, force.wind, sim.step, sim.collide,
    // sim.lifetime, color_ramp, drive(size), drive(opacity), falloff, cull} + the fade
    // {value.attribute, value.map_range} + birth {grid, move, sim.spawn} + render {scale, move,
    // output}.
    assert_eq!(state.doc.graph.nodes().len(), 19);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
}

/// **The snow: the Simulation Zone, in the boot document, end to end** (docs 48 + 49).
///
/// The whole triangle of a particle system, on the real graph, through the real cook:
///
/// 1. **BIRTH** — the zone's `init` is unwired, so the population starts at NOTHING and is
///    entirely born. If the spawn were dead, the scene would stay empty forever.
/// 2. **LIFE** — a flake ACCELERATES: its velocity lives in the state, so each tick's fall is
///    longer than the last. Measured on one identified flake (`id = 0`), because the population
///    changes underneath any average — the survivorship trap this test walked into once already,
///    when it tracked the LOWEST drop and found it *rising* (the lowest drop is the one the disc
///    is about to kill).
/// 3. **DEATH + STEADY STATE** — birth balances death, so the population climbs and then SETTLES
///    instead of growing without bound. A zone that re-seeded when it emptied, or a cull that was
///    only a filter, would both break the plateau.
#[test]
fn the_snow_is_born_accelerates_and_settles_into_a_steady_state() {
    let state = MotionState::new();
    let snow = *state.sinks.last().expect("the snow is the last sink");
    let mut cook = Cook::new();

    let mut counts: Vec<usize> = Vec::new();
    // The fall of flake `id = 0` — the first one ever born — tick by tick, while it lives.
    let mut flake: Vec<f32> = Vec::new();
    for k in 0..=240u64 {
        let t = k as f64 / 60.0;
        let out = cook
            .cook(&state.doc.graph, &state.registry, snow, t)
            .unwrap();
        let s = out[0].as_stream();
        counts.push(s.count());
        if let (Some(Column::Vec2(p)), Some(Column::Scalar(ids))) = (s.get("P"), s.get("id"))
            && let Some(i) = ids.iter().position(|id| *id == 0.0)
        {
            flake.push(p[i][1]);
        }
        cook.advance_tick(&state.doc.graph, &state.registry, t)
            .unwrap();
    }

    // 1. BIRTH: nothing exists on the first tick (dt = 0), and then the sky fills.
    assert_eq!(counts[0], 0, "the zone starts EMPTY - every flake is born");
    assert!(
        counts[60] > 10,
        "a second in, it is snowing: {}",
        counts[60]
    );

    // 2. LIFE: flake 0 accelerates — each fall is longer than the one before.
    assert!(flake.len() > 30, "flake 0 lived long enough to measure");
    let falls: Vec<f32> = flake.windows(2).map(|w| w[0] - w[1]).collect();
    assert!(
        falls.windows(2).all(|w| w[1] >= w[0] - 1e-4),
        "gravity accumulates in the STATE: {falls:?}"
    );
    assert!(
        falls[falls.len() - 1] > falls[0] * 2.0,
        "…and it really is accelerating, not drifting"
    );

    // 3. AGE: the flakes grow old and are COLOURED by how old they are. Both readings come off
    //    the same live state, so a broken `sim.step` age or a mis-wired ramp shows here.
    let out = cook
        .cook(&state.doc.graph, &state.registry, snow, 4.0)
        .unwrap();
    let s = out[0].as_stream();
    let (ages, lifes) = (
        match s.get("age") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("the flakes have no age"),
        },
        match s.get("life") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => panic!("the flakes do not know their life fraction"),
        },
    );
    assert!(
        ages.iter().any(|a| *a > 1.0) && ages.iter().any(|a| *a < 0.3),
        "old flakes and newborns coexist: {:?}..{:?}",
        ages.iter().cloned().fold(f32::MAX, f32::min),
        ages.iter().cloned().fold(f32::MIN, f32::max)
    );
    assert!(
        lifes.iter().all(|t| (0.0..=1.0).contains(t)),
        "the life fraction stays in [0,1]"
    );
    // The colour is DRIVEN by that fraction: a young flake and an old one are not the same
    // colour. (FALSIFIED by a `value.attribute` that reads a missing column: every t would be 0
    // and the whole snowfall would be one flat colour.)
    let tints = match s.get("tint") {
        Some(Column::Vec4(v)) => v.clone(),
        _ => panic!("the ramp never coloured them"),
    };
    let spread = tints.iter().map(|c| c[2]).fold(f32::MIN, f32::max)
        - tints.iter().map(|c| c[2]).fold(f32::MAX, f32::min);
    assert!(
        spread > 0.05,
        "the flakes are coloured by their AGE, not all alike: blue spread {spread}"
    );

    // …and it does not merely get LIGHTER: an old flake is SMALLER and more TRANSPARENT (doc 51 —
    // Enio's smoke: *"ficou claro mas não menor nem transparente"*). Size and alpha are driven by
    // the same life fraction, so the oldest flake on screen is the smallest AND the faintest.
    let sizes = match s.get("size") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("the shrink never wrote a size"),
    };
    let pick = |cmp: fn(&f32, &f32) -> std::cmp::Ordering| {
        lifes
            .iter()
            .enumerate()
            .max_by(|a, b| cmp(a.1, b.1))
            .map(|(i, _)| i)
            .expect("the snow is not empty")
    };
    let (oldest, youngest) = (pick(|a, b| a.total_cmp(b)), pick(|a, b| b.total_cmp(a)));
    assert!(
        tints[oldest][3] < tints[youngest][3] * 0.5,
        "the old flake is FADED: alpha {} vs {}",
        tints[oldest][3],
        tints[youngest][3]
    );
    assert!(
        sizes[oldest][0] < sizes[youngest][0] * 0.5,
        "…and SMALLER: {:?} vs {:?}",
        sizes[oldest],
        sizes[youngest]
    );
    // **`Set`, not `Multiply`.** `size` and `tint` RIDE THE STATE, so a multiplying drive would
    // compound every tick: after four seconds the flakes would be 1e-30 across — a fade that is a
    // function of FRAME COUNT rather than of age, and invisible in any test that only checks
    // "does the old one look different". `Set` is idempotent, so nothing ever falls below the
    // floor its own life maps to.
    assert!(
        sizes.iter().all(|q| q[0] > 0.01),
        "the drive is Set, not Multiply: nothing compounded away ({:?})",
        sizes.iter().map(|q| q[0]).fold(f32::MAX, f32::min)
    );

    // 4. THE GROUND: the flakes LAND on it. Nothing is below the floor (a collider that only
    //    clamped the position would let them ooze through, because their velocity would still
    //    point down), and some are resting ON it — so the floor is being hit, not merely drawn.
    //    The positions here are the RENDERED ones, so the scene's own `motion.move` is added in.
    let ys: Vec<f32> = match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|q| q[1]).collect(),
        _ => panic!("no positions"),
    };
    let floor = -2.0 + 2.4; // the collider's height, in the rendered frame (`move` dy = 2.4)
    let lowest = ys.iter().cloned().fold(f32::MAX, f32::min);
    assert!(
        lowest > floor - 0.05,
        "nothing fell through the floor: lowest {lowest} vs floor {floor}"
    );
    assert!(
        ys.iter().any(|y| *y < floor + 0.15),
        "…and the snow is settling ON it, not hovering above it: lowest {lowest}"
    );

    // 5. DEATH + STEADY STATE: the population settles instead of growing without bound.
    let late = &counts[180..];
    let (lo, hi) = (
        *late.iter().min().unwrap() as f32,
        *late.iter().max().unwrap() as f32,
    );
    assert!(lo > 0.0, "the snow does not die out: birth keeps up");
    assert!(
        hi < lo * 1.35,
        "birth balances death - the population is a plateau, not a ramp: {lo}..{hi}"
    );
}
