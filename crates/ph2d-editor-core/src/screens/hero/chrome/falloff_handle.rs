// ph2d-chrome-sync:z=240 (dispatch priority, ADR-0107; lower = earlier)
//! Painter brush Falloff-curve point handle menu — 2 handle types (Vector /
//! Auto), opened by a secondary-click on a control point (the shell raises
//! `ContextMenuKind::FalloffPointHandle`).
//!
//! editor-core can't depend on the brush crate, so the chosen handle crosses to
//! the shell as the `HandleType` wire u8 parked in
//! `HeroScreen.pending_falloff_point_handle` (drained via
//! `take_pending_falloff_point_handle`): `0 = Auto`, `1 = Vector`. The ids are
//! unique to this menu, so a plain id match suffices (mirror of `point_type.rs`).

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let handle = if id == ids::CTX_MENU_FALLOFF_HANDLE_AUTO {
        0
    } else if id == ids::CTX_MENU_FALLOFF_HANDLE_VECTOR {
        1
    } else {
        return false;
    };
    hero.pending_falloff_point_handle = Some(handle);
    hero.store.close_context_menu();
    true
}
