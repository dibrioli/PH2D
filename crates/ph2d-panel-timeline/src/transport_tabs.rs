//! The view tab strip in the transport bar — see [`crate::tab`] for WHY the panel
//! has two views at all.
//!
//! Its own module rather than another arm of `transport.rs`, which is at the
//! 600-line cap (HR-18): the strip is one coherent thing (measure, paint, register),
//! so it is the honest seam to cut on.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{TabItem, Tabs, TabsVariant, paint_tabs_with_hover};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Theme};

use crate::ids;
use crate::tab::Tab;

/// One tab's width in the strip. Wide enough for the LONGEST label — "Containers" at
/// `TypeToken::Base` with the segmented pill's padding either side. Sized to the longest
/// rather than the average because a segmented pill gives every cell the same box: fit the
/// average and the long one elides, which is the "+ Container" crush one control over
/// (Enio's screenshot, 2026-07-20) in a different widget.
const TAB_W: f32 = 84.0; // LITERAL-PX-OK: one view tab's width

/// How wide the strip paints — the single source `transport`'s flow measures
/// against, and the same number [`paint`] lays out from.
pub(crate) fn width() -> f32 {
    TAB_W * crate::tab::TABS.len() as f32
}

/// The view tabs: `[ Keys | Arrange ]`.
///
/// Segmented rather than underlined: it sits inside a bar of outlined toggle
/// cells, and a pill reads as a chooser at a glance where a hairline underline
/// would read as chrome.
///
/// Both the paint and the hit walk [`crate::tab::TABS`] — the ONE list. Registering
/// from a hand-written second list is exactly how a painted control ends up
/// dispatching nothing ([[feedback_widget_is_done_when_a_test_clicks_it]]); the
/// seam gate clicks each of these for the same reason.
pub(crate) fn paint(ctx: &mut PaintCtx, theme: Theme, x: f32, y: f32, tab: Tab) {
    let rect = Rect::new(x, y, TAB_W * crate::tab::TABS.len() as f32, ROW_H_PX);
    let items: Vec<TabItem> = crate::tab::TABS
        .iter()
        .map(|(id, key)| TabItem::new(*id, ph2d_i18n::tr(key)))
        .collect();
    let tabs = Tabs::new(ids::TIMELINE_TABS, "", items)
        .selected(tab.index())
        .variant(TabsVariant::Segmented);
    // The hot tab, so the strip answers the pointer before the click commits.
    let hovered = crate::tab::TABS
        .iter()
        .position(|(id, _)| ctx.host.store().hot_id() == Some(*id));
    paint_tabs_with_hover(&tabs, hovered, rect, ctx.scene, ctx.text_system, theme);
    for i in 0..crate::tab::TABS.len() {
        ctx.host
            .hit_index_mut()
            .register(tabs.items[i].id, tabs.tab_rect(rect, i));
    }
}
