//! **Color mixing** (a mistura opaca do Digital, `BrushSpec::mistura_opaca`): uma opção a mais no cartão Basics,
//! nos dois painéis. Só onde a lei age — o Pincel do Digital sem o flag da aquarela, como o Fine Thread só no Smear do
//! Digital. Com `Off` os números ficam à vista, esmaecidos ([`ph2d_tool_painter::Dependente::MisturaDesligada`]).

use crate::cards_linhas::{Item, escolha, filete, fila};
use crate::paint_wetpaint::Numero;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Seg};
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{BrushSettings, FerramentaDoPincel, ModoMisturaOpaca, PaintMedia};

/// Os nomes das opções, pela ordem de `ModoMisturaOpaca::TODOS`.
const OPCOES: [&str; 2] = [
    "panel.painter_layers.brush.mix_off",
    "panel.painter_layers.brush.mix_wet_blend",
];

/// Os rótulos dos números, pela ordem de `PAINTER_BRUSH_MIX_FIELDS`.
const ROTULOS: [&str; 6] = [
    "panel.painter_layers.brush.mix_amount",
    "panel.painter_layers.brush.mix_density",
    "panel.painter_layers.brush.mix_stretch",
    "panel.painter_layers.brush.mix_amount_pressure",
    "panel.painter_layers.brush.mix_density_pressure",
    "panel.painter_layers.brush.mix_pigment_amount",
];

/// A lei age neste pincel: o Pincel do Digital, sem a aquarela (`mistura_opaca_dab` só corre na rota opaca).
pub(crate) fn oferecida(b: &BrushSettings) -> bool {
    b.ferramenta() == FerramentaDoPincel::Pincel
        && PaintMedia::from_u8(b.media) == PaintMedia::Digital
        && !b.watercolor
        && !b.wetpaint
}

/// É um número do grupo: o evento segue como `SetValue` para a ferramenta.
pub(crate) fn e_campo(id: NodeId) -> bool {
    pids::PAINTER_BRUSH_MIX_FIELDS.contains(&id)
}

/// Os seis números, com os valores de `b` (todos `0..1`), na ordem do painel (`PAINTER_BRUSH_MIX_ORDEM`).
pub(crate) fn numeros(b: &BrushSettings) -> Vec<Numero> {
    let m = b.mistura_opaca;
    let valores = [
        m.paint_amount,
        m.paint_density,
        m.color_stretch,
        m.amount_pressure,
        m.density_pressure,
        m.pigmento,
    ];
    pids::PAINTER_BRUSH_MIX_ORDEM
        .iter()
        .map(|&i| {
            crate::cards_linhas::numero(
                ROTULOS[i],
                pids::PAINTER_BRUSH_MIX_FIELDS[i],
                valores[i],
                (0.0, 1.0),
                crate::cards_topo::PASSO_DO_PINCEL,
                crate::cards_topo::CASAS_DO_PINCEL,
            )
        })
        .collect()
}

fn indice(b: &BrushSettings) -> usize {
    ModoMisturaOpaca::TODOS
        .iter()
        .position(|m| *m == b.mistura_opaca.modo)
        .unwrap_or(0)
}

/// Com o store MUTÁVEL, antes de pintar (os dois painéis).
pub(crate) fn prepare(store: &mut WidgetStore, b: &BrushSettings) {
    if oferecida(b) {
        crate::cards_wet::arma_numeros(
            store,
            &numeros(b),
            &numeros(&crate::paint_brush::FALLBACK_BRUSH),
        );
    }
}

/// O grupo no cartão Basics, depois de um filete: o rótulo do seletor é o nome do grupo (nunca um 2.º título).
pub(crate) fn grupo_da_mistura(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let y = filete(c, y);
    let atual = indice(b);
    let segs: Vec<Seg<'_>> = OPCOES
        .iter()
        .zip(pids::PAINTER_BRUSH_MIX_MODES)
        .enumerate()
        .map(|(i, (k, id))| Seg {
            id,
            icon: None,
            text: tr(k),
            on: i == atual,
        })
        .collect();
    let y = escolha(c, tr("panel.painter_layers.brush.mix_mode"), &segs, y);
    let itens: Vec<Item> = numeros(b).into_iter().map(Item::Num).collect();
    fila(c, &itens, y)
}

/// O mesmo no painel de antes (`PH2D_CARDS=0`): o seletor segmentado e uma linha por número.
pub(crate) fn paint_de_antes(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    x: f32,
    w: f32,
    y: f32,
    b: &BrushSettings,
) -> f32 {
    use ph2d_editor_core::widget::{
        SegmentedAdaptive, SegmentedOption, measure_segmented_adaptive, paint_segmented_adaptive,
    };
    use ph2d_editor_core::zones::Rect;
    let opcoes = OPCOES
        .iter()
        .zip(pids::PAINTER_BRUSH_MIX_MODES)
        .map(|(k, id)| SegmentedOption::new(id, tr(k)))
        .collect();
    let seg = SegmentedAdaptive::new(
        pids::PAINTER_BRUSH_MIX_MODE,
        tr("panel.painter_layers.brush.mix_mode"),
        opcoes,
    )
    .selected(indice(b));
    let row = ph2d_tokens::ROW_H_PX;
    let h = measure_segmented_adaptive(&seg, w, row, ctx.text_system);
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        paint_segmented_adaptive(
            &seg,
            Rect::new(x, y, w, row),
            scene,
            text_system,
            theme,
            store,
            hit_index,
        );
    }
    let mut y = y + h + ph2d_tokens::control_gap_px();
    for n in numeros(b) {
        y = n.linha_de_antes(ctx, theme, x, w, y);
    }
    y
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cada número lê o seu campo com a faixa `0..1`, e cada opção tem o seu id e nome.
    #[test]
    fn os_numeros_da_mistura_leem_os_seus_campos() {
        let mut b = crate::paint_brush::FALLBACK_BRUSH;
        b.mistura_opaca.color_stretch = 0.25;
        b.mistura_opaca.density_pressure = 0.75;
        b.mistura_opaca.pigmento = 0.125;
        b.mistura_opaca.paint_density = 0.5;
        let n = numeros(&b);
        assert_eq!(n.len(), 6);
        // A ordem do CSP: Amount · Density · Color stretch · Pigment · as pressões.
        assert_eq!(n[1].valor, 0.5, "Density logo depois do Amount");
        assert_eq!(n[2].valor, 0.25);
        assert_eq!(n[3].valor, 0.125, "Pigment depois do Color stretch");
        assert_eq!(n[5].valor, 0.75);
        assert_eq!(tr(ROTULOS[5]), "Pigment");
        assert_eq!(tr(ROTULOS[2]), "Color stretch");
        assert!(n.iter().all(|n| e_campo(n.id) && n.min == 0.0 && n.max == 1.0));
        assert_eq!(tr(OPCOES[1]), "Wet blend");
        assert_eq!(tr(ROTULOS[0]), "Amount");
        b.mistura_opaca.modo = ModoMisturaOpaca::WetBlend;
        assert_eq!(indice(&b), 1);
    }
}
