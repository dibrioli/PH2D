//! Wheel + scrollbar dispatch helpers.
//!
//! Extracted from [`super`] (Track A8). Two responsibilities:
//!
//! 1. [`dispatch_wheel`] — public entry point for wheel / trackpad
//!    events. Finds the panel under the cursor, adjusts its
//!    `panel_scroll`, and clamps against the painter-published
//!    `content_h` / `visible_h` so wheeling past the end doesn't
//!    produce a 1-frame jump.
//! 2. [`scrollbar_panel_for_id`] — maps a scrollbar thumb's
//!    [`ph2d_a11y::NodeId`] back to the panel it scrolls. Pure
//!    routing table; hosts that add new scrollable panels extend
//!    the match here.

use super::super::{InteractiveState, WidgetEvent, WidgetStore};
use bumpalo::Bump;
use bumpalo::collections::Vec as BumpVec;
use ph2d_a11y::NodeId;

/// Wheel / trackpad scroll. Finds the panel under `(x, y)` via
/// [`WidgetStore::panel_at`] and adjusts that panel's
/// `panel_scroll` by `delta_y`. Caller (painter) is responsible
/// for clamping the offset against the panel's `content_h` —
/// dispatch only deltas, doesn't know content height.
pub fn dispatch_wheel<'frame>(
    store: &mut WidgetStore,
    event: ph2d_host::WheelEvent,
    arena: &'frame Bump,
) -> &'frame [WidgetEvent] {
    let events: BumpVec<'frame, WidgetEvent> = BumpVec::new_in(arena);
    // An OPEN dropdown popover scrolls first — it floats on top of any panel, and its rect lives in a
    // dedicated slot (not `panel_rects`) so `panel_at` isn't polluted. Its scroll value + heights use
    // the `panel_scroll`/`panel_*_h` tables keyed by the dropdown id.
    if let Some((dd, rect)) = store.dropdown_popover()
        && rect.contains(event.x, event.y)
        && matches!(
            store.get(dd),
            Some(InteractiveState::Dropdown { open: true, .. })
        )
    {
        let mut next = (store.panel_scroll(dd) - event.delta_y).max(0.0);
        if let Some(content_h) = store.panel_content_h(dd) {
            let visible_h = store.panel_visible_h(dd).unwrap_or(0.0);
            next = next.min((content_h - visible_h).max(0.0));
        }
        store.set_panel_scroll(dd, next);
        return events.into_bump_slice();
    }
    if let Some(panel) = store.panel_at(event.x, event.y) {
        let cur = store.panel_scroll(panel);
        // delta_y > 0 from winit means "scroll forward" / content
        // moves up. We store offset as "how far down content
        // pretends to be" — so positive delta increments the
        // offset (showing content further down).
        let mut next = (cur - event.delta_y).max(0.0);
        // Clamp at the upper bound when the painter has published a
        // content_h for this panel. Without this, wheeling past the
        // last element pushes `next` arbitrarily high; the next
        // paint pass clamps it back, producing a 1-frame "jump"
        // (the user's "saltos indesejados se rodamos a roda no fim").
        if let Some(content_h) = store.panel_content_h(panel) {
            // Prefer the painter-published visible_h (exact body
            // height); fall back to `panel.h - 60` only when the
            // painter hasn't seeded one yet (first frame).
            let visible_h = store.panel_visible_h(panel).unwrap_or_else(|| {
                store
                    .panel_rect(panel)
                    .map(|r| (r.h - 60.0).max(0.0))
                    .unwrap_or(0.0)
            });
            let max_scroll = (content_h - visible_h).max(0.0);
            if next > max_scroll {
                next = max_scroll;
            }
        }
        store.set_panel_scroll(panel, next);
    }
    events.into_bump_slice()
}

/// Maps a scrollbar thumb's hit id back to the panel it scrolls.
/// Returns `None` for non-scrollbar ids. Keeps the panel↔scrollbar
/// mapping in one place — hosts that add new scrollable panels
/// extend this match.
pub(super) fn scrollbar_panel_for_id(id: NodeId) -> Option<NodeId> {
    use crate::ids;
    if id == crate::widget::INSPECTOR_SCROLLBAR_ID {
        Some(ids::INSP_PANEL)
    } else if id == crate::widget::HIERARCHY_SCROLLBAR_ID {
        Some(ids::HIER_PANEL)
    } else if id == crate::widget::GALLERY_SCROLLBAR_ID {
        Some(ids::GAL_PANEL)
    } else if id == crate::widget::GRID_SETTINGS_SCROLLBAR_ID {
        Some(crate::ids::GS_PANEL)
    } else if id == crate::widget::COLOR_EQUALIZATION_SCROLLBAR_ID {
        Some(crate::ids::CEQ_PANEL)
    } else if id == crate::widget::BG_REMOVAL_SCROLLBAR_ID {
        Some(crate::ids::BGR_PANEL)
    } else if id == crate::widget::PADDING_SCROLLBAR_ID {
        Some(crate::ids::PAD_PANEL)
    } else if id == crate::widget::UPSCALE_SCROLLBAR_ID {
        Some(crate::ids::UPS_PANEL)
    } else if id == crate::widget::EQUALIZE_SIZES_SCROLLBAR_ID {
        Some(crate::ids::EQS_PANEL)
    } else if id == crate::widget::PAINTER_LAYERS_SCROLLBAR_ID {
        Some(ids::PAINTER_LAYERS_PANEL)
    } else if id == crate::widget::PAINTER_BRUSH_STUDIO_SCROLLBAR_ID {
        Some(ids::PAINTER_BRUSH_STUDIO_PANEL)
    } else {
        None
    }
}
