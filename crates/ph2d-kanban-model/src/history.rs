//! **O desfazer de UM Kanban** — o molde do `History` do Quadro: cada gesto é um lote de
//! [`KanbanOp`] e guarda-se o lote INVERSO, nunca o documento inteiro. Vive fora do
//! [`crate::KanbanSet`]: o histórico é da sessão, não do ficheiro (ADR-0195 §5).
//!
//! A hora de cada passo vem de quem chama (o `undo_tick` único do editor), para o `Ctrl+Z` comparar
//! pilhas de famílias diferentes sem este crate ter relógio próprio.

use crate::activity::{Actor, apply_logged};
use crate::kanban::{Header, Kanban};
use crate::ops::{KanbanOp, Record, apply_batch};

#[path = "history_fields.rs"]
mod fields;
use fields::{meta_field, to_restore};

/// Passos guardados por Kanban — a mesma ordem do Quadro e do Figma; é memória, não desempenho.
pub const MAX_STEPS: usize = 500;

/// O estado, DEPOIS do gesto, do que uma op do lote inverso repõe.
#[derive(Clone, Debug)]
enum Was {
    Record(Option<Record>),
    Header(Header),
}

/// Um passo: o lote inverso e o estado depois do gesto de cada alvo dele.
#[derive(Clone, Debug)]
struct Step {
    inverse: Vec<KanbanOp>,
    after: Vec<Was>,
}

fn after(k: &Kanban, ops: &[KanbanOp]) -> Vec<Was> {
    ops.iter()
        .map(|op| match op.target() {
            Some(id) => Was::Record(k.record(id)),
            None => Was::Header(k.header.clone()),
        })
        .collect()
}

/// ⭐ O lote de desfazer contra o documento de AGORA, CAMPO A CAMPO (W5, ADR-0181 §3): repõe-se só o
/// campo que ainda tem o valor que o gesto deixou; o que outra pessoa mexeu depois fica como ela o
/// deixou. Sem mais ninguém, é o lote tal como foi guardado.
fn rebase(k: &Kanban, step: Step) -> Vec<KanbanOp> {
    step.inverse
        .into_iter()
        .zip(step.after)
        .filter_map(|(op, was)| match (op, was) {
            (KanbanOp::Header(before), Was::Header(was)) => {
                let (now, was, b) = (k.header.fields(), was.fields(), before.fields());
                let put = to_restore(&now, &was, &b);
                if put.is_empty() {
                    return None;
                }
                k.header.with_fields(&put).map(KanbanOp::Header)
            }
            (KanbanOp::Put(before), Was::Record(was)) => match (k.record(before.id()), was) {
                (Some(now), Some(was)) => {
                    let (n, w, b) = (now.fields(), was.fields(), before.fields());
                    let put = to_restore(&n, &w, &b);
                    if put.is_empty() {
                        return None;
                    }
                    now.with_fields(&put).map(KanbanOp::put)
                }
                (None, None) => Some(KanbanOp::Put(before)),
                _ => None,
            },
            // Desfazer um nascimento é a lápide: só se ninguém a mexeu depois.
            (op @ KanbanOp::Delete(id), Was::Record(was)) => match (k.record(id), was) {
                (Some(now), Some(was)) => {
                    let (n, w) = (now.fields(), was.fields());
                    (meta_field(&n) == meta_field(&w)).then_some(op)
                }
                _ => None,
            },
            _ => None,
        })
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

    /// Desfaz o último gesto (saltando os passos que os colegas esvaziaram). `false` = nada a desfazer.
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
        // Um passo que os colegas esvaziaram (mexeram em todos os seus campos) descarta-se e o gesto
        // passa ao anterior — nunca conta como feito nem deixa um passo vazio na outra pilha.
        while let Some((_, step)) = from.pop() {
            let ops = rebase(k, step);
            if ops.is_empty() {
                continue;
            }
            let inverse = match actor {
                Some(a) => apply_logged(k, ops, a),
                None => apply_batch(k, ops),
            };
            if inverse.is_empty() {
                continue;
            }
            let after = after(k, &inverse);
            to.push((at, Step { inverse, after }));
            return true;
        }
        false
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
#[path = "history_fields_tests.rs"]
mod fields_tests;
#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
