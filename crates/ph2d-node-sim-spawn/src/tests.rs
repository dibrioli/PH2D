//! Guards for `sim.spawn` (O4 birth, doc 49). `super` is the crate root.

use super::*;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("registry builds");
    reg
}

const FPS: f64 = 60.0;

/// The pre-`pulse` shape of [`newborns`]: rate-born elements, each drawn to its row by [`slot`].
///
/// It exists so the gates that predate the port keep asking **exactly what they always asked** —
/// a fixture that reached the same state through the new argument would quietly stop testing the
/// old law the day the new one changed.
fn rate_newborns(template: &Stream, ids: &[u32], scatter: bool, seed: u32) -> Stream {
    let n = template.count();
    let rows: Vec<usize> = ids.iter().map(|id| slot(*id, n, scatter, seed)).collect();
    newborns(template, ids, &rows)
}

/// A pulse stream over `n` rows, firing at the given rows.
fn fires(n: usize, rows: &[usize]) -> Stream {
    let mut v = vec![0.0f32; n];
    for &r in rows {
        v[r] = 1.0;
    }
    Stream::new(n).with(PULSE_COL, Column::Scalar(v))
}

/// **The ordinal WRAPS at `ID_WRAP` (ADR-0136, the audit's C3).** The id is
/// stored as `f32`, whose integers collapse past 2²⁴ — un-wrapped, the CPU
/// would stamp ids the representation cannot hold while the GPU (whose window
/// arrives wrapped by contract) stamps wrapped ones: two answers to "who is
/// this newborn?". The wrap must happen at the ONE observable point, so the
/// slot draw and the stamped id agree with the device bit for bit.
#[test]
fn the_birth_ordinal_wraps_at_the_representable_ceiling() {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::gpu::ID_WRAP;
    // A playhead deep enough that the raw ordinal passes 2²⁴ (rate 4M/s ⇒ ~4.2 s
    // — the audit's own arithmetic: at millions per second the wrap is SECONDS).
    let rate = 4_000_000.0_f64;
    let t = 4.3_f64;
    let raw = born_in(rate, t, 1.0 / FPS);
    assert!(
        raw.start > ID_WRAP,
        "fixture: the raw ordinal must be past 2^24"
    );

    // The node, on the real cook path: template → spawn at that playhead.
    let reg = registry();
    let mut g = Graph::new();
    let tpl = g.add_node("motion.grid");
    g.set_param(tpl, "rows", 2.0);
    g.set_param(tpl, "cols", 2.0);
    let sp = g.add_node("sim.spawn");
    g.set_param(sp, "rate", rate as f32);
    g.connect(Edge {
        from: (tpl, 0),
        to: (sp, 0),
        delayed: false,
    })
    .unwrap();
    assert!(g.validate(&reg).is_ok());
    let mut cook = Cook::new();
    // Two cooks: the first has no history (dt = 0, nothing born).
    cook.cook(&g, &reg, sp, t - 1.0 / FPS).expect("warm-up");
    cook.advance_tick(&g, &reg, t - 1.0 / FPS).expect("tick");
    let out = cook.cook(&g, &reg, sp, t).expect("cooks");
    let stream = out[0].as_stream();
    assert!(stream.count() > 0, "fixture: births must happen");
    let Some(Column::Scalar(ids)) = stream.get("id") else {
        panic!("newborns carry an id column");
    };
    for id in ids {
        assert!(
            *id >= 0.0 && (*id as u32) < ID_WRAP,
            "id {id} escaped the wrap — past 2^24 the f32 lattice collapses ids"
        );
        assert_eq!(*id, (*id as u32) as f32, "every id is an exact f32 integer");
    }
}

/// Every id born over `secs` seconds of 60 fps ticks.
fn run(rate: f64, secs: f64) -> Vec<u32> {
    let ticks = (secs * FPS) as u64;
    let mut ids = Vec::new();
    for k in 1..=ticks {
        let t = k as f64 / FPS;
        ids.extend(born_in(rate, t, 1.0 / FPS));
    }
    ids
}

