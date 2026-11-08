//! ⭐ **O duplo clique numa lista suspensa CHEGA ao painel** (GEF02 fecho 07/11). O clique simples só abre/fecha
//! (sem evento); o duplo emitia nada e o «duplo clique no Layer renomeia» do Inspector só existia nos testes que
//! injetavam o evento. Mutação: tirar o `push` do braço `Dropdown` de `apply_click`.

use super::*;

fn clique(store: &mut WidgetStore, hits: &HitIndex, t: u128) -> Vec<WidgetEvent> {
    let arena = Bump::new();
    let mut out = Vec::new();
    for kind in [PointerKind::Down, PointerKind::Up] {
        let mut e = pointer(kind, 50.0, 15.0);
        e.timestamp_ns = t;
        out.extend(dispatch_pointer(store, hits, e, &arena).iter().cloned());
    }
    out
}

#[test]
fn o_duplo_clique_numa_lista_suspensa_chega_ao_painel_e_a_deixa_fechada() {
    let dd = NodeId(1);
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        dd,
        InteractiveState::Dropdown {
            state: crate::widget::DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );
    let mut hits = HitIndex::new();
    hits.register(dd, Rect::new(0.0, 0.0, 100.0, 30.0));
    let um = clique(&mut store, &hits, 1_000_000_000);
    assert!(
        !um.contains(&WidgetEvent::DoubleClick(dd)),
        "um clique não é duplo: {um:?}"
    );
    let dois = clique(&mut store, &hits, 1_050_000_000);
    assert!(
        dois.contains(&WidgetEvent::DoubleClick(dd)),
        "o duplo não chegou: {dois:?}"
    );
    assert!(
        matches!(
            store.get(dd),
            Some(InteractiveState::Dropdown { open: false, .. })
        ),
        "o duplo deixa a lista fechada"
    );
    // Um clique muito depois volta a ser simples (abre, sem evento).
    let tres = clique(&mut store, &hits, 9_000_000_000);
    assert!(!tres.contains(&WidgetEvent::DoubleClick(dd)));
}

/// O campo do renomear abre com o nome todo seleccionado: o que se escreve substitui. Mutação: a porta sem o
/// `select_all_in_text_widget` («PropsBoxes»).
#[test]
fn renomear_abre_com_o_nome_todo_seleccionado_e_o_que_se_escreve_substitui() {
    let id = NodeId(7);
    let mut store = WidgetStore::with_capacity(4);
    store.register(id, text_input("Props"));
    assert!(focus_text_field_selected(&mut store, id));
    let arena = Bump::new();
    for ch in "Boxes".chars() {
        let _ = dispatch_text_input(&mut store, ch, &arena);
    }
    assert_eq!(store.text(id), Some("Boxes"));
}
