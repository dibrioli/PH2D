//! Os gates do [`super`] — separados por ARQUIVO (o teto de LOC do painel), ainda um `mod`
//! FILHO, para `use super::*` continuar alcançando o que o `populate` tem de privado.

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
        // Grid Stamp: cell size + lattice offset, per axis.
        (
            ids::PAINTER_BRUSH_GRID_CELL[0],
            ids::PAINTER_BRUSH_GRID_CELL_CHIPS[0],
        ),
        (
            ids::PAINTER_BRUSH_GRID_CELL[1],
            ids::PAINTER_BRUSH_GRID_CELL_CHIPS[1],
        ),
        (
            ids::PAINTER_BRUSH_GRID_OFFSET[0],
            ids::PAINTER_BRUSH_GRID_OFFSET_CHIPS[0],
        ),
        (
            ids::PAINTER_BRUSH_GRID_OFFSET[1],
            ids::PAINTER_BRUSH_GRID_OFFSET_CHIPS[1],
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
