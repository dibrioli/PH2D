// ph2d-chrome-sync:z=290 (dispatch priority, ADR-0107; lower = earlier)
//! Timeline **segment preset** menu (W3.E4) — the interpolation leaving a key,
//! chosen by right-clicking its dope-sheet diamond or its graph anchor (the
//! shell's dispatch raises `ContextMenuKind::TimelineSegment`).
//!
//! Three cascade rows (`Ease In/Out/In-Out \u{25b6}`) REPLACE this menu with the
//! easing-family submenu, carrying the segment plus the mode they stand for —
//! the same replace-the-parent cascade `settings_ppm.rs` uses. Every other row
//! is a leaf: it parks a [`TimelineInterpPick`] on the hero for the shell to
//! drain into a `TimelineIntent::SetInterp`.
//!
//! editor-core cannot depend on `ph2d-anim`, so what crosses is the clicked row's
//! id plus the mode wire byte — never an `Interp`. The shell owns that mapping
//! (and a gate there proves every row in both tables resolves to one).

use crate::ids;
use crate::interaction::{
    ContextMenuKind, ContextMenuRequest, TL_NO_EASE_MODE, TimelineInterpPick, WidgetEvent,
};
use crate::screens::hero::HeroScreen;

/// The three cascade rows, paired with the mode each opens.
const CASCADES: [(ph2d_a11y::NodeId, u8); 3] = [
    (ids::CTX_MENU_TL_EASE_IN, ids::TL_EASE_MODE_IN),
    (ids::CTX_MENU_TL_EASE_OUT, ids::TL_EASE_MODE_OUT),
    (ids::CTX_MENU_TL_EASE_INOUT, ids::TL_EASE_MODE_INOUT),
];

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    // Which segment was open? The Down that precedes this Click already CLOSED
    // the menu (`pointer_down.rs`), parking the request in `last_context_menu` —
    // reading only `context_menu()` here is how a menu ships doing nothing.
    // Read before acting: opening the submenu overwrites the request.
    let req = hero
        .store
        .context_menu()
        .or_else(|| hero.store.last_context_menu());
    let (target, key, open_mode) = match req.map(|r| r.kind) {
        Some(ContextMenuKind::TimelineSegment { target, key }) => (target, key, TL_NO_EASE_MODE),
        Some(ContextMenuKind::TimelineSegmentEase { target, key, mode }) => (target, key, mode),
        _ => return false,
    };

    // Cascade: swap in the family submenu for this mode, anchored to the row.
    if let Some(&(_, mode)) = CASCADES.iter().find(|(row, _)| *row == id) {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::TimelineSegmentEase { target, key, mode },
        });
        return true;
    }

    // Leaf rows. A family row is only meaningful under a mode — a stray click on
    // one from the top-level menu (impossible today, but the tables are public)
    // must not park a pick the shell cannot resolve.
    let is_leaf = id == ids::CTX_MENU_TL_HOLD
        || id == ids::CTX_MENU_TL_LINEAR
        || id == ids::CTX_MENU_TL_CUSTOM;
    let is_family = ids::TIMELINE_EASE_MENU.iter().any(|(fam, _, _)| *fam == id);
    if !(is_leaf || (is_family && open_mode != TL_NO_EASE_MODE)) {
        return false;
    }
    hero.pending_timeline_interp = Some(TimelineInterpPick {
        target,
        key,
        item: id,
        mode: open_mode,
    });
    hero.store.close_context_menu();
    // Drop the parked request: it is spent, and leaving it would let a later
    // stray Click on one of these ids re-apply the preset to the same key.
    hero.store.consume_last_context_menu();
    true
}
