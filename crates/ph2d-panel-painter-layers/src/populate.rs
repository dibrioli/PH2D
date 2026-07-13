//! Painter layers `populate` — pre-registers the panel's **fixed-id** widget
//! slots in the `WidgetStore` at host boot (once via `Panel::populate`).
//!
//! Per-row widgets (eye toggle, opacity slider+chip, blend chip, row select)
//! have *dynamic* ids derived from the live layer ids, so they are registered
//! in `paint` via `register_if_absent` (the panel owns `store_mut` there) —
//! they can't be known at boot. Only the chrome buttons live here.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{
    ButtonState, DropdownState, SliderOrientation, SliderState, TextInputState,
};

pub fn populate(store: &mut WidgetStore) {
    register_chrome_buttons(store);
    register_brush_inputs(store);
    register_mask_and_selection(store);
    register_toggles_and_dropdowns(store);
}

/// Header / toolbar / modifier chrome buttons + the "+ Adj" dropdown.
fn register_chrome_buttons(store: &mut WidgetStore) {
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
}

/// Brush-properties inputs: sliders, drag-scrub NumberInputs, slider chips, Composite card.
fn register_brush_inputs(store: &mut WidgetStore) {
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
        ph2d_editor_core::ids::PAINTER_BRUSH_OFFSET,
        ph2d_editor_core::ids::PAINTER_BRUSH_JITTER,
        // Per-dab Jitter Scale / Rotate / Spacing (next to the position Jitter).
        ph2d_editor_core::ids::PAINTER_BRUSH_JITTER_SCALE,
        ph2d_editor_core::ids::PAINTER_BRUSH_JITTER_ROTATE,
        ph2d_editor_core::ids::PAINTER_BRUSH_JITTER_SPACING,
        ph2d_editor_core::ids::PAINTER_BRUSH_DASH_RATIO,
        ph2d_editor_core::ids::PAINTER_BRUSH_DASH_LENGTH,
        ph2d_editor_core::ids::PAINTER_BRUSH_INPUT_SAMPLES,
        // Stabilizer intensity — the single "how regular" knob (reuses the STABILIZE id, now a
        // slider instead of the removed toggle).
        ph2d_editor_core::ids::PAINTER_BRUSH_STABILIZE,
        // Symmetry radial segment-count slider (`3..=12`; its chip uses the mapped-integer link below).
        ph2d_editor_core::ids::PAINTER_BRUSH_SYMMETRY_SEGMENTS,
        // Inpaint heal-reconstruction sliders (`0..1` track; their chips use the mapped-integer link below).
        ph2d_editor_core::ids::PAINTER_INPAINT_PATCH_SLIDER,
        ph2d_editor_core::ids::PAINTER_INPAINT_QUALITY_SLIDER,
        ph2d_editor_core::ids::PAINTER_INPAINT_SEARCH_SLIDER,
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
    // Grain + Shape param fields are drag-scrub NumberInputs (Enio 2026-06-25): Angle / Offset X-Y /
    // Size X-Y / Depth + the per-pattern params. Paint mirrors the live value via `paint_ramp_chip`;
    // this seed sets the TYPE so the dispatch scrubs (vertical/horizontal) instead of sliding.
    for id in [
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_ANGLE,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_OFFSET_X,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_OFFSET_Y,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_SIZE_X,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_SIZE_Y,
    ]
    .into_iter()
    .chain(ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_PARAMS)
    .chain(ph2d_editor_core::ids::PAINTER_BRUSH_STENCIL_FIELDS)
    .chain(ph2d_editor_core::ids::PAINTER_SHAPE_SLIDERS)
    .chain(ph2d_editor_core::ids::PAINTER_SHAPE_PARAMS)
    .chain(ph2d_editor_core::ids::PAINTER_WATERCOLOR_FIELDS)
    .chain(ph2d_editor_core::ids::PAINTER_IMPASTO_FIELDS)
    {
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
    }
    crate::populate_brush_chips::register_brush_slider_chips(store);
    // Composite Brush card: 3 bare per-layer Strength sliders + the enable checkbox + the 6 reorder
    // buttons. Registered here (not in `paint_composite`) so the panel-wiring-parity gate sees the ids.
    for sid in ph2d_editor_core::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH {
        store.register(
            sid,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
    }
    for id in std::iter::once(ph2d_editor_core::ids::PAINTER_BRUSH_COMPOSITE_ENABLE)
        .chain(ph2d_editor_core::ids::PAINTER_BRUSH_COMPOSITE_UP)
        .chain(ph2d_editor_core::ids::PAINTER_BRUSH_COMPOSITE_DOWN)
    {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// Mask section + collapsible headers + Selection section + Deform widgets.
fn register_mask_and_selection(store: &mut WidgetStore) {
    // Mask section: the sub-brush segments + the whole-canvas op buttons + the overlay-colour swatches
    // are all Buttons (Click channel). Registered here (not in `paint_mask`) so panel-wiring-parity + the
    // dispatch see them. The header is a plain collapsible section (no colour dot).
    for id in ph2d_editor_core::ids::PAINTER_MASK_BRUSH
        .iter()
        .chain(ph2d_editor_core::ids::PAINTER_MASK_OP.iter())
        .chain(ph2d_editor_core::ids::PAINTER_MASK_COLOR.iter())
        .chain(std::iter::once(&ph2d_editor_core::ids::PAINTER_MASK_APPLY))
        .copied()
    {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    store.mark_collapsible_section(ph2d_editor_core::ids::PAINTER_MASK_SECTION);
    crate::populate_sections::register_collapsible_sections(store);
    // Selection section (ADR-0103): the Feather + Automatic-threshold sliders (with linked editable chips)
    // and the mode / boolean-op / action segments + the Edit-Selection toggle (all Buttons). Registered
    // here (not in `paint_selection`) so panel-wiring-parity + the dispatch see them.
    for id in [
        ph2d_editor_core::ids::PAINTER_SEL_FEATHER_SLIDER,
        ph2d_editor_core::ids::PAINTER_SEL_THRESHOLD_SLIDER,
        ph2d_editor_core::ids::PAINTER_SEL_STABILIZE_SLIDER,
        ph2d_editor_core::ids::PAINTER_SEL_OPACITY_SLIDER,
        ph2d_editor_core::ids::PAINTER_SEL_OFFSET_SLIDER,
    ] {
        store.register(
            id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.0,
                orientation: SliderOrientation::Horizontal,
            },
        );
    }
    for (slider, chip) in [
        (
            ph2d_editor_core::ids::PAINTER_SEL_FEATHER_SLIDER,
            ph2d_editor_core::ids::PAINTER_SEL_FEATHER_CHIP,
        ),
        (
            ph2d_editor_core::ids::PAINTER_SEL_THRESHOLD_SLIDER,
            ph2d_editor_core::ids::PAINTER_SEL_THRESHOLD_CHIP,
        ),
        (
            ph2d_editor_core::ids::PAINTER_SEL_STABILIZE_SLIDER,
            ph2d_editor_core::ids::PAINTER_SEL_STABILIZE_CHIP,
        ),
        (
            ph2d_editor_core::ids::PAINTER_SEL_OPACITY_SLIDER,
            ph2d_editor_core::ids::PAINTER_SEL_OPACITY_CHIP,
        ),
        (
            ph2d_editor_core::ids::PAINTER_SEL_OFFSET_SLIDER,
            ph2d_editor_core::ids::PAINTER_SEL_OFFSET_CHIP,
        ),
    ] {
        store.register(
            chip,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::new(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number(slider, chip);
        store.set_number_range(chip, 0.0, 1.0, 0.01); // LITERAL-PX-OK: chip 0..1 track step (behaviour value)
    }
    for id in ph2d_editor_core::ids::PAINTER_SEL_MODE_IDS
        .iter()
        .chain(ph2d_editor_core::ids::PAINTER_SEL_OP_IDS.iter())
        .chain(ph2d_editor_core::ids::PAINTER_SEL_ACTION_IDS.iter())
        .chain(ph2d_editor_core::ids::PAINTER_SEL_WAVE5_IDS.iter())
        .chain(ph2d_editor_core::ids::PAINTER_SEL_OFFSET_APPLY_IDS.iter())
        .chain(std::iter::once(&ph2d_editor_core::ids::PAINTER_SEL_EDIT))
        .chain(std::iter::once(&ph2d_editor_core::ids::PAINTER_SEL_CONVERT))
        .chain(std::iter::once(&ph2d_editor_core::ids::PAINTER_SEL_MERGE))
        .chain(std::iter::once(
            &ph2d_editor_core::ids::PAINTER_SEL_SIMPLIFY,
        ))
        .copied()
    {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Deform section (Wave 1) widgets — split into a sibling for the panel file-LOC cap.
    crate::populate_deform::register_deform_widgets(store);
    // Sculpt section. This call is not optional and it is not a formality: a widget that paints, registers
    // a hit rect and is forwarded by `event.rs` is STILL dead under the mouse unless it also carries an
    // `InteractiveState` here — `is_focusable` answers `None => false`, so Down never activates it and the
    // Click never happens. That is exactly how the Impasto light rig shipped inert. The gate that would
    // catch its absence is `tests/seam_sculpt.rs`, and it CLICKS.
    crate::populate_sculpt::register_sculpt_widgets(store);
}

/// Checkbox/action Buttons (swatches, toggles, ramp + stroke-editor actions, Watercolor
/// chrome), Symmetry clickables, tooltips and the Dropdown chips.
fn register_toggles_and_dropdowns(store: &mut WidgetStore) {
    // Colour swatch + Eraser toggle + the Stroke-section "Adjust Strength" toggle —
    // Buttons. MUST be registered here or the dispatcher drops the click (the
    // populate-register gotcha).
    for id in [
        ph2d_editor_core::ids::PAINTER_COLOR_THUMB,
        // Watercolor Paper-colour swatch (the document ground; opens the shared picker).
        ph2d_editor_core::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        ph2d_editor_core::ids::PAINTER_BRUSH_ERASER,
        // "Randomize Color" subsection enable (Color section header checkbox).
        ph2d_editor_core::ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE,
        // Seamless Tiling (wrap-around painting) toggles, X / Y + the Repeat-Image tile preview.
        ph2d_editor_core::ids::PAINTER_BRUSH_TILING_X,
        ph2d_editor_core::ids::PAINTER_BRUSH_TILING_Y,
        ph2d_editor_core::ids::PAINTER_BRUSH_REPEAT_IMAGE,
        ph2d_editor_core::ids::PAINTER_BRUSH_SPACE_ATTEN,
        ph2d_editor_core::ids::PAINTER_BRUSH_ACCUMULATE,
        // "Sync with other tools" checkbox at the top of the brush panel (independent settings by default).
        ph2d_editor_core::ids::PAINTER_BRUSH_SYNC,
        // "Dimensions" checkbox below the Method dropdown (Line only) — dx/dy + corner angles while drawing.
        ph2d_editor_core::ids::PAINTER_BRUSH_LINE_DIMENSIONS,
        ph2d_editor_core::ids::PAINTER_BRUSH_EDGE_TO_EDGE,
        // Clone card: "Set Source" (arms the sample pick) + "Aligned" toggle.
        ph2d_editor_core::ids::PAINTER_BRUSH_CLONE_SET_SOURCE,
        ph2d_editor_core::ids::PAINTER_BRUSH_CLONE_ALIGNED,
        // Texture section: Rake / Random checkboxes (the click still forwards as a Button Click;
        // only the VISUAL is a checkbox).
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAKE,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RANDOM,
        // Shape section: Rake / Random checkboxes + the section reset (clears the image → falloff).
        ph2d_editor_core::ids::PAINTER_SHAPE_RAKE,
        ph2d_editor_core::ids::PAINTER_SHAPE_RANDOM,
        ph2d_editor_core::ids::PAINTER_SHAPE_RESET,
        // "Use Document Layers" capture button + the Per-Layer Color mode toggle (the per-layer
        // checkboxes + swatches are factory ids, registered at paint time). Forward as a Button Click.
        ph2d_editor_core::ids::PAINTER_SHAPE_USE_LAYERS,
        ph2d_editor_core::ids::PAINTER_SHAPE_PER_LAYER_COLOR,
        // Grain Colors ramp: enable + add / remove / invert / B&W + colour-box buttons.
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_ENABLE,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_ADD,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_REMOVE,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_INVERT,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_BW,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_SWATCH,
        // Shape Color ramp: enable + add / remove / invert / B&W + colour-box + section reset.
        ph2d_editor_core::ids::PAINTER_SHAPE_RAMP_ENABLE,
        ph2d_editor_core::ids::PAINTER_SHAPE_RAMP_ADD,
        ph2d_editor_core::ids::PAINTER_SHAPE_RAMP_REMOVE,
        ph2d_editor_core::ids::PAINTER_SHAPE_RAMP_INVERT,
        ph2d_editor_core::ids::PAINTER_SHAPE_RAMP_BW,
        ph2d_editor_core::ids::PAINTER_SHAPE_RAMP_SWATCH,
        ph2d_editor_core::ids::PAINTER_SHAPE_RAMP_RESET,
        // Per-section reset icon buttons (Inspector-Transform pattern).
        ph2d_editor_core::ids::PAINTER_BRUSH_RANDOMIZE_RESET,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RESET,
        ph2d_editor_core::ids::PAINTER_BRUSH_COLOR_RAMP_RESET,
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_RESET,
        // Apply / Apply & Keep — bake the open on-canvas shape (Curve/Free Hand/Ellipse/Polygon); Keep
        // retains the editor for re-apply. Routed in the tool's `route_brush_dab_event`.
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_APPLY,
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_APPLY_KEEP,
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_DELETE,
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_EDIT,
        // Simplify (re-fit the curve) + Merge (fold every shape into one/few curves) + Offset-card Trim.
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_SIMPLIFY,
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_MERGE,
        ph2d_editor_core::ids::PAINTER_BRUSH_OFFSET_TRIM,
        // Multi-shape OPERATION segments (Overlay / Add / Remove) — the boolean mode the next shape uses.
        ph2d_editor_core::ids::PAINTER_STROKE_OP_OVERLAY,
        ph2d_editor_core::ids::PAINTER_STROKE_OP_ADD,
        ph2d_editor_core::ids::PAINTER_STROKE_OP_REMOVE,
        // Save As Object (floppy) — shown beside the Method dropdown only while a curve is drawn. The
        // action is not wired yet (clicking is a deliberate no-op); registering the slot gives it hover /
        // press feedback + a tooltip now, and lets the future route deliver the Click without a re-register.
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_SAVE_OBJECT,
        ph2d_editor_core::ids::PAINTER_BRUSH_TILING_RESET,
        // Watercolor section: the Enable master toggle + the section reset + the Grain-section
        // "Same as Paper" toggle + the Paper section reset. (Pigment merged into the Mix slider; Paper
        // Rake/Random dropped from the UI — redesign 2026-07-07.)
        ph2d_editor_core::ids::PAINTER_WATERCOLOR_ENABLE,
        // Shape section's watercolor "Automatic" toggle (doc 13 #1).
        ph2d_editor_core::ids::PAINTER_SHAPE_WATERCOLOR_AUTO,
        ph2d_editor_core::ids::PAINTER_WATERCOLOR_RESET,
        ph2d_editor_core::ids::PAINTER_WATERCOLOR_GRAN_SAME,
        ph2d_editor_core::ids::PAINTER_WATERCOLOR_PAPER_RESET,
        // Wetness card (doc 13 #9/#10): Dry (end the wet session) / Wet (re-moisten the canvas).
        ph2d_editor_core::ids::PAINTER_WATERCOLOR_DRY_NOW,
        ph2d_editor_core::ids::PAINTER_WATERCOLOR_WET_NOW,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Symmetry section: "Use" / "Circular" checkboxes, the X/Y/Custom axis segments, the two
    // canvas pick buttons (Draw Line / Pick Center), and the section reset — all forward a plain Click.
    for id in ph2d_editor_core::ids::PAINTER_BRUSH_SYMMETRY_CLICKABLE {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Impasto section: the Enable master + section reset + the Depth-Source / Draw-To segments +
    // Show Impasto + the four lamp chips + the lamp's Enable. Every one of them was PAINTED,
    // hit-registered, forwarded by `event.rs` and routed by the tool — and stone dead under the
    // mouse, because none of them was here (Enio 2026-07-12: "nem o checkbox nem se pode
    // selecionar outra luz"). A hit rect is not enough: `dispatch_pointer` only makes a hit ACTIVE
    // when it is focusable, and an id the store never heard of is not. `paint_checkbox_row` and
    // `paint_segmented_group_adaptive` both take the store by `&` — they CANNOT self-register, so
    // this is the only place the wire can be closed. Buttons, not Checkboxes: the check state lives
    // in the TOOL (the paint mirrors `brush.impasto*`), and a Checkbox would emit `Toggled`, which
    // `event.rs` does not forward — registered, and still dead. Gate: `tests/seam_impasto_rig.rs`.
    for id in ph2d_editor_core::ids::PAINTER_IMPASTO_CLICKS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Mouse-over hint for the (not-yet-wired) Save As Object button.
    store.set_tooltip(
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_SAVE_OBJECT,
        "Save As Object",
    );
    // Blend + Falloff + Stroke-Method + Jitter-Unit chips are Dropdowns (generic
    // open/close dispatch); their options are popover buttons.
    for id in [
        ph2d_editor_core::ids::PAINTER_BRUSH_BLEND,
        ph2d_editor_core::ids::PAINTER_BRUSH_FALLOFF,
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_METHOD,
        ph2d_editor_core::ids::PAINTER_BRUSH_JITTER_UNIT,
        // Shape section: source picker (None/Image).
        ph2d_editor_core::ids::PAINTER_SHAPE_KIND,
        // Texture section: Kind picker + Mapping chips.
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_KIND,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_MAPPING,
        // Grain Color Ramp: Mode + Interpolation + Alpha-action chips.
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP,
        ph2d_editor_core::ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE,
        // Shape Color Ramp: Mode + Interpolation + Alpha-action chips.
        ph2d_editor_core::ids::PAINTER_SHAPE_RAMP_MODE,
        ph2d_editor_core::ids::PAINTER_SHAPE_RAMP_INTERP,
        ph2d_editor_core::ids::PAINTER_SHAPE_RAMP_ALPHA_MODE,
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
            ids::PAINTER_BRUSH_JITTER_SPACING,
            // Seamless Tiling toggles + Repeat-Image preview.
            ids::PAINTER_BRUSH_TILING_X,
            ids::PAINTER_BRUSH_TILING_Y,
            ids::PAINTER_BRUSH_REPEAT_IMAGE,
        ] {
            assert!(
                store.get(id).is_some(),
                "toolbar button {id:?} has no store slot — its click would be dropped"
            );
        }
    }

    /// Regression: EVERY brush slider has its editable chip registered, `link_slider_number`-linked,
    /// and `set_number_range`-normalised (Enio 2026-06-26 — the Stroke + Jitter rows were bare
    /// slider+text-readout before; now they all use the canonical slider-with-chip like Size/Strength).
    /// A missing chip slot / link / range silently breaks the slider's number box (drag not
    /// range-proportional, or a chip edit that never reaches the slider).
    #[test]
    fn every_brush_slider_chip_is_registered_linked_and_ranged() {
        let mut store = WidgetStore::with_capacity(64);
        populate(&mut store);
        for (slider, chip) in [
            (ids::PAINTER_BRUSH_SIZE_SLIDER, ids::PAINTER_BRUSH_SIZE_CHIP),
            (
                ids::PAINTER_BRUSH_STRENGTH_SLIDER,
                ids::PAINTER_BRUSH_STRENGTH_CHIP,
            ),
            (
                ids::PAINTER_BRUSH_COLOR_JITTER_HUE,
                ids::PAINTER_BRUSH_COLOR_JITTER_HUE_CHIP,
            ),
            (
                ids::PAINTER_BRUSH_COLOR_JITTER_SAT,
                ids::PAINTER_BRUSH_COLOR_JITTER_SAT_CHIP,
            ),
            (
                ids::PAINTER_BRUSH_COLOR_JITTER_VAL,
                ids::PAINTER_BRUSH_COLOR_JITTER_VAL_CHIP,
            ),
            (ids::PAINTER_BRUSH_RATE, ids::PAINTER_BRUSH_RATE_CHIP),
            (ids::PAINTER_BRUSH_SPACING, ids::PAINTER_BRUSH_SPACING_CHIP),
            (ids::PAINTER_BRUSH_OFFSET, ids::PAINTER_BRUSH_OFFSET_CHIP),
            (
                ids::PAINTER_BRUSH_DASH_RATIO,
                ids::PAINTER_BRUSH_DASH_RATIO_CHIP,
            ),
            (
                ids::PAINTER_BRUSH_DASH_LENGTH,
                ids::PAINTER_BRUSH_DASH_LENGTH_CHIP,
            ),
            (
                ids::PAINTER_BRUSH_INPUT_SAMPLES,
                ids::PAINTER_BRUSH_INPUT_SAMPLES_CHIP,
            ),
            (
                ids::PAINTER_BRUSH_STABILIZE,
                ids::PAINTER_BRUSH_STABILIZE_CHIP,
            ),
            (ids::PAINTER_BRUSH_JITTER, ids::PAINTER_BRUSH_JITTER_CHIP),
            (
                ids::PAINTER_BRUSH_JITTER_SCALE,
                ids::PAINTER_BRUSH_JITTER_SCALE_CHIP,
            ),
            (
                ids::PAINTER_BRUSH_JITTER_ROTATE,
                ids::PAINTER_BRUSH_JITTER_ROTATE_CHIP,
            ),
            (
                ids::PAINTER_BRUSH_JITTER_SPACING,
                ids::PAINTER_BRUSH_JITTER_SPACING_CHIP,
            ),
        ] {
            assert!(
                matches!(store.get(chip), Some(InteractiveState::NumberInput { .. })),
                "chip {chip:?} not registered as a NumberInput"
            );
            assert!(
                store.number_range(chip).is_some(),
                "chip {chip:?} has no set_number_range — its drag won't be range-proportional"
            );
            assert_eq!(
                store.linked_number(slider),
                Some(chip),
                "slider {slider:?} not linked to its chip {chip:?}"
            );
        }
    }
}
