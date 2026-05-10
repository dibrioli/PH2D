//! Inspector painter — currently a blank panel.
//!
//! The placeholder fixture (Properties/Layers/Materials tabs, fake
//! sliders, Behavior section with Checkbox/Toggle/Swatch) was removed
//! when the showcase teardown landed. The next phase repopulates this
//! panel with canonical widget samples driven by the centralized
//! [`crate::widget`] painters.
//!
//! What stays:
//! - Panel surface + title/subtitle from the current `HeroSelection`.
//! - Scroll-ready clip layer so future samples can overflow.
//! - Registration of the floating [`crate::widget::BlenderColorPicker`]
//!   state (parented on [`ids::INSP_BLENDER_PICKER`]). The picker is
//!   painted by [`super::color_picker_demo`], not here.

use super::HeroLayout;
use super::HeroSelection;
use super::ids;
use super::style::{PANEL_HEAD_PAD, paint_panel_surface};
use crate::interaction::{HitIndex, InteractiveState, WidgetEvent, WidgetStore};
use crate::paint::{paint_text, rect_to_vello, resolve};
use crate::widget::{
    ChannelMode, InterpolationMode, TextInputState,
};
use crate::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ColorValue, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Register the floating BlenderColorPicker's retained state + every
/// sub-control hit shim. Inspector chrome itself is currently
/// state-free (no fields yet — sample population is the next phase).
pub fn populate(store: &mut WidgetStore) {
    // BlenderColorPicker retained state — wheel + value-slider hit
    // shims route their picks back into the parent picker.
    store.register(
        ids::INSP_BLENDER_PICKER,
        InteractiveState::BlenderPicker {
            // Initial state: pure black (0, 0, 0). The SV cursor
            // lands at the bottom-left corner (S=0, V=0); the hue
            // strip thumb sits at H=0 (red). Once the user picks a
            // color the BlenderPicker state retains it across the
            // session — this default only applies on fresh start.
            value: ColorValue::from_rgba8(0, 0, 0, 255),
            channel_mode: ChannelMode::Rgb,
            interpolation: InterpolationMode::Perceptual,
            active_palette: 0,
            hsv_h: 0.0,
            hsv_s: 0.0,
        },
    );
    // Seed the picker's mutable palette with the default 12 swatches.
    // "+ swatch" appends; right-click on a swatch removes.
    store.init_blender_palette(
        ids::INSP_BLENDER_PICKER,
        crate::widget::default_palette().swatches.clone(),
    );
    // "+ swatch", eyedropper, drag-handle, wheel, value-slider shims.
    for (id, kind) in [
        (ids::BLENDER_ADD_SWATCH, crate::interaction::BlenderHitKind::AddSwatch),
        (ids::BLENDER_EYEDROPPER, crate::interaction::BlenderHitKind::Eyedropper),
        (ids::BLENDER_DRAG_HANDLE, crate::interaction::BlenderHitKind::DragHandle),
        (ids::BLENDER_WHEEL, crate::interaction::BlenderHitKind::Wheel),
        (ids::BLENDER_VALUE_SLIDER, crate::interaction::BlenderHitKind::ValueSlider),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind,
            },
        );
    }

    // Channel slider shims (4 rows: index 0 = R/H, 1 = G/S, 2 = B/V, 3 = A).
    for (id, idx) in [
        (ids::BLENDER_CHANNEL_0, 0u8),
        (ids::BLENDER_CHANNEL_1, 1),
        (ids::BLENDER_CHANNEL_2, 2),
        (ids::BLENDER_CHANNEL_3, 3),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind: crate::interaction::BlenderHitKind::ChannelSlider(idx),
            },
        );
    }

    // Channel value chips — `NumberInput`s mirrored to the channel
    // sliders. Initial value 0; the painter syncs from the parent
    // `BlenderPicker.value` every frame (when not focused).
    for (id, idx) in [
        (ids::BLENDER_NUM_0, 0u8),
        (ids::BLENDER_NUM_1, 1),
        (ids::BLENDER_NUM_2, 2),
        (ids::BLENDER_NUM_3, 3),
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::new(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_blender_channel(ids::INSP_BLENDER_PICKER, id, idx);
    }

    // Hex field as TextInput — the buffer is pre-allocated with the
    // initial hex string matching the picker's default value.
    store.register(
        ids::BLENDER_HEX,
        InteractiveState::TextInput {
            state: TextInputState::Normal,
            text: "#E7E7E7FF".to_string(),
            caret: 9,
            selection_anchor: None,
        },
    );
    store.link_blender_hex(ids::INSP_BLENDER_PICKER, ids::BLENDER_HEX);

    // Segmented toggle shims (Linear / Perceptual interpolation,
    // RGB / HSV channel modes).
    for (id, kind) in [
        (ids::BLENDER_INTERP_LINEAR, crate::interaction::BlenderHitKind::InterpolationLinear),
        (ids::BLENDER_INTERP_PERCEPTUAL, crate::interaction::BlenderHitKind::InterpolationPerceptual),
        (ids::BLENDER_CHANNEL_RGB, crate::interaction::BlenderHitKind::ChannelRgb),
        (ids::BLENDER_CHANNEL_HSV, crate::interaction::BlenderHitKind::ChannelHsv),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind,
            },
        );
    }

    // Palette swatch shims — slots 0..26. Default palette fills 0..11;
    // "+ swatch" appends into 12..26. Beyond 27, `blender_palette_push`
    // is capped and the painter hides the "+" tile.
    for (id, swatch_idx) in [
        (ids::BLENDER_SWATCH_0, 0u8),
        (ids::BLENDER_SWATCH_1, 1),
        (ids::BLENDER_SWATCH_2, 2),
        (ids::BLENDER_SWATCH_3, 3),
        (ids::BLENDER_SWATCH_4, 4),
        (ids::BLENDER_SWATCH_5, 5),
        (ids::BLENDER_SWATCH_6, 6),
        (ids::BLENDER_SWATCH_7, 7),
        (ids::BLENDER_SWATCH_8, 8),
        (ids::BLENDER_SWATCH_9, 9),
        (ids::BLENDER_SWATCH_10, 10),
        (ids::BLENDER_SWATCH_11, 11),
        (ids::BLENDER_SWATCH_12, 12),
        (ids::BLENDER_SWATCH_13, 13),
        (ids::BLENDER_SWATCH_14, 14),
        (ids::BLENDER_SWATCH_15, 15),
        (ids::BLENDER_SWATCH_16, 16),
        (ids::BLENDER_SWATCH_17, 17),
        (ids::BLENDER_SWATCH_18, 18),
        (ids::BLENDER_SWATCH_19, 19),
        (ids::BLENDER_SWATCH_20, 20),
        (ids::BLENDER_SWATCH_21, 21),
        (ids::BLENDER_SWATCH_22, 22),
        (ids::BLENDER_SWATCH_23, 23),
        (ids::BLENDER_SWATCH_24, 24),
        (ids::BLENDER_SWATCH_25, 25),
        (ids::BLENDER_SWATCH_26, 26),
    ] {
        store.register(
            id,
            InteractiveState::BlenderHit {
                parent: ids::INSP_BLENDER_PICKER,
                kind: crate::interaction::BlenderHitKind::PaletteSwatch(swatch_idx),
            },
        );
    }
}

