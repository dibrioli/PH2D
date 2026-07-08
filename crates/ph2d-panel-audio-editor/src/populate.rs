//! Audio Editor panel widget registration.

use crate::{AEDIT_CLOSE, AEDIT_EXPORT, AEDIT_LOAD, AEDIT_LOOP, AEDIT_PLAY, AEDIT_STOP};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

pub(crate) fn populate(store: &mut WidgetStore) {
    // Floating waveform overlay drag/resize handles — registered as panel-agnostic
    // `BlenderHit`s keyed to `AUDIO_OVERLAY_PANEL`, so the shared dispatch moves +
    // resizes the overlay exactly like the Inspector dock (the shell bridge
    // registers their hit rects each frame + applies the offset/resize).
    for (id, kind) in [
        (ids::AUDIO_OVERLAY_DRAG_HANDLE, BlenderHitKind::DragHandle),
        (ids::AUDIO_OVERLAY_RESIZE_HANDLE, BlenderHitKind::ResizeHandle),
        (ids::AUDIO_OVERLAY_RESIZE_HANDLE_BL, BlenderHitKind::ResizeHandleBl),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::AUDIO_OVERLAY_PANEL,
                kind,
            },
        );
    }

    // Every transport control + Load/Export is a plain Button so the panel's
    // apply_event branch fires on Click. Show/hide is the TopBar
    // `TOPBAR_AUDIO_EDITOR` pill; the dock drag/resize reuse the shared `INSP_*`
    // handles (registered by the Inspector), so none are needed here.
    let button = || InteractiveState::Button {
        state: ButtonState::Normal,
    };
    for id in [
        AEDIT_CLOSE,
        AEDIT_PLAY,
        AEDIT_STOP,
        AEDIT_LOOP,
        AEDIT_LOAD,
        AEDIT_EXPORT,
    ] {
        store.register(id, button());
    }
}
