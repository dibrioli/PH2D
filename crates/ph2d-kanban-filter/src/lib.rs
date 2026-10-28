//! `ph2d-kanban-filter` — o que o Trello faz com os cartões sem os mudar (TrelloClone W6): o
//! **filtro** do quadro ([`Filter`], [`matches`]), a **pesquisa** global `/` com os operadores do
//! Trello ([`search`]), a **ordenação** de uma lista ([`sort_list`]) e o **envelhecimento** do
//! Power-Up Card Aging ([`age_level`]). Tudo puro e determinístico: o «agora», o fuso e o «eu» vêm
//! de quem chama — a UI na W6, as automações na W8.

mod age;
mod filter;
mod search;
mod sort;
#[cfg(test)]
mod test_support;
mod text;

pub use age::{AGE_DAYS, age_level, last_activity, last_activity_index};
pub use filter::{
    ActivityWindow, DueWindow, Filter, MatchMode, count, due_in, filter_entries, matches,
    matches_at,
};
pub use search::{Clause, Has, Hit, Is, Term, parse_query, search};
pub use sort::{SortBy, sort_keys, sort_list};
pub use text::normalize;
