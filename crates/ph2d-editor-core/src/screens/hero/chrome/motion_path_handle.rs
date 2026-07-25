// ph2d-chrome-sync:z=231 (dispatch priority, ADR-0107; lower = earlier)
//! On-canvas **motion-path anchor** handle-type menu (ADR-0141) — the vector Node trio
//! (Corner / Smooth / Symmetric), opened by a secondary-click on a trajectory anchor (the
//! shell raises `ContextMenuKind::MotionPathAnchor { target, i }`).
//!
//! Unlike `curve_point_handle.rs`, the anchor has no persistent selection to fall back on,
//! so its identity travels WITH the menu: the handler reads it back from `last_context_menu`
//! (the Down that precedes this Click already closed the menu, parking the request there —
//! the `timeline_segment.rs` pattern). The chosen kind crosses to the shell as a wire `u8`
//! (`0 = Corner`, `1 = Smooth`, `2 = Symmetric`) parked with the anchor in
//! `HeroScreen.pending_motion_path_handle`; the shell drains it and calls
//! `TimelineDoc::set_path_tangent_kind` (editor-core can't depend on the timeline crate).

use crate::ids;
use crate::interaction::{ContextMenuKind, WidgetEvent};
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let kind = if id == ids::CTX_MENU_PATH_HANDLE_CORNER {
        0
    } else if id == ids::CTX_MENU_PATH_HANDLE_SMOOTH {
        1
    } else if id == ids::CTX_MENU_PATH_HANDLE_SYMMETRIC {
        2
    } else {
        return false;
    };
    // Which anchor? The request was parked in `last_context_menu` when the Down closed the
    // menu — read it (not just `context_menu`, which is already `None` by now) before acting.
    let Some(ContextMenuKind::MotionPathAnchor { target, i }) = hero
        .store
        .context_menu()
        .or_else(|| hero.store.last_context_menu())
        .map(|r| r.kind)
    else {
        return false;
    };
    hero.pending_motion_path_handle = Some((target, i, kind));
    hero.store.close_context_menu();
    // Spend the parked request: leaving it would let a later stray Click on one of these
    // ids re-convert the same anchor.
    hero.store.consume_last_context_menu();
    true
}
