//! Guards for `sim.lifetime` (O4 death by age, doc 50). `super` is the crate root.

use super::*;

fn aged(ages: &[f32]) -> Stream {
    let n = ages.len();
    Stream::new(n)
        .with("age", Column::Scalar(ages.to_vec()))
        .with("id", Column::Scalar((0..n).map(|i| i as f32).collect()))
        .with("P", Column::Vec2(vec![[0.0, 0.0]; n]))
}

fn col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("no `{name}`"),
    }
}

/// **What has outlived its lifetime is gone; what has not, survives — and knows how far through
/// life it is.** `life` is 0 at birth and 1 at the end, which is the number a colour ramp, a
/// scale or a fade actually wants.
///
/// FALSIFIED by a node that only kills (the artist would have to rebuild the fraction out of an
/// age and a parameter they cannot read) and by one that kills at `age > life` rather than
/// `age >= life` — a particle at exactly its lifetime is over.
#[test]
fn the_outlived_die_and_the_survivors_know_their_fraction() {
    let out = reap(&aged(&[0.0, 1.0, 1.9, 2.5]), 2.0, 0.0, 1);
    assert_eq!(out.count(), 3, "the 2.5s-old one is past its 2s life");
    let life = col(&out, "life");
    assert_eq!(life[0], 0.0, "newborn: 0");
    assert_eq!(life[1], 0.5, "halfway: 0.5");
    assert!((life[2] - 0.95).abs() < 1e-6, "nearly gone: 0.95");
}

/// **Variance spreads the deaths by IDENTITY, not by a draw.** Without it, everything born on
/// one tick dies on one tick and the population blinks; with it, each element's lifetime is a
/// hash of its id — so it is the same on a rewind, and the same on a second machine.
///
/// FALSIFIED by a variance that never fires (all lifetimes equal) and by one drawn from a
/// sequence (a scrub would kill a different set).
#[test]
fn variance_spreads_lifetimes_deterministically_by_id() {
    let spans: Vec<f32> = (0..8).map(|id| life_of(id, 2.0, 0.5, 1)).collect();
    assert!(
        spans.windows(2).any(|w| (w[0] - w[1]).abs() > 0.05),
        "the lifetimes differ: {spans:?}"
    );
    assert!(
        spans.iter().all(|s| (1.0..=3.0).contains(s)),
        "…within life x [1-v, 1+v]: {spans:?}"
    );
    let again: Vec<f32> = (0..8).map(|id| life_of(id, 2.0, 0.5, 1)).collect();
    assert_eq!(
        spans, again,
        "a hash of identity, not a draw: replay-stable"
    );
    // Zero variance is exact — a sim that did not ask for spread pays nothing for it.
    assert_eq!(life_of(3, 2.0, 0.0, 1), 2.0);
}

/// A lifetime that rounded away to nothing would be born and killed on the same tick — a
/// flicker, and impossible to debug. The variance is floored.
#[test]
fn a_lifetime_never_rounds_away_to_nothing() {
    for id in 0..64 {
        assert!(life_of(id, 2.0, 1.0, 7) >= 2.0 * MIN_LIFE_FRAC);
    }
}

/// **No age = no death.** Outside a zone nothing grows an `age` (only `sim.step` does), so this
/// node is inert rather than lethal: a lifetime with no simulation behind it kills nothing,
/// because there is no life to run out.
#[test]
fn without_an_age_column_nothing_dies() {
    let plain = Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0]; 3]));
    let out = reap(&plain, 0.001, 0.0, 1);
    assert_eq!(out.count(), 3, "no age, no death");
    assert_eq!(col(&out, "life"), vec![0.0; 3], "…and nobody has aged");
}

/// The survivors keep their ORDER (and every other column with them). Reshuffling the set each
/// tick would make every index-based node downstream — a ramp across the set, an index field —
/// flicker for no reason at all.
#[test]
fn the_survivors_keep_their_order_and_their_columns() {
    let s = aged(&[0.1, 9.0, 0.2, 9.0, 0.3]).with(
        "tint",
        Column::Vec4(vec![
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ]),
    );
    let out = reap(&s, 1.0, 0.0, 1);
    assert_eq!(col(&out, "id"), vec![0.0, 2.0, 4.0], "in their own order");
    match out.get("tint") {
        Some(Column::Vec4(v)) => {
            assert_eq!(v[1], [0.0, 1.0, 0.0, 1.0], "the columns came with them")
        }
        _ => panic!("the tint survived"),
    }
}
