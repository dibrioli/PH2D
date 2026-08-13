//! Gates for `source.shape` (ADR-0154). The cross-side single-door gate (the
//! node's `ctx.param` key == the shell's graph-reader key) lives in the shell,
//! which has both readers; here we pin the KEY function and the kind decode.

use super::*;

/// Every kind index the dropdown offers decodes to a distinct kind, and the label
/// list is index-aligned to the enum (the `Enum` widget stores the index).
/// FALSIFIED by a missing arm or a mis-ordered label.
///
/// ⚠️ **The count is asserted so that growing the catalogue is a DECISION**, and
/// this gate did its job: it fired the moment the list went from eight to
/// forty-three. It was 8 while the node built geometry by hand; it is 43 now that
/// it goes through `ph2d_vec_scene::cook`, which is the fillable half of that
/// crate's 47 (measured — `which_shapes_close` — the other five need a stroke).
#[test]
fn kind_index_round_trips_and_labels_align() {
    assert_eq!(KIND_LABELS.len(), 43, "the fillable catalogue");
    assert_eq!(ALL_KINDS.len(), KIND_LABELS.len(), "a label per kind");
    let mut seen = std::collections::BTreeSet::new();
    for i in 0..KIND_LABELS.len() {
        let k = ShapeKind::from_index(i as f32);
        assert!(seen.insert(k), "index {i} decodes to a DISTINCT kind");
        assert_eq!(k.index(), i, "and the round trip closes");
    }

    // ⚠️ **The wire-format claim, spelled out.** A saved graph stores the INDEX, so
    // these eight positions are file format: moving one renames the shape in every
    // document that already chose it. Appending is the only safe growth, and this
    // is what makes that a rule rather than an intention.
    for (i, want) in [
        (0, ShapeKind::Circle),
        (1, ShapeKind::Square),
        (2, ShapeKind::Ellipse),
        (3, ShapeKind::Rectangle),
        (4, ShapeKind::Polygon),
        (5, ShapeKind::Star),
        (6, ShapeKind::Heart),
        (7, ShapeKind::Gear),
    ] {
        assert_eq!(
            ShapeKind::from_index(i as f32),
            want,
            "o indice {i} e FORMATO DE ARQUIVO e mudou de forma"
        );
    }

    // The index is rounded and clamped: 4.4 → Polygon (4), and out-of-range
    // saturates to the last kind rather than panicking.
    assert_eq!(ShapeKind::from_index(4.4), ShapeKind::Polygon);
    assert_eq!(ShapeKind::from_index(-3.0), ShapeKind::Circle);
    assert_eq!(ShapeKind::from_index(999.0), ShapeKind::IsoPyramid);
}

/// The content key is a pure function of the params: identical params give
/// identical keys (so identical shapes SHARE one `VecPath`), and **any** param
/// change gives a different key. `to_bits` makes it exact — no float-format drift.
/// FALSIFIED by a key that ignores a param (two different shapes would collide).
///
/// ⚠️ **O gate é DERIVADO de [`param::ALL`], como a chave.** Ele enumerava os
/// campos numa lista de mutadores, que é a mesma doença um nível acima: um param
/// acrescentado sem entrar na lista ficaria **não-testado**, e o defeito que isso
/// esconde é o pior de um cache — o controle novo fica inerte DEPOIS da primeira
/// vez, porque a forma antiga volta do cache com a chave velha.
#[test]
fn the_key_is_deterministic_and_separates_every_param() {
    // O default do manifesto, pela mesma rota que o `ctx.param` do nó usa.
    let dflt = |n: &str| {
        MANIFEST
            .params
            .iter()
            .find(|p| p.name == n)
            .map_or(0.0, |p| p.default)
    };
    let base = &dflt;
    assert_eq!(shape_key(base), shape_key(base), "deterministic");

    for name in param::ALL {
        // Cutuca UM param — qualquer valor diferente do default serve, e o
        // `+ 1.0` move os bits de qualquer f32 finito que o manifesto declare.
        let nudged = |n: &str| {
            if n == *name { dflt(n) + 1.0 } else { dflt(n) }
        };
        assert_ne!(
            shape_key(nudged),
            shape_key(base),
            "`{name}` tem de mover a chave, senao duas formas distintas partilham a geometria"
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
