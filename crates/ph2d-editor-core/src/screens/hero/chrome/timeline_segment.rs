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
    let (scope, open_mode) = match req.map(|r| r.kind) {
        Some(ContextMenuKind::TimelineSegment { scope }) => (scope, TL_NO_EASE_MODE),
        Some(ContextMenuKind::TimelineSegmentEase { scope, mode }) => (scope, mode),
        _ => return false,
    };

    // Cascade: swap in the family submenu for this mode, anchored to the row.
    if let Some(&(_, mode)) = CASCADES.iter().find(|(row, _)| *row == id) {
        let (x, y) = super::cascade_anchor(hero, id);
        hero.store.open_context_menu(ContextMenuRequest {
            x,
            y,
            kind: ContextMenuKind::TimelineSegmentEase { scope, mode },
        });
        return true;
    }

    // A top-level leaf is any row of the segment menu that is NOT one of the three
    // cascade rows (those returned just above). Deriving it from the table the menu
    // PAINTS — instead of re-listing the ids here — is what keeps a new row from
    // shipping a menu item that silently does nothing: `Nearest` was added to
    // `TIMELINE_SEGMENT_MENU` and this hand-list forgot it, so clicking it parked
    // no pick and the graph never changed (the enumeration rotted the moment the
    // table grew). A family row is a leaf only under a mode — a stray click on one
    // from the top-level menu must not park a pick the shell cannot resolve.
    let is_top_level_leaf = ids::TIMELINE_SEGMENT_MENU
        .iter()
        .any(|(row, _, _)| *row == id)
        && !CASCADES.iter().any(|(cascade, _)| *cascade == id);
    let is_family = ids::TIMELINE_EASE_MENU.iter().any(|(fam, _, _)| *fam == id);
    if !(is_top_level_leaf || (is_family && open_mode != TL_NO_EASE_MODE)) {
        return false;
    }
    hero.pending_timeline_interp = Some(TimelineInterpPick {
        scope,
        item: id,
        mode: open_mode,
    });
    hero.store.close_context_menu();
    // Drop the parked request: it is spent, and leaving it would let a later
    // stray Click on one of these ids re-apply the preset to the same key.
    hero.store.consume_last_context_menu();
    true
}
