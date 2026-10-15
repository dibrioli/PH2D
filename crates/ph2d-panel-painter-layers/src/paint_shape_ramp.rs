//! The **Shape Color** section — the Shape's colour in two LAYERS (prompt 38): the **Texture Color**
//! (the Shape ramp, indexed by the Shape's texture value) and the **Falloff Color** (a second ramp,
//! indexed by the dab coverage), their order and the Blend between them; with **B&W** on the Shape ramp
//! is the silhouette's grayscale tone instead (auto-on with a Grain). The ramp editor body lives in the
//! shared [`crate::paint_ramp_widget`]; here we build the id-bundles + views, lay the layers out and
//! drain the section's deferred dropdown popovers.

use crate::paint_brush::paint_dropdown_popover;
use crate::paint_brush_rows::paint_dropdown_chip;
use crate::paint_ramp_widget::{BW_W, RampIds, RampView, bw_button, icon_button, paint_ramp_body};
use crate::state;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_painter::ids::{
    painter_shape_color_blend_option_id, painter_shape_falloff_ramp_interp_option_id,
    painter_shape_falloff_ramp_mode_option_id, painter_shape_ramp_alpha_option_id,
    painter_shape_ramp_handle_id, painter_shape_ramp_interp_option_id,
    painter_shape_ramp_mode_option_id,
};
use ph2d_tool_painter::{
    BrushBlend, BrushSettings, MAX_BRUSH_BLEND_MODES, RampAlphaMode, RampColorMode, RampInterp,
};

/// The Shape ramp's fixed widget ids + state hooks (collapsible "Shape Color").
fn shape_ramp_ids() -> RampIds {
    RampIds {
        section: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_SECTION,
        reset: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_RESET,
        enable: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_ENABLE,
        mode: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_MODE,
        interp: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_INTERP,
        alpha_mode: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_ALPHA_MODE,
        bw: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_BW,
        add: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_ADD,
        remove: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_REMOVE,
        invert: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_INVERT,
        edit: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_EDIT,
        swatch: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_SWATCH,
        stop_index: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_STOP_INDEX,
        stop_pos: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_STOP_POS,
        handle: painter_shape_ramp_handle_id,
        set_pending_mode: state::set_pending_shape_ramp_mode_dd,
        set_pending_interp: state::set_pending_shape_ramp_interp_dd,
        set_pending_alpha: state::set_pending_shape_ramp_alpha_dd,
    }
}

/// The Falloff Color ramp's ids (prompt 38) — the section-level ids (section / reset / enable / alpha /
/// B&W) are the Shape ramp's: the layer body paints none of them.
fn falloff_ramp_ids() -> RampIds {
    use ph2d_tool_painter::ids as i;
    RampIds {
        mode: i::PAINTER_SHAPE_FALLOFF_RAMP_MODE,
        interp: i::PAINTER_SHAPE_FALLOFF_RAMP_INTERP,
        add: i::PAINTER_SHAPE_FALLOFF_RAMP_ADD,
        remove: i::PAINTER_SHAPE_FALLOFF_RAMP_REMOVE,
        invert: i::PAINTER_SHAPE_FALLOFF_RAMP_INVERT,
        edit: i::PAINTER_SHAPE_FALLOFF_RAMP_EDIT,
        swatch: i::PAINTER_SHAPE_FALLOFF_RAMP_SWATCH,
        stop_index: i::PAINTER_SHAPE_FALLOFF_RAMP_STOP_INDEX,
        stop_pos: i::PAINTER_SHAPE_FALLOFF_RAMP_STOP_POS,
        handle: i::painter_shape_falloff_ramp_handle_id,
        set_pending_mode: crate::state_shape_cor::set_pending_falloff_mode_dd,
        set_pending_interp: crate::state_shape_cor::set_pending_falloff_interp_dd,
        ..shape_ramp_ids()
    }
}

/// The two layers of the section, in the order they read.
#[derive(Clone, Copy, PartialEq)]
enum Camada {
    Textura,
    Queda,
}

