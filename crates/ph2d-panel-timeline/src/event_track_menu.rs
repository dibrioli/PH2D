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

use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_timeline::{AnimTarget, Extrap, ExtrapSide, TimelineIntent};

use crate::ids;

/// The parked menu request (open or just-closed). The Down that precedes a menu
/// Click already CLOSED the menu, parking the request in `last_context_menu` —
/// reading only `context_menu()` here is how a menu ships doing nothing (the same
/// gotcha the segment cascade documents).
fn parked(host: &dyn PanelHostInternal) -> Option<ContextMenuRequest> {
    host.store()
        .context_menu()
        .or_else(|| host.store().last_context_menu())
}

/// A entidade (e a prop) que a requisição de menu parqueada nomeia, se ela ainda
/// existir no snapshot. `want_path` escolhe QUAL das duas variantes é aceita — o menu
/// de uma track de trajetória e o de uma track comum são tabelas diferentes, e uma
/// linha de um não pode ser honrada a partir da requisição do outro.
fn target_of(
    host: &dyn PanelHostInternal,
    want_path: bool,
) -> Option<(u64, ph2d_timeline::PropKind)> {
    let target = match (parked(host)?.kind, want_path) {
        (ContextMenuKind::TimelineTrackPath { target }, true)
        | (ContextMenuKind::TimelineTrack { target }, false)
        // Uma row de EIXO é "não-trajetória" para efeito de qual menu a pediu: o
        // `Delete Track` dela e o `Convert to Motion Path` vêm da MESMA requisição.
        | (ContextMenuKind::TimelineTrackAxis { target }, false)
        // Time Remap só tem o Delete Track (want_path é falso).
        | (ContextMenuKind::TimelineTrackTimeRemap { target }, false) => target,
        _ => return None,
    };
    crate::state::current_snapshot()
        .tracks
        .iter()
        .find(|t| t.target.get() == target)
        .map(|t| (t.entity, t.prop))
}

/// The RAW `AnimTarget` of the parked track menu — any of the four track menus —
/// if the row still exists in the snapshot. The extrapolation rows carry the raw
/// target through to `SetTrackExtrap`, which speaks `AnimTarget` directly (unlike
/// Delete Track, which resolves to `(entity, prop)` for `Unbind`).
fn raw_track_target(host: &dyn PanelHostInternal) -> Option<u64> {
    let target = match parked(host)?.kind {
        ContextMenuKind::TimelineTrack { target }
        | ContextMenuKind::TimelineTrackAxis { target }
        | ContextMenuKind::TimelineTrackPath { target } => target,
        // Time Remap deliberately excluded: its menu has no extrapolation cascade,
        // so this is never reached from there — but pinning it keeps the exclusion
        // honest if a future edit adds the cascade to that table by mistake.
        _ => return None,
    };
    crate::state::current_snapshot()
        .tracks
        .iter()
        .any(|t| t.target.get() == target)
        .then_some(target)
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

/// **Convert to Motion Path / to Separate Axes** (ADR-0141 §5) — a troca de MODO, que
/// só faz sentido na família de track que tem o outro modo para ir.
pub(crate) fn convert(host: &mut dyn PanelHostInternal, to_path: bool) -> EventOutcome {
    // `want_path` é o inverso: quem vai PARA trajetória está numa row de EIXO.
    if let Some((entity, _)) = target_of(host, !to_path) {
        crate::state::push_intent(TimelineIntent::ConvertPositionMode { entity, to_path });
        host.store_mut().close_context_menu();
        host.store_mut().consume_last_context_menu();
    }
    EventOutcome::Consumed
}

/// **Extrapolation cascade** — a Pre/Post row REPLACES the track menu with the
/// four-mode submenu (plan §6), carrying the row's raw target + the side. Same
/// replace-the-parent cascade the segment ease menu uses; the parent is already
/// closed (the Down parked it), so the submenu opens at its position.
fn open_extrap(host: &mut dyn PanelHostInternal, side_wire: u8) -> EventOutcome {
    if let (Some(target), Some(req)) = (raw_track_target(host), parked(host)) {
        host.store_mut().open_context_menu(ContextMenuRequest {
            x: req.x,
            y: req.y,
            kind: ContextMenuKind::TimelineExtrap {
                target,
                side: side_wire,
            },
        });
    }
    EventOutcome::Consumed
}

/// **Extrapolation mode leaf** — resolve the parked `TimelineExtrap { target, side }`
/// and raise `SetTrackExtrap`. One undo step; a key edit (never a strip intent),
/// so the fade surface is untouched.
fn set_extrap(host: &mut dyn PanelHostInternal, mode: Extrap) -> EventOutcome {
    if let Some(ContextMenuKind::TimelineExtrap { target, side }) = parked(host).map(|r| r.kind) {
        let side = if side == ids::TL_EXTRAP_SIDE_PRE {
            ExtrapSide::Pre
        } else {
            ExtrapSide::Post
        };
        crate::state::push_intent(TimelineIntent::SetTrackExtrap {
            target: AnimTarget::new(target),
            side,
            mode,
        });
        host.store_mut().close_context_menu();
        host.store_mut().consume_last_context_menu();
    }
    EventOutcome::Consumed
}

/// **Expression\u{2026}** — abre o campo de fórmula inline (ADR-0144) na posição do
/// clique, para a track parqueada. Mora aqui (e não no `route`) porque abrir o campo
/// mexe no `TimelinePanelState`, que o `route(host, id)` não recebe — o `apply_event`,
/// que tem o state, chama isto ANTES do `route`.
pub(crate) fn open_expr(
    state: &mut crate::state::TimelinePanelState,
    host: &mut dyn PanelHostInternal,
) -> EventOutcome {
    if let (Some(target), Some(req)) = (raw_track_target(host), parked(host)) {
        crate::expr_edit::open(state, target, req.x, req.y);
        host.store_mut().close_context_menu();
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
    if id == ids::CTX_MENU_TL_TO_PATH {
        return Some(convert(host, true));
    }
    if id == ids::CTX_MENU_TL_TO_AXES {
        return Some(convert(host, false));
    }
    // Extrapolation: the two cascade rows open the submenu; the four leaves set it.
    if id == ids::CTX_MENU_TL_EXTRAP_PRE {
        return Some(open_extrap(host, ids::TL_EXTRAP_SIDE_PRE));
    }
    if id == ids::CTX_MENU_TL_EXTRAP_POST {
        return Some(open_extrap(host, ids::TL_EXTRAP_SIDE_POST));
    }
    if id == ids::CTX_MENU_TL_EXTRAP_HOLD {
        return Some(set_extrap(host, Extrap::Hold));
    }
    if id == ids::CTX_MENU_TL_EXTRAP_LOOP {
        return Some(set_extrap(host, Extrap::Loop));
    }
    if id == ids::CTX_MENU_TL_EXTRAP_PINGPONG {
        return Some(set_extrap(host, Extrap::PingPong));
    }
    if id == ids::CTX_MENU_TL_EXTRAP_CONTINUE {
        return Some(set_extrap(host, Extrap::Continue));
    }
    None
}
