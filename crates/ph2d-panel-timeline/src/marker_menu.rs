//! **O menu do botão direito num marker** (ADR-0143), e só ele.
//!
//! Irmão do `event_track_menu`, e a diferença é o `state`: as duas linhas de EDIÇÃO
//! (Rename / Set Signal) abrem o campo inline, que é estado do PAINEL
//! (`TimelinePanelState.marker_rename`) — não um intent —, então este router recebe
//! `state`, enquanto o do track resolve tudo por intent. A 3ª linha (Delete) é um
//! intent puro, como o Delete Track.
//!
//! ⚠️ **A requisição já está FECHADA quando o Click chega.** O Down que precedeu este
//! Click fechou o menu e parqueou o pedido, então lê-se
//! `context_menu().or_else(last_context_menu())` — ler só o aberto ship um menu que não
//! faz nada (o mesmo gotcha do menu de presets e do track).
//!
//! ⚠️ Nem Rename nem Set Signal abrem um bracket `BeginEdit`/`EndEdit`: o Secondary NÃO
//! captura o ponteiro (não houve `Begin`), e o rename commita depois como seu próprio passo
//! (`RenameMarker`/`SetMarkerSignal`). Um índice que sumiu do documento entre o menu abrir e
//! o Click chegar **expira em silêncio** — o `marker_rename::paint` abandona um índice morto,
//! e `remove_marker` devolve `false` sem tocar nada (nunca acerta outro marker).

use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_timeline::TimelineIntent;

use crate::ids;
use crate::state::{self, MarkerRename, TimelinePanelState};

/// O índice de marker que a requisição de menu parqueada nomeia, se houver uma.
fn marker_index(host: &dyn PanelHostInternal) -> Option<usize> {
    let req = host
        .store()
        .context_menu()
        .or_else(|| host.store().last_context_menu())?;
    match req.kind {
        ContextMenuKind::TimelineMarker { index } => Some(index),
        _ => None,
    }
}

/// Arma o campo inline para o marker parqueado — em modo LABEL ou SINAL. O `paint`
/// do próximo frame semeia + foca, e abandona sozinho se o marker sumiu.
fn open_editor(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    editing_signal: bool,
) -> EventOutcome {
    if let Some(index) = marker_index(host) {
        state.marker_rename = Some(MarkerRename {
            index,
            opened: false,
            editing_signal,
        });
        host.store_mut().close_context_menu();
        // Gasta: um Click perdido neste id depois não pode reabrir o editor.
        host.store_mut().consume_last_context_menu();
    }
    EventOutcome::Consumed
}

/// **Delete Marker** — um intent atômico, como o Unbind do Delete Track (sem bracket).
fn delete_marker(host: &mut dyn PanelHostInternal) -> EventOutcome {
    if let Some(index) = marker_index(host) {
        state::push_intent(TimelineIntent::RemoveMarker { index });
        host.store_mut().close_context_menu();
        host.store_mut().consume_last_context_menu();
    }
    EventOutcome::Consumed
}

/// Encaminha o Click, se ele for de uma linha deste menu. `None` = não é comigo.
pub(crate) fn route(
    state: &mut TimelinePanelState,
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
) -> Option<EventOutcome> {
    if id == ids::CTX_MENU_TL_RENAME_MARKER {
        return Some(open_editor(state, host, false));
    }
    if id == ids::CTX_MENU_TL_SET_SIGNAL {
        return Some(open_editor(state, host, true));
    }
    if id == ids::CTX_MENU_TL_DELETE_MARKER {
        return Some(delete_marker(host));
    }
    None
}
