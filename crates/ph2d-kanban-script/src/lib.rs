//! `ph2d-kanban-script` — **`ph2d.kanban`: os scripts Luau que mexem nos Kanbans do projecto**
//! (TrelloClone W8, lote L; ADR-0188: o script mora no projecto).
//!
//! As linhas da API, os tipos do `ph2d.d.luau` e a ajuda vivem na fonte única
//! (`ph2d_script::api_table_kanban`, `api_help_kanban`); aqui a fiação verdadeira ([`wire`]) e a
//! corrida ([`KanbanScripts`]): ler, escrever pelo `apply_logged` com o autor «script: <nome>», um
//! passo de desfazer por corrida, os mesmos acontecimentos e o mesmo anti-laço do Butler.
//!
//! Quem corre: o host dá o projecto, os históricos e um [`Ctx`]:
//! `scripts.run(nome, fonte, &mut set, &mut histories, &cx)`; depois de cada gesto de uma pessoa,
//! `scripts.react(&mut set, &mut histories, seen.events(&set, &me), &cx)`.

mod api;
mod card;
mod dates;
mod event;
mod handles;
mod lane;
mod run;

pub use api::wire;
pub use dates::{parse_iso, to_iso};
pub use event::{EVENT_NAMES, event_name};
pub use lane::script_member;
pub use run::{Ctx, KanbanScripts, Report, ScriptError};
