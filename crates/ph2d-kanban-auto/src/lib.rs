//! `ph2d-kanban-auto` — o **Butler** do Kanban (TrelloClone W8): regras, botões de cartão e de
//! quadro, comandos de calendário e de prazo; o executor determinístico, a agenda, a frase de cada
//! automação para a i18n e as sugestões a partir do que se repete. Puro: o «agora», o fuso e quem
//! fez o gesto vêm de quem chama; o que sai é um [`Plan`](ph2d_kanban_model::Plan) a aplicar com o
//! autor [`Member::butler`](ph2d_kanban_model::Member::butler).
//!
//! Um gesto da pessoa: `let seen = Seen::plan(&set, &plan);` → aplicar → `react(&set,
//! seen.events(&set, &me), now, &zone)` → aplicar o `Outcome::plan` em nome do Butler.

mod act;
mod agenda;
mod cond;
mod describe;
mod event;
mod run;
mod spec;
mod suggest;
mod vars;

pub use agenda::{Firing, due_commands, due_moment, is_day, latest_occurrence, run_scheduled};
pub use cond::Cx;
pub use describe::{Part, Word, describe};
pub use event::{Event, Seen, What};
pub use run::{
    Fired, MAX_DEPTH, MAX_RUNS, Outcome, Stop, press_board_button, press_card_button, react,
    scope_cards,
};
pub use spec::{
    Action, Added, By, Condition, DateRule, Dest, DueIn, DueWhen, End, Every, Family, FieldTest,
    Kind, SPEC_VERSION, Scope, Sort, Spec, Test, TimeOfDay, Trigger, Unit, specs,
};
pub use suggest::{SUGGEST_MIN, SUGGEST_WINDOW_MS, Suggestion, suggest};
pub use vars::{VARIABLES, expand};