/// **A fractional rate is exact over time.** 7 births a second at 60 fps is 0.116 per tick —
/// and the obvious implementation (`round(rate · dt)` per tick) rounds that to ZERO on every
/// tick and emits **nothing, ever**. Births come from the difference of two floors, so the
/// leftovers accumulate instead of evaporating.
///
/// FALSIFIED by per-tick rounding (0 births) and by per-tick truncation (also 0).
#[test]
fn a_fractional_rate_is_exact_over_time_and_does_not_round_itself_away() {
    let ids = run(7.0, 1.0);
    assert_eq!(ids.len(), 7, "7 per second means 7, at 60 fps: {ids:?}");
    let ten = run(7.0, 10.0);
    assert_eq!(ten.len(), 70, "…and it stays exact over ten seconds");
}

/// **Identity is the birth ordinal**: ids are unique, monotone, and never reused — so a kill
/// node downstream can tell one tick's survivors from the next tick's newborns, and two
/// particles can never claim to be the same particle.
#[test]
fn ids_are_unique_and_monotone() {
    let ids = run(30.0, 3.0);
    assert_eq!(ids.len(), 90);
    assert!(ids.windows(2).all(|w| w[1] > w[0]), "monotone: {ids:?}");
    assert_eq!(ids[0], 0, "the first element ever born is number 0");
}

/// **A paused clock breeds nothing.** `dt = 0` is the first tick of a cook and every paused
/// frame; a sim that spawned on it would multiply while you stared at it.
#[test]
fn a_paused_clock_spawns_nothing() {
    assert!(born_in(50.0, 4.0, 0.0).is_empty(), "paused: no births");
    assert!(born_in(50.0, 0.0, 0.0).is_empty(), "the first tick: none");
    assert!(born_in(0.0, 4.0, 1.0 / FPS).is_empty(), "rate 0: none");
}

/// **A scrub reproduces the same particles.** The ids are a function of the CLOCK, not of a
/// counter the node keeps — so cooking t = 2 s after a rewind gives exactly what the first pass
/// gave.
///
/// FALSIFIED by an id counter carried in a state column: it would count the cook's HISTORY, so a
/// scrub would renumber the world and every element downstream would look brand new.
#[test]
fn a_rewind_reproduces_the_same_births() {
    let first = run(20.0, 2.0);
    let again = run(20.0, 2.0);
    assert_eq!(first, again);
    // …and a single tick in the middle is reproducible on its own, without any history at all.
    let mid: Vec<u32> = born_in(20.0, 1.0, 1.0 / FPS).collect();
    assert_eq!(mid, vec![19], "the 20th birth lands in that tick, always");
}

/// A newborn **inherits its template row, column for column** — position, velocity, size, tint,
/// whatever was authored upstream — and is stamped with its own id.
///
/// It is stamped with NO clock (`sim_t`): `sim.step` must see an element it has never stepped
/// and start it, not hurl it forward by the entire age of the simulation.
#[test]
fn a_newborn_inherits_its_template_row_and_carries_no_clock() {
    let template = Stream::new(2)
        .with("P", Column::Vec2(vec![[1.0, 2.0], [3.0, 4.0]]))
        .with("vel", Column::Vec2(vec![[0.0, -1.0], [0.0, -2.0]]))
        .with("id", Column::Scalar(vec![99.0, 98.0])); // the template's own ids are NOT inherited

    let out = rate_newborns(&template, &[5], false, 1); // round-robin: 5 % 2 = row 1
    assert_eq!(out.count(), 1);
    assert!(matches!(out.get("P"), Some(Column::Vec2(v)) if v[0] == [3.0, 4.0]));
    assert!(matches!(out.get("vel"), Some(Column::Vec2(v)) if v[0] == [0.0, -2.0]));
    assert!(
        matches!(out.get("id"), Some(Column::Scalar(v)) if v[0] == 5.0),
        "its identity is its BIRTH ORDINAL, not the template's id"
    );
    assert!(out.get("sim_t").is_none(), "a newborn carries no clock");
}

