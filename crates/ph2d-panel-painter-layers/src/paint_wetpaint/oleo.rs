//! O cartão **Real Oil** da secção Wet Paint (Óleo Real, doc 49): a caixa, as linhas da óptica do relevo da tela, o
//! Environment e o material, e o popover do Environment. Saiu de `paint_wetpaint.rs` pelo tecto de 600 LOC (só
//! mover, 29/10).

use crate::PaintCtx;
use crate::card::card_row;
use crate::number_field;
use ph2d_i18n::tr;
use ph2d_tool_painter::BrushSettings;

/// O **Real Oil** (Óleo Real, doc 49): a caixa e, só com ela ligada, as seis linhas da óptica do relevo da TELA —
/// Shadow (o Depth e o Gloss são do pincel desde 28/10: Tuning › Material), a luz (altura e direcção, em GRAUS), a fenda (força e alcance, W2 E1) e a força do
/// ambiente (W2 E2), por cima do Environment; depois o MATERIAL (W2 E3-B, [`paint_oleo_material`]). As
/// faixas são as da ferramenta (`PAINTER_WETPAINT_OLEO_FAIXAS`), uma lei para quem pinta e quem prende.
pub(super) fn paint_oleo_real(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    use ph2d_tool_painter::ids::{PAINTER_WETPAINT_OLEO_FAIXAS, PAINTER_WETPAINT_OLEO_FIELDS};
    // A caixa é do TRAÇO (dono 28/10): só a ferramenta que POUSA tinta a lê (a borracha e as da água não).
    let pousa = brush.le(ph2d_tool_painter::LinhaDaFerramenta::DepositoMolhado);
    let mut y = y;
    if pousa {
        y = crate::paint_brush_top::paint_checkbox_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_REAL,
            "panel.painter_layers.wetpaint.oleo_real",
            brush.wet_oleo_real,
        );
    }
    // Os números são da TELA (dono 28/10): à vista com a caixa do pincel ligada OU com tinta com óleo na tela.
    if !(pousa && brush.wet_oleo_real) && !brush.wet_oleo_na_tela {
        return y;
    }
    let linhas = [
        (
            "panel.painter_layers.wetpaint.oleo_shadow",
            brush.wet_oleo_sombra,
            number_field::FINE_STEP,
            2,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_light_height",
            brush.wet_oleo_altitude_graus,
            number_field::ANGLE_STEP,
            0,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_light_direction",
            brush.wet_oleo_azimute_graus,
            number_field::ANGLE_STEP,
            0,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_crevice",
            brush.wet_oleo_fenda,
            number_field::FINE_STEP,
            2,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_crevice_reach",
            brush.wet_oleo_alcance_fenda,
            number_field::CELL_STEP,
            0,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_environment_light",
            brush.wet_oleo_forca_ambiente,
            number_field::FINE_STEP,
            2,
        ),
    ];
    for (((chave, valor, passo, casas), id), (lo, hi)) in linhas
        .into_iter()
        .zip(
            &PAINTER_WETPAINT_OLEO_FIELDS
                [..ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_MATERIAL_DE],
        )
        .zip(PAINTER_WETPAINT_OLEO_FAIXAS)
    {
        y = card_row(
            ctx, theme, x, content_w, y, chave, *id, valor, lo as f32, hi as f32, passo, casas,
        );
    }
    let cur = brush.wet_oleo_ambiente;
    let (y, open) = crate::paint_brush_rows::paint_dropdown_row_de_caixa(
        ctx,
        theme,
        x,
        content_w,
        y,
        "panel.painter_layers.wetpaint.oleo_environment",
        ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_AMBIENTE,
        cur,
        nome_do_ambiente(cur),
    );
    if let Some(r) = open {
        PENDING_AMBIENTE_DD.with(|c| c.set(Some((r, cur))));
    }
    paint_oleo_material(ctx, theme, x, content_w, y, brush)
}

/// Os rótulos das linhas do material (nomes do Blender), pela ordem de `MaterialCartao::CAMPOS`; o que não cabe
/// parte-se em duas linhas ([`crate::duas_linhas`]).
pub(crate) const MATERIAL: [&str; ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_MATERIAL.len()] = [
    "panel.painter_layers.wetpaint.oleo_metallic",
    "panel.painter_layers.wetpaint.oleo_roughness",
    "panel.painter_layers.wetpaint.oleo_ior_level",
    "panel.painter_layers.wetpaint.oleo_coat",
    "panel.painter_layers.wetpaint.oleo_coat_roughness",
    "panel.painter_layers.wetpaint.oleo_sheen",
    "panel.painter_layers.wetpaint.oleo_subsurface",
    "panel.painter_layers.wetpaint.oleo_subsurface_radius",
];

/// O **material** global do Real Oil (W3): os 8 controlos do Principled simplificado, com os nomes do Blender. O
/// reflexo usa o **Environment** (o chip acima).
fn paint_oleo_material(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    content_w: f32,
    y: f32,
    brush: &BrushSettings,
) -> f32 {
    use ph2d_tool_painter::ids::{
        PAINTER_WETPAINT_OLEO_FAIXAS, PAINTER_WETPAINT_OLEO_MATERIAL,
        PAINTER_WETPAINT_OLEO_MATERIAL_DE,
    };
    let mut y = y;
    for k in 0..MATERIAL.len() {
        let (lo, hi) = PAINTER_WETPAINT_OLEO_FAIXAS[PAINTER_WETPAINT_OLEO_MATERIAL_DE + k];
        y = card_row(
            ctx,
            theme,
            x,
            content_w,
            y,
            MATERIAL[k],
            PAINTER_WETPAINT_OLEO_MATERIAL[k],
            brush.wet_oleo_material[k],
            lo as f32,
            hi as f32,
            number_field::FINE_STEP,
            2,
        );
    }
    y
}

/// Os rótulos do **Environment**, pela ordem de `Ambiente::TODOS` (o wire do chip).
const AMBIENTES: [&str; ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_AMBIENTE_COUNT as usize] = [
    "panel.painter_layers.wetpaint.oleo_env_classic",
    "panel.painter_layers.wetpaint.oleo_env_studio",
    "panel.painter_layers.wetpaint.oleo_env_interior",
    "panel.painter_layers.wetpaint.oleo_env_courtyard",
];

fn nome_do_ambiente(i: u8) -> &'static str {
    tr(AMBIENTES
        .get(usize::from(i))
        .copied()
        .unwrap_or(AMBIENTES[1]))
}

thread_local! {
    /// O menu do Environment aberto neste quadro (o chip e o índice de agora), pintado por cima do corpo.
    static PENDING_AMBIENTE_DD: std::cell::Cell<Option<(ph2d_editor_core::zones::Rect, u8)>> =
        const { std::cell::Cell::new(None) };
}

/// O popover do **Environment** (chamado de `paint_brush_popovers`, depois do clip do corpo).
pub(crate) fn paint_wetpaint_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    if let Some((r, cur)) = PENDING_AMBIENTE_DD.with(std::cell::Cell::take) {
        let opcoes = (0..ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_AMBIENTE_COUNT)
            .map(|i| {
                ph2d_editor_core::widget::DropdownOption::new(
                    ph2d_tool_painter::ids::painter_wetpaint_oleo_ambiente_option_id(i),
                    i,
                    nome_do_ambiente(i),
                )
            })
            .collect();
        crate::dropdown_popover::paint_dropdown_popover(
            ctx,
            theme,
            ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_AMBIENTE,
            opcoes,
            r,
            cur,
        );
    }
}