/// Paint the **Shape Color** section at `y`, returning the next `y` (prompt 38): "Use Color Ramp", the
/// section-wide B&W + Alpha line, then the two layers — **Texture Color** (the Shape ramp, indexed by
/// the Shape's texture) and **Falloff Color** (indexed by the coverage) — each with its checkbox and,
/// when on, its ramp editor; the first row carries the swap button; with both on, the top layer's
/// Blend mode + Opacity. With B&W on the ramp is the silhouette's TONE: one ramp, no layers.
pub(crate) fn paint_shape_ramp_section(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let ids = shape_ramp_ids();
    let (mut y, fold) = crate::paint_brush_top::paint_collapsible_section(
        ctx,
        theme,
        x,
        content_w,
        y,
        tr("panel.painter_layers.shape.shape_color"),
        ids.section,
        ids.reset,
    );
    let Some(fold) = fold else {
        return y;
    };
    y = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        ids.enable,
        "panel.painter_layers.ramp.use_color_ramp",
        brush.shape_color_ramp_enabled,
    );
    if !brush.shape_color_ramp_enabled {
        return crate::paint_brush_top::end_fold(ctx, fold, y); // no dead controls when off
    }
    // Smear/Blur/Clone/Eraser paint no colour → the ramp is a B&W coverage tone; force B&W checked + locked.
    let bw_locked = brush.paints_no_color() || brush.eraser || brush.is_mask;
    let count =
        (brush.shape_color_ramp_stop_count as usize).min(brush.shape_color_ramp_stops.len());
    let textura = RampView {
        enabled: true,
        bw: brush.shape_color_ramp_bw || bw_locked,
        bw_locked,
        bw_button: false,
        mode: brush.shape_color_ramp_mode,
        interp: brush.shape_color_ramp_interp,
        alpha_mode: brush.shape_color_ramp_alpha_mode,
        stops: &brush.shape_color_ramp_stops[..count],
        selected_id: state::selected_shape_ramp_stop(),
    };
    // The section-wide line: B&W + the Alpha action.
    let gap = Spacing::Xs.px();
    bw_button(ctx, theme, Rect::new(x, y, BW_W, ROW_H_PX), ids.bw, textura);
    let alpha_rect = Rect::new(
        x + BW_W + gap,
        y,
        (content_w - BW_W - gap).max(0.0),
        ROW_H_PX,
    );
    if paint_dropdown_chip(
        ctx,
        theme,
        ids.alpha_mode,
        textura.alpha_mode,
        tr(RampAlphaMode::from_u8(textura.alpha_mode).name_key()),
        alpha_rect,
    ) {
        state::set_pending_shape_ramp_alpha_dd(Some((alpha_rect, textura.alpha_mode)));
    }
    y += ph2d_tokens::row_pitch_px();
    if textura.bw {
        y = paint_ramp_body(ctx, theme, x, content_w, y, &ids, textura);
        return crate::paint_brush_top::end_fold(ctx, fold, y);
    }
    let qcount =
        (brush.shape_falloff_ramp_stop_count as usize).min(brush.shape_falloff_ramp_stops.len());
    let queda = RampView {
        mode: brush.shape_falloff_ramp_mode,
        interp: brush.shape_falloff_ramp_interp,
        stops: &brush.shape_falloff_ramp_stops[..qcount],
        selected_id: crate::state_shape_cor::selected_falloff_stop(),
        ..textura
    };
    let ordem = if brush.shape_falloff_first {
        [Camada::Queda, Camada::Textura]
    } else {
        [Camada::Textura, Camada::Queda]
    };
    for (n, camada) in ordem.into_iter().enumerate() {
        let (id, chave, ligada) = match camada {
            Camada::Textura => (
                ph2d_tool_painter::ids::PAINTER_SHAPE_TEXTURE_COLOR,
                "panel.painter_layers.ramp.texture_color",
                brush.shape_texture_color,
            ),
            Camada::Queda => (
                ph2d_tool_painter::ids::PAINTER_SHAPE_FALLOFF_COLOR,
                "panel.painter_layers.ramp.falloff_color",
                brush.shape_falloff_color,
            ),
        };
        // The first row carries the swap (which layer is on top) at its right.
        let troca = n == 0;
        let w_caixa = if troca {
            content_w - BW_W - gap
        } else {
            content_w
        };
        if troca {
            icon_button(
                ctx,
                theme,
                Rect::new(x + content_w - BW_W, y, BW_W, ROW_H_PX),
                tr("panel.painter_layers.ramp.swap"),
                ph2d_tool_painter::ids::PAINTER_SHAPE_COLOR_SWAP,
                false,
                false,
            );
        }
        y = crate::paint_brush_top::paint_checkbox_row(
            ctx, theme, x, w_caixa, y, id, chave, ligada,
        );
        if ligada {
            y = match camada {
                Camada::Textura => paint_ramp_body(ctx, theme, x, content_w, y, &ids, textura),
                Camada::Queda => {
                    paint_ramp_body(ctx, theme, x, content_w, y, &falloff_ramp_ids(), queda)
                }
            };
        }
    }
    if brush.shape_texture_color && brush.shape_falloff_color {
        y = paint_blend_rows(ctx, theme, x, content_w, y, brush);
    }
    crate::paint_brush_top::end_fold(ctx, fold, y)
}

