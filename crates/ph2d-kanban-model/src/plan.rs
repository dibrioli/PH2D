//! **Um gesto que atravessa Kanbans** (v3: mover/copiar entre Kanbans, espelhos, mover uma lista).
//!
//! Os históricos são POR Kanban (ADR-0195 §5) e os ids são de um espaço por Kanban: um gesto entre
//! dois Kanbans é um [`Plan`] — um lote por Kanban. [`KanbanSet::apply_plan`] guarda UM passo no
//! [`History`] de cada Kanban tocado, todos com a MESMA hora (o `undo_tick` do gesto, que é único);
//! [`KanbanSet::undo_gesture`] desfaz juntos todos os passos com a hora mais recente — um `Ctrl+Z`
//! desfaz o gesto inteiro, nos dois Kanbans. Cada lado continua a respeitar o que outro escreveu
//! depois (o `rebase` do [`History`]).

use std::collections::BTreeMap;

use ph2d_frac::FracKey;

use crate::activity::Actor;
use crate::history::History;
use crate::ops::KanbanOp;
use crate::records::{KanbanId, ListId};
use crate::set::KanbanSet;

/// Os lotes de um gesto, por Kanban.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    steps: BTreeMap<KanbanId, Vec<KanbanOp>>,
}

impl Plan {
    pub fn push(&mut self, k: KanbanId, op: KanbanOp) {
        self.steps.entry(k).or_default().push(op);
    }

    pub fn extend(&mut self, k: KanbanId, ops: impl IntoIterator<Item = KanbanOp>) {
        self.steps.entry(k).or_default().extend(ops);
    }

    /// O lote do Kanban `k` (vazio = o gesto não lhe toca).
    #[must_use]
    pub fn ops(&self, k: KanbanId) -> &[KanbanOp] {
        self.steps.get(&k).map_or(&[], Vec::as_slice)
    }

    /// Os Kanbans que o gesto toca.
    pub fn kanbans(&self) -> impl Iterator<Item = KanbanId> + '_ {
        self.steps
            .iter()
            .filter(|(_, ops)| !ops.is_empty())
            .map(|(k, _)| *k)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.values().all(Vec::is_empty)
    }

    #[must_use]
    pub fn into_steps(self) -> BTreeMap<KanbanId, Vec<KanbanOp>> {
        self.steps
    }
}

/// Um sítio numa lista: o Kanban, a lista e a posição.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    pub kanban: KanbanId,
    pub list: ListId,
    pub pos: FracKey,
}

/// Uma lista de um Kanban do projecto.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListRef {
    pub kanban: KanbanId,
    pub list: ListId,
}

impl KanbanSet {
    /// Aplica o gesto: um passo no histórico de cada Kanban, todos à hora `at`, com a atividade em
    /// nome de `actor`.
    pub fn apply_plan(
        &mut self,
        plan: Plan,
        histories: &mut BTreeMap<KanbanId, History>,
        at: u64,
        actor: &Actor,
    ) {
        for (id, ops) in plan.into_steps() {
            if let Some(k) = self.doc_mut(id) {
                histories
                    .entry(id)
                    .or_default()
                    .apply_logged(k, ops, at, actor);
            }
        }
    }

    /// Desfaz o gesto mais recente em TODOS os Kanbans que ele tocou (os passos com a hora maior);
    /// o refazer guarda-os à hora `at`. `false` = nada a desfazer.
    pub fn undo_gesture(
        &mut self,
        histories: &mut BTreeMap<KanbanId, History>,
        at: u64,
        actor: &Actor,
    ) -> bool {
        self.swap_gesture(histories, at, actor, true)
    }

    /// Refaz o gesto desfeito mais recente, em todos os Kanbans dele.
    pub fn redo_gesture(
        &mut self,
        histories: &mut BTreeMap<KanbanId, History>,
        at: u64,
        actor: &Actor,
    ) -> bool {
        self.swap_gesture(histories, at, actor, false)
    }

    fn swap_gesture(
        &mut self,
        histories: &mut BTreeMap<KanbanId, History>,
        at: u64,
        actor: &Actor,
        undo: bool,
    ) -> bool {
        let when = |h: &History| if undo { h.undo_at() } else { h.redo_at() };
        let Some(newest) = histories.values().filter_map(when).max() else {
            return false;
        };
        let mut did = false;
        for (id, h) in histories.iter_mut() {
            if when(h) != Some(newest) {
                continue;
            }
            if let Some(k) = self.doc_mut(*id) {
                did |= if undo {
                    h.undo_logged(k, at, actor)
                } else {
                    h.redo_logged(k, at, actor)
                };
            }
        }
        did
    }
}
