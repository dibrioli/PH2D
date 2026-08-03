//! Gates for `source.shape` (ADR-0154). The cross-side single-door gate (the
//! node's `ctx.param` key == the shell's graph-reader key) lives in the shell,
//! which has both readers; here we pin the KEY function and the kind decode.

use super::*;

/// Every kind index the dropdown offers decodes to a distinct kind, and the label
/// list is index-aligned to the enum (the `Enum` widget stores the index).
/// FALSIFIED by a missing arm or a mis-ordered label.
#[test]
fn kind_index_round_trips_and_labels_align() {
    assert_eq!(KIND_LABELS.len(), 8, "eight fillable shape families");
    let mut seen = std::collections::BTreeSet::new();
    for i in 0..KIND_LABELS.len() {
        let k = ShapeKind::from_index(i as f32);
        assert!(seen.insert(k), "index {i} decodes to a DISTINCT kind");
    }
    // The index is rounded and clamped: 4.4 → Polygon (4), and out-of-range
    // saturates to the last kind rather than panicking.
    assert_eq!(ShapeKind::from_index(4.4), ShapeKind::Polygon);
    assert_eq!(ShapeKind::from_index(-3.0), ShapeKind::Circle);
    assert_eq!(ShapeKind::from_index(999.0), ShapeKind::Gear);
}

/// The content key is a pure function of the descriptor: identical params give
/// identical keys (so identical shapes SHARE one `VecPath`), and any single param
/// change gives a different key. `to_bits` makes it exact — no float-format drift.
/// FALSIFIED by a key that ignores a param (two different shapes would collide).
#[test]
fn the_key_is_deterministic_and_separates_every_param() {
    let base = ShapeParams {
        kind: ShapeKind::Star,
        size: 1.0,
        aspect: 1.0,
        sides: 5,
        inner: 0.45,
        corner: 0.0,
        frame: 0.0,
    };
    assert_eq!(shape_key(&base), shape_key(&base), "deterministic");

    // Flip each field in turn — every one must move the key, or two distinct
    // shapes would resolve to the same stored geometry.
    let mutate: &[fn(&mut ShapeParams)] = &[
        |p| p.kind = ShapeKind::Circle,
        |p| p.size = 2.0,
        |p| p.aspect = 1.5,
        |p| p.sides = 6,
        |p| p.inner = 0.5,
        |p| p.corner = 8.0,
        |p| p.frame = 4.0,
    ];
    for (i, m) in mutate.iter().enumerate() {
        let mut v = base;
        m(&mut v);
        assert_ne!(
            shape_key(&v),
            shape_key(&base),
            "param #{i} must change the key"
        );
    }
}

/// `read` over a source that returns the manifest defaults reconstructs the
/// default descriptor — the node's `eval` (reading `ctx.param`, which falls back
/// to the manifest default) sees exactly this. Pins the param NAMES against the
/// manifest so a renamed param can't silently read 0.
#[test]
fn read_over_manifest_defaults_gives_the_default_shape() {
    let default_of = |name: &str| -> f32 {
        MANIFEST
            .params
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.default)
            .unwrap_or_else(|| panic!("param {name} is declared in the manifest"))
    };
    let p = ShapeParams::read(default_of);
    assert_eq!(p.kind, ShapeKind::Circle);
    assert_eq!(p.size, 1.0);
    assert_eq!(p.aspect, 1.0);
    assert_eq!(p.sides, 6);
    assert_eq!(p.inner, 0.45);
    assert_eq!(p.corner, 0.0);
    assert_eq!(p.frame, 0.0);
}

/// Every UI hint names a param the manifest declares (a hint for a non-existent
/// param is a dead row). FALSIFIED by a typo in a hint's `param`.
#[test]
fn every_hint_names_a_declared_param() {
    for hint in PARAM_HINTS {
        assert!(
            MANIFEST.params.iter().any(|s| s.name == hint.param),
            "hint '{}' names a param the manifest declares",
            hint.param
        );
    }
}
