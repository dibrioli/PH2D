//! A seção **EFFECTS** (ADR-0132) — a pilha de Live Path Effects do caminho selecionado.
//!
//! Módulo irmão do [`super`] pelo teto de 600 LOC, par do `populate_effects` (que os
//! REGISTRA — sem isso o botão fica pintado e morto).
//!
//! # O que esta seção oferece, e o que ela deliberadamente NÃO oferece
//!
//! **Um Trim por caminho.** O motor aceita uma pilha de N efeitos e a ordem deles muda a
//! geometria (há gate). A UI expõe **um** porque só existe **um tipo** de efeito: empilhar
//! dois Trims idênticos é curiosidade, não valor, e reordenar dois iguais não significa
//! nada. Quando o 2º tipo chegar, a pergunta *"em que ordem?"* fica real e a lista com
//! reordenação nasce com ela — não antes.
//!
//! Por isso o botão é um **TOGGLE**: "Add" sobre um caminho que já tem Trim criaria o
//! segundo, que a UI não sabe mostrar. Um botão que promete o que a tela não representa é
//! pior que um botão que falta.

use super::*;

/// Quantas casas decimais o chip mostra. Os três parâmetros são frações do comprimento, e
/// 1% é o passo — duas casas é exatamente o que o artista consegue distinguir.
const DECIMALS: usize = 2;

impl BodyCtx<'_> {
    /// A seção Effects. Devolve o `y` avançado — e devolve o `y` de entrada intocado quando
    /// não há caminho selecionado, que é o que faz o `step` não emitir separador órfão.
    pub(crate) fn effects_section(&mut self, y: f32) -> f32 {
        let (mut y, collapsed) = self.section_header(
            ids::VECTOR_SECTION_EFFECTS,
            tr("panel.vector.section.effects"),
            y,
        );
        if collapsed {
            return y;
        }
        let trim = state::trim();
        // O toggle: o mesmo botão põe e tira. O rótulo diz qual das duas coisas ele fará.
        let label = if trim.is_some() {
            "Remove Trim Path"
        } else {
            "Add Trim Path"
        };
        y = self.action_button(ids::VECTOR_FX_TRIM, label, y);

        // Os parâmetros só existem quando o efeito existe. Não são "dimmed": um controle
        // apagado que ainda despacha mente, e um que não despacha é um botão morto.
        let Some((start, end, offset)) = trim else {
            return y;
        };
        // A tabela é TIPADA e nomeia o `ph2d_a11y::NodeId` de propósito: é assim que o gate
        // HR-12 reconhece que esta seção fala a11y (o `paint_envelope` usa o mesmo idioma).
        let rows: [(&str, ph2d_a11y::NodeId, ph2d_a11y::NodeId, f64); 3] = [
            (
                "Start",
                ids::VECTOR_FX_TRIM_START,
                ids::VECTOR_FX_TRIM_START_NUM,
                start,
            ),
            (
                "End",
                ids::VECTOR_FX_TRIM_END,
                ids::VECTOR_FX_TRIM_END_NUM,
                end,
            ),
            (
                "Offset",
                ids::VECTOR_FX_TRIM_OFFSET,
                ids::VECTOR_FX_TRIM_OFFSET_NUM,
                offset,
            ),
        ];
        for (label, slider, chip, v) in rows {
            // Fração `0..=1`: o track do slider É o valor do documento (o Bend do Envelope é
            // o contra-exemplo, e é por isso que a conversão dele mora no `event.rs`).
            #[allow(clippy::cast_possible_truncation)]
            let track = v as f32;
            y = self.slider_row(label, slider, chip, track, v, &format!("{v:.DECIMALS$}"), y);
        }
        y
    }
}
