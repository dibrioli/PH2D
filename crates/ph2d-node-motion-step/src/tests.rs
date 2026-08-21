//! Gates for `motion.step` — the staircase, its three limit modes, and the
//! RESET (doc 89 fam. 7). Split from `lib.rs` for the LOC cap; a `#[path]`
//! CHILD, so `use super::*` still reaches the crate's private items.

use super::*;

fn dot(x: f32) -> Stream {
    Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]]))
}
fn fire(v: f32) -> Stream {
    Stream::new(1).with(PULSE_COL, Column::Scalar(vec![v]))
}
/// The product door with **nothing wired to `reset`** — the shipped shape of a
/// `motion.step` the artist has not given a reset to, and the reason the gates
/// below are the byte-identity proof rather than a rewrite of them.
fn stepped(input: &Stream, pulse: &Stream, state: &Stream, p: &Params) -> Stream {
    step(input, pulse, state, &Stream::new(0), p)
}

fn params(count_max: i64, mode: LimitMode) -> Params {
    Params {
        channel: 0, // X
        step: 1.0,
        count_max,
        mode,
        reset_to: 0.0,
    }
}
fn count(s: &Stream) -> f32 {
    match s.get(COUNT_COL).unwrap() {
        Column::Scalar(v) => v[0],
        _ => panic!(),
    }
}
fn x(s: &Stream) -> f32 {
    match s.get("P").unwrap() {
        Column::Vec2(v) => v[0][0],
        _ => panic!(),
    }
}

/// FALSIFICATION of edge-safety: a pulse HELD high for several ticks advances
/// the count exactly ONCE (on the rising edge), not once per tick. Counting
/// `pulse > 0.5` every tick — the bug — would reach 5 after a 5-tick hold.
#[test]
fn a_held_pulse_counts_once_not_once_per_tick() {
    let p = params(16, LimitMode::Wrap);
    let mut state = Stream::new(1);
    // The pulse rises at tick 0 and STAYS high for five ticks.
    for _ in 0..5 {
        state = stepped(&dot(0.0), &fire(1.0), &state, &p);
    }
    assert_eq!(count(&state), 1.0, "one rising edge = one count, not five");
    // It drops, then rises again → the second edge counts.
    state = stepped(&dot(0.0), &fire(0.0), &state, &p);
    state = stepped(&dot(0.0), &fire(1.0), &state, &p);
    assert_eq!(count(&state), 2.0, "the next rising edge counts once more");
}

/// Wrap mode: the count climbs 0..N-1 and returns HOME (0) on the Nth pulse —
/// the displacement is `count · step`, so the grid slides out and snaps back.
#[test]
fn wrap_mode_returns_home_after_count_max() {
    let p = params(4, LimitMode::Wrap); // counts 0,1,2,3 then wraps
    let mut state = Stream::new(1);
    let mut seq = Vec::new();
    // Ten single-tick pulses (a 0 between each so every 1.0 is a fresh edge).
    for _ in 0..10 {
        state = stepped(&dot(0.0), &fire(1.0), &state, &p);
        seq.push(count(&state));
        state = stepped(&dot(0.0), &fire(0.0), &state, &p);
    }
    // 1,2,3,0,1,2,3,0,1,2 — home at every 4th pulse.
    assert_eq!(seq, vec![1.0, 2.0, 3.0, 0.0, 1.0, 2.0, 3.0, 0.0, 1.0, 2.0]);
}

/// Clamp mode: the count plateaus at N-1 and never wraps — the staircase holds
/// at the top instead of returning home.
#[test]
fn clamp_mode_plateaus_at_the_top() {
    let p = params(3, LimitMode::Clamp); // 0,1,2 then holds at 2
    let mut state = Stream::new(1);
    let mut seq = Vec::new();
    for _ in 0..6 {
        state = stepped(&dot(0.0), &fire(1.0), &state, &p);
        seq.push(count(&state));
        state = stepped(&dot(0.0), &fire(0.0), &state, &p);
    }
    assert_eq!(seq, vec![1.0, 2.0, 2.0, 2.0, 2.0, 2.0], "holds at N-1");
}

