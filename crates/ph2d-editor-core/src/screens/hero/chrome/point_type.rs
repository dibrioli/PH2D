// ph2d-chrome-sync:z=220 (dispatch priority, ADR-0107; lower = earlier)
//! Vector Direct-Select point-type menu — 4 vertex continuity kinds
//! (Corner / Smooth / Asymmetric / Auto), opened by a secondary-click on a
//! vertex (the shell raises `ContextMenuKind::VectorPointType`).
//!
//! editor-core can't depend on the vector-doc crate, so the chosen kind crosses
//! to the shell as a 0..=3 index parked in `HeroScreen.pending_vector_point_type`
//! (drained via `take_pending_vector_point_type`). The index order is the
//! canonical `VertexKind` mapping the shell expects:
//! `0 = Corner (Free)`, `1 = Smooth (Mirror)`, `2 = Asymmetric (Aligned)`,
//! `3 = Auto`. The ids are unique to this menu, so a plain id match suffices
//! (mirror of `theme.rs`).

use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    let index = if id == ids::CTX_MENU_POINT_TYPE_CORNER {
        0
    } else if id == ids::CTX_MENU_POINT_TYPE_SMOOTH {
        1
    } else if id == ids::CTX_MENU_POINT_TYPE_ASYMMETRIC {
        2
    } else if id == ids::CTX_MENU_POINT_TYPE_AUTO {
        3
    } else {
        return false;
    };
    hero.pending_vector_point_type = Some(index);
    hero.store.close_context_menu();
    true
}
