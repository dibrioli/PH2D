//! Os gates do [`super`] — separados por ARQUIVO (o teto de LOC do painel), ainda um `mod`
//! FILHO, para `use super::*` continuar alcançando o que o `populate` tem de privado.

use super::*;

/// Regression: every plain-action toolbar button that `paint` paints + hit-registers MUST get a
/// store slot here, or the dispatch drops its click (the dead-button class — a missing
/// `PAINTER_LAYERS_ADD_TEXTURE` slot made the whole Texture-layer feature unreachable).
#[test]
fn action_toolbar_buttons_have_store_slots() {
    let mut store = WidgetStore::with_capacity(32);
    populate(&mut store);
    for id in [
        ph2d_tool_painter::ids::PAINTER_LAYERS_ADD,
        ph2d_tool_painter::ids::PAINTER_LAYERS_GROUP,
        ph2d_tool_painter::ids::PAINTER_LAYERS_DUPLICATE,
        ph2d_tool_painter::ids::PAINTER_LAYERS_DELETE,
        ph2d_tool_painter::ids::PAINTER_LAYERS_ADD_TEXTURE,
        ph2d_tool_painter::ids::PAINTER_LAYERS_MASK,
        ph2d_tool_painter::ids::PAINTER_LAYERS_CLIP,
        // Randomize Color enable + its H/S/V + Jitter Scale/Rotate sliders — a missing slot here
        // is the dead-control class (the click/drag would be silently dropped).
        ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_ENABLE,
        ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_HUE,
        ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_SAT,
        ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_VAL,
        ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER_SCALE,
        ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER_ROTATE,
        ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER_SPACING,
        // Seamless Tiling toggles + Repeat-Image preview.
        ph2d_tool_painter::ids::PAINTER_BRUSH_TILING_X,
        ph2d_tool_painter::ids::PAINTER_BRUSH_TILING_Y,
        ph2d_tool_painter::ids::PAINTER_BRUSH_REPEAT_IMAGE,
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
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_SIZE_SLIDER,
            ph2d_tool_painter::ids::PAINTER_BRUSH_SIZE_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_STRENGTH_SLIDER,
            ph2d_tool_painter::ids::PAINTER_BRUSH_STRENGTH_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_HUE,
            ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_HUE_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_SAT,
            ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_SAT_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_VAL,
            ph2d_tool_painter::ids::PAINTER_BRUSH_COLOR_JITTER_VAL_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_RATE,
            ph2d_tool_painter::ids::PAINTER_BRUSH_RATE_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_SPACING,
            ph2d_tool_painter::ids::PAINTER_BRUSH_SPACING_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_OFFSET,
            ph2d_tool_painter::ids::PAINTER_BRUSH_OFFSET_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_DASH_RATIO,
            ph2d_tool_painter::ids::PAINTER_BRUSH_DASH_RATIO_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_DASH_LENGTH,
            ph2d_tool_painter::ids::PAINTER_BRUSH_DASH_LENGTH_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_INPUT_SAMPLES,
            ph2d_tool_painter::ids::PAINTER_BRUSH_INPUT_SAMPLES_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_STABILIZE,
            ph2d_tool_painter::ids::PAINTER_BRUSH_STABILIZE_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER,
            ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER_SCALE,
            ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER_SCALE_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER_ROTATE,
            ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER_ROTATE_CHIP,
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER_SPACING,
            ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER_SPACING_CHIP,
        ),
        // Grid Stamp: cell size + lattice offset, per axis.
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_CELL[0],
            ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_CELL_CHIPS[0],
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_CELL[1],
            ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_CELL_CHIPS[1],
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_OFFSET[0],
            ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_OFFSET_CHIPS[0],
        ),
        (
            ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_OFFSET[1],
            ph2d_tool_painter::ids::PAINTER_BRUSH_GRID_OFFSET_CHIPS[1],
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
    // ⭐ **E as barras do card Line — da TABELA, não desta lista** (2026-09-16). Elas eram as únicas
    //    barras de pincel sem chip, e a lista acima nunca as conheceu: *uma lista escrita à mão mede
    //    os sítios de que já se suspeita*.
    let barras = crate::line_barras::todas();
    assert!(
        barras.len() >= 12,
        "a tabela do card Line tem {} barras distintas — encolheu, ou a varredura partiu-se",
        barras.len()
    );
    for b in barras {
        assert!(
            matches!(store.get(b.slider), Some(InteractiveState::Slider { .. })),
            "a barra {} não tem slider registado",
            b.chave
        );
        assert!(
            matches!(
                store.get(b.chip),
                Some(InteractiveState::NumberInput { .. })
            ),
            "a barra {} não tem chip registado",
            b.chave
        );
        assert!(
            store.number_range(b.chip).is_some(),
            "o chip da barra {} não tem faixa",
            b.chave
        );
        assert_eq!(
            store.linked_number(b.slider),
            Some(b.chip),
            "a barra {} não liga o slider ao chip dela",
            b.chave
        );
    }
}
