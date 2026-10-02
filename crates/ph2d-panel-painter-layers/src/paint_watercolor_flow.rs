//! As linhas da **forma da borda** no cartão Wash (BUGS_painter #31): o padrão de **Flow** do Ragged
//! Edge, o Size e o Angle dele (só com um padrão — o Classic não tem escala nem direção), e o **Paper
//! Edge**. Irmão de `paint_watercolor.rs` pelo tecto de LOC do painel.

use crate::card::card_row;
use crate::paint_brush_rows::paint_dropdown_row;
use crate::{number_field, state};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::DropdownOption;
use ph2d_tool_painter::{BrushSettings, FLOW_KINDS, FLOW_SIZE_MAX, FLOW_SIZE_MIN, TextureKind};

const ANGLE_MAX: f32 = 360.0; // LITERAL-PX-OK: Flow Angle range (degrees)

/// Quantas linhas estas pintam — o `card_frame` dimensiona a moldura por este número.
pub(crate) fn flow_row_count(brush: &BrushSettings) -> usize {
    if classic(brush) { 2 } else { 4 }
}

fn classic(brush: &BrushSettings) -> bool {
    TextureKind::from_u8(brush.flow_kind) == TextureKind::None
}

/// O rótulo de uma opção do Flow: `None` é o **Classic**, o resto é o nome do padrão.
fn rotulo(k: TextureKind) -> &'static str {
    if k == TextureKind::None {
        ph2d_i18n::tr("panel.painter_layers.watercolor.flow_classic")
    } else {
        ph2d_i18n::tr(k.name_key())
    }
}

/// Pinta as linhas da forma da borda a partir de `y`; devolve o `y` seguinte.
pub(crate) fn paint_flow_rows(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let (mut y, open) = paint_dropdown_row(
        ctx,
        theme,
        x,
        w,
        y,
        "panel.painter_layers.watercolor.flow",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_FLOW_KIND,
        brush.flow_kind,
        rotulo(TextureKind::from_u8(brush.flow_kind)),
    );
    if let Some(r) = open {
        state::set_pending_flow_kind_dd(Some((r, brush.flow_kind)));
    }
    if !classic(brush) {
        y = card_row(
            ctx,
            theme,
            x,
            w,
            y,
            "panel.painter_layers.watercolor.flow_size",
            ph2d_tool_painter::ids::PAINTER_WATERCOLOR_FLOW_SIZE,
            brush.flow_size,
            FLOW_SIZE_MIN,
            FLOW_SIZE_MAX,
            number_field::FINE_STEP,
            2,
        );
        y = card_row(
            ctx,
            theme,
            x,
            w,
            y,
            "panel.painter_layers.watercolor.flow_angle",
            ph2d_tool_painter::ids::PAINTER_WATERCOLOR_FLOW_ANGLE,
            f32::from(brush.flow_angle),
            0.0,
            ANGLE_MAX,
            number_field::ANGLE_STEP,
            0,
        );
    }
    card_row(
        ctx,
        theme,
        x,
        w,
        y,
        "panel.painter_layers.watercolor.paper_edge",
        ph2d_tool_painter::ids::PAINTER_WATERCOLOR_PAPER_EDGE,
        brush.paper_edge,
        0.0,
        1.0,
        number_field::FINE_STEP,
        2,
    )
}

/// Drena o popover do Flow (chamado do passe dos popovers, depois do clip do corpo).
pub(crate) fn paint_flow_popover(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((chip_rect, cur)) = state::take_pending_flow_kind_dd() {
        let options: Vec<DropdownOption<u8>> = FLOW_KINDS
            .iter()
            .map(|&k| {
                DropdownOption::new(
                    ph2d_tool_painter::ids::painter_flow_kind_option_id(k.to_u8()),
                    k.to_u8(),
                    rotulo(k),
                )
            })
            .collect();
        crate::paint_brush::paint_dropdown_popover(
            ctx,
            theme,
            ph2d_tool_painter::ids::PAINTER_WATERCOLOR_FLOW_KIND,
            options,
            chip_rect,
            cur,
        );
    }
}
