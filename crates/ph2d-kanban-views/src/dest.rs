//! As linhas do destino «Move to» (Kanban → listas), pela pesquisa: o seletor que o Planner mostra.

use ph2d_kanban_model::{KanbanSet, ListRef};

/// Uma linha da lista de destinos.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Row {
    /// O nome do Kanban (não se escolhe).
    Kanban(String),
    List(ListRef, String),
}

/// As linhas que a pesquisa `q` deixa: um Kanban cujo nome casa mostra todas as listas; senão, só
/// as listas que casam (e o Kanban delas por cima).
#[must_use]
pub fn rows(set: &KanbanSet, q: &str) -> Vec<Row> {
    let q = q.trim().to_lowercase();
    let mut out = Vec::new();
    for k in set.iter() {
        let all = q.is_empty() || k.header.name.to_lowercase().contains(&q);
        let lists: Vec<Row> = k
            .lists_in_order()
            .into_iter()
            .filter(|l| !l.archived && (all || l.title.to_lowercase().contains(&q)))
            .map(|l| {
                Row::List(
                    ListRef {
                        kanban: k.id,
                        list: l.id,
                    },
                    l.title.clone(),
                )
            })
            .collect();
        if !lists.is_empty() {
            out.push(Row::Kanban(k.header.name.clone()));
            out.extend(lists);
        }
    }
    out
}

/// A 1.ª lista das linhas (a escolha de partida).
#[must_use]
pub fn first_pick(rows: &[Row]) -> Option<ListRef> {
    rows.iter().find_map(|r| match r {
        Row::List(l, _) => Some(*l),
        Row::Kanban(_) => None,
    })
}