/// The top layer's **Blend** mode chip and its **Opacity** field.
fn paint_blend_rows(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: BrushSettings,
) -> f32 {
    let r = Rect::new(x, y, content_w, ROW_H_PX);
    let m = brush.shape_color_blend;
    if paint_dropdown_chip(
        ctx,
        theme,
        ph2d_tool_painter::ids::PAINTER_SHAPE_COLOR_BLEND,
        m,
        tr(BrushBlend::from_u8(m).name_key()),
        r,
    ) {
        crate::state_shape_cor::set_pending_blend_dd(Some((r, m)));
    }
    let sec = crate::seccoes::seccao_da_chave(ctx.text_system, "panel.painter_layers.ramp.opacity");
    crate::number_field::paint_num_row(
        ctx,
        theme,
        x,
        content_w,
        y + ph2d_tokens::row_pitch_px(),
        tr("panel.painter_layers.ramp.opacity"),
        ph2d_tool_painter::ids::PAINTER_SHAPE_COLOR_MIX,
        brush.shape_color_mix.clamp(0.0, 1.0),
        0.0,
        1.0,
        crate::number_field::FINE_STEP,
        2,
        sec,
    )
}

/// Deferred paint of the Shape ramp's open Mode / Interpolation / Alpha dropdown popovers.
pub(crate) fn paint_shape_ramp_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip, cur)) = state::take_pending_shape_ramp_mode_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_MODE,
            mode_options(),
            chip,
            cur,
        );
    }
    if let Some((chip, cur)) = state::take_pending_shape_ramp_interp_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_INTERP,
            interp_options(),
            chip,
            cur,
        );
    }
    if let Some((chip, cur)) = crate::state_shape_cor::take_pending_falloff_mode_dd() {
        let opts = (0..RampColorMode::COUNT)
            .map(|m| {
                DropdownOption::new(
                    painter_shape_falloff_ramp_mode_option_id(m),
                    m,
                    tr(RampColorMode::from_u8(m).label_key()),
                )
            })
            .collect();
        let id = ph2d_tool_painter::ids::PAINTER_SHAPE_FALLOFF_RAMP_MODE;
        paint_dropdown_popover(ctx, theme, id, opts, chip, cur);
    }
    if let Some((chip, cur)) = crate::state_shape_cor::take_pending_falloff_interp_dd() {
        let opts = (0..RampInterp::COUNT)
            .map(|i| {
                DropdownOption::new(
                    painter_shape_falloff_ramp_interp_option_id(i),
                    i,
                    tr(RampInterp::from_u8(i).label_key()),
                )
            })
            .collect();
        let id = ph2d_tool_painter::ids::PAINTER_SHAPE_FALLOFF_RAMP_INTERP;
        paint_dropdown_popover(ctx, theme, id, opts, chip, cur);
    }
    if let Some((chip, cur)) = crate::state_shape_cor::take_pending_blend_dd() {
        let opts = (0..MAX_BRUSH_BLEND_MODES)
            .map(|m| {
                DropdownOption::new(
                    painter_shape_color_blend_option_id(m),
                    m,
                    tr(BrushBlend::from_u8(m).name_key()),
                )
            })
            .collect();
        let id = ph2d_tool_painter::ids::PAINTER_SHAPE_COLOR_BLEND;
        paint_dropdown_popover(ctx, theme, id, opts, chip, cur);
    }
    if let Some((chip, cur)) = state::take_pending_shape_ramp_alpha_dd() {
        paint_dropdown_popover(
            ctx,
            theme,
            ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_ALPHA_MODE,
            alpha_options(),
            chip,
            cur,
        );
    }
}

fn mode_options() -> Vec<DropdownOption<u8>> {
    (0..RampColorMode::COUNT)
        .map(|m| {
            DropdownOption::new(
                painter_shape_ramp_mode_option_id(m),
                m,
                tr(RampColorMode::from_u8(m).label_key()),
            )
        })
        .collect()
}

fn interp_options() -> Vec<DropdownOption<u8>> {
    (0..RampInterp::COUNT)
        .map(|i| {
            DropdownOption::new(
                painter_shape_ramp_interp_option_id(i),
                i,
                tr(RampInterp::from_u8(i).label_key()),
            )
        })
        .collect()
}

fn alpha_options() -> Vec<DropdownOption<u8>> {
    (0..RampAlphaMode::COUNT)
        .map(|m| {
            DropdownOption::new(
                painter_shape_ramp_alpha_option_id(m),
                m,
                ph2d_i18n::tr(RampAlphaMode::from_u8(m).name_key()),
            )
        })
        .collect()
}
