//! On-canvas **Curve / Free Hand** editor point-handle menu — the five vector-app continuity kinds
//! (Free / Aligned / Symmetric / Vector / Auto), opened by a secondary-click on a control point (the
//! shell raises `ContextMenuKind::CurvePointHandle`).
//!
//! editor-core can't depend on the brush/tool crate, so the chosen kind crosses to the shell as a wire
//! `u8` parked in `HeroScreen.pending_curve_point_handle` (`0 = Free`, `1 = Aligned`, `2 = Vector`,
//! `3 = Auto`, `4 = Symmetric`); the shell drains it and calls `set_curve_handle_kind` on the point.
//! The ids are unique to this menu, so a plain id match suffices (mirror of `falloff_handle.rs`).

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let kind = if id == ids::CTX_MENU_CURVE_HANDLE_FREE {
        0
    } else if id == ids::CTX_MENU_CURVE_HANDLE_ALIGNED {
        1
    } else if id == ids::CTX_MENU_CURVE_HANDLE_VECTOR {
        2
    } else if id == ids::CTX_MENU_CURVE_HANDLE_AUTO {
        3
    } else if id == ids::CTX_MENU_CURVE_HANDLE_SYMMETRIC {
        4
    } else {
        return false;
    };
    hero.pending_curve_point_handle = Some(kind);
    hero.store.close_context_menu();
    true
}
