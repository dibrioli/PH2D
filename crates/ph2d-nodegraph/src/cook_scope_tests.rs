//! Time-scope tests (plan §1.5, M2.N1) — split from `cook_tests.rs` for the
//! HR-18 LOC cap. Declared there as a `#[path]` sibling, so `super` is the
//! test harness (the node ops + `ops()`), and `super::super` is `cook`.

use super::*;

// ───────────────────────── Time scopes (plan §1.5, M2.N1) ─────────────
// The remapper is always `test.delay` (a plain passthrough): scopes are
// keyed by NodeId, so the substrate never needs to know a node TYPE means
// "remap" — the domain layer decides and hands over the map.

use crate::time::{TimeMap, TimeMode};

fn tmap(mode: TimeMode, duration: f64, offset: f64) -> TimeMap {
    TimeMap {
        mode,
        scale: 1.0,
        offset,
        duration,
    }
}

/// `clock -> remapper`. The remap rewrites the clock of everything ABOVE
/// it; the remapper itself (and anything below) stays on the outer clock.
#[test]
fn a_time_scope_rewrites_only_the_upstream_clock() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let remap = g.add_node("test.delay");
    g.connect(Edge {
        from: (clock, 0),
        to: (remap, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    let mut cook = Cook::new();

    // Unscoped: the subtree sees the real playhead.
    let out = cook.cook(&g, &o, remap, 3.0).unwrap();
    assert_eq!(out_scalars(&out[0]), vec![3.0]);

    // Frozen at 5s: the subtree is pinned there no matter the playhead.
    let scopes: TimeScopes = [(remap, tmap(TimeMode::Freeze, 1.0, 5.0))].into();
    let out = cook.cook_scoped(&g, &o, remap, 3.0, &scopes).unwrap();
    assert_eq!(out_scalars(&out[0]), vec![5.0]);
    let out = cook.cook_scoped(&g, &o, remap, 99.0, &scopes).unwrap();
    assert_eq!(out_scalars(&out[0]), vec![5.0], "frozen means frozen");

    // An identity map is dropped: same lane, same behaviour as unscoped.
    let ident: TimeScopes = [(remap, TimeMap::default())].into();
    let out = cook.cook_scoped(&g, &o, remap, 7.0, &ident).unwrap();
    assert_eq!(out_scalars(&out[0]), vec![7.0]);
}

/// **Freeze is free** (plan §1.5): its `t'` is constant, so the upstream
/// subtree's fingerprint never changes and the memo answers every frame —
/// one cook for the life of the still, no matter how long the playhead runs.
/// Counted, not assumed.
#[test]
fn a_freeze_scope_costs_one_cook_no_matter_the_playhead() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let remap = g.add_node("test.delay");
    g.connect(Edge {
        from: (clock, 0),
        to: (remap, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    let mut cook = Cook::new();
    let scopes: TimeScopes = [(remap, tmap(TimeMode::Freeze, 1.0, 5.0))].into();

    for frame in 0..10 {
        let t = f64::from(frame) / 60.0;
        let out = cook.cook_scoped(&g, &o, remap, t, &scopes).unwrap();
        assert_eq!(out_scalars(&out[0]), vec![5.0]);
    }
    assert_eq!(
        o.clock.calls.load(Ordering::Relaxed),
        1,
        "a frozen subtree must cook once, then live off the memo"
    );

    // Contrast: unscoped, every distinct playhead is a real recompute.
    for frame in 0..10 {
        cook.cook(&g, &o, remap, f64::from(frame) / 60.0).unwrap();
    }
    assert_eq!(o.clock.calls.load(Ordering::Relaxed), 11);
}

/// `Loop` cycles the upstream clock: the subtree replays its first
/// `duration` seconds forever, and a lap returns the same VALUE as the
/// first pass.
///
/// It does **not** return it from the cache. The memo is single-slot per
/// `(node, scope)` — it holds the last cook, not a `playhead -> value` map
/// — so a lap that comes back 2 s later finds the slot holding some other
/// phase and recomputes. Making laps free needs a playhead-keyed multi-slot
/// memo with an eviction budget (the "política de cache explícita por nó"
/// ADR-0032 §3 already flags as future work). Freeze, whose `t'` never
/// moves, IS free today (see the test above).
#[test]
fn a_loop_scope_cycles_the_upstream_clock() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let remap = g.add_node("test.delay");
    g.connect(Edge {
        from: (clock, 0),
        to: (remap, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    let mut cook = Cook::new();
    let scopes: TimeScopes = [(remap, tmap(TimeMode::Loop, 2.0, 0.0))].into();

    let at = |cook: &mut Cook, t: f64| {
        out_scalars(&cook.cook_scoped(&g, &o, remap, t, &scopes).unwrap()[0])
    };
    assert_eq!(at(&mut cook, 0.5), vec![0.5]);
    assert_eq!(at(&mut cook, 1.5), vec![1.5]);
    assert_eq!(at(&mut cook, 2.5), vec![0.5], "the second lap repeats");
    assert_eq!(at(&mut cook, 4.5), vec![0.5], "and the third");
    // Re-cooking the SAME phase back-to-back does hit the memo.
    let before = o.clock.calls.load(Ordering::Relaxed);
    assert_eq!(at(&mut cook, 4.5), vec![0.5]);
    assert_eq!(o.clock.calls.load(Ordering::Relaxed), before);
}

/// A diamond where one arm crosses a remapper: the shared upstream is
/// cooked once PER SCOPE and each lane stays memoized. A `NodeId`-only memo
/// (the pre-M2.N1 cache) would alias the two lanes, so every re-cook of the
/// same tick would thrash both arms — and `advance_tick` could snapshot the
/// remapped stream as a `pre` value.
#[test]
fn a_diamond_crossing_a_remap_keeps_each_arm_memoized() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let remap = g.add_node("test.delay");
    let join = g.add_node("test.acc"); // out = in0 + in1, both forward here
    g.connect(Edge {
        from: (clock, 0),
        to: (join, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (clock, 0),
        to: (remap, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (remap, 0),
        to: (join, 1),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    let mut cook = Cook::new();
    let scopes: TimeScopes = [(remap, tmap(TimeMode::Freeze, 1.0, 5.0))].into();

    // Live arm at t=1, frozen arm at 5 → 6. Each arm cooked the clock once.
    let out = cook.cook_scoped(&g, &o, join, 1.0, &scopes).unwrap();
    assert_eq!(out_scalars(&out[0]), vec![6.0]);
    assert_eq!(o.clock.calls.load(Ordering::Relaxed), 2, "one per lane");

    // Re-cook the same tick: fully memoized, both lanes intact.
    let out = cook.cook_scoped(&g, &o, join, 1.0, &scopes).unwrap();
    assert_eq!(out_scalars(&out[0]), vec![6.0]);
    assert_eq!(
        o.clock.calls.load(Ordering::Relaxed),
        2,
        "aliased lanes would recompute both arms on every cook"
    );
}

/// A sequential node (one reading a `pre`) inside a remapped scope is
/// refused, loudly. Its state is a recurrence over the OUTER tick: under
/// `Loop` it would be asked to relive ticks it already integrated. Better a
/// cook error the editor can badge than a plausible wrong trajectory.
#[test]
fn a_sequential_node_inside_a_time_scope_is_refused() {
    let mut g = Graph::new();
    let generator = g.add_node("test.gen");
    let acc = g.add_node("test.acc");
    let remap = g.add_node("test.delay");
    g.connect(Edge {
        from: (generator, 0),
        to: (acc, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (acc, 0),
        to: (acc, 1),
        delayed: true,
    })
    .unwrap();
    g.connect(Edge {
        from: (acc, 0),
        to: (remap, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    let mut cook = Cook::new();

    // Unscoped, the accumulator is perfectly legal.
    assert!(cook.cook(&g, &o, remap, 0.0).is_ok());

    // Under a remap, cooking it is refused — naming the offending node.
    // (`CookValue` is not `PartialEq`, so compare the error, not the whole
    // `Result`.)
    let scopes: TimeScopes = [(remap, tmap(TimeMode::Loop, 2.0, 0.0))].into();
    assert_eq!(
        cook.cook_scoped(&g, &o, remap, 1.0, &scopes).err(),
        Some(CookError::SequentialInTimeScope { node: acc })
    );
}

/// Editing a remap param mints a fresh `ScopeKey` for every value it passes
/// through — a slider drag would otherwise strand one full copy of the
/// subtree per intermediate value, forever. Lanes not visited in a frame
/// die with the tick; the root lane never does.
#[test]
fn stale_scope_lanes_are_pruned_when_the_tick_advances() {
    let mut g = Graph::new();
    let clock = g.add_node("test.clock");
    let remap = g.add_node("test.delay");
    g.connect(Edge {
        from: (clock, 0),
        to: (remap, 0),
        delayed: false,
    })
    .unwrap();
    let o = ops();
    let mut cook = Cook::new();

    // Drag a `Freeze` offset across 20 values, one frame each.
    for i in 0..20 {
        let scopes: TimeScopes = [(remap, tmap(TimeMode::Freeze, 1.0, f64::from(i) * 0.1))].into();
        cook.cook_scoped(&g, &o, remap, 1.0, &scopes).unwrap();
        cook.advance_tick_scoped(&g, &o, 1.0, &scopes).unwrap();
    }
    // Two nodes × (root lane + the one live scope lane) = 4, not 40+.
    assert!(
        cook.cache.len() <= 4,
        "stale scope lanes leaked: {} entries",
        cook.cache.len()
    );
    assert!(
        cook.cache.keys().any(|(_, k)| *k == SCOPE_ROOT),
        "the root lane survives pruning"
    );
}
