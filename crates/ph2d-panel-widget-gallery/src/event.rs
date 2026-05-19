//! Widget Gallery panel `apply_event` — ADR-0029 Phase C.3 port.
//!
//! Migrated from `ph2d_panel_widget_gallery::apply_event_thunk` (which
//! mutated `hero.view.widget_gallery_visible` directly) to the typed
//! contract `(state: &mut WidgetGalleryState, host: &mut dyn
//! PanelHostInternal, ev: WidgetEvent)`. Visibility flips now go
//! through `host.set_panel_visible("widget_gallery", ...)` (canonical
//! Phase C visibility model).

use crate::WidgetGalleryPanel;
use crate::state::WidgetGalleryState;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

pub(crate) fn apply_event(
    _state: &mut WidgetGalleryState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    if let WidgetEvent::Click(id) = ev
        && (id == ids::TOPBAR_WIDGET_GALLERY || id == ids::GAL_CLOSE)
    {
        let next = !host.panel_visible(WidgetGalleryPanel::ID);
        host.set_panel_visible(WidgetGalleryPanel::ID, next);
        return EventOutcome::Consumed;
    }
    EventOutcome::Ignored
}
