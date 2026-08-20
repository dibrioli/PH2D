//! **Um clique DENTRO de um modal não o fecha.**
//!
//! O caminho do Down fecha qualquer menu aberto antes do foco/clique normal — e tem de fechar,
//! senão clicar no canvas com um menu aberto não o dispensa. A excepção são as superfícies em que
//! *escolher* e *confirmar* são dois cliques dentro de uma pergunta só.
//!
//! ⚠️ **Este é o gate do caminho de PONTEIRO**, e é outro do que o `apply_event` exercita: um teste
//! que só chama `apply_event` vê a escolha chegar ao store e passa **mesmo que o Down tenha fechado
//! o modal um instante antes** — no produto, o artista veria o modal desaparecer ao escolher a
//! resolução, e a folha nunca nasceria. *A pergunta «o modal continua aberto?» só o pipeline
//! responde.*

use super::*;
use crate::interaction::{ContextMenuKind, ContextMenuRequest};

/// Um store com o modal da folha aberto e a primeira resolução hit-indexada.
fn sheet_modal_open() -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(4);
    let (_, size_id) = crate::ids::CTX_MENU_SHEET_SIZES[0];
    store.register(size_id, InteractiveState::Plain);
    store.register(NodeId(9_001), InteractiveState::Plain);
    store.open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::SheetSizeDialog,
    });
    let mut hits = HitIndex::new();
    hits.register(size_id, Rect::new(0.0, 0.0, 40.0, 20.0));
    hits.register(NodeId(9_001), Rect::new(200.0, 200.0, 40.0, 20.0));
    (store, hits)
}

#[test]
fn clicking_a_sheet_resolution_keeps_the_modal_open() {
    let (mut store, hits) = sheet_modal_open();
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 5.0, 5.0),
        &arena,
    );
    assert!(
        matches!(
            store.context_menu().map(|r| r.kind),
            Some(ContextMenuKind::SheetSizeDialog)
        ),
        "escolher a resolucao fechou o modal — o Create deixaria de existir e a folha nunca nasceria"
    );
}

/// **Controle positivo**, e sem ele o teste acima passa por o modal nunca fechar: um clique FORA
/// tem de o dispensar.
#[test]
fn clicking_outside_the_sheet_modal_closes_it() {
    let (mut store, hits) = sheet_modal_open();
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 205.0, 205.0),
        &arena,
    );
    assert!(
        store.context_menu().is_none(),
        "um clique fora do modal tem de o dispensar"
    );
}
