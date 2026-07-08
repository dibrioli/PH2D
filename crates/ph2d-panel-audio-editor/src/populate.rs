//! Audio Editor panel widget registration.

use crate::{AEDIT_CLOSE, AEDIT_EXPORT, AEDIT_LOAD, AEDIT_LOOP, AEDIT_PLAY, AEDIT_STOP};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;

pub(crate) fn populate(store: &mut WidgetStore) {
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
