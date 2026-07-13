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

use super::strobe::{RAIN_Y, SEA_DRAFT, SEA_LEVEL, SEA_WAVE_AMP, SNOW_FLOOR_Y};
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
    // 20 nodes: the zone's interior {sim.zone, combine, force.wind, force.buoyancy, sim.step,
    // sim.collide, sim.lifetime, color_ramp, drive(size), drive(opacity), falloff, cull} + the fade
    // {value.attribute, value.map_range} + birth {distribute_poisson, move, sim.spawn} + render
    // {scale, move, output}.
    assert_eq!(state.doc.graph.nodes().len(), 20);
    assert!(state.doc.graph.validate(&state.registry).is_ok());
}

/// **The snow: the Simulation Zone, in the boot document, end to end** (docs 48 + 49).
///
/// The whole triangle of a particle system, on the real graph, through the real cook:
///
/// 1. **BIRTH** — the zone's `init` is unwired, so the population starts at NOTHING and is
///    entirely born. If the spawn were dead, the scene would stay empty forever.
/// 2. **LIFE** — a flake ACCELERATES: **while it is in the air**, its velocity lives in the state,
///    so each tick's fall is longer than the last. Measured on one identified flake (`id = 0`),
///    because the population changes underneath any average — the survivorship trap this test
///    walked into once already, when it tracked the LOWEST drop and found it *rising* (the lowest
///    drop is the one the disc is about to kill).
/// 3. **THE SEA** — and then it stops accelerating, because it hits the water (doc 60). The flake
///    plunges, the sea pushes back, and it comes to rest floating on the surface — not resting on
///    the bed and not passing through. "In the air" is not a time window here but the *waterline*,
///    which is what makes this gate survive a re-tuned demo.
/// 4. **AGE** — they whiten, shrink and fade as they melt, all three driven off the one `life`
///    column (docs 50 + 51).
/// 5. **THE WORLD** — on the whole population: nothing is below the bed, and the flakes that
///    finished falling are FLOATING near the waterline rather than piled on the bottom.
/// 6. **DEATH + STEADY STATE** — birth balances death, so the population climbs and then SETTLES
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

    // 2. LIFE: flake 0 accelerates — each fall is longer than the one before, WHILE IT IS IN THE
    //    AIR. The waterline (plus the swell, which reaches SEA_WAVE_AMP above the still level) is
    //    where the air ends, so this window is read off the world, not off a tick count.
    assert!(flake.len() > 30, "flake 0 lived long enough to measure");
    // **The sink is the scene as DRAWN.** `motion.move` shifts the whole population by `RAIN_Y`
    // on its way to the output, so every `y` read off it is in DISPLAY space while the sea's
    // constants are in SIM space. Read one, compare against the other, and the gate is measuring
    // a sea 2.4 units below the one the flakes fall into
    // ([[feedback_derived_coordinate_seed_must_match_sample]] — it caught me here).
    let sea = SEA_LEVEL + RAIN_Y;
    let bed = SNOW_FLOOR_Y + RAIN_Y;
    let waterline = sea + SEA_WAVE_AMP;
    let airborne: Vec<f32> = flake
        .iter()
        .copied()
        .take_while(|y| *y > waterline)
        .collect();
    assert!(
        airborne.len() > 20,
        "flake 0 spent {} ticks above the water - too few to measure a fall",
        airborne.len()
    );
    let falls: Vec<f32> = airborne.windows(2).map(|w| w[0] - w[1]).collect();
    assert!(
        falls.windows(2).all(|w| w[1] >= w[0] - 1e-4),
        "gravity accumulates in the STATE: {falls:?}"
    );
    assert!(
        falls[falls.len() - 1] > falls[0] * 2.0,
        "…and it really is accelerating, not drifting"
    );

    // 3. THE SEA catches it. The flake plunges past the still-water line (it arrives with a second
    //    and a half of gravity behind it), taps the shallow bed, and the water lifts it back — so
    //    it ENDS afloat: within a draft of the surface, above the bed, and no longer falling.
    //    Without the buoyancy it would simply lie on the bed; without the bed it would keep going.
    let deepest = flake.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        deepest < sea,
        "the flake never even broke the surface ({deepest} vs {sea})"
    );
    assert!(
        (deepest - bed).abs() < 0.02,
        "the splash should TAP the bed and be STOPPED by it (`sim.collide` is what ends the \
         plunge - the sea alone would just slow it): deepest {deepest} vs bed {bed}"
    );
    let settled = flake[flake.len() - 1];
    assert!(
        (settled - sea).abs() < SEA_DRAFT + SEA_WAVE_AMP + 0.05,
        "the flake should be FLOATING at the end of its life, not sunk or launched: {settled} \
         against a sea at {sea}"
    );
    assert!(
        settled > deepest + 0.05,
        "it plunged to {deepest} and the water never brought it back up (ended at {settled})"
    );

    // 4. AGE: the flakes grow old and are COLOURED by how old they are. Both readings come off
    //    the same live state, so a broken `sim.step` age or a mis-wired ramp shows here.
    let out = cook
        .cook(&state.doc.graph, &state.registry, snow, 4.0)
        .unwrap();
    let s = out[0].as_stream();
    let (ages, lives) = (
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
        lives.iter().all(|t| (0.0..=1.0).contains(t)),
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
        lives
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

    // 5. THE WORLD, on the whole population: the SEA holds them and the BED stops them.
    //
    //    This used to read "the flakes land on the floor and rest there" — and it did that by
    //    writing `-2.0 + 2.4` out by hand, a magic number that duplicated the demo's own constant
    //    instead of importing it. When the sea arrived (doc 60) the bed moved, and the literal went
    //    on pointing at a patch of empty water while the gate stayed green about it. It reads the
    //    constants now.
    //
    //    What the scene actually does: nothing is below the bed (a collider that only clamped the
    //    position would let them ooze through, because their velocity would still point down), and
    //    the population is FLOATING — the flakes that are done falling sit around the waterline,
    //    not on the bed. Take `force.buoyancy` out and they all pile up on the bottom, which is
    //    what this second assertion is here to notice.
    let ys: Vec<f32> = match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|q| q[1]).collect(),
        _ => panic!("no positions"),
    };
    let lowest = ys.iter().cloned().fold(f32::MAX, f32::min);
    assert!(
        lowest > bed - 0.05,
        "nothing fell through the bed: lowest {lowest} vs bed {bed}"
    );
    let afloat = ys
        .iter()
        .filter(|y| (**y - sea).abs() < SEA_DRAFT + SEA_WAVE_AMP + 0.05)
        .count();
    assert!(
        afloat >= 3,
        "the sea should be holding the flakes that finished falling, but only {afloat} of {} are \
         anywhere near the waterline at {sea}",
        ys.len()
    );

    // 6. DEATH + STEADY STATE: the population settles instead of growing without bound.
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

