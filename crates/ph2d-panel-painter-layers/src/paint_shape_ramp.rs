//! The **Shape Color** section — the Shape's colour in two LAYERS (prompt 38): the **Texture Color**
//! (the Shape ramp, indexed by the Shape's texture value) and the **Falloff Color** (a second ramp,
//! indexed by the dab coverage), their order and the Blend between them; with **B&W** on the Shape ramp
//! is the silhouette's grayscale tone instead (auto-on with a Grain). The ramp editor body lives in the
//! shared [`crate::paint_ramp_widget`]; here we build the id-bundles + views, lay the layers out and
//! drain the section's deferred dropdown popovers.

use crate::paint_brush::paint_dropdown_popover;
use crate::paint_ramp_widget::{RampIds, RampView, icon_button, paint_ramp_body, ramp_body_height};
use crate::state;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::Spacing;
use ph2d_tool_painter::ids::{
    painter_shape_color_blend_option_id, painter_shape_falloff_ramp_interp_option_id,
    painter_shape_falloff_ramp_mode_option_id, painter_shape_ramp_handle_id,
    painter_shape_ramp_interp_option_id, painter_shape_ramp_mode_option_id,
};
use ph2d_tool_painter::{
    BrushBlend, BrushSettings, MAX_BRUSH_BLEND_MODES, RampColorMode, RampInterp,
};

/// The Shape ramp's fixed widget ids + state hooks (collapsible "Shape Color").
fn shape_ramp_ids() -> RampIds {
    RampIds {
        section: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_SECTION,
        reset: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_RESET,
        enable: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_ENABLE,
        mode: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_MODE,
        interp: ph2d_tool_painter::ids::PAINTER_SHAPE_RAMP_INTERP,
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
    }
}

/// The Falloff Color ramp's ids (prompt 38) — the section-level ids (section / reset / enable / B&W) are
/// the Shape ramp's: the layer body paints none of them.
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

const MOVE_W: f32 = 72.0; // LITERAL-PX-OK: the "Move Up" button — wide enough for its label

/// Paint the **Shape Color** section at `y`, returning the next `y` (prompt 38): "Use Color Ramp", the
/// section-wide **B&W** row, then one CARD per layer — **Texture Color** (the Shape ramp,
/// indexed by the Shape's texture) and **Falloff Color** (indexed by the coverage), in stack order (the
/// first card is on top). Each card: "Enabled" (the lower one also carries "Move Up"), and when on its
/// ramp editor + **Opacity**; the top card adds the **Blend** when both are on. With B&W on the ramp is
/// the silhouette's TONE: one ramp, no layers.
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
    // Smear/Blur/Clone/Eraser paint no colour → the ramp is a B&W coverage tone (forced: no B&W row).
    let bw_locked = brush.paints_no_color() || brush.eraser || brush.is_mask;
    if !bw_locked {
        y = crate::paint_brush_top::paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            ids.bw,
            "panel.painter_layers.ramp.bw",
            brush.shape_color_ramp_bw,
        );
    }
    let count =
        (brush.shape_color_ramp_stop_count as usize).min(brush.shape_color_ramp_stops.len());
    let textura = RampView {
        enabled: true,
        bw: brush.shape_color_ramp_bw || bw_locked,
        bw_locked,
        bw_button: false,
        mode: brush.shape_color_ramp_mode,
        interp: brush.shape_color_ramp_interp,
        alpha: None, // the colour's alpha always scales coverage (`BRUSH_RAMP_ALPHA`)
        stops: &brush.shape_color_ramp_stops[..count],
        selected_id: state::selected_shape_ramp_stop(),
    };
    // ⛔ Com B&W os DOIS cartões ficam (dono 15/10: «B&W está fechando um dos color ramps. Não permita»):
    // o tom é a pilha das duas camadas, mostradas em cinza (`textura.bw` viaja para a `queda`).
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
    // Sem textura no Shape a Texture Color não tem o que colorir (a ferramenta di-lo: `TexturaSemForma`):
    // o cartão dela fica INTEIRO esmaecido e sem clique, e o Blend some (só a Falloff pinta).
    let textura_inerte =
        state::current_inercias().inerte(ph2d_tool_painter::Dependente::TexturaSemForma);
    if textura_inerte {
        let store = ctx.host.store_mut();
        if store.picker_target() == Some(ids.swatch) {
            store.set_picker_target(None);
        }
    }
    let ambas = brush.shape_texture_color && !textura_inerte && brush.shape_falloff_color;
    for (n, camada) in ordem.into_iter().enumerate() {
        let (titulo, id, ligada, opacidade, id_op, view, rids) = match camada {
            Camada::Textura => (
                "panel.painter_layers.ramp.texture_color",
                ph2d_tool_painter::ids::PAINTER_SHAPE_TEXTURE_COLOR,
                brush.shape_texture_color,
                brush.shape_texture_opacity,
                ph2d_tool_painter::ids::PAINTER_SHAPE_TEXTURE_OPACITY,
                textura,
                ids,
            ),
            Camada::Queda => (
                "panel.painter_layers.ramp.falloff_color",
                ph2d_tool_painter::ids::PAINTER_SHAPE_FALLOFF_COLOR,
                brush.shape_falloff_color,
                brush.shape_falloff_opacity,
                ph2d_tool_painter::ids::PAINTER_SHAPE_FALLOFF_OPACITY,
                queda,
                falloff_ramp_ids(),
            ),
        };
        let cartao = Cartao {
            titulo,
            corpo: Corpo {
                id,
                ligada,
                rids,
                view,
                id_op,
                opacidade,
            },
            de_cima: n == 0,
            blend: (n == 0 && ambas).then_some(brush.shape_color_blend),
            inerte: camada == Camada::Textura && textura_inerte,
        };
        y = paint_layer_card(ctx, theme, x, content_w, y, &cartao);
    }
    crate::paint_brush_top::end_fold(ctx, fold, y)
}

