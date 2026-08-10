//! Os gates do `pulse.threshold` — a histerese, as três direções e o **debounce**.
//!
//! ⚠️ Vivem num arquivo IRMÃO por teto de LOC, e seguem sendo módulo **FILHO**
//! (`#[path]`), porque o que eles medem é privado: `step`, `step_one`,
//! `debounce_one` e `EdgeDir` nunca saem da crate, e um teste de fora só
//! alcançaria a porta do `NodeOp` — que é o produto, não a lei.

use super::*;

/// A 60 fps tick, the clock every fixture below runs on.
const DT: f32 = 1.0 / 60.0;
/// The premise the pre-debounce fixtures declare instead of inheriting: a
/// default that moves must not leave them silently testing something else.
const NO_DEBOUNCE: f32 = 0.0;

/// Feed a ramp up then down through the Schmitt trigger, carrying the output
/// back as `state` (what the `pre` self-loop does). It must fire ONCE on the
/// way up (crossing `rise`) and, with hysteresis, only re-arm after the
/// signal drops below the separate, lower `fall`.
#[test]
fn it_fires_once_on_the_rising_edge_and_holds_through_the_hysteresis_band() {
    let (rise, fall) = (0.6, 0.3);
    // A signal that climbs past `rise`, dips into the band (0.3..0.6) WITHOUT
    // falling below `fall`, climbs again, then finally drops below `fall`.
    let signal = [0.0, 0.5, 0.7, 0.9, 0.4, 0.8, 0.2, 0.7];
    let mut state = Stream::new(1);
    let mut fired = Vec::new();
    for &v in &signal {
        let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, v]]));
        state = step(
            &input,
            &state,
            Trigger {
                rise,
                fall,
                edge: EdgeDir::Rise,
                channel: 1,
                debounce: NO_DEBOUNCE,
            },
            DT,
        );
        let p = match state.get(PULSE_COL).unwrap() {
            Column::Scalar(s) => s[0],
            _ => panic!(),
        };
        fired.push(p);
    }
    // Rises at index 2 (0.5→0.7 crosses 0.6). Index 4 (0.4) stays in the band
    // → still armed → NO refire at index 5. Index 6 (0.2 < fall) disarms →
    // index 7 (0.7) re-fires. Exactly two pulses; the band swallowed the dip.
    assert_eq!(fired, vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
}

/// FALSIFICATION of the hysteresis: with a SINGLE threshold (fall == rise),
/// the same in-band dip re-arms and the signal chatters — the extra pulse
/// the two-threshold design exists to suppress.
#[test]
fn a_single_threshold_chatters_where_the_schmitt_stays_quiet() {
    let signal = [0.0, 0.7, 0.4, 0.8]; // dip to 0.4 is below a single 0.6...
    let run = |rise: f32, fall: f32| {
        let mut state = Stream::new(1);
        let mut count = 0;
        for &v in &signal {
            let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, v]]));
            state = step(
                &input,
                &state,
                Trigger {
                    rise,
                    fall,
                    edge: EdgeDir::Rise,
                    channel: 1,
                    debounce: NO_DEBOUNCE,
                },
                DT,
            );
            if let Some(Column::Scalar(s)) = state.get(PULSE_COL) {
                count += (s[0] > 0.5) as i32;
            }
        }
        count
    };
    // Single threshold (fall==rise): 0.4 disarms, 0.8 re-fires → 2 pulses.
    assert_eq!(run(0.6, 0.6), 2, "single threshold chatters");
    // Schmitt (fall 0.3 < rise 0.6): 0.4 stays armed → 1 pulse.
    assert_eq!(run(0.6, 0.3), 1, "hysteresis suppresses the chatter");
}

#[test]
fn the_fall_direction_fires_on_the_disarm_crossing() {
    let signal = [0.0, 0.8, 0.1]; // arm at 0.8, disarm at 0.1.
    let mut state = Stream::new(1);
    let mut fired = Vec::new();
    for &v in &signal {
        let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, v]]));
        state = step(
            &input,
            &state,
            Trigger {
                rise: 0.5,
                fall: 0.3,
                edge: EdgeDir::Fall,
                channel: 1,
                debounce: NO_DEBOUNCE,
            },
            DT,
        );
        if let Some(Column::Scalar(s)) = state.get(PULSE_COL) {
            fired.push(s[0]);
        }
    }
    // Rise edge is ignored; the pulse is on the fall (index 2).
    assert_eq!(fired, vec![0.0, 0.0, 1.0]);
}

/// The `pulse` output is 1.0 only for ONE tick, even while the signal stays
/// high — the Rive "true for a short time". A sustained level is NOT a train
/// of pulses.
#[test]
fn a_sustained_high_signal_fires_exactly_one_pulse() {
    let mut state = Stream::new(1);
    let mut total = 0.0;
    for _ in 0..10 {
        let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, 1.0]]));
        state = step(
            &input,
            &state,
            Trigger {
                rise: 0.5,
                fall: 0.3,
                edge: EdgeDir::Rise,
                channel: 1,
                debounce: NO_DEBOUNCE,
            },
            DT,
        );
        if let Some(Column::Scalar(s)) = state.get(PULSE_COL) {
            total += s[0];
        }
    }
    assert_eq!(total, 1.0, "one edge, not one-per-tick-held-high");
}

