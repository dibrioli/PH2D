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
//!
//! B5(4) — cada passo leva a HORA do [`undo_tick`]: o `Ctrl+Z` de um documento compara-a com a da
//! última operação de abas e desfaz a mais recente (doc 15).

use std::sync::atomic::{AtomicU64, Ordering};

use crate::{BoardDoc, BoardOp, Element, Field, Team, TeamOp, apply_batch};

static CLOCK: AtomicU64 = AtomicU64::new(0);

/// ⭐ B5(4) — o relógio ÚNICO das pilhas de desfazer (de cada quadro, das abas, da cena): só cresce, e
/// quem o lê compara horas — a mais alta é o passo mais recente (feito, desfeito ou refeito).
#[must_use]
pub fn undo_tick() -> u64 {
    CLOCK.fetch_add(1, Ordering::Relaxed) + 1
}

/// Passos guardados por quadro. ⚠️ É memória, não desempenho: um passo é um lote de elementos
/// INTEIROS (o `Put` leva o estado anterior), e um quadro grande com gestos sobre selecções grandes
/// enche-o depressa. 500 é a ordem do Excalidraw/Figma; acima disto ninguém desfaz à mão.
pub const MAX_STEPS: usize = 500;

/// Um passo: o lote inverso do documento (com o estado de cada elemento DEPOIS do gesto), ou o
/// inverso de um passo da equipa.
#[derive(Clone, Debug)]
enum Step {
    /// O lote inverso, o estado depois do gesto e — no botão «Rascunho ↔ Final» do quadro inteiro — o
    /// MODO do quadro a repor e o outro (`Board::sketch`; auditoria 21/10: o `Ctrl+Z` desfazia as
    /// formas e deixava o modo).
    Doc(Vec<BoardOp>, Vec<Option<Element>>, Option<(bool, bool)>),
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

/// Cada pilha guarda `(hora, passo)`: a hora é a do [`undo_tick`] quando o passo lá entrou.
#[derive(Clone, Debug, Default)]
pub struct History {
    undo: Vec<(u64, Step)>,
    redo: Vec<(u64, Step)>,
    /// A hora do último passo NOVO (não um desfazer nem um refazer); `0` = nenhum.
    last_push: u64,
    /// O modo do quadro que o último desfazer/refazer repôs ([`Self::take_mode`]).
    mode: Option<bool>,
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
            self.push(Step::Doc(inverse, after, None));
        }
    }

    /// Aplica `ops` e muda o MODO do quadro de `was` para `now` no MESMO passo (o botão «Rascunho ↔
    /// Final» sem selecção): desfazê-lo repõe `was` ([`Self::take_mode`]).
    pub fn apply_mode(&mut self, doc: &mut BoardDoc, ops: Vec<BoardOp>, [was, now]: [bool; 2]) {
        let inverse = apply_batch(doc, ops);
        if !inverse.is_empty() || was != now {
            let after = after(doc, &inverse);
            self.push(Step::Doc(inverse, after, Some((was, now))));
        }
    }

    /// O modo do quadro que o último desfazer/refazer repôs, se o passo o mudava — quem tem o
    /// [`crate::Board`] escreve-o em `Board::sketch`.
    pub fn take_mode(&mut self) -> Option<bool> {
        self.mode.take()
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
        self.last_push = undo_tick();
        self.undo.push((self.last_push, step));
        if self.undo.len() > MAX_STEPS {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Desfaz o último gesto do DOCUMENTO. `false` = nada a desfazer, ou o último passo é da equipa
    /// (fica na pilha para [`Self::undo_team`]).
    pub fn undo(&mut self, doc: &mut BoardDoc) -> bool {
        let done = Self::swap_doc(&mut self.undo, &mut self.redo, doc);
        self.mode = done.flatten();
        done.is_some()
    }

    /// Refaz o último desfeito do DOCUMENTO (o par de [`Self::undo`]).
    pub fn redo(&mut self, doc: &mut BoardDoc) -> bool {
        let done = Self::swap_doc(&mut self.redo, &mut self.undo, doc);
        self.mode = done.flatten();
        done.is_some()
    }

    fn swap_doc(
        from: &mut Vec<(u64, Step)>,
        to: &mut Vec<(u64, Step)>,
        doc: &mut BoardDoc,
    ) -> Option<Option<bool>> {
        let Some((_, Step::Doc(..))) = from.last() else {
            return None;
        };
        let Some((_, Step::Doc(step, was, mode))) = from.pop() else {
            unreachable!("visto acima");
        };
        let inverse = apply_batch(doc, rebase(doc, step, was));
        let after = after(doc, &inverse);
        to.push((
            undo_tick(),
            Step::Doc(inverse, after, mode.map(|(a, b)| (b, a))),
        ));
        Some(mode.map(|(back, _)| back))
    }

    /// Desfaz o último passo se é da EQUIPA. `false` = não é (ou não há).
    pub fn undo_team(&mut self, team: &mut Team) -> bool {
        Self::swap_team(&mut self.undo, &mut self.redo, team)
    }

    /// Refaz o último desfeito se é da EQUIPA.
    pub fn redo_team(&mut self, team: &mut Team) -> bool {
        Self::swap_team(&mut self.redo, &mut self.undo, team)
    }

    fn swap_team(from: &mut Vec<(u64, Step)>, to: &mut Vec<(u64, Step)>, team: &mut Team) -> bool {
        let Some((_, Step::Team(op))) = from.last().cloned() else {
            return false;
        };
        from.pop();
        // O fio pode ter saído entretanto (apagar a 1.ª mensagem não se desfaz): o passo gasta-se.
        if let Some(inverse) = team.apply(op) {
            to.push((undo_tick(), Step::Team(inverse)));
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

    /// B5(4) — a hora do passo que o próximo desfazer leva (`None` = nada a desfazer).
    #[must_use]
    pub fn undo_at(&self) -> Option<u64> {
        self.undo.last().map(|s| s.0)
    }

    /// B5(4) — a hora do passo que o próximo refazer leva: a de quando foi DESFEITO.
    #[must_use]
    pub fn redo_at(&self) -> Option<u64> {
        self.redo.last().map(|s| s.0)
    }

    /// B5(4) — a hora do último passo novo (`0` = nenhum): um refazer de abas mais velho que ela já não
    /// vale (o artista fez outra coisa depois de desfazer).
    #[must_use]
    pub fn last_push(&self) -> u64 {
        self.last_push
    }
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
