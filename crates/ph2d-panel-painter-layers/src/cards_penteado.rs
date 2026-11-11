//! **Bristle Combing:** os fios que penteiam a tinta, um grupo do cartão Shape separado do tapete de cerdas, que é outra
//! coisa (dono 07/11). Os valores são knobs do motor, gravados no preset pela chave de sempre; os ids são os do Tuning,
//! que as macros gravaram. Fora do Wet Paint, esmaecidos: [`ph2d_tool_painter::Dependente::PenteadoForaDoMolhado`].

use crate::cards_linhas::{Item, fila, grupo, numero};
use crate::number_field::Padrao;
use crate::paint_wetpaint::Numero;
use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{BrushSettings, Escala, LinhaDoPenteado};

/// O rótulo de cada fio. A largura leva a unidade no rótulo; os outros repetem o nome que o dono já conhece.
fn rotulo(chave: &str) -> &'static str {
    match chave {
        "bristleLaneWidth" => "card.painter.shape.bristle_width",
        "bristleHeight" => "card.painter.shape.bristle_relief",
        "bristleTipRagged" => "card.painter.shape.ragged_bristle_tips",
        _ => "card.painter.shape.bristle_follow_size",
    }
}

/// B35.13: o rótulo (o nome que o Tuning já usava; os de px levam a unidade, como a largura) e a dica de cada cerda.
fn rotulo_da_cerda(chave: &str) -> (&'static str, &'static str) {
    match chave {
        "bristlePressure" => (
            "panel.wet_tuning.knob.bristlePressure",
            "panel.wet_tuning.tip.bristlePressure",
        ),
        "bristleNoiseLength" => (
            "card.painter.shape.bristle_noise_length",
            "panel.wet_tuning.tip.bristleNoiseLength",
        ),
        "bristleWave" => (
            "panel.wet_tuning.knob.bristleWave",
            "panel.wet_tuning.tip.bristleWave",
        ),
        "bristleWaveLength" => (
            "card.painter.shape.bristle_wave_length",
            "panel.wet_tuning.tip.bristleWaveLength",
        ),
        "bristleOscillation" => (
            "card.painter.shape.bristle_oscillation",
            "panel.wet_tuning.tip.bristleOscillation",
        ),
        "bristleSpan" => (
            "card.painter.shape.bristle_span",
            "panel.wet_tuning.tip.bristleSpan",
        ),
        "bristleSpanLength" => (
            "card.painter.shape.bristle_span_length",
            "panel.wet_tuning.tip.bristleSpanLength",
        ),
        "bristleNoiseHue" => (
            "panel.wet_tuning.knob.bristleNoiseHue",
            "panel.wet_tuning.tip.bristleNoiseHue",
        ),
        "bristleNoiseSat" => (
            "panel.wet_tuning.knob.bristleNoiseSat",
            "panel.wet_tuning.tip.bristleNoiseSat",
        ),
        "bristleNoiseLight" => (
            "panel.wet_tuning.knob.bristleNoiseLight",
            "panel.wet_tuning.tip.bristleNoiseLight",
        ),
        "bristleLift" => (
            "panel.wet_tuning.knob.bristleLift",
            "panel.wet_tuning.tip.bristleLift",
        ),
        "bristleRemove" => (
            "panel.wet_tuning.knob.bristleRemove",
            "panel.wet_tuning.tip.bristleRemove",
        ),
        "bristleStreak" => (
            "panel.wet_tuning.knob.bristleStreak",
            "panel.wet_tuning.tip.bristleStreak",
        ),
        "bristleStreakLight" => (
            "panel.wet_tuning.knob.bristleStreakLight",
            "panel.wet_tuning.tip.bristleStreakLight",
        ),
        "bristleStreakLength" => (
            "card.painter.shape.bristle_streak_length",
            "panel.wet_tuning.tip.bristleStreakLength",
        ),
        "bristleStreakCut" => (
            "panel.wet_tuning.knob.bristleStreakCut",
            "panel.wet_tuning.tip.bristleStreakCut",
        ),
        _ => (
            "panel.wet_tuning.knob.tuftArc",
            "panel.wet_tuning.tip.tuftArc",
        ),
    }
}

fn escala(l: &LinhaDoPenteado) -> Escala {
    Escala {
        min: l.min as f32,
        max: l.max as f32,
        um: l.max as f32,
        step: l.passo,
    }
}

