//! The view tab strip in the transport bar — see [`crate::tab`] for WHY the panel
//! has views at all.
//!
//! **The container you are inside is a TAB in this strip** (Enio, 2026-07-23): the
//! strip is `[ Keys | Containers | <C1> | <C2>… | Arrange ]`, with the container
//! levels of the trail sitting between Containers and Arrange, the innermost one
//! selected. Entering a container adds its tab (born checked); leaving is any of the
//! three fixed tabs. That replaces the old free-floating breadcrumb (a `Scene`
//! button that just fell into Arrange, plus a `Container` button beside it) — two
//! controls outside the group, one of them redundant with a tab already there.
//!
//! Its own module rather than another arm of `transport.rs`, which is at the
//! 600-line cap (HR-18): the strip is one coherent thing (measure, paint, register),
//! so it is the honest seam to cut on.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{TabItem, Tabs, TabsVariant, paint_tabs_with_hover};
use ph2d_editor_core::zones::Rect;
use ph2d_timeline::TimelineViewSnapshot;
use ph2d_tokens::{ROW_H_PX, Theme};

use crate::ids;
use crate::tab::Tab;

/// One tab's width in the strip. Wide enough for the LONGEST fixed label —
/// "Containers" at `TypeToken::Base` with the segmented pill's padding either side.
/// A container tab's name elides into this same box (the segmented pill gives every
/// cell one width), which is the same trade the breadcrumb made with its fixed
/// `SEG_W`.
const TAB_W: f32 = 84.0; // LITERAL-PX-OK: one view tab's width

/// **The tabs on screen right now**, as `(id, label)` — the fixed three plus one per
/// container level of the trail, between Containers and Arrange. The ONE list the
/// paint, the hit registration and the selection all walk, so a container tab cannot
/// be painted at a column the click resolves to a different level.
fn items(snap: &TimelineViewSnapshot) -> Vec<(ph2d_a11y::NodeId, String)> {
    let mut out = vec![
        (
            ids::TIMELINE_TAB_KEYS,
            ph2d_i18n::tr("panel.timeline.tab.keys").to_owned(),
        ),
        (
            ids::TIMELINE_TAB_CONTAINERS,
            ph2d_i18n::tr("panel.timeline.tab.containers").to_owned(),
        ),
    ];
    // The container levels: the breadcrumb trail MINUS its scene root (slot 0), which
    // the Arrange tab already is. Slot indices are preserved (`crumb`'s id is
    // `TIMELINE_CRUMB[slot]`), so the existing crumb click handler resolves each to
    // the right depth through the same `depth_of_slot` the elision uses.
    for (slot, (_depth, label)) in crate::breadcrumb::trail(snap)
        .into_iter()
        .enumerate()
        .skip(1)
    {
        out.push((ids::TIMELINE_CRUMB[slot], label));
    }
    out.push((
        ids::TIMELINE_TAB_ARRANGE,
        ph2d_i18n::tr("panel.timeline.tab.arrange").to_owned(),
    ));
    out
}

/// How many container tabs sit between Containers and Arrange (the trail minus its
/// scene root).
fn crumb_count(snap: &TimelineViewSnapshot) -> usize {
    crate::breadcrumb::trail(snap).len().saturating_sub(1)
}

/// **Which cell is highlighted.** Inside a container (`Containers` tab, non-empty
/// trail) the SELECTED cell is the innermost container tab, not "Containers" — that
/// is what "the container tab is born checked" means. The Containers tab itself is
/// the LIST, reached by tapping it (which pops the trail to the root).
fn selected(tab: Tab, snap: &TimelineViewSnapshot) -> usize {
    let k = crumb_count(snap);
    match tab {
        Tab::Keys => 0,
        // [Keys, Containers, c0..c_{k-1}, Arrange] — Arrange is the last cell.
        Tab::Arrange => 2 + k,
        Tab::Containers if k == 0 => 1,
        // The innermost container tab (the trail's last level = where you are).
        Tab::Containers => 1 + k,
    }
}

/// How wide the strip paints — the single source `transport`'s flow measures
/// against, and the same list [`paint`] lays out from.
pub(crate) fn width(snap: &TimelineViewSnapshot) -> f32 {
    TAB_W * items(snap).len() as f32
}