#[test]
fn an_inverted_band_is_clamped_not_honoured() {
    // fall > rise would be a nonsense inverted band; clamp fall down to rise
    // (degenerates to a single threshold, never a negative-width band).
    let (p, a) = step_one(0.7, false, 0.5, 0.9, EdgeDir::Rise);
    assert_eq!(
        (p, a),
        (1.0, 1.0),
        "still arms at rise; the bad fall is ignored"
    );
}

/// Run `signal` through the trigger at `dt`, carrying the output back as
/// `state` (what the `pre` self-loop does), and return the pulse per tick.
fn run(signal: &[f32], edge: EdgeDir, debounce: f32, dt: f32) -> Vec<f32> {
    let mut state = Stream::new(1);
    let mut fired = Vec::new();
    for &v in signal {
        let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, v]]));
        state = step(
            &input,
            &state,
            Trigger {
                rise: 0.6,
                fall: 0.3,
                edge,
                channel: 1,
                debounce,
            },
            dt,
        );
        match state.get(PULSE_COL).unwrap() {
            Column::Scalar(s) => fired.push(s[0]),
            _ => panic!(),
        }
    }
    fired
}

/// A chattering gesture: crosses the band cleanly, over and over, fast.
fn chatter(ticks: usize) -> Vec<f32> {
    (0..ticks)
        .map(|i| if i % 2 == 0 { 0.8 } else { 0.1 })
        .collect()
}

/// **The debounce swallows the second crossing, and the CONTROL is the same
/// signal without it.** Both crossings are legitimate — they clear the whole
/// hysteresis band — so this is exactly the repique the band cannot reach.
#[test]
fn the_debounce_swallows_a_second_crossing_that_arrives_too_fast() {
    // arm, disarm, arm again — three ticks, ~50 ms apart at 60 fps.
    let signal = [0.0, 0.7, 0.1, 0.7];
    let free = run(&signal, EdgeDir::Rise, NO_DEBOUNCE, DT);
    assert_eq!(
        free,
        vec![0.0, 1.0, 0.0, 1.0],
        "the CONTROL: the hysteresis is an amplitude guard and lets both through"
    );
    let held = run(&signal, EdgeDir::Rise, 0.2, DT);
    assert_eq!(
        held,
        vec![0.0, 1.0, 0.0, 0.0],
        "a 200 ms window silences the crossing that lands 33 ms later"
    );
}

/// **The debounce owns the OUTPUT, never the state machine** — the law that
/// keeps the trigger alive after its own quiet window.
///
/// ⚠️ **The fixture parks the signal INSIDE the hysteresis band, and that is
/// the whole test.** Outside the band the Schmitt re-syncs in a single tick,
/// so a latch frozen through the window is corrected the moment the freeze
/// ends and the bug is invisible — the first version of this gate held the
/// signal LOW and stayed **green under the mutation it exists to kill**. In
/// the band `armed_now == prev_armed`, so the disarm that happened during the
/// window is lost **for good**: the machine believes it is still armed, the
/// next real rise is not an edge, and the node goes quiet forever.
#[test]
fn the_debounce_silences_the_output_never_the_state_machine() {
    // arm+fire, disarm inside the window, then drift in the band (0.3, 0.6)
    // until it has drained, then a clean rise.
    let mut signal = vec![0.0, 0.7, 0.1];
    signal.extend(std::iter::repeat_n(0.45, 14));
    signal.push(0.7);
    let fired = run(&signal, EdgeDir::Rise, 0.2, DT);
    let ticks: Vec<usize> = fired
        .iter()
        .enumerate()
        .filter(|(_, p)| **p > 0.5)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        ticks,
        vec![1, signal.len() - 1],
        "the trigger survives its own quiet window: {fired:?}"
    );
}

/// **A zero debounce is the world before the param**, and the oracle is the raw
/// Schmitt itself: the same signal walked through `step_one` by hand, with no
/// countdown anywhere. Equality is exact — the neutral does not merely look the
/// same, it takes the same arithmetic.
#[test]
fn a_zero_debounce_is_the_world_before_it() {
    let signal = chatter(40);
    let through_the_node = run(&signal, EdgeDir::Both, NO_DEBOUNCE, DT);

    let mut armed = false;
    let raw: Vec<f32> = signal
        .iter()
        .map(|&v| {
            let (p, a) = step_one(v, armed, 0.6, 0.3, EdgeDir::Both);
            armed = a > 0.5;
            p
        })
        .collect();
    assert_eq!(
        through_the_node, raw,
        "the neutral is the raw Schmitt, tick for tick"
    );
}

