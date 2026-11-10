//! Os gestos da Preview: o `×` fecha, o botão do título pede tocar/parar à família, a alça arrasta.

use crate::{PixelPreviewPanel, PixelPreviewPanelState, ids};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::pixel_panel::{PixelFrameAction, PixelRequest};
use ph2d_editor_core::widget::ButtonState;
use ph2d_i18n::tr;

pub(crate) fn populate(store: &mut WidgetStore) {
    crate::crt::populate(store);
    store.set_tooltip(ids::CRT, tr("panel.pixel.preview.crt_tip"));
    for id in [ids::PREVIEW_CLOSE, ids::PLAY, ids::CRT] {
        let state = ButtonState::Normal;
        store.register(id, InteractiveState::Button { state });
    }
    store.set_tooltip(ids::PLAY, tr("panel.pixel.preview.play"));
    for (id, kind) in [
        (ids::DRAG, BlenderHitKind::DragHandle),
        (ids::RESIZE, BlenderHitKind::ResizeHandle),
        (ids::RESIZE_BL, BlenderHitKind::ResizeHandleBl),
    ] {
        let parent = PixelPreviewPanel::NODE_ID;
        store.register(id, InteractiveState::BlenderHit { parent, kind });
    }
}

pub(crate) fn apply_event(
    _st: &mut PixelPreviewPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        WidgetEvent::Click(id) if id == ids::PREVIEW_CLOSE => {
            host.set_panel_visible(PixelPreviewPanel::ID, false);
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) if id == ids::PLAY => {
            host.bus_mut()
                .push(EditorAction::Pixel(PixelRequest::FrameAction(
                    PixelFrameAction::PlayPreview,
                )));
            EventOutcome::Consumed
        }
        WidgetEvent::Click(id) | WidgetEvent::Toggled(id) if id == ids::CRT => {
            crate::crt::alterna();
            EventOutcome::Consumed
        }
        WidgetEvent::ValueChanged(id) if crate::crt::numero(host.store(), id) => {
            EventOutcome::Consumed
        }
        _ => EventOutcome::Ignored,
    }
}
