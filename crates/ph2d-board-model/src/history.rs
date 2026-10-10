//! **O desfazer de UM quadro** (plano §1.4): cada gesto é um lote de [`BoardOp`] e o que se guarda é
//! o lote INVERSO que [`apply_batch`] devolve — nunca o documento inteiro (100 mil elementos por
//! gesto não cabem). Desfazer aplica o inverso, e o inverso DELE é o refazer.
//!
//! Os passos da equipa ([`TeamOp`]: resolver, mover a bolha) vivem na MESMA pilha, pela ordem em que
//! se fizeram — o `Ctrl+Z` desfaz o último, seja do desenho ou do comentário. O documento e a equipa
//! são campos irmãos do [`crate::Board`]: [`History::undo`] só desfaz um passo do documento e pára
//! (devolve `false`) diante de um da equipa, que [`History::undo_team`] desfaz.
//!
//! Vive fora do [`crate::BoardSet`] de propósito: o histórico é da sessão, não do ficheiro.

use crate::{BoardDoc, BoardOp, Element, Field, Team, TeamOp, apply_batch};

/// Passos guardados por quadro. ⚠️ É memória, não desempenho: um passo é um lote de elementos
/// INTEIROS (o `Put` leva o estado anterior), e um quadro grande com gestos sobre selecções grandes
/// enche-o depressa. 500 é a ordem do Excalidraw/Figma; acima disto ninguém desfaz à mão.
pub const MAX_STEPS: usize = 500;

/// Um passo: o lote inverso do documento (com o estado de cada elemento DEPOIS do gesto), ou o
/// inverso de um passo da equipa.
#[derive(Clone, Debug)]
enum Step {
    Doc(Vec<BoardOp>, Vec<Option<Element>>),
    Team(TeamOp),
}

/// O estado, agora, de cada elemento que o lote `ops` repõe.
fn after(doc: &BoardDoc, ops: &[BoardOp]) -> Vec<Option<Element>> {
    ops.iter()
        .map(|op| doc.elements.get(&op.id()).cloned())
        .collect()
}

/// ⭐ O lote de desfazer contra o documento de AGORA (ADR-0181 §3): cada registo volta ao valor do
/// passo só se ainda é o que o gesto deixou — o que outra pessoa mexeu depois fica (o Excalidraw,
/// `colab_oracle` c11), e um elemento que outra apagou continua apagado (c12). Sem mais ninguém, é
/// o lote tal como foi guardado.
fn rebase(doc: &BoardDoc, ops: Vec<BoardOp>, was: Vec<Option<Element>>) -> Vec<BoardOp> {
    ops.into_iter()
        .zip(was)
        .map(|(op, was)| {
            let (BoardOp::Put(before), Some(was)) = (&op, was) else {
                return op;
            };
            let Some(now) = doc.elements.get(&before.id).filter(|now| **now != was) else {
                return op;
            };
            let mut el = now.clone();
            for f in Field::ALL {
                if now.read(f) == was.read(f)
                    && let Some(v) = before.read(f)
                {
                    el.write(f, &v);
                }
            }
            BoardOp::Put(el)
        })
        .collect()
}

#[derive(Clone, Debug, Default)]
pub struct History {
    undo: Vec<Step>,
    redo: Vec<Step>,
}

impl History {
    /// Aplica um gesto ao documento e guarda o passo. Um lote que não mudou nada não é passo.
    pub fn apply(&mut self, doc: &mut BoardDoc, ops: Vec<BoardOp>) {
        let inverse = apply_batch(doc, ops);
        self.record(inverse, doc);
    }

    /// Guarda um passo já aplicado a `doc` (também um arrasto que o mexeu quadro a quadro):
    /// `inverse` é o lote que o devolve ao início do gesto. Vazio não é passo.
    pub fn record(&mut self, inverse: Vec<BoardOp>, doc: &BoardDoc) {
        if !inverse.is_empty() {
            let after = after(doc, &inverse);
            self.push(Step::Doc(inverse, after));
        }
    }

    /// Aplica um passo da equipa e guarda-o. `false` = não mudou nada (não é passo).
    pub fn apply_team(&mut self, team: &mut Team, op: TeamOp) -> bool {
        let Some(inverse) = team.apply(op) else {
            return false;
        };
        self.push(Step::Team(inverse));
        true
    }

    fn push(&mut self, step: Step) {
        self.undo.push(step);
        if self.undo.len() > MAX_STEPS {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Desfaz o último gesto do DOCUMENTO. `false` = nada a desfazer, ou o último passo é da equipa
    /// (fica na pilha para [`Self::undo_team`]).
    pub fn undo(&mut self, doc: &mut BoardDoc) -> bool {
        let Some(Step::Doc(..)) = self.undo.last() else {
            return false;
        };
        let Some(Step::Doc(step, was)) = self.undo.pop() else {
            unreachable!("visto acima");
        };
        let inverse = apply_batch(doc, rebase(doc, step, was));
        let after = after(doc, &inverse);
        self.redo.push(Step::Doc(inverse, after));
        true
    }

    /// Refaz o último desfeito do DOCUMENTO (o par de [`Self::undo`]).
    pub fn redo(&mut self, doc: &mut BoardDoc) -> bool {
        let Some(Step::Doc(..)) = self.redo.last() else {
            return false;
        };
        let Some(Step::Doc(step, was)) = self.redo.pop() else {
            unreachable!("visto acima");
        };
        let inverse = apply_batch(doc, rebase(doc, step, was));
        let after = after(doc, &inverse);
        self.undo.push(Step::Doc(inverse, after));
        true
    }

    /// Desfaz o último passo se é da EQUIPA. `false` = não é (ou não há).
    pub fn undo_team(&mut self, team: &mut Team) -> bool {
        Self::swap_team(&mut self.undo, &mut self.redo, team)
    }

    /// Refaz o último desfeito se é da EQUIPA.
    pub fn redo_team(&mut self, team: &mut Team) -> bool {
        Self::swap_team(&mut self.redo, &mut self.undo, team)
    }

    fn swap_team(from: &mut Vec<Step>, to: &mut Vec<Step>, team: &mut Team) -> bool {
        let Some(Step::Team(op)) = from.last().cloned() else {
            return false;
        };
        from.pop();
        // O fio pode ter saído entretanto (apagar a 1.ª mensagem não se desfaz): o passo gasta-se.
        if let Some(inverse) = team.apply(op) {
            to.push(Step::Team(inverse));
        }
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

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
