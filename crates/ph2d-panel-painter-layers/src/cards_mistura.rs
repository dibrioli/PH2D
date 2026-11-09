//! O cartão **Paint Mixing** (dono 10/11: «junte e mude de nome»), logo depois do Basics: o ÚNICO lugar de como as
//! cores se misturam, nos meios onde isso age.
//!
//! * **Pigment** (`pigment`/`pigment_mix`, id `PAINTER_WATERCOLOR_MIX`): `0` = como luz, `1` = como tinta. UM número
//!   que as duas misturas leem — a tinta sobre a tela (`blend_over_pigmento`) e o pincel com o que apanhou no Wet
//!   blend (`mistura_opaca_dab`). Até 10/11 o Wet blend tinha o seu próprio `Pigment` (`mistura_opaca.pigmento`) num
//!   cartão Color Mixing, e o cartão Mixing o outro; medido (`diag_os_dois_pigmentos`): os dois somavam no Digital
//!   com Wet blend e o do Wet blend era MORTO com `Off` e fora do Digital. Linha onde `pigment_offered`.
//! * **Color mixing** (a mistura opaca do Digital, `BrushSpec::mistura_opaca`): o seletor, o Sample e os números, só
//!   no Pincel do Digital sem a aquarela. Com `Off` ficam à vista, esmaecidos
//!   ([`ph2d_tool_painter::Dependente::MisturaDesligada`]); o Pigment não esmaece (age com `Off`).

use crate::cards_linhas::{Item, fila};
use crate::paint_wetpaint::Numero;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget};
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{
    AmostraDaMistura, BrushSettings, FerramentaDoPincel, ModoMisturaOpaca, PaintMedia,
};

/// Os nomes das opções, pela ordem de `ModoMisturaOpaca::TODOS`.
const OPCOES: [&str; 2] = [
    "panel.painter_layers.brush.mix_off",
    "panel.painter_layers.brush.mix_wet_blend",
];

/// Os nomes das opções do **Sample**, pela ordem de `AmostraDaMistura::TODAS`.
const AMOSTRAS: [&str; 2] = [
    "panel.painter_layers.brush.mix_sample_this_layer",
    "panel.painter_layers.brush.mix_sample_all_visible",
];

/// O rótulo do seletor Color mixing.
const MODO: &str = "panel.painter_layers.brush.mix_mode";

/// O rótulo e a dica do seletor Sample.
const AMOSTRA: &str = "panel.painter_layers.brush.mix_sample";
const DICA_DA_AMOSTRA: &str = "panel.painter_layers.brush.mix_sample_tip";

/// Os rótulos dos números, pela ordem de `PAINTER_BRUSH_MIX_FIELDS`.
const ROTULOS: [&str; 5] = [
    "panel.painter_layers.brush.mix_amount",
    "panel.painter_layers.brush.mix_density",
    "panel.painter_layers.brush.mix_stretch",
    "panel.painter_layers.brush.mix_amount_pressure",
    "panel.painter_layers.brush.mix_density_pressure",
];

/// O Pigment age neste pincel (o meio ∧ o gesto, `pigment_offered`; no Wet Paint só a lei da sim).
pub(crate) fn pigmento_oferecido(b: &BrushSettings) -> bool {
    let gesto = !b.wetpaint || b.le(ph2d_tool_painter::LinhaDaFerramenta::MisturaDoPigmento);
    !b.is_inpaint && b.pigment_offered && gesto
}

/// O cartão está à vista: alguma das duas linhas age.
pub(crate) fn presente(b: &BrushSettings) -> bool {
    oferecida(b) || pigmento_oferecido(b)
}

/// A mistura opaca (Wet blend) age neste pincel: o Pincel do Digital, sem a aquarela (`mistura_opaca_dab` só corre na rota opaca).
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

