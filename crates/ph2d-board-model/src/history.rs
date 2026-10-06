//! **O desfazer de UM quadro** (plano §1.4): cada gesto é um lote de [`BoardOp`] e o que se guarda é
//! o lote INVERSO que [`apply_batch`] devolve — nunca o documento inteiro (100 mil elementos por
//! gesto não cabem). Desfazer aplica o inverso, e o inverso DELE é o refazer.
//!
//! Vive fora do [`crate::BoardSet`] de propósito: o histórico é da sessão, não do ficheiro.

use crate::{BoardDoc, BoardOp, apply_batch};

/// Passos guardados por quadro. ⚠️ É memória, não desempenho: um passo é um lote de elementos
/// INTEIROS (o `Put` leva o estado anterior), e um quadro grande com gestos sobre selecções grandes
/// enche-o depressa. 500 é a ordem do Excalidraw/Figma; acima disto ninguém desfaz à mão.
pub const MAX_STEPS: usize = 500;

#[derive(Clone, Debug, Default)]
pub struct History {
    undo: Vec<Vec<BoardOp>>,
    redo: Vec<Vec<BoardOp>>,
}

impl History {
    /// Aplica um gesto ao documento e guarda o passo. Um lote que não mudou nada não é passo.
    pub fn apply(&mut self, doc: &mut BoardDoc, ops: Vec<BoardOp>) {
        let inverse = apply_batch(doc, ops);
        self.record(inverse);
    }

    /// Guarda um passo já aplicado ao vivo (um arrasto que mexeu no documento quadro a quadro):
    /// `inverse` é o lote que o devolve ao início do gesto. Vazio não é passo.
    pub fn record(&mut self, inverse: Vec<BoardOp>) {
        if inverse.is_empty() {
            return;
        }
        self.undo.push(inverse);
        if self.undo.len() > MAX_STEPS {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Desfaz o último gesto. `false` = nada a desfazer.
    pub fn undo(&mut self, doc: &mut BoardDoc) -> bool {
        let Some(step) = self.undo.pop() else {
            return false;
        };
        self.redo.push(apply_batch(doc, step));
        true
    }

    /// Refaz o último desfeito. `false` = nada a refazer.
    pub fn redo(&mut self, doc: &mut BoardDoc) -> bool {
        let Some(step) = self.redo.pop() else {
            return false;
        };
        self.undo.push(apply_batch(doc, step));
        true
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    #[must_use]
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
