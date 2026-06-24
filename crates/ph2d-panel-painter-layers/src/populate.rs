//! Painter layers `populate` — pre-registers the panel's **fixed-id** widget
//! slots in the `WidgetStore` at host boot (once via `Panel::populate`).
//!
//! Per-row widgets (eye toggle, opacity slider+chip, blend chip, row select)
//! have *dynamic* ids derived from the live layer ids, so they are registered
//! in `paint` via `register_if_absent` (the panel owns `store_mut` there) —
//! they can't be known at boot. Only the chrome buttons live here.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, DropdownState, SliderOrientation, SliderState};

pub fn populate(store: &mut WidgetStore) {
    let buttons = [
        ph2d_editor_core::ids::PAINTER_LAYERS_CLOSE,
        // Action toolbar (below the header): New layer / Group / Duplicate /
        // Delete. MUST be registered here or the dispatcher drops the click
        // (paint + hit_index alone is not enough — feedback-panel-populate-register).
        ph2d_editor_core::ids::PAINTER_LAYERS_ADD,
        ph2d_editor_core::ids::PAINTER_LAYERS_GROUP,
        ph2d_editor_core::ids::PAINTER_LAYERS_DUPLICATE,
        ph2d_editor_core::ids::PAINTER_LAYERS_DELETE,
        // "+ Texture" — creates a Texture layer (plain Button, not a Dropdown).
        ph2d_editor_core::ids::PAINTER_LAYERS_ADD_TEXTURE,
        // Modifier toolbar (acts on the active layer): Mask / Clip / Lock / Ref.
        ph2d_editor_core::ids::PAINTER_LAYERS_MASK,
        ph2d_editor_core::ids::PAINTER_LAYERS_CLIP,
        ph2d_editor_core::ids::PAINTER_LAYERS_ALPHA_LOCK,
        ph2d_editor_core::ids::PAINTER_LAYERS_REFERENCE,
        // Dock-mode toggle ("Brush") in the header (mode C).
        ph2d_editor_core::ids::PAINTER_LAYERS_TOGGLE_DOCK,
        // "Apply" CTA — commits the composite to the sprite (shared id with the
        // sidebar panel).
        ph2d_editor_core::ids::PAINTER_APPLY,
    ];
    for id in buttons {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // "+ Adj" is a Dropdown, not a plain Button (W4 T4.15): clicking it toggles
    // the 24-kind picker popover open/closed via the generic Dropdown dispatch
    // (which emits no Click — the panel observes `open` via the store), exactly
    // like the per-row blend chip. Painted as an icon button; the open popover
    // is a deferred pass in `paint`.
    store.register(
        ph2d_editor_core::ids::PAINTER_LAYERS_ADD_ADJUSTMENT,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: None,
        },
    );

    // Brush-properties view (fixed-id, so registered here, not per-frame): the
    // Size / Hardness / Flow / Strength sliders. The panel paints them off the
    // published `BrushSettings` snapshot; the values here are placeholders
    // overwritten by the drag / the snapshot each frame.
    let brush_sliders = [
        ph2d_editor_core::ids::PAINTER_BRUSH_SIZE_SLIDER,
        ph2d_editor_core::ids::PAINTER_BRUSH_STRENGTH_SLIDER,
        // Randomize Color amounts (painted only when the subsection is enabled).
        ph2d_editor_core::ids::PAINTER_BRUSH_COLOR_JITTER_HUE,
        ph2d_editor_core::ids::PAINTER_BRUSH_COLOR_JITTER_SAT,
        ph2d_editor_core::ids::PAINTER_BRUSH_COLOR_JITTER_VAL,
        // Stroke section sliders (all `0..1` track; the tool maps each to its range).
        ph2d_editor_core::ids::PAINTER_BRUSH_RATE,
        ph2d_editor_core::ids::PAINTER_BRUSH_SPACING,
        ph2d_editor_core::ids::PAINTER_BRUSH_JITTER,
        // Per-dab Jitter Scale / Rotate (next to the position Jitter).
        ph2d_editor_core::ids::PAINTER_BRUSH_JITTER_SCALE,
        ph2d_editor_core::ids::PAINTER_BRUSH_JITTER_ROTATE,
        // Repeat-Image Aspect Ratio X / Y (shown when Repeat Image is on).
        ph2d_editor_core::ids::PAINTER_BRUSH_TILE_ASPECT_X,
        ph2d_editor_core::ids::PAINTER_BRUSH_TILE_ASPECT_Y,
        ph2d_editor_core::ids::PAINTER_BRUSH_DASH_RATIO,
        ph2d_editor_core::ids::PAINTER_BRUSH_DASH_LENGTH,
        ph2d_editor_core::ids::PAINTER_BRUSH_INPUT_SAMPLES,
        // Stabilizer intensity — the single "how regular" knob (reuses the STABILIZE id, now a
        // slider instead of the removed toggle).
        ph2d_editor_core::ids::PAINTER_BRUSH_STABILIZE,
        // Texture section sliders (Angle / Offset X-Y / Size X-Y; all `0..1` track).
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_ANGLE,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_SIZE_Y,
        // Per-pattern parameter sliders (painted only when the active kind exposes the slot).
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_PARAM_0,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_PARAM_1,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_PARAM_2,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_PARAM_3,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_PARAM_4,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_PARAM_5,
    ];
    for id in brush_sliders {
        store.register(
            id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
    }
    // Colour swatch + Eraser toggle + the Stroke-section "Adjust Strength" toggle —
    // Buttons. MUST be registered here or the dispatcher drops the click (the
    // populate-register gotcha).
    for id in [
        ph2d_editor_core::ids::PAINTER_COLOR_THUMB,
        ph2d_editor_core::ids::PAINTER_BRUSH_ERASER,
        // "Randomize Color" subsection enable (Color section header checkbox).
        ph2d_editor_core::ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE,
        // Seamless Tiling (wrap-around painting) toggles, X / Y + the Repeat-Image tile preview.
        ph2d_editor_core::ids::PAINTER_BRUSH_TILING_X,
        ph2d_editor_core::ids::PAINTER_BRUSH_TILING_Y,
        ph2d_editor_core::ids::PAINTER_BRUSH_REPEAT_IMAGE,
        ph2d_editor_core::ids::PAINTER_BRUSH_SPACE_ATTEN,
        ph2d_editor_core::ids::PAINTER_BRUSH_EDGE_TO_EDGE,
        // Texture section: "New" (momentary) + Rake / Random toggles.
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_NEW,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAKE,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RANDOM,
        // Color Ramp: enable toggle + add / remove stop + colour-box buttons.
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_ADD,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_REMOVE,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Blend + Falloff + Stroke-Method + Jitter-Unit chips are Dropdowns (generic
    // open/close dispatch); their options are popover buttons.
    for id in [
        ph2d_editor_core::ids::PAINTER_BRUSH_BLEND,
        ph2d_editor_core::ids::PAINTER_BRUSH_FALLOFF,
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_METHOD,
        ph2d_editor_core::ids::PAINTER_BRUSH_JITTER_UNIT,
        // Texture section: Kind picker + Mapping chips.
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_KIND,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_MAPPING,
        // Color Ramp: Mode + Interpolation + Alpha-action chips.
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE,
    ] {
        store.register(
            id,
            InteractiveState::Dropdown {
                state: DropdownState::Normal,
                open: false,
                selected_index: Some(0),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::ids;

    /// Regression: every plain-action toolbar button that `paint` paints + hit-registers MUST get a
    /// store slot here, or the dispatch drops its click (the dead-button class — a missing
    /// `PAINTER_LAYERS_ADD_TEXTURE` slot made the whole Texture-layer feature unreachable).
    #[test]
    fn action_toolbar_buttons_have_store_slots() {
        let mut store = WidgetStore::with_capacity(32);
        populate(&mut store);
        for id in [
            ids::PAINTER_LAYERS_ADD,
            ids::PAINTER_LAYERS_GROUP,
            ids::PAINTER_LAYERS_DUPLICATE,
            ids::PAINTER_LAYERS_DELETE,
            ids::PAINTER_LAYERS_ADD_TEXTURE,
            ids::PAINTER_LAYERS_MASK,
            ids::PAINTER_LAYERS_CLIP,
            // Randomize Color enable + its H/S/V + Jitter Scale/Rotate sliders — a missing slot here
            // is the dead-control class (the click/drag would be silently dropped).
            ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE,
            ids::PAINTER_BRUSH_COLOR_JITTER_HUE,
            ids::PAINTER_BRUSH_COLOR_JITTER_SAT,
            ids::PAINTER_BRUSH_COLOR_JITTER_VAL,
            ids::PAINTER_BRUSH_JITTER_SCALE,
            ids::PAINTER_BRUSH_JITTER_ROTATE,
            // Seamless Tiling toggles + Repeat-Image preview + its Aspect sliders.
            ids::PAINTER_BRUSH_TILING_X,
            ids::PAINTER_BRUSH_TILING_Y,
            ids::PAINTER_BRUSH_REPEAT_IMAGE,
            ids::PAINTER_BRUSH_TILE_ASPECT_X,
            ids::PAINTER_BRUSH_TILE_ASPECT_Y,
        ] {
            assert!(
                store.get(id).is_some(),
                "toolbar button {id:?} has no store slot — its click would be dropped"
            );
        }
    }
}