/// Zigzag mode: a triangle — up to N-1 then back down to 0 and up again
/// (period `2(N-1)`). This is the smooth ping-pong sweep the demo uses.
#[test]
fn zigzag_mode_pingpongs_up_then_down() {
    let p = params(4, LimitMode::Zigzag); // triangle 0..3..0, period 6
    let mut state = Stream::new(1);
    let mut seq = Vec::new();
    for _ in 0..8 {
        state = stepped(&dot(0.0), &fire(1.0), &state, &p);
        seq.push(count(&state));
        state = stepped(&dot(0.0), &fire(0.0), &state, &p);
    }
    // 1,2,3,2,1,0,1,2 — climbs to 3, folds back to 0, climbs again.
    assert_eq!(seq, vec![1.0, 2.0, 3.0, 2.0, 1.0, 0.0, 1.0, 2.0]);
}

/// FALSIFICATION of "apply to fresh input, never compound": after three
/// counts the X displacement is `count · step` off the FRESH base (3·1 = 3),
/// NOT the sum of every step applied to an ever-growing state (1+2+3 = 6).
#[test]
fn the_displacement_never_compounds_across_ticks() {
    let p = params(16, LimitMode::Wrap);
    let mut state = Stream::new(1);
    for _ in 0..3 {
        // Every tick feeds the SAME fresh base X = 10.0.
        state = stepped(&dot(10.0), &fire(1.0), &state, &p);
        state = stepped(&dot(10.0), &fire(0.0), &state, &p);
    }
    // count 3, step 1 → X = 10 + 3 = 13, not 10 + (1+2+3) = 16.
    assert_eq!(count(&state), 3.0);
    assert_eq!(
        x(&state),
        13.0,
        "displacement is count·step off the fresh base"
    );
}

/// A degenerate `count_max = 1` never divides by zero and stays home (count 0,
/// zero displacement) no matter how many pulses arrive.
#[test]
fn count_max_one_stays_home_without_dividing_by_zero() {
    let p = params(1, LimitMode::Zigzag); // period 2(N-1) would be 0 — guarded
    let mut state = Stream::new(1);
    for _ in 0..5 {
        state = stepped(&dot(0.0), &fire(1.0), &state, &p);
        state = stepped(&dot(0.0), &fire(0.0), &state, &p);
    }
    assert_eq!(count(&state), 0.0);
    assert_eq!(x(&state), 0.0, "no displacement, no panic");
}

/// A STABLE multi-instance stream steps in lockstep under a global beat
/// (audit 2026-07-10: every test above uses n = 1, so the positional
/// pairing across rows was untested). Three dots, two beats: every row
/// carries the same count and the same displacement off its own base.
#[test]
fn a_stable_multi_instance_stream_steps_in_lockstep() {
    let p = params(16, LimitMode::Wrap);
    let three = || {
        Stream::new(3).with(
            "P",
            Column::Vec2(vec![[0.0, 0.0], [10.0, 0.0], [20.0, 0.0]]),
        )
    };
    let fire3 = |v: f32| Stream::new(3).with(PULSE_COL, Column::Scalar(vec![v; 3]));
    let mut state = Stream::new(3);
    for _ in 0..2 {
        state = stepped(&three(), &fire3(1.0), &state, &p);
        state = stepped(&three(), &fire3(0.0), &state, &p);
    }
    match state.get(COUNT_COL).unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![2.0; 3], "one count, all rows"),
        _ => panic!(),
    }
    match state.get("P").unwrap() {
        // count 2 · step 1 off each row's OWN fresh base.
        Column::Vec2(v) => assert_eq!(v, &vec![[2.0, 0.0], [12.0, 0.0], [22.0, 0.0]]),
        _ => panic!(),
    }
}

