//! **A família do EDITOR DE SCRIPTS** (CodeEditor Onda 1, `docs/CodeEditor/02_onda1.md`).
//!
//! Um script do projeto abre-se numa ABA ao lado da `Cena` (o trait
//! [`ph2d_editor_core::doc_surface::DocumentSurface`]): [`CodeDocs`] guarda um documento por
//! [`ScriptId`] aberto, responde ao teclado e ao rato da aba activa e pinta-a pela `ph2d-panel-code`.
//! A shell instala-a uma vez e chama [`frame`] uma vez por quadro — é aí que a família troca com a
//! cena: abre o que o cartão pediu, entrega o texto salvo à `ScriptLibrary`, lê as recargas (o selo),
//! o console e o diagnóstico, e pede a gravação do projeto.
//!
//! [`ScriptId`]: ph2d_script::ScriptId

mod actions;
mod cards;
pub mod commands;
pub use ph2d_code_assist::complete;
mod complete_doc;
mod console;
pub mod context_menu;
mod debug;
mod debug_card;
mod doc;
mod docs;
mod docs_probe;
mod draft;
mod drop_doc;
mod edit;
mod find;
mod folds;
mod format;
mod frame;
mod goto;
mod help;
use ph2d_code_assist::help_doc;
mod help_input;
mod hint;
mod inlay;
mod intel;
mod kanban;
mod keys;
mod link;
mod lint;
pub mod list;
use ph2d_code_assist::nav;
mod note;
mod object_rename;
pub mod options_card;
mod places;
mod play_problems;
mod pointer;
mod problem_list;
pub use ph2d_code_assist::problems;
mod reach;
use ph2d_code_assist::recovery;
mod refs;
mod rename;
mod scene_names;
use ph2d_code_assist::semantic;
use ph2d_code_assist::skeleton;
pub mod smoke;
pub mod smoke_drive;
use ph2d_code_assist::snippet;
mod snippet_doc;
mod strict;
mod surface;
pub mod tab_menu;

pub use debug::GameStop;
pub use docs::{CodeDocs, GrammarProbe, ParkedFootprint, SaveKind};
pub use format::FormatServer;
pub use frame::frame;
pub use help::{HELP_TAG, help_doc, member_id};
pub use help_doc::{DeadLink, dead_links, link_count};
pub use intel::IntelServer;
pub use object_rename::object_renamed;

/// A declaração da família à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "code",
    routers: smoke::ROUTERS,
};
