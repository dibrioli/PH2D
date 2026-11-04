//! D17 — **o menu do cartão de DOCUMENTO** (Kanban, Quadro): «Open» pede a aba (o mesmo que o
//! duplo-clique) e «Delete…» abre a pergunta de apagar. Os verbos de prefab não chegam cá.

use ph2d_asset_index::AssetRef;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids::{CTX_MENU_ASSET_DELETE, CTX_MENU_ASSET_OPEN, asset_cell_id};
use ph2d_editor_core::interaction::drag_payload::DragPayload;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_panel_asset_browser::state::{AssetBrowserState, probe_set_painted};
use ph2d_panel_asset_browser::{AssetBrowserPanel, PANEL_ID};
use ph2d_ui_testkit::MockPanelHost;

/// O painel aberto, `doc` pintado na célula 0 e o menu `kind` já fechado (como o `pointer_down`).
fn staged(doc: AssetRef, kind: ContextMenuKind) -> (MockPanelHost, AssetBrowserState) {
    let mut host = crate::corpo_de_antes();
    host.set_panel_visible(PANEL_ID, true);
    probe_set_painted(vec![doc]);
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind,
    });
    host.store_mut().close_context_menu();
    (host, AssetBrowserState::default())
}

fn doc_menu() -> ContextMenuKind {
    ContextMenuKind::AssetDocCard {
        cell: asset_cell_id(0),
    }
}

/// **Mutação que deve sangrar:** apagar o braço de `CTX_MENU_ASSET_OPEN` no `event.rs`.
#[test]
fn open_on_a_document_card_asks_for_its_tab() {
    for (doc, payload) in [
        (AssetRef::Kanban { id: 5 }, DragPayload::Kanban { id: 5 }),
        (AssetRef::Board { id: 6 }, DragPayload::Board { id: 6 }),
    ] {
        let (mut host, mut st) = staged(doc, doc_menu());
        let out = host.apply_panel_event::<AssetBrowserPanel>(
            &mut st,
            WidgetEvent::Click(CTX_MENU_ASSET_OPEN),
        );
        assert_eq!(out, EventOutcome::Consumed);
        let drained: Vec<_> = host.bus_mut().drain().collect();
        assert_eq!(drained, vec![EditorAction::OpenAsset(payload)]);
    }
}

/// **Mutação que deve sangrar:** trocar `ConfirmDeleteKanban` por `ConfirmDeleteBoard` no `event.rs`.
#[test]
fn delete_on_a_document_card_opens_the_matching_confirmation() {
    for (doc, want) in [
        (
            AssetRef::Kanban { id: 5 },
            ContextMenuKind::ConfirmDeleteKanban { kanban: 5 },
        ),
        (
            AssetRef::Board { id: 6 },
            ContextMenuKind::ConfirmDeleteBoard { board: 6 },
        ),
    ] {
        let (mut host, mut st) = staged(doc, doc_menu());
        let out = host.apply_panel_event::<AssetBrowserPanel>(
            &mut st,
            WidgetEvent::Click(CTX_MENU_ASSET_DELETE),
        );
        assert_eq!(out, EventOutcome::Consumed);
        assert_eq!(host.store().context_menu().map(|r| r.kind), Some(want));
        assert!(
            host.bus().is_empty(),
            "apagar passa pela pergunta, nunca direto"
        );
    }
}

/// O item do documento só age sobre o menu do documento, e só sobre um documento: um pedido de
/// prefab pendente, ou uma célula que já pinta um prefab, não se apaga por este caminho.
#[test]
fn the_document_verbs_refuse_a_prefab_menu_or_a_prefab_cell() {
    let prefab = AssetRef::Component { stable_id: 77 };
    let card = ContextMenuKind::AssetCard {
        cell: asset_cell_id(0),
    };
    for (doc, kind) in [(prefab, card), (prefab, doc_menu())] {
        for id in [CTX_MENU_ASSET_OPEN, CTX_MENU_ASSET_DELETE] {
            let (mut host, mut st) = staged(doc, kind);
            let out = host.apply_panel_event::<AssetBrowserPanel>(&mut st, WidgetEvent::Click(id));
            assert_eq!(out, EventOutcome::Ignored);
            assert!(host.bus().is_empty());
            assert!(host.store().context_menu().is_none());
        }
    }
}

/// «Remove from Library» voltou a ser só o verbo de receita: num documento (se um pedido velho o
/// trouxesse) vai ao shell como verbo, que o recusa — nunca abre a pergunta de apagar.
#[test]
fn remove_from_library_no_longer_deletes_a_document() {
    let card = ContextMenuKind::AssetCard {
        cell: asset_cell_id(0),
    };
    let (mut host, mut st) = staged(AssetRef::Kanban { id: 5 }, card);
    host.apply_panel_event::<AssetBrowserPanel>(
        &mut st,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_ASSET_REMOVE),
    );
    assert!(
        host.store().context_menu().is_none(),
        "abriu a pergunta de apagar"
    );
}