/// Os cinco números, com os valores de `b` (todos `0..1`), na ordem do painel (`PAINTER_BRUSH_MIX_ORDEM`).
pub(crate) fn numeros(b: &BrushSettings) -> Vec<Numero> {
    let m = b.mistura_opaca;
    let valores = [
        m.paint_amount,
        m.paint_density,
        m.color_stretch,
        m.amount_pressure,
        m.density_pressure,
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

fn indice_da_amostra(b: &BrushSettings) -> usize {
    AmostraDaMistura::TODAS
        .iter()
        .position(|a| *a == b.mistura_opaca.amostra)
        .unwrap_or(0)
}

/// A dica do Sample: a da inércia (Color mixing em `Off`) toma o lugar da explicação.
fn poe_dicas(store: &mut WidgetStore) {
    for id in pids::PAINTER_BRUSH_MIX_SAMPLES {
        if crate::esmaecer::poe_dica(store, &[id]).is_none() {
            store.set_tooltip(id, tr(DICA_DA_AMOSTRA));
        }
    }
}

/// Com o store MUTÁVEL, antes de pintar (os dois painéis).
pub(crate) fn prepare(store: &mut WidgetStore, b: &BrushSettings) {
    if oferecida(b) {
        crate::cards_wet::arma_numeros(
            store,
            &numeros(b),
            &numeros(&crate::paint_brush::FALLBACK_BRUSH),
        );
        poe_dicas(store);
    }
}

/// O título do cartão (o id dele é `PAINTER_BRUSH_MIX_SECTION`, a dobra do painel de antes; nasce dobrado).
pub(crate) fn titulo() -> &'static str {
    tr("panel.painter_layers.brush.paint_mixing")
}

/// O corpo do cartão Paint Mixing: o Pigment primeiro (age sempre), à esquerda do Color mixing — dois irmãos curtos
/// dividem a linha (05 §4) —, depois o Sample e os números que dependem dele.
pub(crate) fn corpo(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let pigmento = pigmento_oferecido(b);
    if !oferecida(b) {
        return if pigmento {
            crate::cards_nativos::linha_do_pigmento(c, y)
        } else {
            y
        };
    }
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
    let y = if pigmento {
        let modo = Cell::new(Widget::Choice(&segs)).label(tr(MODO));
        c.pair(crate::cards_nativos::celula_do_pigmento(), modo, y)
    } else {
        c.full(Some(tr(MODO)), Widget::Choice(&segs), y)
    };
    let atual = indice_da_amostra(b);
    let segs: Vec<Seg<'_>> = AMOSTRAS
        .iter()
        .zip(pids::PAINTER_BRUSH_MIX_SAMPLES)
        .enumerate()
        .map(|(i, (k, id))| Seg {
            id,
            icon: None,
            text: tr(k),
            on: i == atual,
        })
        .collect();
    // O rótulo esmaece com os botões, como o dos números: a camada da linha inteira, sem o véu por opção.
    let y = crate::cards_linhas::com_lei(c, pids::PAINTER_BRUSH_MIX_SAMPLES[0], y, |c, y| {
        c.full(Some(tr(AMOSTRA)), Widget::Choice(&segs), y)
    });
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
    let seletores = [
        (
            pids::PAINTER_BRUSH_MIX_MODE,
            MODO,
            OPCOES.iter().zip(pids::PAINTER_BRUSH_MIX_MODES),
            indice(b),
            &[][..],
        ),
        (
            pids::PAINTER_BRUSH_MIX_SAMPLE,
            AMOSTRA,
            AMOSTRAS.iter().zip(pids::PAINTER_BRUSH_MIX_SAMPLES),
            indice_da_amostra(b),
            &pids::PAINTER_BRUSH_MIX_SAMPLES[..],
        ),
    ];
    let row = ph2d_tokens::ROW_H_PX;
    let mut y = if pigmento_oferecido(b) {
        crate::paint_pigment::paint_pigment_row(ctx, theme, x, w, y, b)
    } else {
        y
    };
    for (grupo, rotulo, opcoes, atual, lei) in seletores {
        let opcoes = opcoes
            .map(|(k, id)| SegmentedOption::new(id, tr(k)))
            .collect();
        let seg = SegmentedAdaptive::new(grupo, tr(rotulo), opcoes).selected(atual);
        let h = measure_segmented_adaptive(&seg, w, row, ctx.text_system);
        // O Sample esmaece com os números quando a mistura está em `Off` (a mesma camada da linha inerte).
        crate::esmaecer::talvez_esmaecido(ctx, theme, Rect::new(x, y, w, h), lei, |ctx| {
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
        });
        y += h + ph2d_tokens::control_gap_px();
    }
    poe_dicas(ctx.host.store_mut());
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
        b.mistura_opaca.paint_density = 0.5;
        let n = numeros(&b);
        assert_eq!(
            n.len(),
            5,
            "o Pigment não é número da mistura opaca: é o do cartão, um só"
        );
        assert_eq!(n[1].valor, 0.5, "Density logo depois do Amount");
        assert_eq!(n[2].valor, 0.25);
        assert_eq!(n[4].valor, 0.75);
        assert!(ROTULOS.iter().all(|k| tr(k) != "Pigment"));
        assert_eq!(tr(ROTULOS[2]), "Color stretch");
        assert!(
            n.iter()
                .all(|n| e_campo(n.id) && n.min == 0.0 && n.max == 1.0)
        );
        assert_eq!(tr(OPCOES[1]), "Wet blend");
        assert_eq!(tr(ROTULOS[0]), "Amount");
        b.mistura_opaca.modo = ModoMisturaOpaca::WetBlend;
        assert_eq!(indice(&b), 1);
        assert_eq!(tr(AMOSTRAS[1]), "All visible layers");
        b.mistura_opaca.amostra = AmostraDaMistura::TodasVisiveis;
        assert_eq!(indice_da_amostra(&b), 1);
    }
}