/// The graph's sinks, the way the bridge computes them each frame.
fn sinks_of(state: &MotionState) -> Vec<NodeId> {
    let mut ids: Vec<NodeId> = state
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|i| i.type_name == "motion.output")
        .map(|i| i.id)
        .collect();
    ids.sort();
    ids
}

/// Drive the REAL pump for `ticks` fixed steps, exactly as `motion_bridge` does — so what
/// these guards exercise is the runtime the shell runs, not a hand-rolled stand-in.
fn run(state: &mut MotionState, ticks: u64) {
    const FIXED_DT: f64 = 1.0 / 60.0;
    state.sinks = sinks_of(state);
    // The editor's ONE clock (W4.T7). Motion keeps no transport of its own any more, so the
    // helper drives the same `Playhead` the bridge drives and derives the tick with the same
    // function (`motion_bridge::motion_tick`) — a counter rolled by hand here would be the
    // stand-in this helper exists to avoid, and a second derivation of the tick is the trap
    // this module has already fallen into.
    //
    // It is born PAUSED and `advance` is a no-op while it is (the bridge auto-plays when the
    // tool opens). Without the `play` the tick never leaves 0, the sim never runs, and a guard
    // about "state the load must forget" would be measuring an empty cook.
    let mut playhead = ph2d_core::Playhead::new(FIXED_DT);
    playhead.play();
    let scopes = ph2d_node_motion_time_remap::time_scopes(&state.doc.graph, &state.registry);
    // The shell cooks the CURRENT tick before it ever advances (the catch-up cook of a
    // paused frame), so tick 0 — the sim's seed — is always the first one cooked. Advancing
    // first instead drops the pump straight into its SCRUB path with an empty checkpoint
    // ring, and nothing comes out. Mirroring the bridge is the point of this helper.
    for step in 0..=ticks {
        if step > 0 {
            playhead.advance();
        }
        let tick = crate::render_loop::motion_bridge::motion_tick(&playhead, FIXED_DT);
        state.pump.advance_or_scrub_scoped(
            &state.doc.graph,
            &state.registry,
            &state.sinks,
            tick,
            |t| t as f64 * FIXED_DT,
            state.default_uv_rect,
            state.default_size,
            &scopes,
        );
    }
}