/// A reset stream shaped like the pulse one (the port carries the same
/// `PULSE` type, so it carries the same column).
fn res(v: f32) -> Stream {
    Stream::new(1).with(PULSE_COL, Column::Scalar(vec![v]))
}

/// Climb the staircase, then send a reset: the count is home and so is the
/// DISPLACEMENT — without this the `count_tick` only ever grows, and lowering
/// `count_max` just refolds the same tick.
#[test]
fn a_reset_returns_the_staircase_home() {
    let p = params(16, LimitMode::Wrap);
    let mut state = Stream::new(1);
    for _ in 0..4 {
        state = stepped(&dot(0.0), &fire(1.0), &state, &p);
        state = stepped(&dot(0.0), &fire(0.0), &state, &p);
    }
    assert_eq!(
        count(&state),
        4.0,
        "the fixture must have somewhere to fall FROM"
    );
    assert_eq!(x(&state), 4.0, "…and the displacement climbed with it");

    state = step(&dot(0.0), &fire(0.0), &state, &res(1.0), &p);
    assert_eq!(count(&state), 0.0, "a reset returns the count home");
    assert_eq!(x(&state), 0.0, "…and the displacement with it");
    // And it STAYS home rather than resuming from four.
    state = stepped(&dot(0.0), &fire(1.0), &state, &p);
    assert_eq!(
        count(&state),
        1.0,
        "the next pulse counts from home, not from four"
    );
}

/// **The asymmetry that is the whole design, in one fixture:** a held PULSE
/// counts once (it accumulates, so the edge matters) while a held RESET holds
/// the counter down (it is idempotent, so the level is the honest reading —
/// TD's `While On`, for free, with no Reset Condition param to pick it).
///
/// The control is the other half of the same run: the identical pulse train
/// without the reset does climb.
#[test]
fn a_held_reset_holds_where_a_held_pulse_counts_once() {
    let p = params(16, LimitMode::Wrap);

    // Held reset, with pulses arriving THROUGH it: five edges, still home.
    //
    // ⚠️ The count is asserted at EVERY tick of the hold, not once at the end:
    // a reset read as an EDGE lets each rising pulse push the count to 1 and
    // only looks home again on the tick the pulse falls, so an end-of-loop
    // assertion would be measuring the one instant the defect is invisible
    // (the mutation found this fixture before the fixture found the mutation).
    let mut held = Stream::new(1);
    for i in 0..10 {
        let pulse = if i % 2 == 0 { 1.0 } else { 0.0 };
        held = step(&dot(0.0), &fire(pulse), &held, &res(1.0), &p);
        assert_eq!(
            count(&held),
            0.0,
            "tick {i}: a held reset holds the counter down while it is held"
        );
        assert_eq!(x(&held), 0.0, "tick {i}: …and the displacement with it");
    }

    // CONTROL: the same pulse train with the reset absent really does climb —
    // otherwise the gate would be measuring a train that never counted.
    let mut free = Stream::new(1);
    for _ in 0..5 {
        free = stepped(&dot(0.0), &fire(1.0), &free, &p);
        free = stepped(&dot(0.0), &fire(0.0), &free, &p);
    }
    assert_eq!(
        count(&free),
        5.0,
        "the fixture's pulses do count when nothing holds them"
    );
}

/// The reset WINS the tick it shares with a pulse — `reset_to`, never one past
/// it. The word says so, and the order in `advance_tick` is what makes it true.
#[test]
fn the_reset_wins_the_tick_it_shares_with_a_pulse() {
    let p = params(16, LimitMode::Wrap);
    let mut state = Stream::new(1);
    state = stepped(&dot(0.0), &fire(1.0), &state, &p);
    state = stepped(&dot(0.0), &fire(0.0), &state, &p);
    assert_eq!(count(&state), 1.0);
    // A RISING pulse and the reset land together.
    state = step(&dot(0.0), &fire(1.0), &state, &res(1.0), &p);
    assert_eq!(
        count(&state),
        0.0,
        "reset + pulse is the reset value, not one past it"
    );
}

