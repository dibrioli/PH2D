//! A geometria de uma fila de abas e o salto de id.
//!
//! ⚠️ **As leis que precisam do REGISTRY de painéis não moram aqui** — nesta crate o
//! `test_support::ensure_panel_registry` é um `{}`, e uma varredura sobre zero painéis passa sobre
//! nada. Elas vivem em `ph2d-panel-registry-init/tests/`.

use super::*;

fn bar() -> Rect {
    Rect::new(1062.0, 28.0, 304.0, TAB_BAR_H)
}

#[test]
fn the_tabs_fill_the_row_and_never_overlap() {
    for n in 1..=4 {
        let rects = tab_rects(bar(), n);
        assert_eq!(rects.len(), n, "n={n}");
        for w in rects.windows(2) {
            assert!(
                w[0].x + w[0].w <= w[1].x + 0.001,
                "n={n}: duas abas partilham pixel ({:?} / {:?})",
                w[0],
                w[1]
            );
        }
        let last = rects.last().unwrap();
        assert!(
            last.x + last.w <= bar().x + bar().w + 0.001,
            "n={n}: a última aba sai da faixa ({last:?})"
        );
    }
}

/// ⭐ O piso é de LEGIBILIDADE: acima dele as abas encolhem; abaixo, **transbordam**.
#[test]
fn a_row_that_cannot_show_a_label_overflows_instead_of_shrinking() {
    let many = tab_rects(bar(), 40);
    assert!(
        many.len() < 40,
        "40 abas em 304 px couberam todas — cada uma teria {} px",
        bar().w / 40.0
    );
    assert!(!many.is_empty(), "o transbordo comeu a fila inteira");
    for r in &many {
        assert!(
            r.w >= MIN_TAB_W - 0.001,
            "uma aba do transbordo ficou abaixo do piso: {r:?}"
        );
    }
}

#[test]
fn an_empty_row_has_no_tabs() {
    assert!(tab_rects(bar(), 0).is_empty());
    assert!(tab_rects(Rect::new(0.0, 0.0, 0.0, 0.0), 3).is_empty());
}

/// ⛔ **O controlo de que o salto de id é uma BIJECÇÃO** — ele não pode criar colisões que o
/// espaço de ids de painel já não tivesse, e não pode devolver o próprio id do painel.
#[test]
fn the_tab_id_is_a_bijection_and_never_the_panels_own_id() {
    let ids = [NodeId(1), NodeId(2), NodeId(0xdead_beef), NodeId(u64::MAX)];
    for a in ids {
        assert_ne!(tab_node_id(a), a, "a aba herdou o id do painel ({a:?})");
        assert_eq!(tab_node_id(tab_node_id(a)), a, "o salto não é involutivo");
        for b in ids {
            if a != b {
                assert_ne!(
                    tab_node_id(a),
                    tab_node_id(b),
                    "dois painéis distintos deram a mesma aba"
                );
            }
        }
    }
}