/// **A loaded document must cook exactly like a freshly booted one** — byte for byte.
///
/// This is the whole of `install()` in one assertion, and it is sharp because the Motion
/// runtime is keyed by NODE ID. Node ids are small integers that the next document reuses
/// for entirely different nodes, so anything the previous document left behind does not
/// merely linger — it gets ADOPTED by whatever node inherits its number.
///
/// The `Cook` is the worst of them, because it is not a cache: it holds the simulation's
/// **living state**. Here the snow is run for two seconds first, so the cook is full of
/// falling flakes with velocities and ages; then the very same document is loaded back.
/// If `install` merely marked the pump dirty (or reset the graph but not the transport),
/// the "loaded" scene would resume mid-snowfall at t=2s while a booted one starts from an
/// empty sky — and the two would disagree on the first frame.
///
/// FALSIFIED by dropping any single line of `install`: keep the pump and the snow keeps
/// falling; keep the transport and the playhead starts in the future.
#[test]
fn a_loaded_document_cooks_exactly_like_a_freshly_booted_one() {
    let mut used = MotionState::new();
    run(&mut used, 120); // two seconds of snow: the cook is now FULL of live flakes
    assert!(
        !used.pump.instances.is_empty(),
        "the sim really did accumulate state to forget"
    );

    // Save it, and load it straight back into the state that has been running.
    let text = used.doc.to_text();
    used.load_text(&text).expect("its own text round-trips");

    // A fresh boot of the same document — the reference for what a load must look like.
    let mut booted = MotionState::new();

    for state in [&mut used, &mut booted] {
        run(state, 30);
    }
    let bytes = |s: &MotionState| bytemuck::cast_slice::<_, u8>(&s.pump.instances).to_vec();
    assert_eq!(
        used.pump.instances.len(),
        booted.pump.instances.len(),
        "the loaded document is mid-snowfall — the previous run's sim survived the load"
    );
    assert_eq!(
        bytes(&used),
        bytes(&booted),
        "a loaded document must be indistinguishable from a booted one"
    );
}

/// The load forgets what the previous document was pointing AT. A probe, a flow digest and
/// (sharpest) the panel's SELECTION all name nodes by raw id — carry any of them across a
/// load and they silently re-target whichever node inherited the number, so the params
/// panel would edit a node the artist never picked.
#[test]
fn loading_forgets_every_id_that_named_the_previous_document() {
    let mut state = MotionState::new();
    let victim = state.doc.graph.nodes()[0].id;
    state.probe = Some(victim);
    state.probe_ring.push(1.0);
    state.flow_digest.insert(victim.0, 7);
    state.history.begin(&state.doc);

    let text = state.doc.to_text();
    state.load_text(&text).expect("round-trips");

    assert_eq!(state.probe, None, "the probe named a node in the old graph");
    assert!(state.probe_ring.is_empty());
    assert!(state.flow_digest.is_empty(), "the digests were keyed by id");
    assert!(
        !state.history.can_undo(),
        "undo belongs to the document that was edited, not to the file that replaced it"
    );
    // "A loaded document starts at tick 0" is NOT asserted here, and that is a MOVE, not a
    // deletion: after W4.T7 the clock is the editor's ONE `Playhead`, which `MotionState` does
    // not own and cannot rewind. The rewind lives in `App::project_load` (`project.rs`), and it
    // is gated there — `project::tests::a_loaded_project_rewinds_the_clock_and_starts_a_fresh_history`
    // drives the real Ctrl+O on a real (windowless) `App`. What the assertion was PROTECTING
    // against is the guard right below.
}