/// TD's *Reset Value*: the staircase restarts where the artist said, and the
/// displacement follows (the count is what `step` multiplies).
#[test]
fn the_reset_lands_on_the_authored_value() {
    let mut p = params(16, LimitMode::Wrap);
    p.reset_to = 3.0;
    let mut state = Stream::new(1);
    for _ in 0..6 {
        state = stepped(&dot(0.0), &fire(1.0), &state, &p);
        state = stepped(&dot(0.0), &fire(0.0), &state, &p);
    }
    assert_eq!(count(&state), 6.0);
    state = step(&dot(0.0), &fire(0.0), &state, &res(1.0), &p);
    assert_eq!(count(&state), 3.0, "it restarts at the authored value");
    assert_eq!(
        x(&state),
        3.0,
        "…and the displacement is that count times the step"
    );
    state = stepped(&dot(0.0), &fire(1.0), &state, &p);
    assert_eq!(count(&state), 4.0, "…and climbs from there");
}

/// Letting go of the reset must not manufacture an edge: the reset leaves
/// `count_prev` alone, so a counting pulse that was ALREADY high when the reset
/// arrived stays counted rather than counting again the moment it is released.
#[test]
fn letting_go_of_the_reset_does_not_manufacture_an_edge() {
    let p = params(16, LimitMode::Wrap);
    let mut state = Stream::new(1);
    // The pulse rises and stays high; the reset arrives on top of it and goes.
    state = stepped(&dot(0.0), &fire(1.0), &state, &p);
    state = step(&dot(0.0), &fire(1.0), &state, &res(1.0), &p);
    assert_eq!(count(&state), 0.0);
    // Reset released, the counting pulse STILL high — no new rising edge.
    state = stepped(&dot(0.0), &fire(1.0), &state, &p);
    assert_eq!(
        count(&state),
        0.0,
        "the pulse never fell, so releasing the reset must not re-count it"
    );
    // It falls and rises again → THAT is an edge.
    state = stepped(&dot(0.0), &fire(0.0), &state, &p);
    state = stepped(&dot(0.0), &fire(1.0), &state, &p);
    assert_eq!(count(&state), 1.0, "a real edge still counts");
}

// ─────────────────────────────────────────────────────────────────────────────
// Doc 89 folha 07 — o pareamento. Estes gates correm o `step` INTEIRO ao longo de
// vários tiques, não a função `pairing` isolada (essa tem gates próprios em
// `pairing_tests.rs`): o que a célula alegava é que a ESCADA sai errada, e uma
// escada só existe depois de dois tiques.
// ─────────────────────────────────────────────────────────────────────────────

/// Uma stream de `n` peças com identidade — os `id` são a coluna que o emitter
/// estampa.
fn with_ids(ids: &[f32]) -> Stream {
    let p: Vec<[f32; 2]> = ids.iter().map(|k| [k * 10.0, 0.0]).collect();
    Stream::new(ids.len())
        .with("P", Column::Vec2(p))
        .with("id", Column::Scalar(ids.to_vec()))
}
fn fire_n(n: usize, v: f32) -> Stream {
    Stream::new(n).with(PULSE_COL, Column::Scalar(vec![v; n]))
}
fn counts(s: &Stream) -> Vec<f32> {
    match s.get(COUNT_COL).unwrap() {
        Column::Scalar(v) => v.clone(),
        _ => panic!(),
    }
}

