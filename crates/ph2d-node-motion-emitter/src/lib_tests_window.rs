//! **Quantos existem, e quais** — a LEI DE CONTAGEM do emissor, separada de `lib_tests.rs` no
//! teto de LOC do HR-18 pela costura que o arquivo já tinha: o pai responde *como cada partícula
//! sai* (a velocidade, o lugar de nascimento, a direção) e este *quais existem neste instante* —
//! a janela viva, o espaço de ids, o teto e os degenerados.
//!
//! NETO de `lib.rs`, então `use super::*` ainda alcança o `spec()` e o `ids_of` em vez de os
//! copiar — um segundo `spec()` seria uma segunda resposta a *como é um emissor intocado?*.

use super::*;
use ph2d_nodegraph::gpu::CountLawCtx;

/// **Identity survives any rate, at any playhead** — the invariant that lets
/// this engine aim at millions of particles instead of thousands.
///
/// This gate was written RED, asserting the opposite. Ids are stored as `f32`
/// (the stream's only scalar column), so a spawn index past 2²⁴ used to
/// collapse onto its neighbour: measured, rate 4.000.000 at t = 5 s gave
/// 4096 particles and **2049 distinct ids**, and both pairings — the CPU's
/// `BTreeMap<id,row>` and the GPU's `id − prev_first` — then handed two
/// particles the same prior state, in silence.
///
/// The emitter's own doc filed this under "≈ 4,8 days at rate 40, out of
/// scope". That was TRUE while the rate slider stopped at 200 (23 hours). It
/// stopped being true the moment the ceiling moved — 23 minutes at 12.000/s,
/// **4 seconds** at the millions-per-second a real particle sim wants. A note
/// that says "unreachable" is a claim about a number someone else can change.
///
/// Now the index WRAPS at [`ID_WRAP`] and every consumer reads identity as a
/// difference inside one window, which is orders of magnitude smaller than
/// the wrap period. So there is no rate at which this breaks.
#[test]
fn identity_is_exact_at_any_rate_because_it_wraps() {
    let distinct_ids = |rate: f32, t: f32| {
        let mut sp = spec();
        sp.spawn = Spawn::Continuous { rate };
        sp.life = 1.0;
        sp.max = 4096;
        let mut ids = ids_of(&emit(&sp, t));
        let n = ids.len();
        ids.sort_by(f32::total_cmp);
        ids.dedup();
        (n, ids.len())
    };
    // Well under the old cliff, far past it, and absurdly past it.
    for (rate, t) in [
        (200.0, 5.0),
        (12_000.0, 5.0),
        (4_000_000.0, 5.0),     // used to collapse to 2049 of 4096
        (4_000_000.0, 3_600.0), // an hour in: 1,4e10 spawns, ~858 wraps
    ] {
        let (n, distinct) = distinct_ids(rate, t);
        assert!(
            n > 1,
            "rate {rate} at t={t}: the fixture must have a window"
        );
        assert_eq!(
            distinct, n,
            "rate {rate} at t={t}: every id in a window must be its own"
        );
    }
    // …and the wrap is REACHED, not merely tolerated — otherwise the loop
    // above would be four restatements of "we stayed under 2²⁴".
    let w = window(Spawn::Continuous { rate: 4_000_000.0 }, 1.0, 4096, 3_600.0);
    assert!(
        (w.first as u64) < u64::from(ID_WRAP),
        "the stored index lives inside the exact range"
    );
    assert!(
        4_000_000.0f64 * 3_600.0 > f64::from(ID_WRAP),
        "the fixture must actually pass the wrap point"
    );
    // The age stays a small, honest number instead of the difference of two
    // large ones: at rate 4e6 the old `t − id/rate` was 3600.0 − 3600.0.
    assert!(
        w.age_first > 0.0 && w.age_first <= 1.0,
        "the oldest particle's age must be within `life`, got {}",
        w.age_first
    );
}

#[test]
fn the_alive_set_is_the_id_window_born_within_life() {
    // rate 10, life 1 → at t = 1.55 the alive ids are those born in
    // (0.55, 1.55]: k/10 ∈ (0.55, 1.55] → k ∈ 6..=15.
    let s = emit(&spec(), 1.55);
    assert_eq!(ids_of(&s).first().copied(), Some(6.0));
    assert_eq!(ids_of(&s).last().copied(), Some(15.0));
    assert_eq!(s.count(), 10);
    // Ids are strictly ascending (oldest first) so Index reads as age order.
    assert!(ids_of(&s).windows(2).all(|w| w[1] > w[0]));
}

