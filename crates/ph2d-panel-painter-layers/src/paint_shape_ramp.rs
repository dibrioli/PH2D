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

const MOVE_W: f32 = 72.0; // LITERAL-PX-OK: the "Move Up" button — wide enough for its label

/// Paint the **Shape Color** section at `y`, returning the next `y` (prompt 38): "Use Color Ramp", the
/// section-wide **B&W** and **Alpha** rows, then one CARD per layer — **Texture Color** (the Shape ramp,
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
    let alpha = brush.shape_color_ramp_alpha_mode;
    let (ny, aberto) = crate::paint_brush_rows::paint_dropdown_row(
        ctx,
        theme,
        x,
        content_w,
        y,
        "panel.painter_layers.ramp.alpha",
        ids.alpha_mode,
        alpha,
        tr(RampAlphaMode::from_u8(alpha).name_key()),
    );
    if let Some(r) = aberto {
        state::set_pending_shape_ramp_alpha_dd(Some((r, alpha)));
    }
    y = ny;
    let count =
        (brush.shape_color_ramp_stop_count as usize).min(brush.shape_color_ramp_stops.len());
    let textura = RampView {
        enabled: true,
        bw: brush.shape_color_ramp_bw || bw_locked,
        bw_locked,
        bw_button: false,
        mode: brush.shape_color_ramp_mode,
        interp: brush.shape_color_ramp_interp,
        alpha_mode: alpha,
        stops: &brush.shape_color_ramp_stops[..count],
        selected_id: state::selected_shape_ramp_stop(),
    };
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
    let ambas = brush.shape_texture_color && brush.shape_falloff_color;
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
        let de_cima = n == 0;
        let com_blend = de_cima && ambas;
        let iw = crate::card::card_inner_w(content_w);
        let (linhas, corpo) = if ligada {
            (2 + usize::from(com_blend), ramp_body_height(iw, false))
        } else {
            (1, 0.0)
        };
        let (ix, iw, mut ry, proximo) = crate::card::card_frame_com_extra(
            ctx,
            theme,
            x,
            content_w,
            y,
            tr(titulo),
            linhas,
            corpo,
        );
        // The lower card's TITLE line carries "Move Up" (brings it to the top), so its rows keep the
        // section's column.
        if !de_cima {
            let h = ry - y;
            icon_button(
                ctx,
                theme,
                Rect::new(
                    ix + iw - MOVE_W,
                    y + Spacing::Xs.px(),
                    MOVE_W,
                    h - Spacing::Xs.px(),
                ),
                tr("panel.painter_layers.ramp.move_up"),
                ph2d_tool_painter::ids::PAINTER_SHAPE_COLOR_SWAP,
                false,
                false,
            );
        }
        ry = crate::paint_brush_top::paint_checkbox_row(
            ctx,
            theme,
            ix,
            iw,
            ry,
            id,
            "panel.painter_layers.ramp.enabled",
            ligada,
        );
        if ligada {
            ry = paint_ramp_body(ctx, theme, ix, iw, ry, &rids, view);
            ry = crate::card::card_row(
                ctx,
                theme,
                ix,
                iw,
                ry,
                "panel.painter_layers.ramp.opacity",
                id_op,
                opacidade.clamp(0.0, 1.0),
                0.0,
                1.0,
                crate::number_field::FINE_STEP,
                2,
            );
            if com_blend {
                let m = brush.shape_color_blend;
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
        }
        y = proximo;
    }
    crate::paint_brush_top::end_fold(ctx, fold, y)
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

#[cfg(test)]
mod tests {
    //! A secção Shape Color com as duas camadas (prompt 38) pinta e regista os controlos de cada estado —
    //! uma linha não pintada não regista hit rect, logo nenhum clique a alcança — e nenhum se sobrepõe.
    use ph2d_a11y::NodeId;
    use ph2d_editor_core::panel::{PaintCtx, PanelHostInternal};
    use ph2d_editor_core::screens::HeroLayout;
    use ph2d_editor_core::zones::Rect;
    use ph2d_text::TextSystem;
    use ph2d_tokens::Theme;
    use ph2d_tool_painter::{BrushSettings, PANEL_RAMP_STOPS, ids};
    use ph2d_ui_testkit::MockPanelHost;
    use ph2d_vector::VectorScene;

