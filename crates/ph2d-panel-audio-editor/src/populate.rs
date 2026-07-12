//! Audio Editor panel widget registration.

use crate::{
    AEDIT_BATCH_LUFS, AEDIT_CLOSE, AEDIT_CODEC_NEXT, AEDIT_CODEC_PREV, AEDIT_CUT, AEDIT_DC,
    AEDIT_EXPORT, AEDIT_FADE_IN, AEDIT_FADE_OUT, AEDIT_FX_ADD, AEDIT_FX_APPLY, AEDIT_FX_BYPASS,
    AEDIT_FX_CANCEL, AEDIT_FX_DOWN, AEDIT_FX_LOAD_IR, AEDIT_FX_NEXT, AEDIT_FX_PARAMS,
    AEDIT_FX_PREV, AEDIT_FX_REMOVE, AEDIT_FX_RESET, AEDIT_FX_STAGE_ONS, AEDIT_FX_STAGES,
    AEDIT_FX_UP, AEDIT_GAIN_DOWN, AEDIT_GAIN_UP, AEDIT_INVERT, AEDIT_LOAD, AEDIT_LOOP,
    AEDIT_LOOP_CLEAR, AEDIT_LOOP_SET, AEDIT_LOOP_XFADE, AEDIT_MARK_ADD, AEDIT_MARK_DEL, AEDIT_MONO,
    AEDIT_NAME, AEDIT_NORM_LUFS, AEDIT_NORMALIZE, AEDIT_OGG_QUALITY, AEDIT_PLAY,
    AEDIT_PRESET_APPLY, AEDIT_PRESET_LOAD, AEDIT_PRESET_NEXT, AEDIT_PRESET_PREV, AEDIT_PRESET_SAVE,
    AEDIT_REDO, AEDIT_REVERSE, AEDIT_SEC_DELIVERY, AEDIT_SEC_EDIT, AEDIT_SEC_FX, AEDIT_SEC_LOOP,
    AEDIT_SEC_MARKERS, AEDIT_SEC_SPECTRAL, AEDIT_SEC_TRANSPORT, AEDIT_SEC_VARIATIONS,
    AEDIT_SILENCE, AEDIT_SPEC_AMOUNT, AEDIT_SPEC_DENOISE, AEDIT_SPEC_LEARN, AEDIT_SPEC_REPAIR,
    AEDIT_SPEC_VIEW, AEDIT_STOP, AEDIT_TRIM, AEDIT_UNDO, AEDIT_VAR_ADD, AEDIT_VAR_ADD_FOLDER,
    AEDIT_VAR_GAIN, AEDIT_VAR_LOAD, AEDIT_VAR_PITCH, AEDIT_VAR_PLAY, AEDIT_VAR_REMOVE,
    AEDIT_VAR_ROWS, AEDIT_VAR_SAVE, AEDIT_VAR_STRATEGY_NEXT, AEDIT_VAR_STRATEGY_PREV,
    AEDIT_VAR_WEIGHT_DOWN, AEDIT_VAR_WEIGHT_UP, delivery_state, loop_state, spectral_state,
    variation_state,
};
use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{BlenderHitKind, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

pub(crate) fn populate(store: &mut WidgetStore) {
    populate_sections(store);

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
        AEDIT_BATCH_LUFS,
        // Edit ops (W2).
        AEDIT_UNDO,
        AEDIT_REDO,
        AEDIT_NORMALIZE,
        AEDIT_NORM_LUFS,
        AEDIT_REVERSE,
        AEDIT_DC,
        AEDIT_INVERT,
        AEDIT_MONO,
        AEDIT_GAIN_DOWN,
        AEDIT_GAIN_UP,
        // Range ops (block 2b).
        AEDIT_TRIM,
        AEDIT_CUT,
        AEDIT_SILENCE,
        AEDIT_FADE_IN,
        AEDIT_FADE_OUT,
        // Effects rack (W3 block 3a): selector + Apply.
        AEDIT_FX_PREV,
        AEDIT_FX_NEXT,
        AEDIT_FX_RESET,
        AEDIT_FX_APPLY,
        AEDIT_FX_CANCEL,
        // Effects chain (W3 block 3b): edit the chain + the global A/B.
        AEDIT_FX_ADD,
        AEDIT_FX_REMOVE,
        AEDIT_FX_UP,
        AEDIT_FX_DOWN,
        AEDIT_FX_BYPASS,
        // Chain presets (W3): factory selector + Apply · Save · Load.
        AEDIT_PRESET_PREV,
        AEDIT_PRESET_NEXT,
        AEDIT_PRESET_APPLY,
        AEDIT_PRESET_SAVE,
        AEDIT_PRESET_LOAD,
        // Loop points (W6): Set from selection · Clear.
        AEDIT_LOOP_SET,
        AEDIT_LOOP_CLEAR,
        // Markers (W6): Add at playhead · Delete nearest.
        AEDIT_MARK_ADD,
        AEDIT_MARK_DEL,
        // Variation containers (W6): Add · Add Folder · Remove · Play · strategy
        // selector · Weight ÷2/×2 · Save · Load.
        AEDIT_VAR_ADD,
        AEDIT_VAR_ADD_FOLDER,
        AEDIT_VAR_REMOVE,
        AEDIT_VAR_PLAY,
        AEDIT_VAR_STRATEGY_PREV,
        AEDIT_VAR_STRATEGY_NEXT,
        AEDIT_VAR_WEIGHT_DOWN,
        AEDIT_VAR_WEIGHT_UP,
        AEDIT_VAR_SAVE,
        AEDIT_VAR_LOAD,
        // Delivery (W6): the codec selector. It drives the cost readout AND the export.
        AEDIT_CODEC_PREV,
        AEDIT_CODEC_NEXT,
        // The Convolution Reverb's room (W6): an IR is a resource, not a parameter.
        AEDIT_FX_LOAD_IR,
        // Spectral (W5): the view toggle and the two repair tools.
        AEDIT_SPEC_VIEW,
        AEDIT_SPEC_REPAIR,
        AEDIT_SPEC_LEARN,
        AEDIT_SPEC_DENOISE,
    ] {
        store.register(id, button());
    }

    // Denoise Amount — normalized 0..1.
    store.register(
        AEDIT_SPEC_AMOUNT,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: spectral_state::DEFAULT_AMOUNT,
            orientation: SliderOrientation::Horizontal,
        },
    );

    // Ogg Vorbis quality — normalized 0..1, which is also the codec's own range.
    store.register(
        AEDIT_OGG_QUALITY,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: delivery_state::DEFAULT_QUALITY_NORM,
            orientation: SliderOrientation::Horizontal,
        },
    );

    // Variation list rows — clicking a row selects it (Remove / Weight target).
    for id in AEDIT_VAR_ROWS {
        store.register(id, button());
    }

    // Variation pitch/gain jitter sliders — normalized 0..1 (the shell maps them to
    // a `± semitones` / `± dB` range), seeded neutral.
    for id in [AEDIT_VAR_PITCH, AEDIT_VAR_GAIN] {
        store.register(
            id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: variation_state::DEFAULT_JITTER_NORM,
                orientation: SliderOrientation::Horizontal,
            },
        );
    }

    // Loop crossfade slider — normalized 0..1 (the shell maps it to ms), seeded at
    // the section's default position.
    store.register(
        AEDIT_LOOP_XFADE,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: loop_state::DEFAULT_XFADE_NORM,
            orientation: SliderOrientation::Horizontal,
        },
    );

    // Chain rows: the row selects the stage, the eye toggles it in/out of the render.
    for id in AEDIT_FX_STAGES.into_iter().chain(AEDIT_FX_STAGE_ONS) {
        store.register(id, button());
    }

    // Effects rack parameter sliders. Values are **normalized 0..1**; the shell
    // maps them onto real DSP ranges. The paint step seeds them from the selected
    // effect's preset whenever the kind changes.
    for id in AEDIT_FX_PARAMS {
        store.register(
            id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
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

/// Register the collapsible section headers and their initial fold state. Split out of
/// `populate` to keep it under the panel fn-LOC cap.
fn populate_sections(store: &mut WidgetStore) {
    // Collapsible section headers — bare hit rects (no `InteractiveState`); the
    // dispatch folds them via `toggle_collapsed` on click. Same chrome as the Sprite
    // Inspector and the Audio Mixer.
    for id in [
        AEDIT_SEC_TRANSPORT,
        AEDIT_SEC_EDIT,
        AEDIT_SEC_FX,
        AEDIT_SEC_LOOP,
        AEDIT_SEC_SPECTRAL,
        AEDIT_SEC_MARKERS,
        AEDIT_SEC_VARIATIONS,
        AEDIT_SEC_DELIVERY,
    ] {
        store.mark_collapsible_section(id);
    }
    // What you reach for once per ASSET (loop region, markers, variation sets, delivery)
    // starts FOLDED, so the panel opens on what you touch on every pass. Each folded
    // header still carries its readout, so nothing is hidden — only unstacked.
    // `populate` runs once, at `HeroScreen::new`, so this is an initial state and not a
    // per-frame override.
    for id in [
        AEDIT_SEC_LOOP,
        AEDIT_SEC_SPECTRAL,
        AEDIT_SEC_MARKERS,
        AEDIT_SEC_VARIATIONS,
        AEDIT_SEC_DELIVERY,
    ] {
        store.set_collapsed(id, true);
    }
}