/// **UMA PEÇA QUE SOBREVIVE À ROTAÇÃO DO CONJUNTO MANTÉM A ESCADA DELA.**
///
/// É a célula inteira, encenada: três peças sobem dois degraus; depois duas
/// morrem e uma nasce. O sobrevivente (`id 9`) tem de continuar em **2**, e o
/// recém-nascido (`id 11`) tem de começar em **0**.
///
/// ⚠️ **A lei antiga daria `[2, 2]`** — posicionalmente o `9` de hoje leria a
/// linha 0 e o `11` leria a linha 1, ambas com o tique 2. O gate exige o `0`
/// explicitamente, senão um regresso ao pareamento posicional passava verde numa
/// cena onde "as peças contam".
#[test]
fn a_survivor_keeps_its_staircase_when_the_set_churns() {
    let p = params(16, LimitMode::Wrap);
    let born = with_ids(&[7.0, 8.0, 9.0]);
    let mut state = Stream::new(0);
    for _ in 0..2 {
        state = stepped(&born, &fire_n(3, 1.0), &state, &p);
        state = stepped(&born, &fire_n(3, 0.0), &state, &p);
    }
    assert_eq!(counts(&state), vec![2.0; 3], "os três subiram dois degraus");

    // O conjunto roda: 7 e 8 morrem, 11 nasce. Um tique sem pulso, para que só o
    // pareamento decida o número.
    let churned = with_ids(&[9.0, 11.0]);
    let after = stepped(&churned, &fire_n(2, 0.0), &state, &p);
    assert_eq!(
        counts(&after),
        vec![2.0, 0.0],
        "o id 9 leva a escada dele; o id 11 é novo e semeia em 0"
    );
}

/// **UM BATIMENTO GLOBAL (uma linha só) ALCANÇA TODAS AS PEÇAS.**
///
/// ⚠️ **Este é o segundo defeito da célula, e a lei antiga esticava com zeros:**
/// só o elemento 0 contava, e os outros ficavam parados para sempre. A leitura
/// na tela era *"o Step não faz nada"* — o pior modo de falha que esta casa
/// conhece, porque não parece um bug, parece um knob morto.
#[test]
fn a_one_row_pulse_is_a_global_beat_not_a_pulse_for_element_zero() {
    let p = params(16, LimitMode::Wrap);
    let four = || {
        Stream::new(4).with(
            "P",
            Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
        )
    };
    let global = |v: f32| Stream::new(1).with(PULSE_COL, Column::Scalar(vec![v]));
    let mut state = Stream::new(4);
    state = stepped(&four(), &global(1.0), &state, &p);
    assert_eq!(counts(&state), vec![1.0; 4], "todas contaram uma vez");

    // …e a beira continua a valer: segurar o batimento não conta outra vez.
    // ⚠️ É aqui que se vê porque o `count_prev` tem de sair DIFUNDIDO: se ele
    // guardasse a coluna crua (uma linha), as peças 1..3 leriam `prev = 0` contra
    // `pulse = 1` e fabricariam uma beira por quadro.
    state = stepped(&four(), &global(1.0), &state, &p);
    assert_eq!(counts(&state), vec![1.0; 4], "segurar não conta de novo");
    state = stepped(&four(), &global(0.0), &state, &p);
    state = stepped(&four(), &global(1.0), &state, &p);
    assert_eq!(
        counts(&state),
        vec![2.0; 4],
        "largar e bater conta a segunda"
    );
}

/// **UM RESET GLOBAL TAMBÉM ALCANÇA TODAS AS PEÇAS** — a porta irmã lê a mesma
/// escada, e um botão de reset é exactamente uma linha só.
#[test]
fn a_one_row_reset_returns_the_whole_set_home() {
    let p = params(16, LimitMode::Wrap);
    let four = || {
        Stream::new(4).with(
            "P",
            Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
        )
    };
    let mut state = Stream::new(4);
    let none = Stream::new(0);
    for _ in 0..3 {
        state = step(&four(), &fire_n(4, 1.0), &state, &none, &p);
        state = step(&four(), &fire_n(4, 0.0), &state, &none, &p);
    }
    assert_eq!(counts(&state), vec![3.0; 4]);
    let global_reset = Stream::new(1).with(PULSE_COL, Column::Scalar(vec![1.0]));
    state = step(&four(), &fire_n(4, 0.0), &state, &global_reset, &p);
    assert_eq!(counts(&state), vec![0.0; 4], "o botão zera o conjunto todo");
}
