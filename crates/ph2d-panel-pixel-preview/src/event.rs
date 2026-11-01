//! Os gestos da Preview: o `×` fecha, o botão do título pede tocar/parar à família, a alça arrasta.

use crate::{PixelPreviewPanel, PixelPreviewPanelState, ids};
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetEvent, WidgetStore};
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_editor_core::pixel_panel::{PixelFrameAction, PixelRequest};
use ph2d_editor_core::widget::ButtonState;
use ph2d_i18n::tr;

pub(crate) fn populate(store: &mut WidgetStore) {
    for id in [ids::CLOSE, ids::PLAY] {
        let state = ButtonState::Normal;
        store.register(id, InteractiveState::Button { state });
    }
    store.set_tooltip(ids::PLAY, tr("panel.pixel.preview.play"));
    store.register(
        ids::DRAG,
        InteractiveState::BlenderHit {
            parent: PixelPreviewPanel::NODE_ID,
            kind: BlenderHitKind::DragHandle,
        },
    );
}

pub(crate) fn apply_event(
    _st: &mut PixelPreviewPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    match ev {
        WidgetEvent::Click(id) if id == ids::CLOSE => {
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
        _ => EventOutcome::Ignored,
    }
}