/// What a layer card's body paints: "Enabled", and when on its ramp editor + Opacity.
struct Corpo<'a> {
    id: ph2d_a11y::NodeId,
    ligada: bool,
    rids: RampIds,
    view: RampView<'a>,
    id_op: ph2d_a11y::NodeId,
    opacidade: f32,
}

/// One layer card: its title, body, whether it is on top (the lower one carries "Move Up"), the Blend
/// it shows (the top card, with both layers on) and whether it is INERT (the Texture Color with no Shape
/// texture: the whole card dimmed, under a hit-less clip — 2.º smoke do dono, 15/10).
struct Cartao<'a> {
    titulo: &'static str,
    corpo: Corpo<'a>,
    de_cima: bool,
    blend: Option<u8>,
    inerte: bool,
}

/// Paint one layer [`Cartao`] at `y`, returning the next `y`.
fn paint_layer_card(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    c: &Cartao,
) -> f32 {
    let iw = crate::card::card_inner_w(content_w);
    let (linhas, extra) = if c.corpo.ligada {
        (
            2 + usize::from(c.blend.is_some()),
            ramp_body_height(iw, false),
        )
    } else {
        (1, 0.0)
    };
    let (ix, iw, ry, proximo) =
        crate::card::card_frame_com_extra(ctx, theme, x, content_w, y, tr(c.titulo), linhas, extra);
    // The lower card's TITLE line carries "Move Up" (brings it to the top), so its rows keep the
    // section's column.
    if !c.de_cima {
        let topo = y + Spacing::Xs.px();
        icon_button(
            ctx,
            theme,
            Rect::new(ix + iw - MOVE_W, topo, MOVE_W, ry - topo),
            tr("panel.painter_layers.ramp.move_up"),
            ph2d_tool_painter::ids::PAINTER_SHAPE_COLOR_SWAP,
            false,
            false,
        );
    }
    if c.inerte {
        let area = Rect::new(ix, ry, iw, (proximo - ry).max(0.0));
        ctx.host
            .hit_index_mut()
            .push_clip(Rect::new(ix, ry, 0.0, 0.0));
        crate::esmaecer::esmaecido(ctx, theme, area, |ctx| {
            pinta_corpo(ctx, theme, ix, iw, ry, &c.corpo);
        });
        ctx.host.hit_index_mut().pop_clip();
        return proximo;
    }
    let ry = pinta_corpo(ctx, theme, ix, iw, ry, &c.corpo);
    if let (Some(m), true) = (c.blend, c.corpo.ligada) {
        let (_, aberto) = crate::paint_brush_rows::paint_dropdown_row(
            ctx,
            theme,
            ix,
            iw,
            ry,
            "panel.painter_layers.ramp.blend",
            ph2d_tool_painter::ids::PAINTER_SHAPE_COLOR_BLEND,
            m,
            tr(BrushBlend::from_u8(m).name_key()),
        );
        if let Some(r) = aberto {
            crate::state_shape_cor::set_pending_blend_dd(Some((r, m)));
        }
    }
    proximo
}

/// A layer card's body without the Blend, at `ry`; returns the next `ry`.
fn pinta_corpo(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    ix: f32,
    iw: f32,
    ry: f32,
    c: &Corpo,
) -> f32 {
    let ry = crate::paint_brush_top::paint_checkbox_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        c.id,
        "panel.painter_layers.ramp.enabled",
        c.ligada,
    );
    if !c.ligada {
        return ry;
    }
    let ry = paint_ramp_body(ctx, theme, ix, iw, ry, &c.rids, c.view);
    crate::card::card_row(
        ctx,
        theme,
        ix,
        iw,
        ry,
        "panel.painter_layers.ramp.opacity",
        c.id_op,
        c.opacidade.clamp(0.0, 1.0),
        0.0,
        1.0,
        crate::number_field::FINE_STEP,
        2,
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

#[cfg(test)]
#[path = "paint_shape_ramp_tests.rs"]
mod tests;
