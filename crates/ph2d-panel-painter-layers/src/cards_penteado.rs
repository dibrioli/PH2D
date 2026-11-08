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
        .any(|k| pids::wet_tuning_chip_id(k) == id)
}

/// Os quatro números, com os valores de `b`.
pub(crate) fn numeros(b: &BrushSettings) -> Vec<Numero> {
    b.wet_knobs
        .penteado()
        .iter()
        .map(|l| {
            let e = escala(l);
            numero(
                rotulo(l.chave),
                pids::wet_tuning_chip_id(l.chave),
                l.valor as f32,
                (e.min, e.max),
                e.step,
                e.decimals(),
            )
        })
        .collect()
}

/// As mesmas linhas para o caminho sem cartões (`PH2D_CARDS=0`).
pub(crate) fn padroes(b: &BrushSettings) -> Vec<Padrao> {
    b.wet_knobs
        .penteado()
        .iter()
        .map(|l| {
            (
                tr(rotulo(l.chave)),
                pids::wet_tuning_chip_id(l.chave),
                l.valor as f32,
                escala(l),
            )
        })
        .collect()
}

/// O grupo, depois de um filete: o que é do tapete fica acima.
pub(crate) fn grupo_do_penteado(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let y = grupo(c, "card.painter.shape.bristle_combing", y);
    let itens: Vec<Item> = numeros(b).into_iter().map(Item::Num).collect();
    fila(c, &itens, y)
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
