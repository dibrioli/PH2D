//! **O desfazer de UM Kanban** — o molde do `History` do Quadro: cada gesto é um lote de
//! [`KanbanOp`] e guarda-se o lote INVERSO, nunca o documento inteiro. Vive fora do
//! [`crate::KanbanSet`]: o histórico é da sessão, não do ficheiro (ADR-0195 §5).
//!
//! A hora de cada passo vem de quem chama (o `undo_tick` único do editor), para o `Ctrl+Z` comparar
//! pilhas de famílias diferentes sem este crate ter relógio próprio.

use crate::activity::{Actor, apply_logged};
use crate::kanban::Kanban;
use crate::ops::{KanbanOp, Record, apply_batch};

/// Passos guardados por Kanban — a mesma ordem do Quadro e do Figma; é memória, não desempenho.
pub const MAX_STEPS: usize = 500;

/// Um passo: o lote inverso e o estado, DEPOIS do gesto, de cada registo que ele repõe.
#[derive(Clone, Debug)]
struct Step {
    inverse: Vec<KanbanOp>,
    after: Vec<Option<Record>>,
}

fn after(k: &Kanban, ops: &[KanbanOp]) -> Vec<Option<Record>> {
    ops.iter()
        .map(|op| op.target().and_then(|id| k.record(id)))
        .collect()
}

/// O lote de desfazer contra o documento de AGORA: um registo que outra pessoa mexeu depois do gesto
/// fica como ela o deixou. Sem mais ninguém (antes da W5, sempre), é o lote tal como foi guardado.
fn rebase(k: &Kanban, step: Step) -> Vec<KanbanOp> {
    step.inverse
        .into_iter()
        .zip(step.after)
        .filter(|(op, was)| op.target().is_none_or(|id| k.record(id) == *was))
        .map(|(op, _)| op)
        .collect()
}

/// As duas pilhas de um Kanban: `(hora, passo)`.
#[derive(Clone, Debug, Default)]
pub struct History {
    undo: Vec<(u64, Step)>,
    redo: Vec<(u64, Step)>,
}

impl History {
    /// Aplica um gesto e guarda o passo com a hora `at`. Um lote que não mudou nada não é passo.
    pub fn apply(&mut self, k: &mut Kanban, ops: Vec<KanbanOp>, at: u64) {
        let inverse = apply_batch(k, ops);
        self.record(inverse, k, at);
    }

    /// [`Self::apply`] e a atividade do gesto em nome de `actor` (que o desfazer não apaga).
    pub fn apply_logged(&mut self, k: &mut Kanban, ops: Vec<KanbanOp>, at: u64, actor: &Actor) {
        let inverse = apply_logged(k, ops, actor);
        self.record(inverse, k, at);
    }

    /// Guarda um passo já aplicado a `k` (um arrasto que o mexeu quadro a quadro): `inverse` devolve-o
    /// ao início do gesto. Vazio não é passo.
    pub fn record(&mut self, inverse: Vec<KanbanOp>, k: &Kanban, at: u64) {
        if inverse.is_empty() {
            return;
        }
        let after = after(k, &inverse);
        self.undo.push((at, Step { inverse, after }));
        if self.undo.len() > MAX_STEPS {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    /// Desfaz o último gesto. `false` = nada a desfazer.
    pub fn undo(&mut self, k: &mut Kanban, at: u64) -> bool {
        Self::swap(&mut self.undo, &mut self.redo, k, at, None)
    }

    /// Refaz o último desfeito.
    pub fn redo(&mut self, k: &mut Kanban, at: u64) -> bool {
        Self::swap(&mut self.redo, &mut self.undo, k, at, None)
    }

    /// [`Self::undo`] que regista na atividade o que o desfazer fez (é outra ação, como no Trello).
    pub fn undo_logged(&mut self, k: &mut Kanban, at: u64, actor: &Actor) -> bool {
        Self::swap(&mut self.undo, &mut self.redo, k, at, Some(actor))
    }

    /// [`Self::redo`] com a atividade.
    pub fn redo_logged(&mut self, k: &mut Kanban, at: u64, actor: &Actor) -> bool {
        Self::swap(&mut self.redo, &mut self.undo, k, at, Some(actor))
    }

    fn swap(
        from: &mut Vec<(u64, Step)>,
        to: &mut Vec<(u64, Step)>,
        k: &mut Kanban,
        at: u64,
        actor: Option<&Actor>,
    ) -> bool {
        let Some((_, step)) = from.pop() else {
            return false;
        };
        let ops = rebase(k, step);
        let inverse = match actor {
            Some(a) => apply_logged(k, ops, a),
            None => apply_batch(k, ops),
        };
        let after = after(k, &inverse);
        to.push((at, Step { inverse, after }));
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

    /// A hora do passo que o próximo desfazer leva.
    #[must_use]
    pub fn undo_at(&self) -> Option<u64> {
        self.undo.last().map(|s| s.0)
    }

    /// A hora do passo que o próximo refazer leva (a de quando foi desfeito).
    #[must_use]
    pub fn redo_at(&self) -> Option<u64> {
        self.redo.last().map(|s| s.0)
    }
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
