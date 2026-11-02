//! D17 — as LINHAS do menu de um cartão de documento (Kanban, Quadro) e de um prefab nos Assets. O
//! despacho que escolhe o kind é provado em `interaction/dispatch/tests/asset_doc_menu.rs`.

use super::menu_rows;
use crate::interaction::ContextMenuKind;
use ph2d_a11y::NodeId;

const CELL: NodeId = NodeId(7_101);

fn row_ids(kind: ContextMenuKind) -> Vec<NodeId> {
    menu_rows(kind).iter().map(|(id, _, _)| *id).collect()
}

/// **Mutação que deve sangrar:** `AssetDocCard` a devolver as linhas do prefab.
#[test]
fn a_document_card_menu_is_open_and_delete() {
    assert_eq!(
        row_ids(ContextMenuKind::AssetDocCard { cell: CELL }),
        vec![
            crate::ids::CTX_MENU_ASSET_OPEN,
            crate::ids::CTX_MENU_ASSET_DELETE
        ],
        "um documento abre-se ou apaga-se — nada de verbos de prefab"
    );
}

/// Controle: o prefab continua com o menu dele, intacto.
#[test]
fn a_prefab_card_menu_keeps_its_verbs() {
    let rows = row_ids(ContextMenuKind::AssetCard { cell: CELL });
    assert_eq!(rows.first(), Some(&crate::ids::CTX_MENU_ASSET_EDIT));
    assert!(rows.contains(&crate::ids::CTX_MENU_ASSET_REMOVE));
    assert!(!rows.contains(&crate::ids::CTX_MENU_ASSET_OPEN));
    assert!(!rows.contains(&crate::ids::CTX_MENU_ASSET_DELETE));
}
