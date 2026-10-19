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
pub mod commands;
pub mod complete;
mod complete_doc;
mod console;
mod doc;
mod docs;
mod docs_probe;
mod edit;
mod find;
mod folds;
mod format;
mod frame;
mod goto;
mod hint;
mod inlay;
mod intel;
mod keys;
mod link;
mod lint;
pub mod list;
mod note;
mod places;
mod pointer;
pub mod problems;
mod reach;
mod recovery;
mod refs;
mod rename;
mod semantic;
mod skeleton;
pub mod smoke;
pub mod smoke_drive;
mod snippet;
mod snippet_doc;
mod strict;

pub use docs::{CodeDocs, GrammarProbe, ParkedFootprint, SaveKind};
pub use format::FormatServer;
pub use frame::frame;
pub use intel::IntelServer;

/// A declaração da família à shell.
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "code",
    routers: smoke::ROUTERS,
};
