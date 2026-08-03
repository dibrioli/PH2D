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
        corner: 0.0,
        star_depth: 0.45,
        cleft: 0.2,
        tooth_depth: 0.35,
        hole: 0.45,
    };
    assert_eq!(shape_key(&base), shape_key(&base), "deterministic");

    // Flip each field in turn — every one must move the key, or two distinct
    // shapes would resolve to the same stored geometry.
    let mutate: &[fn(&mut ShapeParams)] = &[
        |p| p.kind = ShapeKind::Circle,
        |p| p.size = 2.0,
        |p| p.aspect = 1.5,
        |p| p.sides = 6,
        |p| p.corner = 0.3,
        |p| p.star_depth = 0.6,
        |p| p.cleft = 0.3,
        |p| p.tooth_depth = 0.4,
        |p| p.hole = 0.5,
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
    assert_eq!(p.corner, 0.0);
    assert_eq!(p.star_depth, 0.45);
    assert_eq!(p.cleft, 0.2);
    assert_eq!(p.tooth_depth, 0.35);
    assert_eq!(p.hole, 0.45);
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

/// Every gate names declared params (`param` + `when`) and lists only valid kind
/// indices, and only params PAST `size` are gated (`kind`/`size` are always shown).
/// FALSIFIED by a typo, an out-of-range kind index, or gating a base control.
#[test]
fn every_gate_names_declared_params_and_valid_kinds() {
    let declared = |name: &str| MANIFEST.params.iter().any(|s| s.name == name);
    for g in PARAM_GATES {
        assert!(declared(g.param), "gate param '{}' is declared", g.param);
        assert!(declared(g.when), "gate when '{}' is declared", g.when);
        assert!(
            g.param != param::KIND && g.param != param::SIZE,
            "kind/size are always shown — not gated"
        );
        assert!(!g.values.is_empty(), "a gate with no values hides forever");
        for &v in g.values {
            assert!(
                (0..KIND_LABELS.len() as i32).contains(&v),
                "gate value {v} is a valid kind index"
            );
        }
    }
}
