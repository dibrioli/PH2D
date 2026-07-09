//! Audio Editor panel widget registration.

use crate::{
    AEDIT_BITCRUSH, AEDIT_CLOSE, AEDIT_COMPRESS, AEDIT_CUT, AEDIT_DC, AEDIT_EXPORT, AEDIT_FADE_IN,
    AEDIT_FADE_OUT, AEDIT_GAIN_DOWN, AEDIT_GAIN_UP, AEDIT_HIGHPASS, AEDIT_INVERT, AEDIT_LOAD,
    AEDIT_LOOP, AEDIT_LOWPASS, AEDIT_NAME, AEDIT_NORM_LUFS, AEDIT_NORMALIZE, AEDIT_PLAY,
    AEDIT_REDO, AEDIT_REVERSE, AEDIT_SATURATE, AEDIT_SILENCE, AEDIT_STOP, AEDIT_TRIM, AEDIT_UNDO,
    AEDIT_WIDEN,
};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, TextInputState};

pub(crate) fn populate(store: &mut WidgetStore) {
    // Floating waveform overlay drag/resize handles — registered as panel-agnostic
    // `BlenderHit`s keyed to `AUDIO_OVERLAY_PANEL`, so the shared dispatch moves +
    // resizes the overlay exactly like the Inspector dock (the shell bridge
    // registers their hit rects each frame + applies the offset/resize).
    for (id, kind) in [
        (ids::AUDIO_OVERLAY_DRAG_HANDLE, BlenderHitKind::DragHandle),
        (
            ids::AUDIO_OVERLAY_RESIZE_HANDLE,
            BlenderHitKind::ResizeHandle,
        ),
        (
            ids::AUDIO_OVERLAY_RESIZE_HANDLE_BL,
            BlenderHitKind::ResizeHandleBl,
        ),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::AUDIO_OVERLAY_PANEL,
                kind,
            },
        );
    }
    // Body hit-barrier: the overlay floats over the canvas, so its empty body must
    // swallow clicks (mirror of the registry z-walk's per-panel barrier) or clicks
    // between the handles fall through to the canvas tool. `Plain` = hittable no-op.
    store.register(ids::AUDIO_OVERLAY_PANEL, InteractiveState::Plain);

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
        // Edit ops (W2).
        AEDIT_UNDO,
        AEDIT_REDO,
        AEDIT_NORMALIZE,
        AEDIT_NORM_LUFS,
        AEDIT_REVERSE,
        AEDIT_DC,
        AEDIT_INVERT,
        AEDIT_GAIN_DOWN,
        AEDIT_GAIN_UP,
        // Range ops (block 2b).
        AEDIT_TRIM,
        AEDIT_CUT,
        AEDIT_SILENCE,
        AEDIT_FADE_IN,
        AEDIT_FADE_OUT,
        // Effects rack (W3 block 1).
        AEDIT_LOWPASS,
        AEDIT_HIGHPASS,
        AEDIT_COMPRESS,
        AEDIT_SATURATE,
        AEDIT_BITCRUSH,
        AEDIT_WIDEN,
    ] {
        store.register(id, button());
    }

    // Clip name — an editable TextInput (mirror of the Inspector entity-name box).
    store.register(
        AEDIT_NAME,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
}
