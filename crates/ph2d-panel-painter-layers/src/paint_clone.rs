//! The **Clone** card (shown only in Clone mode): a "Set Source" button that arms the on-canvas
//! sample pick, an "Aligned" toggle (offset persists across strokes), and a one-line hint until a
//! source is sampled. The fixed-id widgets are registered in `crate::populate` and their clicks
//! forwarded by `crate::event`'s brush-click whitelist. Split from `paint_brush` for the LOC cap.

use crate::paint::register_button;
use crate::paint_brush_top::paint_checkbox_row;
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{Button, ButtonKind, paint_button};
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, ROW_H_PX, TypeToken};
use ph2d_tool_painter::BrushSettings;

/// Paint the Clone card and return the next `y`. Only called in Clone mode (`brush.is_clone`).
pub(crate) fn paint_clone_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    // "Set Source" button — arms the pick mode; the next canvas click samples the source anchor. Reads
    // its store state so hover / mouse-down feedback shows, and paints Accent while armed (stays
    // "checked" until the canvas click samples the source) — mirror of the Symmetry pick buttons.
    let id = ph2d_tool_painter::ids::PAINTER_BRUSH_CLONE_SET_SOURCE;
    let label = if brush.clone_sample_armed {
        tr("panel.painter_layers.clone.sample_hint")
    } else if brush.clone_has_source {
        tr("panel.painter_layers.clone.set_source_resample")
    } else {
        tr("panel.painter_layers.clone.set_source")
    };
    let state = ctx.host.store().button_visual(id);
    let kind = if brush.clone_sample_armed {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let btn = Button::new(id, label).kind(kind).visual(state);
    let rect =
        ph2d_editor_core::property_row::caixa_do_botao(ctx.text_system, x, content_w, y, label);
    paint_button(&btn, rect, ctx.scene, ctx.text_system, theme);
    register_button(ctx.host.store_mut(), id);
    ctx.host.hit_index_mut().register(id, rect);
    let mut y = y + ph2d_tokens::row_pitch_px();

    // "Aligned" toggle — keep the source→dest offset fixed across strokes.
    y = paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        ph2d_tool_painter::ids::PAINTER_BRUSH_CLONE_ALIGNED,
        "panel.painter_layers.clone.aligned",
        brush.clone_aligned,
    );

    // Hint until a source is sampled.
    if !brush.clone_has_source {
        let font = TypeToken::Sm.px();
        paint_text(
            ctx.text_system,
            ctx.scene,
            tr("panel.painter_layers.clone.hint"),
            x,
            y + (ROW_H_PX - font) * 0.5,
            font,
            content_w,
            resolve(ColorToken::Text2, theme),
        );
        y += ROW_H_PX;
    }
    y
}
