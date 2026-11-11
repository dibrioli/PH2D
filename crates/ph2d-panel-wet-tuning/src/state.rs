//! Panel state: the [`BrushSettings`] snapshot the shell publishes each frame
//! (the painter-layers `set_current_brush` pattern — the wet knob table, the
//! flags and the Paper-slot arm all ride inside it).

use ph2d_tool_painter::BrushSettings;
use std::cell::RefCell;

thread_local! {
    /// Live snapshot published by the shell before each `paint`. `None` until
    /// the first push — the panel then paints the defaults (which equal the
    /// engine's boot, gate-pinned in the tool).
    static CURRENT: RefCell<Option<BrushSettings>> = const { RefCell::new(None) };
    /// O espelho da barra ainda por chegar depois de uma edição no chip (B35.4): `(barra, valor enviado)`.
    static ESPELHO: std::cell::Cell<Option<(ph2d_a11y::NodeId, f64)>> = const { std::cell::Cell::new(None) };
}

/// Folga, em ulps de `f32` sobre a faixa, da ida e volta valor → pista → valor do espelho.
const ULPS_DO_ESPELHO: f64 = 8.0; // LITERAL-PX-OK: numeric tolerance (math, not design)

/// Lembra que o chip acabou de enviar o valor exacto; o `ValueChanged` da barra que o despacho espelha a seguir
/// (`push_mirrored_slider_event`) é a mesma edição.
pub(crate) fn espera_espelho(slider: ph2d_a11y::NodeId, valor: f64) {
    ESPELHO.with(|e| e.set(Some((slider, valor))));
}

/// `true` = este `ValueChanged` da barra é o espelho da edição do chip (consome a espera). O espelho faz a conta em
/// `f32` (`apply_chip_value_with_mirror`): casa-se dentro da precisão dela sobre a faixa, nunca ao bit.
pub(crate) fn e_o_espelho(row: &crate::rows::TuneRow, pista: f32) -> bool {
    ESPELHO.with(|e| e.take()).is_some_and(|(s, v)| {
        s == row.slider
            && (row.value_of(pista) - v).abs()
                <= f64::from(f32::EPSILON) * ULPS_DO_ESPELHO * (row.max - row.min)
    })
}
/// Zero-size panel state (everything lives in the store + the snapshot).
#[derive(Default)]
pub struct WetTuningPanelState;

/// Publish this frame's brush snapshot (the shell's bridge calls this beside
/// the painter-layers publish — same source, same cadence).
pub fn set_current_brush(brush: Option<BrushSettings>) {
    CURRENT.with(|c| *c.borrow_mut() = brush);
}

/// The snapshot the paint/event halves read. With nothing published yet the
/// panel paints a default tool's settings — the engine boot, which is what
/// an untouched section authors anyway.
pub(crate) fn current() -> BrushSettings {
    CURRENT
        .with(|c| *c.borrow())
        .unwrap_or_else(|| ph2d_tool_painter::PainterTool::default().brush_settings())
}
