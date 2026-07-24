//! **O menu do botão direito numa track row**, e só ele.
//!
//! Split de `event.rs` sob o teto de 600 LOC (e o de 200 do `apply_event`), e uma
//! unidade por direito próprio: as duas linhas fazem a MESMA dança — ler a requisição
//! que o Down já parqueou, resolver o `AnimTarget` cru contra o snapshot, e levantar um
//! intent. Duas cópias dessa dança divergiriam na próxima linha do menu.
//!
//! ⚠️ **A requisição já está FECHADA quando o Click chega.** O Down que precedeu este
//! Click fechou o menu e parqueou o pedido, então lê-se
//! `context_menu().or_else(last_context_menu())` — ler só o aberto ship um menu que não
//! faz nada (o mesmo gotcha que o menu de presets documenta).
//!
//! ⚠️ E uma row que sumiu do snapshot desde que o menu abriu (deletada no meio)
//! **resolve para nada**: a ação expira com o alvo dela, em vez de acertar outra track.

use ph2d_editor_core::interaction::ContextMenuKind;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_timeline::TimelineIntent;

use crate::ids;

/// A entidade (e a prop) que a requisição de menu parqueada nomeia, se ela ainda
/// existir no snapshot. `want_path` escolhe QUAL das duas variantes é aceita — o menu
/// de uma track de trajetória e o de uma track comum são tabelas diferentes, e uma
/// linha de um não pode ser honrada a partir da requisição do outro.
fn target_of(
    host: &dyn PanelHostInternal,
    want_path: bool,
) -> Option<(u64, ph2d_timeline::PropKind)> {
    let req = host
        .store()
        .context_menu()
        .or_else(|| host.store().last_context_menu())?;
    let target = match (req.kind, want_path) {
        (ContextMenuKind::TimelineTrackPath { target }, true)
        | (ContextMenuKind::TimelineTrack { target }, false) => target,
        _ => return None,
    };
    crate::state::current_snapshot()
        .tracks
        .iter()
        .find(|t| t.target.get() == target)
        .map(|t| (t.entity, t.prop))
}

/// **Auto-Orient** (ADR-0141 §6) — só existe no menu de uma track de TRAJETÓRIA.
pub(crate) fn auto_orient(host: &mut dyn PanelHostInternal) -> EventOutcome {
    if let Some((entity, _)) = target_of(host, true) {
        crate::state::push_intent(TimelineIntent::ToggleAutoOrient { entity });
    }
    EventOutcome::Consumed
}

/// **Delete Track** — binding + keys num passo de undo.
pub(crate) fn delete_track(host: &mut dyn PanelHostInternal) -> EventOutcome {
    if let Some((entity, prop)) = target_of(host, false) {
        crate::state::push_intent(TimelineIntent::Unbind { entity, prop });
        host.store_mut().close_context_menu();
        // Gasta: um Click perdido neste id depois não pode apagar outra vez.
        host.store_mut().consume_last_context_menu();
    }
    EventOutcome::Consumed
}

/// Encaminha o Click, se ele for de uma linha deste menu. `None` = não é comigo.
pub(crate) fn route(
    host: &mut dyn PanelHostInternal,
    id: ph2d_a11y::NodeId,
) -> Option<EventOutcome> {
    if id == ids::CTX_MENU_TL_AUTO_ORIENT {
        return Some(auto_orient(host));
    }
    if id == ids::CTX_MENU_TL_DELETE_TRACK {
        return Some(delete_track(host));
    }
    None
}
