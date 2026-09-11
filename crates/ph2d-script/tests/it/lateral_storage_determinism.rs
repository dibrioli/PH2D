//! HR-5 + HR-16 determinism test for the per-entity lateral store.
//!
//! Same inputs must produce byte-identical `snapshot()` output across
//! repeated runs. Alphabetical key order is verified explicitly so
//! a future swap from `BTreeMap` → other backing storage would fail
//! this test loudly.

use ph2d_script::{PodValue, StateTable};

fn populate(s: &StateTable) {
    // Insert in non-alphabetical, non-sorted order on purpose.
    s.set(3, "z", PodValue::Number(30.0));
    s.set(1, "b", PodValue::Number(11.0));
    s.set(2, "a", PodValue::Number(20.0));
    s.set(1, "a", PodValue::Number(10.0));
    s.set(3, "a", PodValue::Bool(true));
    s.set(1, "c", PodValue::String("c1".into()));
}

#[test]
fn snapshot_is_stable_across_repeated_runs() {
    let s1 = StateTable::new();
    populate(&s1);
    let s2 = StateTable::new();
    populate(&s2);
    assert_eq!(s1.snapshot(), s2.snapshot());
}

#[test]
fn snapshot_order_is_sorted_by_key_then_field() {
    let s = StateTable::new();
    populate(&s);
    let snap = s.snapshot();

    let actual_order: Vec<(u64, String)> = snap.iter().map(|(k, f, _)| (*k, f.clone())).collect();
    let expected_order = vec![
        (1, "a".to_string()),
        (1, "b".to_string()),
        (1, "c".to_string()),
        (2, "a".to_string()),
        (3, "a".to_string()),
        (3, "z".to_string()),
    ];
    assert_eq!(actual_order, expected_order);
}

#[test]
fn keys_per_entity_are_alphabetical() {
    let s = StateTable::new();
    s.set(7, "delta", PodValue::Nil);
    s.set(7, "alpha", PodValue::Nil);
    s.set(7, "charlie", PodValue::Nil);
    s.set(7, "bravo", PodValue::Nil);
    let keys = s.keys(7);
    assert_eq!(keys, vec!["alpha", "bravo", "charlie", "delta"]);
}

#[test]
fn empty_entity_has_zero_keys() {
    let s = StateTable::new();
    assert_eq!(s.keys(123).len(), 0);
}