/// **Why the load rewinds the clock** — the Motion half of a gate whose other half is
/// `project::tests::a_loaded_project_rewinds_the_clock_and_starts_a_fresh_history`.
///
/// The rewind reads like bookkeeping ("a new document starts at zero"), which is exactly how a
/// line like that gets deleted by someone tidying up. It is not bookkeeping. `install` hands the
/// pump a **brand-new `MotionCookPump`** — no ticks cooked, an empty checkpoint ring — and the
/// pump decides between *advance* and *scrub* by comparing the tick it is asked for with the last
/// one it cooked. Ask a fresh pump for tick 120 and it does not refuse and it does not start over:
/// it **SCRUBS** there. So a clock left where the previous document had wandered to does not
/// merely display the wrong number — it opens the loaded scene TWO SECONDS IN, with the snow
/// already falling, where a boot opens on an empty sky.
///
/// That is the failure the removed `MotionTransport` assertion used to catch, restated at the
/// level where it is still true.
#[test]
fn a_clock_that_was_not_rewound_opens_the_document_mid_scene() {
    const FIXED_DT: f64 = 1.0 / 60.0;

    // The editor's clock after two seconds of play — the session Ctrl+O walks into.
    let mut playhead = ph2d_core::Playhead::new(FIXED_DT);
    playhead.seek(2.0);
    let inherited = crate::render_loop::motion_bridge::motion_tick(&playhead, FIXED_DT);
    assert_eq!(inherited, 120, "two seconds of fixed ticks");

    // The freshly installed document (what a load produces: a new pump, not one tick cooked)
    // driven at that INHERITED tick.
    let mut loaded = MotionState::new();
    loaded.sinks = sinks_of(&loaded);
    let scopes = ph2d_node_motion_time_remap::time_scopes(&loaded.doc.graph, &loaded.registry);
    loaded.pump.advance_or_scrub_scoped(
        &loaded.doc.graph,
        &loaded.registry,
        &loaded.sinks,
        inherited,
        |t| t as f64 * FIXED_DT,
        loaded.default_uv_rect,
        loaded.default_size,
        &scopes,
    );

    // …against the same document on a REWOUND clock: tick 0, the sky still empty (the premise
    // of `the_snow_is_born_...`: `counts[0] == 0` — every flake is yet to be born).
    let mut booted = MotionState::new();
    run(&mut booted, 0);

    assert!(
        booted.pump.instances.is_empty(),
        "a rewound clock opens on an EMPTY sky: {} instances",
        booted.pump.instances.len()
    );
    assert!(
        !loaded.pump.instances.is_empty(),
        "an un-rewound clock opens the document with the snow already in the air - the load \
         would hand the artist the scene MID-SCENE instead of at its start"
    );
}

/// A document survives the trip through the project file: the graph the artist built comes
/// back with the same nodes, wires, params and LAYOUT (the text format carries `[layout]`,
/// so a loaded graph is not a pile of cards at the origin).
#[test]
fn a_saved_document_comes_back_whole() {
    let mut state = MotionState::new();
    // An edit that is the artist's, not the boot document's: a moved card + a dialled param.
    let node = state.doc.graph.nodes()[0].id;
    state
        .doc
        .graph
        .set_pos(node, ph2d_nodegraph::graph::Pos { x: 123.0, y: -45.0 });
    let before = state.doc.to_text();

    let mut loaded = MotionState::new();
    loaded.load_text(&before).expect("round-trips");

    assert_eq!(
        loaded.doc.to_text(),
        before,
        "the canonical text is a fixed point of save -> load"
    );
    assert_eq!(
        loaded.doc.graph.layout().get(&node),
        Some(&ph2d_nodegraph::graph::Pos { x: 123.0, y: -45.0 }),
        "the cards come back where the artist left them"
    );
    assert!(
        loaded.doc.graph.validate(&loaded.registry).is_ok(),
        "and the loaded graph is still well-typed"
    );
}
