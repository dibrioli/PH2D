//! A amostra da cor SECUNDÁRIA (L11) → o seletor OKLCH partilhado, como as amostras do Impasto: o clique abre-o
//! (ou fecha) apontado para ela, semeado com a cor que ela mostra; a escolha volta por `SelectOption` pintado em
//! [`crate::marca_cor`].

use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::panel::PanelHostInternal;

/// A amostra que este módulo serve.
pub(super) fn is_marca_swatch(id: NodeId) -> bool {
    id == ph2d_tool_painter::ids::PAINTER_MARCA_SECONDARY
}

/// O clique na amostra.
pub(super) fn on_swatch_click(host: &mut dyn PanelHostInternal, id: NodeId) {
    if host.store().picker_target() == Some(id) {
        host.store_mut().set_picker_target(None);
        return;
    }
    let enc = |c: f32| (c.clamp(0.0, 1.0) * 255.0 + 0.5) as u8; // LITERAL-PX-OK: sRGB 8-bit normalize
    // Sem instantâneo ainda: a secundária de fábrica (BRANCO, `CorDaMao::NEUTRA`).
    let rgba = state::current_brush()
        .map(|b| {
            let c = b.marca_secundaria;
            [enc(c[0]), enc(c[1]), enc(c[2]), 255u8]
        })
        .unwrap_or([255, 255, 255, 255]);
    let store = host.store_mut();
    store.set_blender_value(
        core_ids::INSP_BLENDER_PICKER,
        ph2d_tokens::ColorValue::from_rgba8(rgba[0], rgba[1], rgba[2], rgba[3]),
    );
    store.set_widget_color(id, rgba);
    store.set_picker_target(Some(id));
}
