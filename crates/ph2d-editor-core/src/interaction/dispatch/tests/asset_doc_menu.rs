//! D17 — **o botão direito num cartão de DOCUMENTO (Kanban, Quadro) abre o menu do documento**, e
//! num prefab continua a abrir o do prefab. Antes os dois abriam o do prefab: seis itens mortos num
//! documento e «Remove from Library» a apagá-lo. Aqui, o KIND que o despacho abre; as linhas de cada
//! kind são do `screens::hero::menu_rows` (`menu_rows_tests.rs`).

use super::*;
use crate::interaction::ContextMenuKind;

const DOC: NodeId = NodeId(7_101);
const PREFAB: NodeId = NodeId(7_102);

/// Dois cartões pintados, o primeiro um documento.
fn two_cards() -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(DOC, InteractiveState::Plain);
    store.register(PREFAB, InteractiveState::Plain);
    store.set_asset_cells([(DOC, 0), (PREFAB, 1)].into_iter().collect());
    store.set_asset_doc_cells([DOC].into_iter().collect());
    let mut hits = HitIndex::new();
    hits.register(DOC, Rect::new(0.0, 0.0, 40.0, 40.0));
    hits.register(PREFAB, Rect::new(100.0, 0.0, 40.0, 40.0));
    (store, hits)
}

fn right_click(store: &mut WidgetStore, hits: &HitIndex, x: f32) -> Option<ContextMenuKind> {
    let arena = Bump::new();
    let _ = dispatch_pointer(
        store,
        hits,
        PointerEvent {
            button: ph2d_host::PointerButton::Secondary,
            ..pointer(PointerKind::Down, x, 5.0)
        },
        &arena,
    );
    store.context_menu().map(|r| r.kind)
}

/// **Mutação que deve sangrar:** abrir sempre `AssetCard` no `pointer_down_menus`.
#[test]
fn right_click_on_a_document_card_opens_the_document_menu_with_open_and_delete() {
    let (mut store, hits) = two_cards();
    let kind = right_click(&mut store, &hits, 5.0);
    assert_eq!(kind, Some(ContextMenuKind::AssetDocCard { cell: DOC }));
}

/// Controle: o prefab continua com o menu dele, intacto.
#[test]
fn right_click_on_a_prefab_card_still_opens_the_prefab_menu() {
    let (mut store, hits) = two_cards();
    let kind = right_click(&mut store, &hits, 105.0);
    assert_eq!(kind, Some(ContextMenuKind::AssetCard { cell: PREFAB }));
}

/// O censo de cartões manda: painel fechado (cartões vazios) ⇒ nenhum documento sobra.
#[test]
fn a_document_cell_is_no_longer_one_when_the_cards_are_gone() {
    let (mut store, _) = two_cards();
    assert!(store.is_asset_doc_cell(DOC));
    assert!(!store.is_asset_doc_cell(PREFAB));
    store.set_asset_cells(std::collections::BTreeMap::new());
    assert!(!store.is_asset_doc_cell(DOC));
}
