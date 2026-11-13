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
    // + a janela de confirmação do Delete do preset (só do painel: o tool vê o Delete confirmado).
    for id in ph2d_tool_painter::ids::PAINTER_WETPAINT_CLICKS
        .into_iter()
        .chain(ph2d_tool_painter::ids::PAINTER_WETPAINT_PRESET_DELETE_DIALOG)
        .chain([ph2d_tool_painter::ids::PAINTER_PERFORMANCE_PALPITE])
    {
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
    store.set_tooltip(
        ph2d_tool_painter::ids::PAINTER_WETPAINT_PIGMENT_MIXING,
        tr("panel.painter_layers.wetpaint.pigment_mixing_tip"),
    );
    // O Wet bloom e o Tip pickup só agem sobre a tinta MOLHADA de outra cor (dono 25/10, R4: «sem efeito»); o
    // Diffusion leva o nome inteiro, «Pigment diffusion».
    for (id, dica) in [
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_DIFFUSION,
            "panel.painter_wet_cartao.diffusion_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_WET_BLOOM,
            "panel.painter_wet_cartao.wet_bloom_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_TIP_PICKUP,
            "panel.painter_wet_cartao.tip_pickup_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_REWET,
            "panel.painter_wet_cartao.rewet_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_REWET_DRAG,
            "panel.painter_wet_cartao.rewet_drag_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_SOLUBILITY,
            "panel.painter_wet_cartao.solubility_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_STAINING,
            "panel.painter_wet_cartao.staining_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_GLAZE,
            "panel.painter_wet_cartao.glaze_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_COVER,
            "panel.painter_wet_cartao.cover_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_DRY_COVER,
            "panel.painter_wet_cartao.dry_cover_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_WATER_DRAG,
            "panel.painter_wet_cartao.water_drag_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_WET_SOFTEN,
            "panel.painter_wet_cartao.wet_soften_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_DRAG,
            "panel.painter_wet_cartao.drag_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_EMBOSS,
            "panel.painter_wet_cartao.emboss_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_OPAQUE,
            "panel.painter_wet_cartao.opaque_tip",
        ),
        // Os controlos de lei da E10 (F3 §9).
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_MEDIUM,
            "panel.painter_wet_cartao.paint_type_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_DILUTION,
            "panel.painter_wet_cartao.dilution_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_VOL_OPACITY,
            "panel.painter_wet_cartao.volume_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_BODY,
            "panel.painter_wet_cartao.body_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_SMOOTH_EDGES,
            "panel.painter_wet_cartao.smooth_edges_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_HARDNESS,
            "panel.painter_wet_cartao.hardness_tip",
        ),
        (
            ph2d_tool_painter::ids::PAINTER_WETPAINT_ABSORBENCY,
            "panel.painter_wet_cartao.absorbency_tip",
        ),
    ] {
        store.set_tooltip(id, tr(dica));
    }
}
