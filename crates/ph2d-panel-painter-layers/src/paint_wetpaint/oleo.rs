//! O cartão **Real Oil** da secção Wet Paint (Óleo Real, doc 49): a caixa, as linhas da óptica do relevo da tela, o
//! Environment e o material, e o popover do Environment. Saiu de `paint_wetpaint.rs` pelo tecto de 600 LOC (só
//! mover, 29/10).

use super::Numero;
use crate::PaintCtx;
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
    let (optica, material) = numeros(brush, brush);
    if optica.is_empty() {
        return y;
    }
    for n in &optica {
        y = n.linha_de_antes(ctx, theme, x, content_w, y);
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
    material
        .iter()
        .fold(y, |y, n| n.linha_de_antes(ctx, theme, x, content_w, y))
}

/// Os números do Real Oil: a óptica e o material. Vazios sem óleo à vista. Os números são da TELA (dono 28/10): à
/// vista com a caixa do pincel ligada OU com tinta com óleo na tela. Condições de `leis`, valores de `valores`.
pub(crate) fn numeros(leis: &BrushSettings, valores: &BrushSettings) -> (Vec<Numero>, Vec<Numero>) {
    use ph2d_tool_painter::ids::{
        PAINTER_WETPAINT_OLEO_FAIXAS, PAINTER_WETPAINT_OLEO_FIELDS,
        PAINTER_WETPAINT_OLEO_MATERIAL_DE,
    };
    let pousa = leis.le(ph2d_tool_painter::LinhaDaFerramenta::DepositoMolhado);
    if !(pousa && leis.wet_oleo_real) && !leis.wet_oleo_na_tela {
        return (Vec::new(), Vec::new());
    }
    let linhas = [
        (
            "panel.painter_layers.wetpaint.oleo_shadow",
            valores.wet_oleo_sombra,
            number_field::FINE_STEP,
            2,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_light_height",
            valores.wet_oleo_altitude_graus,
            number_field::ANGLE_STEP,
            0,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_light_direction",
            valores.wet_oleo_azimute_graus,
            number_field::ANGLE_STEP,
            0,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_crevice",
            valores.wet_oleo_fenda,
            number_field::FINE_STEP,
            2,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_crevice_reach",
            valores.wet_oleo_alcance_fenda,
            number_field::CELL_STEP,
            0,
        ),
        (
            "panel.painter_layers.wetpaint.oleo_environment_light",
            valores.wet_oleo_forca_ambiente,
            number_field::FINE_STEP,
            2,
        ),
    ];
    let optica = linhas
        .into_iter()
        .zip(&PAINTER_WETPAINT_OLEO_FIELDS[..PAINTER_WETPAINT_OLEO_MATERIAL_DE])
        .zip(PAINTER_WETPAINT_OLEO_FAIXAS)
        .map(|(((chave, valor, passo, casas), id), (lo, hi))| Numero {
            chave,
            id: *id,
            valor,
            min: lo as f32,
            max: hi as f32,
            passo,
            casas,
        })
        .collect();
    (optica, numeros_do_material(valores))
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
fn numeros_do_material(valores: &BrushSettings) -> Vec<Numero> {
    use ph2d_tool_painter::ids::{
        PAINTER_WETPAINT_OLEO_FAIXAS, PAINTER_WETPAINT_OLEO_MATERIAL,
        PAINTER_WETPAINT_OLEO_MATERIAL_DE,
    };
    (0..MATERIAL.len())
        .map(|k| {
            let (lo, hi) = PAINTER_WETPAINT_OLEO_FAIXAS[PAINTER_WETPAINT_OLEO_MATERIAL_DE + k];
            Numero {
                chave: MATERIAL[k],
                id: PAINTER_WETPAINT_OLEO_MATERIAL[k],
                valor: valores.wet_oleo_material[k],
                min: lo as f32,
                max: hi as f32,
                passo: number_field::FINE_STEP,
                casas: 2,
            }
        })
        .collect()
}

/// Os rótulos do **Environment**, pela ordem de `Ambiente::TODOS` (o wire do chip).
const AMBIENTES: [&str; ph2d_tool_painter::ids::PAINTER_WETPAINT_OLEO_AMBIENTE_COUNT as usize] = [
    "panel.painter_layers.wetpaint.oleo_env_classic",
    "panel.painter_layers.wetpaint.oleo_env_studio",
    "panel.painter_layers.wetpaint.oleo_env_interior",
    "panel.painter_layers.wetpaint.oleo_env_courtyard",
];

pub(crate) fn nome_do_ambiente(i: u8) -> &'static str {
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

/// A lista aberta num cartão: a caixa é dele, a lista é esta.
pub(crate) fn abre_lista(r: ph2d_editor_core::zones::Rect, atual: u8) {
    PENDING_AMBIENTE_DD.with(|c| c.set(Some((r, atual))));
}

/// O popover do **Environment** (chamado de `paint_brush_popovers`, depois do clip do corpo).
pub(crate) fn paint_wetpaint_popovers(ctx: &mut PaintCtx, theme: ph2d_tokens::Theme) {
    super::tipo_de_tinta::paint_tipo_popover(ctx, theme);
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
