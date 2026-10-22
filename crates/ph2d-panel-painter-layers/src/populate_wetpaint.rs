//! A secção **Wet Paint** do `populate` (filho de `populate.rs`, o tecto de 600 LOC dos painéis): os cliques, a
//! superfície do disco do Tilt e a dica da Live Water.

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::ButtonState;
use ph2d_i18n::tr;

/// A secção **Wet Paint** (doc 22 + W1g): os cliques, a superfície do disco do Tilt e a dica da Live
/// Water — partida de `populate::register_toggles_and_dropdowns` pelo tecto de 200 LOC de função dos painéis.
pub(crate) fn register_wetpaint(store: &mut WidgetStore) {
    // Wet Paint section (doc 22): Enable + reset + the 7 tool buttons + the
    // tilt toggle + the canvas actions + Show Wet + Paper + Tuning — ONE
    // membership list shared with the paint and the click forward.
    for id in ph2d_tool_painter::ids::PAINTER_WETPAINT_CLICKS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // The TILT pad is a CurvePoint drag surface; the paint refreshes the
    // carried canvas rect per frame — this registration is the focusability
    // half (the wiring-parity law: painted + hit-indexed but absent here =
    // dead under the mouse).
    store.register(
        ph2d_tool_painter::ids::PAINTER_WETPAINT_TILT_PAD,
        InteractiveState::CurvePoint {
            parent: ph2d_tool_painter::ids::PAINTER_WETPAINT_TILT_PAD,
            channel: 0,
            index: 0,
            canvas: ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1.0, 1.0),
        },
    );
    // O pino da preview viva (um Toggle: o despacho genérico vira-o).
    crate::preview_pino::regista(store);
    // A dica da Live Water (W1g): o custo medido de a ligar.
    store.set_tooltip(
        ph2d_tool_painter::ids::PAINTER_WETPAINT_LIVE_WATER,
        tr("panel.painter_layers.wetpaint.live_water_tip"),
    );
}