#[test]
fn particles_are_born_and_die_on_schedule() {
    // At t=0 only particle 0 exists; by t just under 1 life later it still
    // does; a hair past, it is gone (its successors remain).
    assert_eq!(ids_of(&emit(&spec(), 0.0)), vec![0.0]);
    assert!(ids_of(&emit(&spec(), 0.99)).contains(&0.0), "still alive");
    assert!(!ids_of(&emit(&spec(), 1.01)).contains(&0.0), "died at life");
}

#[test]
fn scrubbing_backwards_reproduces_the_scene_exactly() {
    // The reference's stateful emitter could not do this. Ask for a time,
    // get the same particles — no matter what was asked before.
    let forward = emit(&spec(), 2.5);
    for t in [0.3, 7.1, 0.0, 2.5] {
        let _ = emit(&spec(), t);
    }
    let revisited = emit(&spec(), 2.5);
    assert_eq!(ids_of(&forward), ids_of(&revisited));
    assert_eq!(forward.get("vel"), revisited.get("vel"));
}

#[test]
fn launch_velocity_follows_angle_and_speed() {
    // Zero spread, angle 90 (up in this Y-up world) → vel = (0, speed).
    let s = emit(&spec(), 0.5);
    match s.get("vel").unwrap() {
        Column::Vec2(v) => {
            for q in v {
                assert!(q[0].abs() < 1e-4, "no lateral component");
                assert!((q[1] - 2.0).abs() < 1e-4, "speed straight up");
            }
        }
        _ => panic!("vel"),
    }
    // And they all start at the origin — displacement is the integrator's.
    match s.get("P").unwrap() {
        Column::Vec2(v) => assert!(v.iter().all(|p| *p == [1.0, -1.0])),
        _ => panic!("P"),
    }
}

#[test]
fn spread_fans_the_cone_but_never_past_its_half_angle() {
    let mut sp = spec();
    sp.spread = 60.0;
    sp.spawn = Spawn::Continuous { rate: 200.0 }; // plenty of samples
    let s = emit(&sp, 1.0);
    let Column::Vec2(v) = s.get("vel").unwrap() else {
        panic!("vel")
    };
    // angle 90 ± 30 → the x-component is bounded by speed·sin(30°) = 1.0.
    let max_lateral = v.iter().map(|q| q[0].abs()).fold(0.0f32, f32::max);
    assert!(
        max_lateral <= 1.01,
        "cone half-angle respected: {max_lateral}"
    );
    assert!(max_lateral > 0.5, "the cone actually fans out");
    // All still flying upward.
    assert!(v.iter().all(|q| q[1] > 0.0));
}

#[test]
fn the_cap_keeps_the_newest_particles() {
    let mut sp = spec();
    sp.max = 3;
    let s = emit(&sp, 1.55); // would be 10 alive
    assert_eq!(s.count(), 3);
    assert_eq!(ids_of(&s), vec![13.0, 14.0, 15.0], "the newest three");
}

#[test]
fn a_dead_or_absurd_emitter_yields_an_empty_stream_not_a_panic() {
    let mut sp = spec();
    sp.spawn = Spawn::Continuous { rate: 0.0 };
    assert_eq!(emit(&sp, 5.0).count(), 0);
    let mut sp = spec();
    sp.life = 0.0;
    assert_eq!(emit(&sp, 5.0).count(), 0);
    // A negative playhead (a scrub past zero) emits nothing rather than
    // hashing negative ids.
    assert_eq!(emit(&spec(), -1.0).count(), 0);
    // `emit` honours whatever cap it is handed…
    let mut sp = spec();
    sp.spawn = Spawn::Continuous { rate: 1e6 };
    sp.max = 4096;
    assert_eq!(emit(&sp, 10.0).count(), 4096);
    // …and the HARD ceiling is asserted on the count law itself rather than
    // by building the stream: at 4M elements that would allocate ~176 MB to
    // learn a number `window` already knows. It is also the only place worth
    // asserting it, now that `emit` and the GPU `count_law` are the same
    // function — the two paths cannot disagree about `n` any more, which is
    // the point of unifying them.
    assert_eq!(
        window(Spawn::Continuous { rate: 1e9 }, 1e9, MAX_ALIVE, 10.0).count,
        MAX_ALIVE,
        "an absurd ask clamps to the ceiling"
    );
    let gpu_window = GPU_KERNEL.count_law.expect("the emitter is a generator");
    assert_eq!(
        gpu_window(&CountLawCtx {
            inputs: &[],
            param: &|_| 1e9,
            playhead: 10.0,
            dt: 0.0,
        })
        .count,
        MAX_ALIVE,
        "the GPU path reaches the same ceiling through the same law"
    );
}