/// Apply a [`WidgetEvent`] against Inspector widgets. Stub — the
/// inspector chrome has no interactive widgets of its own right now.
pub fn apply_event(_store: &mut WidgetStore, _event: WidgetEvent) -> bool {
    false
}

#[allow(clippy::too_many_arguments)]
pub fn paint_inspector(
    layout: &HeroLayout,
    selection: Option<&HeroSelection>,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    _hit_index: &mut HitIndex,
    store: &WidgetStore,
) {
    let rect = layout.inspector;
    paint_panel_surface(rect, scene, theme);

    let title = selection
        .map(|s| s.label.as_str())
        .unwrap_or("(no selection)");
    let sub = "Inspector";

    let title_y = rect.y + 18.0;
    paint_text(
        text_system,
        scene,
        title,
        rect.x + PANEL_HEAD_PAD,
        title_y,
        TypeToken::Md.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text1, theme),
    );
    paint_text(
        text_system,
        scene,
        sub,
        rect.x + PANEL_HEAD_PAD,
        title_y + TypeToken::Md.px() + 4.0,
        TypeToken::Xs.px(),
        rect.w - PANEL_HEAD_PAD * 2.0,
        resolve(ColorToken::Text3, theme),
    );

    let div_y = title_y + TypeToken::Md.px() + TypeToken::Xs.px() + 16.0;
    let div = Rect::new(
        rect.x + PANEL_HEAD_PAD,
        div_y,
        rect.w - PANEL_HEAD_PAD * 2.0,
        1.0,
    );
    scene.fill_rect(rect_to_vello(div), resolve(ColorToken::Border, theme));

    // Scroll-ready clip layer — kept even though the body is empty,
    // so the next phase (canonical sample population) can paint
    // into the clipped region and inherit wheel-scroll behavior
    // without re-wiring the chrome.
    let content_top = div_y + Spacing::Md.px();
    let content_bottom = rect.y + rect.h - 4.0;
    let _scroll_y = store.panel_scroll(ids::INSP_PANEL).max(0.0);
    let clip = ph2d_vector::Rect::new(
        rect.x as f64,
        content_top as f64,
        (rect.x + rect.w) as f64,
        content_bottom as f64,
    );
    scene.push_clip(&clip);
    // Body intentionally empty — sample widgets land here next phase.
    scene.pop_layer();
}
