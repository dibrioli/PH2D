//! Os **Dropdowns** de id fixo do painel (os chips de menu: abrir/fechar pelo despacho genérico, as opções são
//! botões do popover). Saíram de `populate.rs` pelo tecto de 600 LOC (só mover, 29/10).

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::DropdownState;

/// Regista cada chip de menu do painel como `Dropdown` — sem isto o chip pinta, regista o toque e fica morto.
pub(crate) fn register_dropdowns(store: &mut WidgetStore) {
    // Blend + Falloff + Stroke-Method + Jitter-Unit chips are Dropdowns (generic
    // open/close dispatch); their options are popover buttons.
    for id in [
        // The Paint Mode (medium) chip at the head of the appearance half.
        ph2d_tool_painter::ids::PAINTER_BRUSH_MEDIA,
        // Card Line: o chip `Type` dos tipos de traço procedural (plano 38 W2).
        ph2d_tool_painter::ids::PAINTER_LINE_TYPE,
        ph2d_tool_painter::ids::PAINTER_BRUSH_BLEND,
        ph2d_tool_painter::ids::PAINTER_BRUSH_FALLOFF,
        ph2d_editor_core::ids::PAINTER_BRUSH_STROKE_METHOD,
        ph2d_tool_painter::ids::PAINTER_BRUSH_JITTER_UNIT,
        // Shape section: source picker (None/Image) + the Follow dropdown (Off/Rake/Flow).
        ph2d_tool_painter::ids::PAINTER_SHAPE_KIND,
        ph2d_tool_painter::ids::PAINTER_SHAPE_FOLLOW,
        // Texture section: Kind picker + Mapping chips.
        ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_KIND,
        ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_MAPPING,
        // Grain Color Ramp: Mode + Interpolation chips, and a Texture LAYER's Alpha-action one.
        ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_RAMP_MODE,
        ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_RAMP_INTERP,
        ph2d_tool_painter::ids::PAINTER_BRUSH_TEXTURE_RAMP_ALPHA_MODE,
        // Shape Color Ramp: Mode + Interpolation chips.
        ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_MODE,
        ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_INTERP,
        // Shape Color layers: the Falloff ramp's Mode + Interpolation, the layers' Blend.
        ph2d_tool_painter::ids::PAINTER_SHAPE_FALLOFF_RAMP_MODE,
        ph2d_tool_painter::ids::PAINTER_SHAPE_FALLOFF_RAMP_INTERP,
        ph2d_tool_painter::ids::PAINTER_SHAPE_COLOR_BLEND,
    ]
    .into_iter()
    // O bloco de presets do Wet Painter (W1): Preset · política · os valores rápidos dos volumes.
    .chain(ph2d_tool_painter::ids::PAINTER_WETPAINT_PRESET_DROPDOWNS)
    // O Environment do cartão Real Oil (W1c).
    .chain([ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_AMBIENTE])
    // O cartão Pen: o menu da rotação e o do Reset (neutro · preset, dono 25/10).
    .chain([
        ph2d_tool_painter::ids::PAINTER_PEN_ROTATION,
        ph2d_tool_painter::ids::PAINTER_PEN_RESET,
    ])
    // Os menus da PONTA (L9/L18): a ponta de píxel, o modo do grão, o controlo da profundidade.
    .chain(
        (0..ph2d_tool_painter::N_LINHAS_DA_PONTA)
            .filter(|&i| crate::ponta_linhas::e_menu(i))
            .map(|i| ph2d_tool_painter::ids::PAINTER_PONTA_FIELDS[i]),
    ) {
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
