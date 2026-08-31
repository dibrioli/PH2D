//! A geometria das abas de layout e o salto de id.

use super::*;

fn bar() -> Rect {
    Rect::new(0.0, 0.0, 1366.0, 28.0)
}

#[test]
fn the_tabs_hug_the_right_edge_and_never_overlap() {
    let mut text = TextSystem::without_system_fonts();
    let tabs = tab_rects(bar(), 260.0, &mut text);
    assert_eq!(tabs.len(), TaskLayout::ALL.len());
    for w in tabs.windows(2) {
        assert!(
            w[0].1.x + w[0].1.w <= w[1].1.x + 0.001,
            "duas abas partilham pixel ({:?} / {:?})",
            w[0].1,
            w[1].1
        );
    }
    let last = tabs.last().unwrap().1;
    assert!(
        last.x + last.w <= bar().x + bar().w + 0.001,
        "a última aba sai da barra ({last:?})"
    );
    assert!(
        tabs[0].1.x > bar().w * 0.5,
        "as abas não estão encostadas à direita ({:?})",
        tabs[0].1
    );
}

/// ⛔ **Uma aba por cima de um menu é um clique que troca de tarefa quando o artista queria abrir o
/// ficheiro.** Sem espaço, elas não se pintam.
#[test]
fn the_tabs_never_sit_on_top_of_a_menu_title() {
    let mut text = TextSystem::without_system_fonts();
    // Uma barra estreita, com os menus a ocupar quase tudo.
    let narrow = Rect::new(0.0, 0.0, 300.0, 28.0);
    assert!(
        tab_rects(narrow, 260.0, &mut text).is_empty(),
        "as abas invadiram os títulos dos menus numa barra estreita"
    );
    // E o controlo: com espaço, elas aparecem.
    assert!(!tab_rects(bar(), 260.0, &mut text).is_empty());
}

/// ⛔ **O salto de id é uma BIJECÇÃO**, e nenhuma aba herda o id de outra coisa.
#[test]
fn the_layout_tab_ids_are_distinct_and_reversible() {
    let mut seen = std::collections::BTreeSet::new();
    for l in TaskLayout::ALL {
        let id = tab_node_id(l);
        assert!(seen.insert(id), "duas abas partilham o id {id:?}");
        assert_eq!(layout_for_tab(id), Some(l));
    }
    assert_eq!(layout_for_tab(NodeId(1)), None);
    // ⚠️ E não colide com o salto das abas de PAINEL, que é o vizinho mais próximo.
    for l in TaskLayout::ALL {
        assert_ne!(
            tab_node_id(l),
            super::super::slot_tabs::tab_node_id(NodeId(1)),
            "uma aba de layout tem o id de uma aba de painel"
        );
    }
}
