//! Widget Gallery `apply_event`: a pílula da TopBar / o fecho (X) alternam a visibilidade; os
//! cartões agem pela tabela em [`crate::valores`].

use crate::WidgetGalleryPanel;
use crate::state::WidgetGalleryState;
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};

pub(crate) fn apply_event(
    state: &mut WidgetGalleryState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let WidgetEvent::Click(id) = ev else {
        return EventOutcome::Ignored;
    };
    if id == ids::TOPBAR_WIDGET_GALLERY || id == ids::GAL_CLOSE {
        let next = !host.panel_visible(WidgetGalleryPanel::ID);
        host.set_panel_visible(WidgetGalleryPanel::ID, next);
        return EventOutcome::Consumed;
    }
    if crate::valores::on_click(state, host, id) {
        EventOutcome::Consumed
    } else {
        EventOutcome::Ignored
    }
}