/// **Birth, life and death, through the real registry** — the whole triangle, in a zone.
///
/// The population is born (`sim.spawn` + `motion.combine`), lives (`sim.step`) and, without a
/// kill, only ever GROWS: the newborns persist in the state, tick after tick, which is precisely
/// what a stateless emitter cannot do and what makes this a simulation.
#[test]
fn a_zone_that_spawns_grows_a_population() {
    let reg = registry();
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid"); // the birth template: a single point
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    let zone = g.add_node("sim.zone");
    let spawn = g.add_node("sim.spawn");
    let merge = g.add_node("motion.combine");
    let step = g.add_node("sim.step");
    g.set_param(spawn, "rate", 30.0);

    for (from, fp, to, tp, delayed) in [
        (seed, 0u16, spawn, 0u16, false),
        (zone, 0, merge, 0, true), // the engine-managed state entry: last tick's population
        (spawn, 0, merge, 1, false), // …and this tick's newborns
        (merge, 0, step, 0, false),
        (step, 0, zone, 1, false),
    ] {
        g.connect(Edge {
            from: (NodeId(from.0), fp),
            to: (NodeId(to.0), tp),
            delayed,
        })
        .expect("wire");
    }
    // The zone's `init` is left EMPTY: the population is entirely born, not seeded.
    assert!(g.validate(&reg).is_ok());

    let mut cook = Cook::new();
    let mut counts = Vec::new();
    for k in 0..=120u64 {
        let t = k as f64 / FPS;
        let out = cook.cook(&g, &reg, zone, t).expect("cooks");
        counts.push(out[0].as_stream().count());
        cook.advance_tick(&g, &reg, t).expect("tick closes");
    }
    assert_eq!(counts[0], 0, "nothing is born on the first tick (dt = 0)");
    assert!(
        counts.windows(2).all(|w| w[1] >= w[0]),
        "a population with no kill never shrinks: {counts:?}"
    );
    // 30/s for 2 s ≈ 60 (the first tick spawns none).
    let last = *counts.last().unwrap();
    assert!(
        (58..=60).contains(&last),
        "30 a second for two seconds: {last}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// The `pulse` port (2026-08-10) — the P0 of the conference's sheet 12: until it
// landed, NOTHING in the simulation was triggered by an event.
// ─────────────────────────────────────────────────────────────────────────────

/// **The default REDUCES.** With no pulse column on the port, not one extra element is born and
/// the rate-born keep the FULL `ID_WRAP` period they have always had.
///
/// The second half is the one that bites: carving the pulse half unconditionally would be
/// invisible in every ordinary document (ordinals that small are nowhere near 2²³) and would
/// silently renumber a long-running one. The fixture reaches past the boundary on purpose.
///
/// ⚠️ **It asks the NODE, not the arithmetic.** The first draft asserted `k % ID_WRAP == k`,
/// which is a fact about `%` — true no matter what `eval` does with it, so the mutation it
/// exists to catch (splitting the span unconditionally) sailed straight through it.
#[test]
fn an_unconnected_pulse_is_the_world_before_it() {
    use ph2d_nodegraph::gpu::ID_WRAP;
    let empty = Stream::new(0);
    let (ids, rows) = pulse_born(&empty, 4, 8, 1.0, 1.0 / FPS);
    assert!(ids.is_empty() && rows.is_empty(), "no column, no births");

    // A playhead deep enough that the ordinal is past the pulse half and short of the full one:
    // 4M/s × 2.2 s ≈ 8.8M, and 2²³ = 8.39M < 8.8M < 2²⁴.
    let (rate, t) = (4_000_000.0_f64, 2.2_f64);
    let raw = born_in(rate, t, 1.0 / FPS);
    assert!(
        raw.start > PULSE_ID_BASE && raw.end < ID_WRAP,
        "fixture: the ordinal must live between the two periods"
    );

    let reg = registry();
    let mut g = Graph::new();
    let tpl = g.add_node("motion.grid");
    let sp = g.add_node("sim.spawn");
    g.set_param(sp, "rate", rate as f32);
    g.connect(Edge {
        from: (tpl, 0),
        to: (sp, 0),
        delayed: false,
    })
    .unwrap();
    assert!(g.validate(&reg).is_ok());
    let mut cook = Cook::new();
    cook.cook(&g, &reg, sp, t - 1.0 / FPS).expect("warm-up");
    cook.advance_tick(&g, &reg, t - 1.0 / FPS).expect("tick");
    let out = cook.cook(&g, &reg, sp, t).expect("cooks");
    let Some(Column::Scalar(ids)) = out[0].as_stream().get("id") else {
        panic!("newborns carry an id column");
    };
    assert!(
        ids.iter().all(|id| *id as u32 >= PULSE_ID_BASE),
        "with no pulse wired the ordinal keeps the FULL period — a halved one would fold \
         these ids back into the low half: {:?}",
        &ids[..ids.len().min(4)]
    );
}

/// **A pulse gives birth AT THE ROW THAT FIRED**, `burst` of them, in row order.
///
/// Born at the row, not at a hashed slot: that is what makes it an *event* rather than a second
/// way to spell `rate`. The template's own columns come with it, so the artist aims and colours
/// the birth with ordinary nodes upstream — the vocabulary this node never had to invent.
#[test]
fn a_pulse_gives_birth_at_the_row_that_fired() {
    let template = Stream::new(4)
        .with(
            "P",
            Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
        )
        .with("tint", Column::Scalar(vec![10.0, 11.0, 12.0, 13.0]));

    let (ids, rows) = pulse_born(&fires(4, &[1, 3]), 4, 2, 1.0, 1.0 / FPS);
    assert_eq!(rows, vec![1, 1, 3, 3], "two each, at the rows that fired");
    assert_eq!(ids.len(), 4);

    let out = newborns(&template, &ids, &rows);
    assert!(
        matches!(out.get("P"), Some(Column::Vec2(v)) if v == &[[1.0, 0.0], [1.0, 0.0], [3.0, 0.0], [3.0, 0.0]]),
        "born where the event happened"
    );
    assert!(
        matches!(out.get("tint"), Some(Column::Scalar(v)) if v == &[11.0, 11.0, 13.0, 13.0]),
        "and it inherits every column of that row"
    );
}

/// **The two births are numbered by two clocks, so they live in disjoint halves.**
///
/// A rate-born and a pulse-born wearing the same id would be paired as ONE element by every
/// state node downstream (`motion.integrate`, `motion.spring`, `motion.delay` all key on `id`) —
/// a mispairing that shows up as a particle inheriting a stranger's velocity, with nothing on
/// screen to say why.
#[test]
fn the_two_kinds_of_birth_never_share_an_id() {
    let rate: Vec<u32> = (0..2000).map(|k| k % PULSE_ID_BASE).collect();
    assert!(
        rate.iter().all(|id| *id < PULSE_ID_BASE),
        "rate-born stay in the low half while a pulse is wired"
    );
    for tick in [0u64, 1, 7, 12_345] {
        let (ids, _) = pulse_born(&fires(2, &[0, 1]), 2, 3, tick as f64 / FPS, 1.0 / FPS);
        assert!(
            ids.iter().all(|id| *id >= PULSE_ID_BASE),
            "pulse-born stay in the high half: {ids:?}"
        );
    }
}

/// **The pulse-born id is a pure function of the CLOCK**, which is the property the whole node
/// is built on: rewind, re-cook, and the same elements come back with the same ids.
///
/// It cannot be the birth ordinal (*how many pulses have fired* is HISTORY, and a counter is
/// what this node refuses), so it is the TICK ordinal — and the two halves of this gate are
/// exactly the two things that buys: the ids MOVE between ticks, and they REPEAT on a rewind.
#[test]
fn a_rewind_reproduces_the_same_pulse_births() {
    let dt = 1.0 / FPS;
    let at = |k: u64| pulse_born(&fires(2, &[0]), 2, 2, k as f64 * dt, dt).0;
    assert_ne!(at(30), at(31), "a different tick, different ids");
    assert_eq!(at(30), at(30), "the same tick, the same ids");
    // The gap between two ticks is the whole per-tick reservation, so a tick's ids can never
    // reach into its neighbour's.
    let (a, b) = (at(30), at(31));
    assert_eq!(
        u64::from(b[0]) - u64::from(a[0]),
        PULSE_IDS_PER_TICK,
        "one tick apart is one reservation apart"
    );
}

/// **The tick cap is honoured, and it is the same number for both kinds of birth.** A frame that
/// is already late is the worst possible moment to build thousands of elements.
#[test]
fn a_pulse_storm_is_capped_at_the_same_ceiling_as_a_rate_storm() {
    let n = 1000;
    let all: Vec<usize> = (0..n).collect();
    let (ids, rows) = pulse_born(&fires(n, &all), n, 8, 1.0, 1.0 / FPS);
    assert_eq!(ids.len(), MAX_PER_TICK as usize, "capped");
    assert_eq!(rows.len(), ids.len(), "one row per newborn, always");
    assert_eq!(
        pulse_born(&fires(4, &[0]), 4, 0, 1.0, 1.0 / FPS).0.len(),
        0,
        "a burst of zero is a port that does nothing, not a panic"
    );
    assert_eq!(
        pulse_born(&fires(4, &[0]), 4, 2, 1.0, 0.0).0.len(),
        0,
        "a paused clock gives birth to nothing, by pulse or by rate"
    );
}

/// **End to end, through the real registry: a beat gives birth.**
///
/// The population grows in STEPS at the beat instead of the steady trickle a rate gives — and
/// the control is the same graph with the pulse unwired and `rate = 0`, which must stay empty
/// forever. Without that half the gate would pass over a node that spawned on its own.
#[test]
fn a_beat_gives_birth_inside_a_zone() {
    fn run(wire_pulse: bool) -> Vec<usize> {
        let reg = registry();
        let mut g = Graph::new();
        let seed = g.add_node("motion.grid");
        g.set_param(seed, "rows", 1.0);
        g.set_param(seed, "cols", 1.0);
        let beat = g.add_node("pulse.beat");
        g.set_param(beat, "period", 0.25);
        let zone = g.add_node("sim.zone");
        let spawn = g.add_node("sim.spawn");
        g.set_param(spawn, "rate", 0.0); // ONLY the pulse may give birth here
        g.set_param(spawn, "burst", 5.0);
        let merge = g.add_node("motion.combine");
        let step = g.add_node("sim.step");

        let mut wires: Vec<(NodeId, u16, NodeId, u16, bool)> = vec![
            (seed, 0, spawn, 0, false),
            (seed, 0, beat, 0, false),
            (beat, 0, beat, 1, true), // the family's edge memory
            (zone, 0, merge, 0, true),
            (spawn, 0, merge, 1, false),
            (merge, 0, step, 0, false),
            (step, 0, zone, 1, false),
        ];
        if wire_pulse {
            wires.push((beat, 0, spawn, 1, false));
        }
        for (from, fp, to, tp, delayed) in wires {
            g.connect(Edge {
                from: (from, fp),
                to: (to, tp),
                delayed,
            })
            .expect("wire");
        }
        assert!(g.validate(&reg).is_ok());

        let mut cook = Cook::new();
        let mut counts = Vec::new();
        for k in 0..=90u64 {
            let t = k as f64 / FPS;
            let out = cook.cook(&g, &reg, zone, t).expect("cooks");
            counts.push(out[0].as_stream().count());
            cook.advance_tick(&g, &reg, t).expect("tick closes");
        }
        counts
    }

    let wired = run(true);
    let control = run(false);
    assert!(
        control.iter().all(|c| *c == 0),
        "rate 0 and no pulse: nothing is ever born — the control: {control:?}"
    );
    let last = *wired.last().unwrap();
    assert!(last >= 25, "a quarter-second beat over 1.5 s, 5 each: {last}");
    assert!(
        wired.windows(2).all(|w| w[1] >= w[0]),
        "a population with no kill never shrinks: {wired:?}"
    );
    // The point of an EVENT: the growth is in steps, not a trickle. Most ticks add nothing.
    let quiet = wired.windows(2).filter(|w| w[1] == w[0]).count();
    assert!(
        quiet > wired.len() / 2,
        "births arrive in bursts, not every tick ({quiet} quiet of {})",
        wired.len() - 1
    );
}
