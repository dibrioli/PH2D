//! O editor da curva Custom: a Bézier que se publica e o arrasto que vira intents.

use super::*;
use ph2d_timeline::{AnimTarget, Extrap, KeyId, PropKind};

fn chave(id: u64, t: f64, value: f32, interp: Interp, selected: bool) -> KeyView {
    KeyView {
        id: KeyId::new(id),
        t_seconds: t,
        value,
        interp,
        selected,
        roving: false,
    }
}

fn faixa(target: u64, keys: Vec<KeyView>) -> TrackView {
    TrackView {
        target: AnimTarget::new(target),
        prop: PropKind::TranslationX,
        entity: 1,
        missing: false,
        buffer_ghost: None,
        pre: Extrap::Hold,
        post: Extrap::Hold,
        expr: None,
        keys,
    }
}

fn snap(tracks: Vec<TrackView>) -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        tracks,
        fps: 24.0,
        ..Default::default()
    }
}

/// A `Bezier` publica-se como está; a `BezierW` normaliza-se pela variação até à seguinte (é a
/// mesma curva); parada, ou sem seguinte, não tem forma normalizada.
#[test]
fn the_card_editor_shows_the_keys_own_normalized_bezier() {
    let bez = Interp::bezier(0.25, 0.1, 0.75, 1.2);
    let t = faixa(
        9,
        vec![
            chave(1, 0.0, 0.0, bez, false),
            chave(2, 1.0, 5.0, bez, false),
        ],
    );
    assert_eq!(normalizada(&t, 0), Some([0.25, 0.1, 0.75, 1.2]));
    let w = Interp::bezier_w(0.3, 2.0, 0.6, -5.0);
    let t = faixa(
        9,
        vec![chave(1, 0.0, 0.0, w, false), chave(2, 1.0, 10.0, w, false)],
    );
    let n = normalizada(&t, 0).unwrap();
    assert!((n[1] - 0.2).abs() < 1e-6 && (n[3] - 0.5).abs() < 1e-6);
    // O valor ao longo da normalizada é o mesmo que a BezierW dá (é a mesma curva).
    let normal = Interp::bezier(0.3, f64::from(n[1]), 0.6, f64::from(n[3]));
    for u in [0.25, 0.5, 0.75] {
        assert!((normal.value(0.0, 10.0, u) - w.value(0.0, 10.0, u)).abs() < 1e-4);
    }
    let parada = faixa(
        9,
        vec![chave(1, 0.0, 3.0, w, false), chave(2, 1.0, 3.0, w, false)],
    );
    assert_eq!(normalizada(&parada, 0), None);
    assert_eq!(normalizada(&parada, 1), None);
}

/// Um arrasto = um passo de undo; uma chave fora da seleção muda só ela, dentro dela a seleção
/// inteira; uma coluna passa a ser a seleção antes de mudar.
#[test]
fn a_handle_drag_is_one_undo_step_over_the_menus_scope() {
    let lin = Interp::Linear;
    let s = snap(vec![
        faixa(
            9,
            vec![
                chave(1, 0.0, 0.0, lin, false),
                chave(2, 1.0, 5.0, lin, true),
            ],
        ),
        faixa(8, vec![chave(3, 1.0, 0.0, lin, false)]),
    ]);
    let edit = |scope, begin, end| TimelineBezierEdit {
        scope,
        handles: [0.2, 0.0, 0.8, 1.0],
        begin,
        end,
    };
    let interp = Interp::bezier(f64::from(0.2_f32), 0.0, f64::from(0.8_f32), 1.0);
    let so_ela = S::Key { target: 9, key: 1 };
    assert_eq!(
        intents(edit(so_ela, true, true), &s),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::SetInterp {
                target: AnimTarget::new(9),
                key: KeyId::new(1),
                interp,
            },
            TimelineIntent::EndEdit,
        ]
    );
    let na_selecao = S::Key { target: 9, key: 2 };
    assert_eq!(
        intents(edit(na_selecao, false, false), &s),
        vec![TimelineIntent::SetSelectedInterp { interp }]
    );
    let coluna = S::Column {
        t_bits: 1.0_f64.to_bits(),
    };
    let v = intents(edit(coluna, false, false), &s);
    assert_eq!(v.first(), Some(&TimelineIntent::ClearSelection));
    assert_eq!(v.len(), 4, "limpa, junta as duas chaves do instante, muda");
    assert_eq!(
        v.last(),
        Some(&TimelineIntent::SetSelectedInterp { interp })
    );
}

/// O lote do store: o 1.º leva o começo, o último o fim; depois do fim o editor volta a desenhar o
/// que a timeline publica.
#[test]
fn the_store_hands_the_drag_over_in_batches() {
    let mut store = WidgetStore::default();
    let scope = S::Key { target: 9, key: 1 };
    store.set_key_card_bezier(Some([0.1, 0.1, 0.9, 0.9]));
    store.begin_key_card_bez(scope, 1, [0.1, 0.1, 0.9, 0.9]);
    store.move_key_card_bez((0.5, 1.4));
    let a = store.take_key_card_bez_edit().unwrap();
    assert!(a.begin && !a.end);
    assert_eq!(a.handles, [0.1, 0.1, 0.5, 1.4]);
    assert_eq!(store.key_card_bezier(), Some([0.1, 0.1, 0.5, 1.4]));
    assert_eq!(
        store.take_key_card_bez_edit(),
        None,
        "nada de novo, nada a drenar"
    );
    store.end_key_card_bez();
    let b = store.take_key_card_bez_edit().unwrap();
    assert!(!b.begin && b.end);
    assert_eq!(store.key_card_bezier(), Some([0.1, 0.1, 0.9, 0.9]));
    store.end_key_card_bez();
    assert_eq!(
        store.take_key_card_bez_edit(),
        None,
        "largar sem arrasto não faz nada"
    );
}