/// É um campo do grupo: o evento do número segue como `SetValue` para a ferramenta.
pub(crate) fn e_campo(id: NodeId) -> bool {
    pids::CERDAS_DO_PENTEADO
        .iter()
        .chain(&pids::CERDAS_NO_SHAPE)
        .any(|k| pids::wet_tuning_chip_id(k) == id)
}

/// Os quatro números, com os valores de `b`.
pub(crate) fn numeros(b: &BrushSettings) -> Vec<Numero> {
    b.wet_knobs
        .penteado()
        .iter()
        .map(|l| numero_da_linha(l, rotulo(l.chave)))
        .collect()
}

fn numero_da_linha(l: &LinhaDoPenteado, chave: &'static str) -> Numero {
    let e = escala(l);
    numero(
        chave,
        pids::wet_tuning_chip_id(l.chave),
        l.valor as f32,
        (e.min, e.max),
        e.step,
        e.decimals(),
    )
}

/// B35.13: as cerdas que se leem com `b` (ponta Bristles, pai ligado), com os valores de `valores`.
pub(crate) fn numeros_das_cerdas(b: &BrushSettings, valores: &BrushSettings) -> Vec<Numero> {
    valores
        .wet_knobs
        .cerdas(b)
        .iter()
        .map(|l| numero_da_linha(l, rotulo_da_cerda(l.chave).0))
        .collect()
}

/// Arma os números do penteado e das cerdas (o ↺ é a fábrica) e põe a dica de cada cerda.
pub(crate) fn arma(store: &mut ph2d_editor_core::interaction::WidgetStore, b: &BrushSettings) {
    let fabrica = crate::paint_brush::fabrica_do_pincel(b);
    crate::cards_wet::arma_numeros(store, &numeros(b), &numeros(&fabrica));
    crate::cards_wet::arma_numeros(
        store,
        &numeros_das_cerdas(b, b),
        &numeros_das_cerdas(b, &fabrica),
    );
    for l in b.wet_knobs.cerdas(b) {
        let id = pids::wet_tuning_chip_id(l.chave);
        if crate::esmaecer::poe_dica(store, &[id]).is_none() {
            store.set_tooltip(id, tr(rotulo_da_cerda(l.chave).1));
        }
    }
}

/// As mesmas linhas para o caminho sem cartões (`PH2D_CARDS=0`).
pub(crate) fn padroes(b: &BrushSettings) -> Vec<Padrao> {
    let penteado = b.wet_knobs.penteado().map(|l| (rotulo(l.chave), l));
    let cerdas = b
        .wet_knobs
        .cerdas(b)
        .into_iter()
        .map(|l| (rotulo_da_cerda(l.chave).0, l));
    penteado
        .into_iter()
        .chain(cerdas)
        .map(|(chave, l)| {
            (
                tr(chave),
                pids::wet_tuning_chip_id(l.chave),
                l.valor as f32,
                escala(&l),
            )
        })
        .collect()
}

/// O grupo, depois de um filete: o que é do tapete fica acima.
pub(crate) fn grupo_do_penteado(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let y = grupo(c, "card.painter.shape.bristle_combing", y);
    let itens: Vec<Item> = numeros(b).into_iter().map(Item::Num).collect();
    let y = fila(c, &itens, y);
    let cerdas: Vec<Item> = numeros_das_cerdas(b, b)
        .into_iter()
        .map(Item::Num)
        .collect();
    if cerdas.is_empty() {
        return y;
    }
    let y = grupo(c, "panel.wet_tuning.group.bristles", y);
    fila(c, &cerdas, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cada linha lê o knob dela, com a faixa do motor, e o rótulo certo.
    #[test]
    fn as_quatro_linhas_leem_os_seus_knobs() {
        let b = crate::paint_brush::FALLBACK_BRUSH;
        let n = numeros(&b);
        let l = b.wet_knobs.penteado();
        assert_eq!(n.len(), 4);
        for ((linha, m), k) in n.iter().zip(l).zip(pids::CERDAS_DO_PENTEADO) {
            assert_eq!(linha.id, pids::wet_tuning_chip_id(k));
            assert_eq!(linha.max, m.max as f32);
            assert!(e_campo(linha.id));
        }
        assert_eq!(tr(rotulo("bristleLaneWidth")), "Bristle width (px)");
        assert_eq!(tr(rotulo("bristleHeight")), "Bristle relief");
    }
}