    /// O corpo do painel pintado com `brush` e a secção Shape Color aberta: cada id com o seu rect.
    fn hits(brush: BrushSettings) -> Vec<(NodeId, Rect)> {
        crate::state::set_current_brush(Some(brush));
        let mut host = MockPanelHost::with_panel::<crate::PainterLayersPanel>();
        host.store_mut()
            .set_collapsed(ids::PAINTER_SHAPE_RAMP_SECTION, false);
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let viewport = Rect::new(0.0, 0.0, 360.0, 8000.0);
        let layout = HeroLayout::for_viewport(viewport);
        {
            let mut ctx = PaintCtx {
                host: &mut host,
                layout: &layout,
                slot: layout
                    .slot_rects(ph2d_editor_core::screens::slot::SlotSet::ANY_DOCK)
                    .get(ph2d_editor_core::screens::slot::Slot::RightTop),
                viewport,
                scene: &mut scene,
                text_system: &mut text,
            };
            let rect = Rect::new(0.0, 0.0, 320.0, 8000.0);
            crate::paint_brush::paint_brush_body(&mut ctx, Theme::default(), rect, 0.0);
        }
        crate::state::set_current_brush(None);
        host.hit_index_mut().iter_registrations().collect()
    }

    fn base() -> BrushSettings {
        let mut stops = [[0.0f32; 6]; PANEL_RAMP_STOPS];
        stops[0] = [0.0, 1.0, 0.0, 0.0, 1.0, 0.0];
        stops[1] = [1.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        BrushSettings {
            shape_color_ramp_enabled: true,
            shape_color_ramp_stops: stops,
            shape_color_ramp_stop_count: 2,
            shape_falloff_ramp_stops: stops,
            shape_falloff_ramp_stop_count: 2,
            shape_falloff_color: false,
            ..crate::paint_brush::FALLBACK_BRUSH
        }
    }

    #[test]
    fn as_duas_camadas_pintam_os_controlos_de_cada_estado() {
        let tem = |h: &[(NodeId, Rect)], id| h.iter().any(|(i, _)| *i == id);
        let so_textura = hits(base());
        for id in [
            ids::PAINTER_SHAPE_TEXTURE_COLOR,
            ids::PAINTER_SHAPE_FALLOFF_COLOR,
            ids::PAINTER_SHAPE_COLOR_SWAP,
            ids::PAINTER_SHAPE_RAMP_BW,
            ids::PAINTER_SHAPE_RAMP_SWATCH,
        ] {
            assert!(tem(&so_textura, id), "Texture Color só: falta {id:?}");
        }
        for id in [
            ids::PAINTER_SHAPE_FALLOFF_RAMP_SWATCH,
            ids::PAINTER_SHAPE_COLOR_BLEND,
            ids::PAINTER_SHAPE_FALLOFF_OPACITY,
        ] {
            assert!(
                !tem(&so_textura, id),
                "Texture Color só: {id:?} não devia estar pintado"
            );
        }
        let ambas = hits(BrushSettings {
            shape_falloff_color: true,
            ..base()
        });
        let novos = [
            ids::PAINTER_SHAPE_TEXTURE_COLOR,
            ids::PAINTER_SHAPE_FALLOFF_COLOR,
            ids::PAINTER_SHAPE_COLOR_SWAP,
            ids::PAINTER_SHAPE_FALLOFF_RAMP_ADD,
            ids::PAINTER_SHAPE_FALLOFF_RAMP_SWATCH,
            ids::PAINTER_SHAPE_COLOR_BLEND,
            ids::PAINTER_SHAPE_TEXTURE_OPACITY,
            ids::PAINTER_SHAPE_FALLOFF_OPACITY,
            ids::PAINTER_SHAPE_RAMP_SWATCH,
        ];
        for id in novos {
            assert!(tem(&ambas, id), "as duas camadas: falta {id:?}");
        }
        let rects: Vec<Rect> = novos
            .iter()
            .filter_map(|id| ambas.iter().find(|(i, _)| i == id).map(|(_, r)| *r))
            .collect();
        for (a, ra) in rects.iter().enumerate() {
            for rb in &rects[a + 1..] {
                let cruza = ra.x < rb.x + rb.w
                    && rb.x < ra.x + ra.w
                    && ra.y < rb.y + rb.h
                    && rb.y < ra.y + ra.h;
                assert!(
                    !cruza,
                    "dois controlos das camadas sobrepõem-se: {ra:?} × {rb:?}"
                );
            }
        }
        let bw = hits(BrushSettings {
            shape_color_ramp_bw: true,
            shape_falloff_color: true,
            ..base()
        });
        assert!(
            tem(&bw, ids::PAINTER_SHAPE_RAMP_SWATCH),
            "B&W: o tom tem a sua rampa"
        );
        for id in [
            ids::PAINTER_SHAPE_TEXTURE_COLOR,
            ids::PAINTER_SHAPE_FALLOFF_COLOR,
            ids::PAINTER_SHAPE_COLOR_BLEND,
        ] {
            assert!(!tem(&bw, id), "B&W (tom): {id:?} não devia estar pintado");
        }
    }
}
