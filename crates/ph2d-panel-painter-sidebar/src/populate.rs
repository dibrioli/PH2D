//! Painter sidebar `populate` — pre-registers widget IDs no `WidgetStore`
//! ao boot do host (uma vez via `Panel::populate`).
//!
//! Seed values vêm de [`PainterUiSnapshot::default`] (que espelha
//! `PainterParams::default`); host overwrites cada frame do live snapshot.
//! Pattern espelha BgRemoval/Padding.
//!
//! **Audit (2026-05-28):**
//! - W-1/W-2/Z-1: chips são display-space (px / %) ⇒ link via
//!   `link_slider_number_mapped_integer` (NÃO identity `link_slider_number`),
//!   senão digitar no chip clampa o slider ao máximo (split-brain 2026-05-27).
//! - W-3: seed vem do `PainterUiSnapshot::default()`, não de literais ⇒ seed
//!   e o fallback do `paint` nunca divergem.
//! - Y-7/T2.2: undo/redo buttons só são registrados quando pintados (não
//!   nesta wave) — registrar sem paint deixa roteamento morto.
//! - T2.4: o modifier square JÁ é pintado (`paint::paint_modifier_square`),
//!   então é registrado aqui como `Button` (dispatcher rastreia hover/press
//!   + roteia o `Click` → `ToggleEyedropper`).

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_painter::{
    PainterUiSnapshot, opacity_chip_mapping, opacity01_to_pct, size_chip_mapping, size01_to_px,
};

pub fn populate(store: &mut WidgetStore) {
    // Close (X) button.
    store.register(
        ph2d_editor_core::ids::PAINTER_SIDEBAR_CLOSE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );

    // Modifier square (T2.4) — eyedropper-while-held affordance. Painted in
    // `paint::paint_modifier_square`, so registering it here is live (not
    // dead routing). Dispatcher tracks hover/press + emits its `Click`.
    store.register(
        ids::MODIFIER_SQUARE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );

    // Sliders + chips. Seed do snapshot default (SSOT), chip display em
    // unidade natural (px / %) via os helpers de conversão.
    let seed = PainterUiSnapshot::default();
    register_slider_chip_pair(
        store,
        ids::SIZE_SLIDER,
        ids::SIZE_CHIP,
        seed.size01,
        size01_to_px(seed.size01) as f64,
    );
    register_slider_chip_pair(
        store,
        ids::OPACITY_SLIDER,
        ids::OPACITY_CHIP,
        seed.opacity01,
        opacity01_to_pct(seed.opacity01) as f64,
    );

    // Link slider ↔ chip com a affine mapping correta (DIRETRIZ §5.2 regra
    // 1 + canon BgRemoval/Padding). `_integer` porque px e % são inteiros.
    let (size_scale, size_offset) = size_chip_mapping();
    store.link_slider_number_mapped_integer(
        ids::SIZE_SLIDER,
        ids::SIZE_CHIP,
        size_scale,
        size_offset,
    );
    let (op_scale, op_offset) = opacity_chip_mapping();
    store.link_slider_number_mapped_integer(
        ids::OPACITY_SLIDER,
        ids::OPACITY_CHIP,
        op_scale,
        op_offset,
    );
}

fn register_slider_chip_pair(
    store: &mut WidgetStore,
    slider_id: NodeId,
    chip_id: NodeId,
    slider_value: f32,
    chip_display_value: f64,
) {
    store.register(
        slider_id,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: slider_value,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        chip_id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: chip_display_value,
            buffer: format!("{chip_display_value:.3}"),
            caret: 0,
            last_committed: chip_display_value,
            selection_anchor: None,
        },
    );
}