/// Paint the view tabs, register each cell's hit, and highlight [`selected`].
///
/// Both the paint and the hit walk the SAME [`items`] list; registering from a
/// hand-written second list is exactly how a painted control ends up dispatching
/// nothing ([[feedback_widget_is_done_when_a_test_clicks_it]]). The fixed cells
/// route through `tab::TABS` and the container cells through `TIMELINE_CRUMB`, the
/// two handlers that already exist.
pub(crate) fn paint(
    ctx: &mut PaintCtx,
    theme: Theme,
    x: f32,
    y: f32,
    tab: Tab,
    snap: &TimelineViewSnapshot,
) {
    let cells = items(snap);
    let rect = Rect::new(x, y, TAB_W * cells.len() as f32, ROW_H_PX);
    let tab_items: Vec<TabItem> = cells
        .iter()
        .map(|(id, label)| TabItem::new(*id, label.clone()))
        .collect();
    let tabs = Tabs::new(ids::TIMELINE_TABS, "", tab_items)
        .selected(selected(tab, snap))
        .variant(TabsVariant::Segmented);
    // The hot cell, so the strip answers the pointer before the click commits.
    let hovered = cells
        .iter()
        .position(|(id, _)| ctx.host.store().hot_id() == Some(*id));
    paint_tabs_with_hover(&tabs, hovered, rect, ctx.scene, ctx.text_system, theme);
    for i in 0..cells.len() {
        ctx.host
            .hit_index_mut()
            .register(tabs.items[i].id, tabs.tab_rect(rect, i));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(crumbs: &[&str]) -> TimelineViewSnapshot {
        TimelineViewSnapshot {
            crumbs: crumbs
                .iter()
                .enumerate()
                .map(|(i, n)| (i, (*n).to_string()))
                .collect(),
            ..TimelineViewSnapshot::default()
        }
    }

    /// At the root the strip is exactly the fixed three, in order.
    #[test]
    fn the_root_strip_is_the_fixed_three() {
        let s = snap(&[]);
        let cells: Vec<_> = items(&s).into_iter().map(|(id, _)| id).collect();
        assert_eq!(
            cells,
            vec![
                ids::TIMELINE_TAB_KEYS,
                ids::TIMELINE_TAB_CONTAINERS,
                ids::TIMELINE_TAB_ARRANGE
            ]
        );
        assert_eq!(selected(Tab::Keys, &s), 0);
        assert_eq!(selected(Tab::Containers, &s), 1);
        assert_eq!(selected(Tab::Arrange, &s), 2);
    }

    /// **Entering a container inserts its tab BETWEEN Containers and Arrange, born
    /// checked** (Enio, 2026-07-23). Its label is the container name; its id is the
    /// crumb slot so the existing navigation handler resolves it.
    #[test]
    fn entering_a_container_inserts_its_tab_born_checked() {
        let s = snap(&["Walk"]);
        let cells = items(&s);
        assert_eq!(cells.len(), 4, "Keys | Containers | Walk | Arrange");
        assert_eq!(
            cells[2].0,
            ids::TIMELINE_CRUMB[1],
            "the container's slot id"
        );
        assert_eq!(cells[2].1, "Walk", "labelled with the container's name");
        assert_eq!(cells[3].0, ids::TIMELINE_TAB_ARRANGE, "Arrange stays last");
        // On the Containers tab inside the container, the CONTAINER tab is the selected
        // one — that is "born checked", not the Containers list.
        assert_eq!(
            selected(Tab::Containers, &s),
            2,
            "the container tab, not the Containers list"
        );
        assert_eq!(selected(Tab::Arrange, &s), 3);
        assert_eq!(selected(Tab::Keys, &s), 0);
    }

    /// Nesting deeper keeps every level as a tab and selects the INNERMOST.
    #[test]
    fn nesting_deeper_selects_the_innermost() {
        let s = snap(&["A", "B"]);
        let cells = items(&s);
        assert_eq!(cells.len(), 5, "Keys | Containers | A | B | Arrange");
        assert_eq!(cells[2].0, ids::TIMELINE_CRUMB[1]);
        assert_eq!(cells[3].0, ids::TIMELINE_CRUMB[2]);
        assert_eq!(
            selected(Tab::Containers, &s),
            3,
            "the innermost container (B) is where you are"
        );
    }
}
