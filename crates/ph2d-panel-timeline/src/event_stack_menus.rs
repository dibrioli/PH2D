//! **Os dois menus do botão direito da PILHA de clips** — o da lane (ADR-0115 B5) e o da strip
//! (B6).
//!
//! Cortados de `event.rs` por assunto, pelo tecto de 600 linhas por ficheiro de painel: os dois
//! fazem a mesma dança que o irmão [`crate::event_track_menu`] documenta — ler a requisição
//! PARQUEADA, confirmar que o alvo ainda existe no snapshot, gastar a requisição.

use crate::state::{self, TimelinePanelState};
use ph2d_editor_core::ids::{
    CTX_MENU_TL_LANE_ADDITIVE, CTX_MENU_TL_LANE_DELETE, CTX_MENU_TL_LANE_RENAME,
    CTX_MENU_TL_STRIP_DELETE, CTX_MENU_TL_STRIP_DUPLICATE, CTX_MENU_TL_STRIP_ENTER,
    CTX_MENU_TL_STRIP_LOOP, CTX_MENU_TL_STRIP_PINGPONG, CTX_MENU_TL_STRIP_RESET_SPEED,
    TIMELINE_LANE_MENU, TIMELINE_STRIP_MENU,
};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_timeline::TimelineIntent;

/// The lane's right-click menu (ADR-0115 B5): how it blends, and whether it stays.
///
/// Same contract as `strip_menu_click` — read the PARKED request, confirm the lane
/// still exists, spend the request. `Delete Lane` lives here rather than on the row
/// because the row has no width for a third button; that is a layout fact, not a
/// judgement about how often a lane gets deleted.
pub(crate) fn lane_menu_click(
    state: &mut TimelinePanelState,
    id: ph2d_editor_core::NodeId,
    host: &mut dyn PanelHostInternal,
) -> Option<EventOutcome> {
    use ph2d_editor_core::interaction::ContextMenuKind;
    use ph2d_timeline::LaneMode;

    if !TIMELINE_LANE_MENU.iter().any(|(r, _, _)| *r == id) {
        return None;
    }
    let req = host
        .store()
        .context_menu()
        .or_else(|| host.store().last_context_menu());
    let Some(ContextMenuKind::TimelineLane { lane }) = req.map(|r| r.kind) else {
        return Some(EventOutcome::Consumed);
    };
    if lane < crate::state::current_snapshot().lanes.len() {
        if id == CTX_MENU_TL_LANE_RENAME {
            // Rename opens the field instead of pushing an intent — the animator
            // types, and Enter commits `RenameLane` (`clip_rename::commit`). It is the
            // one menu item that is a gesture START, not a one-shot.
            crate::clip_rename::open_lane(state, lane);
        } else if id == CTX_MENU_TL_LANE_DELETE {
            state::push_intent(TimelineIntent::RemoveLane { lane });
        } else {
            // The two modes. `Additive` is named explicitly and `Override` is the
            // fallback, so a row added to the table without an arm here lands on
            // Override — which the seam test refuses to let pass silently.
            let mode = if id == CTX_MENU_TL_LANE_ADDITIVE {
                LaneMode::Additive
            } else {
                LaneMode::Override
            };
            state::push_intent(TimelineIntent::SetLaneMode { lane, mode });
        }
    }
    host.store_mut().close_context_menu();
    host.store_mut().consume_last_context_menu();
    Some(EventOutcome::Consumed)
}

/// The strip's right-click menu (ADR-0115 B6). `None` means "not one of ours".
///
/// Same two gotchas the track menu documents, for the same reasons: the Down that
/// preceded this Click already CLOSED the menu (so read
/// `context_menu().or_else(last_context_menu())` — reading only the open one
/// ships a menu that does nothing), and the request is CONSUMED after it lands
/// (so a later stray Click on the id cannot duplicate the strip a second time).
///
/// The request names the strip by its stable id, and the snapshot is asked to
/// confirm it still exists: a strip deleted between the menu opening and the row
/// being clicked resolves to nothing. The action expires with its target.
pub(crate) fn strip_menu_click(
    state: &mut TimelinePanelState,
    id: ph2d_editor_core::NodeId,
    host: &mut dyn PanelHostInternal,
) -> Option<EventOutcome> {
    use ph2d_editor_core::interaction::ContextMenuKind;
    use ph2d_timeline::{StripId, StripLoop};

    if !TIMELINE_STRIP_MENU.iter().any(|(r, _, _)| *r == id) {
        return None;
    }
    let req = host
        .store()
        .context_menu()
        .or_else(|| host.store().last_context_menu());
    let Some(ContextMenuKind::TimelineStrip { lane, strip }) = req.map(|r| r.kind) else {
        return Some(EventOutcome::Consumed);
    };
    let id_ = StripId(strip);
    let snap = crate::state::current_snapshot();
    let live = snap
        .lanes
        .get(lane)
        .is_some_and(|l| l.strips.iter().any(|s| s.id == id_));
    // **Entering is not an edit**, so it does not raise an intent: it moves the panel's own
    // view state, and the shell reads it before the next drain. It is also the one row that
    // means nothing on a clip strip — `container` is `None` there, and the row goes inert
    // rather than acting on the wrong thing.
    if id == CTX_MENU_TL_STRIP_ENTER {
        if let Some(c) = snap
            .lanes
            .get(lane)
            .and_then(|l| l.strips.iter().find(|s| s.id == id_))
            .and_then(|s| s.container)
        {
            // The step remembers the STRIP, not just the container: the instance the
            // animator means is the one under this very click, and it is what keeps the
            // interior's ruler mapped (and scrubb-able) at every playhead time
            // (`ph2d_timeline::entry_map`).
            state::enter_container(ph2d_timeline::EnterStep {
                container: c,
                lane,
                strip: Some(id_),
            });
            // A container's interior is the **Containers** tab's half — Arrange is always
            // the scene (`tab::Tab::scene_root`). Entering is therefore a change of TAB, and
            // walking in without it would leave the animator on a tab that has just stopped
            // publishing the trail: the click would do nothing visible at all.
            crate::state::set_tab(state, crate::tab::Tab::Containers);
        }
        host.store_mut().close_context_menu();
        host.store_mut().consume_last_context_menu();
        return Some(EventOutcome::Consumed);
    }
    if live {
        let intent = if id == CTX_MENU_TL_STRIP_DUPLICATE {
            TimelineIntent::DuplicateStrip { lane, id: id_ }
        } else if id == CTX_MENU_TL_STRIP_DELETE {
            TimelineIntent::RemoveStrip { lane, id: id_ }
        } else if id == CTX_MENU_TL_STRIP_RESET_SPEED {
            TimelineIntent::SetStripSpeed {
                lane,
                id: id_,
                speed: 1.0,
            }
        } else {
            // The three source modes. Exhaustive over what remains of the table —
            // and if a row is ever added without landing here, `strip_menu_click`
            // would silently set Once, so the seam test proves each row raises the
            // intent it names rather than merely raising SOMETHING.
            let loop_mode = if id == CTX_MENU_TL_STRIP_LOOP {
                StripLoop::Loop
            } else if id == CTX_MENU_TL_STRIP_PINGPONG {
                StripLoop::PingPong
            } else {
                StripLoop::Once
            };
            TimelineIntent::SetStripLoop {
                lane,
                id: id_,
                loop_mode,
            }
        };
        state::push_intent(intent);
    }
    host.store_mut().close_context_menu();
    host.store_mut().consume_last_context_menu();
    Some(EventOutcome::Consumed)
}
