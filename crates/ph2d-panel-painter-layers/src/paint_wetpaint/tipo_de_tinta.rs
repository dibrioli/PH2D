//! O seletor **Paint type** (Water · Oil) do cartão Wet Paint (E10f, dono 01/11): o `Knob::Medium` só tem dois
//! pontos, por isso é um menu, no idioma da linha Environment do Real Oil ([`super::oleo`]): o nome à esquerda, o
//! chip na caixa de um campo, o ponto do Auto Key. Wire = o índice (`0` Water, `1` Oil).

use crate::PaintCtx;
use ph2d_i18n::tr;
use ph2d_tool_painter::BrushSettings;
use ph2d_tool_painter::ids::{
    PAINTER_WETPAINT_MEDIUM, PAINTER_WETPAINT_MEDIUM_COUNT, painter_wetpaint_medium_option_id,
};

/// Os rótulos das opções, pela ordem do wire.
const OPCOES: [&str; PAINTER_WETPAINT_MEDIUM_COUNT as usize] = [
    "panel.painter_layers.wetpaint.paint_type_water",
    "panel.painter_layers.wetpaint.paint_type_oil",
];

pub(crate) fn nome(i: u8) -> &'static str {
    tr(OPCOES.get(usize::from(i)).copied().unwrap_or(OPCOES[0]))
}

thread_local! {
    /// O menu do Paint type aberto neste quadro (o chip e o índice de agora), pintado por cima do corpo.
    static PENDING_TIPO_DD: std::cell::Cell<Option<(ph2d_editor_core::zones::Rect, u8)>> =
        const { std::cell::Cell::new(None) };
}

/// A linha **Paint type**: Oil quando `Medium > 0,5`, senão Water.
pub(super) fn paint_tipo_de_tinta(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    let cur = u8::from(brush.wet_knobs.e_oleo());
    let (y, open) = crate::paint_brush_rows::paint_dropdown_row_de_caixa(
        ctx,
        theme,
        x,
        content_w,
        y,
        "panel.painter_layers.wetpaint.paint_type",
        PAINTER_WETPAINT_MEDIUM,
        cur,
        nome(cur),
    );
    if let Some(r) = open {
        PENDING_TIPO_DD.with(|c| c.set(Some((r, cur))));
    }
    y
}

/// A lista aberta num cartão: a caixa é dele, a lista é esta.
pub(crate) fn abre_lista(r: ph2d_editor_core::zones::Rect, atual: u8) {
    PENDING_TIPO_DD.with(|c| c.set(Some((r, atual))));
}

/// O popover do **Paint type** (chamado de `paint_wetpaint_popovers`, depois do clip do corpo).
pub(crate) fn paint_tipo_popover(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((r, cur)) = PENDING_TIPO_DD.with(std::cell::Cell::take) {
        let opcoes = (0..PAINTER_WETPAINT_MEDIUM_COUNT)
            .map(|i| {
                ph2d_editor_core::widget::DropdownOption::new(
                    painter_wetpaint_medium_option_id(i),
                    i,
                    nome(i),
                )
            })
            .collect();
        crate::dropdown_popover::paint_dropdown_popover(
            ctx,
            theme,
            PAINTER_WETPAINT_MEDIUM,
            opcoes,
            r,
            cur,
        );
    }
}
