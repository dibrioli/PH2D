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

use crate::{BoardDoc, BoardOp, Team, TeamOp, apply_batch};

/// Passos guardados por quadro. ⚠️ É memória, não desempenho: um passo é um lote de elementos
/// INTEIROS (o `Put` leva o estado anterior), e um quadro grande com gestos sobre selecções grandes
/// enche-o depressa. 500 é a ordem do Excalidraw/Figma; acima disto ninguém desfaz à mão.
pub const MAX_STEPS: usize = 500;

/// Um passo: o lote inverso do documento, ou o inverso de um passo da equipa.
#[derive(Clone, Debug)]
enum Step {
    Doc(Vec<BoardOp>),
    Team(TeamOp),
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
        self.record(inverse);
    }

    /// Guarda um passo já aplicado ao vivo (um arrasto que mexeu no documento quadro a quadro):
    /// `inverse` é o lote que o devolve ao início do gesto. Vazio não é passo.
    pub fn record(&mut self, inverse: Vec<BoardOp>) {
        if !inverse.is_empty() {
            self.push(Step::Doc(inverse));
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
        let Some(Step::Doc(_)) = self.undo.last() else {
            return false;
        };
        let Some(Step::Doc(step)) = self.undo.pop() else {
            unreachable!("visto acima");
        };
        self.redo.push(Step::Doc(apply_batch(doc, step)));
        true
    }

    /// Refaz o último desfeito do DOCUMENTO (o par de [`Self::undo`]).
    pub fn redo(&mut self, doc: &mut BoardDoc) -> bool {
        let Some(Step::Doc(_)) = self.redo.last() else {
            return false;
        };
        let Some(Step::Doc(step)) = self.redo.pop() else {
            unreachable!("visto acima");
        };
        self.undo.push(Step::Doc(apply_batch(doc, step)));
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