/// **The window is measured in SECONDS, not in ticks** — the guard against a
/// countdown that decrements by one per tick, which would make the same
/// authored number mean four different things at four frame rates.
///
/// The oracle is the silence itself, in WALL TIME: the mean gap between
/// pulses, times `dt`. ⚠️ The tolerance is DERIVED, not picked — the signal
/// only crosses on tick boundaries, so once the window drains the next arm can
/// be up to one full chatter period (two ticks) away, and at 15 fps that is
/// 133 ms of honest quantisation. Measured: **0,200 s at 60 fps · 0,267 at 15**,
/// against an authored 0,200.
#[test]
fn the_window_is_measured_in_seconds_not_in_ticks() {
    const TICKS: usize = 60;
    const WINDOW: f32 = 0.2;
    let signal = chatter(TICKS);
    let count = |dt: f32| {
        run(&signal, EdgeDir::Rise, WINDOW, dt)
            .iter()
            .filter(|p| **p > 0.5)
            .count()
    };
    let free = run(&signal, EdgeDir::Rise, NO_DEBOUNCE, 1.0 / 60.0)
        .iter()
        .filter(|p| **p > 0.5)
        .count();
    let (fast, slow) = (count(1.0 / 60.0), count(1.0 / 15.0));

    // The CONTROL: without a window this signal fires on every other tick.
    assert_eq!(free, TICKS / 2, "the un-debounced chatter");
    assert!(
        fast < free && slow < free,
        "both clocks are gated at all ({fast} and {slow} of {free})"
    );
    // A tick-counting countdown would give the SAME number at both rates.
    assert!(
        slow > fast,
        "the slow clock fits fewer ticks in the same second ({fast} vs {slow})"
    );
    // And the silence itself is the authored duration, at either clock.
    for (dt, hits) in [(1.0f32 / 60.0, fast), (1.0f32 / 15.0, slow)] {
        let gap_seconds = TICKS as f32 * dt / hits as f32;
        let tolerance = 2.0 * dt;
        assert!(
            (gap_seconds - WINDOW).abs() <= tolerance,
            "at dt={dt} the silence measured {gap_seconds} s against {WINDOW}"
        );
    }
}

/// **Where the countdown stops expiring** — the sonda behind `PARAM_HARD_MAX`
/// for `debounce`. The window lives in `f32` seconds and is decremented by
/// `dt`, so above some magnitude `cool - dt == cool` and the node would go
/// quiet **for good, silently**. That is a limit of *representation*, which is
/// the only kind this param has: nothing else about a long window costs
/// anything.
///
/// `cargo test -p ph2d-node-pulse-threshold -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição: imprime a escada, não afirma"]
fn the_debounce_ceiling_is_where_the_countdown_stops_expiring() {
    for e in 10..26 {
        let x = (2.0f32).powi(e);
        let step_60 = x - 1.0 / 60.0 < x;
        let step_15 = x - 1.0 / 15.0 < x;
        println!("2^{e:<2} = {x:>12} s | drena a 60 fps: {step_60:<5} | a 15 fps: {step_15}");
    }
}

/// **The typed ceiling is the last magnitude that still drains**, at the worst
/// clock the app runs. The second half is what makes the number a MEASUREMENT
/// instead of a preference: one octave further, the countdown stands still.
#[test]
fn the_debounce_hard_max_is_the_last_window_that_still_drains() {
    let ceiling = PARAM_HARD_MAX
        .iter()
        .find(|h| h.param == "debounce")
        .expect("debounce has a typed ceiling")
        .max;
    // The fastest clock is the harshest test: the smallest decrement.
    let dt = 1.0f32 / 240.0;
    assert!(
        ceiling - dt < ceiling,
        "{ceiling} s still counts down at 240 fps"
    );
    assert!(
        ceiling * 2.0 - dt >= ceiling * 2.0,
        "and {} s does not — the ceiling sits ON the cliff, not near it",
        ceiling * 2.0
    );
}

/// `Both` fires on the arm AND the disarm crossing — one pulse each way
/// (audit 2026-07-10: the third `EdgeDir` was untested). Rise-only and
/// Fall-only each see exactly one of the two.
#[test]
fn the_both_direction_fires_on_arm_and_disarm() {
    let signal = [0.0, 0.8, 0.1]; // arm at 0.8, disarm at 0.1.
    let run = |edge: EdgeDir| {
        let mut state = Stream::new(1);
        let mut fired = Vec::new();
        for &v in &signal {
            let input = Stream::new(1).with("P", Column::Vec2(vec![[0.0, v]]));
            state = step(
                &input,
                &state,
                Trigger {
                    rise: 0.5,
                    fall: 0.3,
                    edge,
                    channel: 1,
                    debounce: NO_DEBOUNCE,
                },
                DT,
            );
            if let Some(Column::Scalar(s)) = state.get(PULSE_COL) {
                fired.push(s[0]);
            }
        }
        fired
    };
    assert_eq!(run(EdgeDir::Both), vec![0.0, 1.0, 1.0], "both crossings");
    assert_eq!(run(EdgeDir::Rise), vec![0.0, 1.0, 0.0], "arm only");
    assert_eq!(run(EdgeDir::Fall), vec![0.0, 0.0, 1.0], "disarm only");
}
