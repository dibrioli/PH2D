//! As linhas da **forma da borda** no cartão Wash (BUGS_painter #31): o padrão de **Flow** do Ragged
//! Edge, a PRÉ-VISUALIZAÇÃO dele (a faixa partilhada das outras texturas), o Size e o Angle (só com um
//! padrão — o Classic não tem textura, escala nem direção), e o **Paper Edge**. Irmão de `paint_watercolor.rs` pelo tecto de LOC do painel.

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

/// A altura que a pré-visualização do Flow soma ao cartão — a MESMA faixa das outras pré-visualizações
/// do painel ([`crate::paint_texture::altura_do_preview`] + o intervalo). O Classic não tem textura, e
/// não tem pré-visualização (como o Paper e o Grain em `None`).
pub(crate) fn altura_extra_do_preview(brush: &BrushSettings, iw: f32) -> f32 {
    if classic(brush) {
        0.0
    } else {
        crate::paint_texture::altura_do_preview(iw) + ph2d_tokens::control_gap_px()
    }
}

/// O padrão do Flow como a pré-visualização partilhada o lê: o slot Grain do snapshot sobrescrito pelo
/// Flow, com o Size que o MOTOR amostra (`flow_size_efetivo`). Sem rampa (é um campo de deslocamento).
fn flow_preview_view(brush: &BrushSettings) -> BrushSettings {
    let mut v = *brush;
    v.texture_kind = brush.flow_kind;
    v.texture_params = brush.flow_params;
    v.texture_size = brush.flow_size_efetivo;
    v.texture_offset = [0.0, 0.0];
    v.texture_angle_deg = brush.flow_angle;
    v.texture_ramp_enabled = false;
    v
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
        // ── A pré-visualização: a faixa partilhada (como a do Shape, do Grain e do Paper) ──
        let imagem = (TextureKind::from_u8(brush.flow_kind) == TextureKind::Image)
            .then(state::current_brush_flow_image)
            .flatten();
        y = crate::paint_texture::paint_texture_preview(
            ctx,
            theme,
            x,
            w,
            y,
            flow_preview_view(brush),
            imagem,
        );
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
