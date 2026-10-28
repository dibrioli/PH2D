//! ⭐ **Duas vistas, um documento** (ADR-0195 §3): a aba e o elemento do Quadro são cada um uma
//! [`KanbanView`] — a chave do Kanban e o que é SÓ DESSA vista. O que ela mostra é
//! [`KanbanView::shown`], uma projecção pura do documento: duas vistas sobre o mesmo Kanban não
//! sincronizam nada, leem o mesmo registo.
//!
//! Recolher uma lista é da vista (o Trello: «recolher é pessoal»), nunca do documento — senão recolher
//! na aba recolhia no canvas e na máquina do colega.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::kanban::Kanban;
use crate::records::{CardId, KanbanId, ListId};
use crate::set::KanbanSet;

/// O estado de UMA vista de um Kanban. Grava-se com quem a tem (o elemento do Quadro, a aba),
/// nunca no documento.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KanbanView {
    pub kanban: KanbanId,
    pub collapsed: BTreeSet<ListId>,
    /// A rolagem horizontal do quadro e a vertical de cada lista, em px da vista.
    pub scroll_x: f64,
    pub scroll_y: BTreeMap<ListId, f64>,
}

/// Uma lista como a vista a mostra.
#[derive(Clone, Debug, PartialEq)]
pub struct ShownList {
    pub id: ListId,
    pub title: String,
    pub collapsed: bool,
    pub cards: Vec<CardId>,
}

/// O que uma vista mostra agora (`None` = o Kanban já não existe).
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub name: String,
    pub lists: Vec<ShownList>,
}

impl KanbanView {
    #[must_use]
    pub fn of(kanban: KanbanId) -> Self {
        Self {
            kanban,
            ..Self::default()
        }
    }

    /// Recolhe ou abre a lista `list` NESTA vista.
    pub fn toggle_collapsed(&mut self, list: ListId) {
        if !self.collapsed.remove(&list) {
            self.collapsed.insert(list);
        }
    }

    /// O que esta vista mostra do conjunto `set`.
    #[must_use]
    pub fn shown(&self, set: &KanbanSet) -> Option<Shown> {
        set.get(self.kanban).map(|k| self.project(k))
    }

    fn project(&self, k: &Kanban) -> Shown {
        Shown {
            name: k.header.name.clone(),
            lists: k
                .lists_in_order()
                .into_iter()
                .map(|l| ShownList {
                    id: l.id,
                    title: l.title.clone(),
                    collapsed: self.collapsed.contains(&l.id),
                    cards: k.cards_in(l.id).into_iter().map(|c| c.id).collect(),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
