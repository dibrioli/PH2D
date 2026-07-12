//! The **relief line** of a Painter layer row — Impasto depth + how it meets the relief below
//! (`docs/Painter/16_impasto_plano_implementacao.md` §10.8). Sibling of [`crate::paint_rows`], split
//! out for the panel LOC cap; the row painter calls [`paint_relief_line`].
//!
//! It is line 3 of a row, and it exists **only on layers that carry relief** (`Layer::has_relief`,
//! projected by the tool from its height map). So a document nobody has sculpted shows none of this
//! anywhere in the panel — and a layer that has been sculpted shows it forever, which is the point: the
//! brush's own Depth is baked into each stroke as it lands and afterwards reaches only the LAST one.
//! This is the one live handle on everything ever laid down on the layer.

use crate::paint::register_button;
use ph2d_editor_core::ids::{PainterLayerWidget, painter_layer_widget_id};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    ButtonState, Slider, SliderState, flat_button_surface, paint_slider,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, StrokeToken};
use ph2d_tool_painter::{Layer, ReliefComposite};

/// The relief-composite chip column ("Add" / "Level").
const MODE_CHIP_W: f32 = 46.0; // LITERAL-PX-OK: two-word chip at the Base font

// The Impasto-depth track. A DOMAIN range (a signed unit depth), not a design value: negative inverts
// the relief, so the slider's centre is zero and the two halves mean opposite things.
const DEPTH_MIN: f32 = -1.0; // LITERAL-PX-OK: signed unit depth — negative inverts the relief
const DEPTH_MAX: f32 = 1.0; // LITERAL-PX-OK: signed unit depth

/// Line 3 of a relief-bearing row: **Impasto depth** + how it meets the relief below.
///
/// The layout is line 2's, deliberately: a bare slider, a plain readout, one chip. The depth of a
/// layer's paint is the opacity of its thickness — it belongs beside opacity, in the same shape, not in
/// a modal panel somewhere else. And the composite chip is the blend-mode chip's true sibling: blend
/// mode says how this layer's COLOUR meets the layer below; this says how its HEIGHT does.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_relief_line(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    layer: &Layer,
    x: f32,
    content_right: f32,
    y: f32,
    font: f32,
    cell_gap: f32,
) {
    let id = layer.id;
    // ── The slider is SIGNED, so one control does Add / Ignore / Subtract. The plan named those as
    // three modes of a composite enum; they are three readings of one number (`+` piles, `0` mutes, `-`
    // carves), and a mode enum that duplicates a slider is exactly the second gain this section already
    // had to kill once (the old "Amount"). What is left in the enum is the one thing a scale cannot
    // say: **Level**.
    let slider_id = painter_layer_widget_id(id.0, PainterLayerWidget::ImpastoDepth);
    let norm = (layer.impasto_depth - DEPTH_MIN) / (DEPTH_MAX - DEPTH_MIN);
    crate::paint_rows::register_bare_slider(ctx.host.store_mut(), slider_id, norm);
    let slider_w =
        (content_right - x - crate::paint_rows::OPACITY_PCT_W - MODE_CHIP_W - cell_gap * 2.0)
            .max(0.0);
    let st = ctx
        .host
        .store()
        .slider(slider_id)
        .map(|(s, _)| s)
        .unwrap_or(SliderState::Normal);
    let mut slider = Slider::new(slider_id, "").accent(true).state(st);
    slider.value = norm;
    let track = Rect::new(x, y, slider_w, ROW_H_PX);
    paint_slider(&slider, track, ctx.scene, theme);
    ctx.host.hit_index_mut().register(slider_id, track);

    let pct = (layer.impasto_depth * crate::paint_rows::PCT_SCALE).round();
    paint_text(
        ctx.text_system,
        ctx.scene,
        &format!("{pct:.0}%"),
        x + slider_w + cell_gap,
        y + (ROW_H_PX - font) * 0.5,
        font,
        crate::paint_rows::OPACITY_PCT_W,
        resolve(ColorToken::Text2, theme),
    );

    // ── The composite chip: a click cycles Add ↔ Level. Two states, so it names the state it is IN
    // (like the blend chip), never the one it would go to.
    let chip_id = painter_layer_widget_id(id.0, PainterLayerWidget::ImpastoLevel);
    let chip = Rect::new(
        x + slider_w + cell_gap + crate::paint_rows::OPACITY_PCT_W + cell_gap,
        y,
        MODE_CHIP_W,
        ROW_H_PX,
    );
    let leveling = matches!(layer.impasto_composite, ReliefComposite::Level);
    fill_rounded_rect(
        ctx.scene,
        chip,
        Radius::Sm.px(),
        resolve(flat_button_surface(ButtonState::Normal), theme),
    );
    if leveling {
        stroke_rounded_rect(
            ctx.scene,
            chip,
            Radius::Sm.px(),
            StrokeToken::Default.px(),
            resolve(ColorToken::Accent, theme),
        );
    }
    paint_text(
        ctx.text_system,
        ctx.scene,
        if leveling { "Level" } else { "Add" },
        chip.x + cell_gap,
        chip.y + (ROW_H_PX - font) * 0.5,
        font,
        MODE_CHIP_W,
        resolve(
            if leveling {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            },
            theme,
        ),
    );
    register_button(ctx.host.store_mut(), chip_id);
    ctx.host.hit_index_mut().register(chip_id, chip);
}
