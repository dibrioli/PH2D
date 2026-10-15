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

mod console;
mod doc;
mod docs;
mod edit;
mod find;
mod frame;
mod keys;
mod pointer;
pub mod problems;

pub use docs::{CodeDocs, SaveKind};
pub use frame::frame;

/// A declaração da família à shell. Sem cenas de smoke ainda (o `PH2D_CODE_SMOKE` é o E15).
pub const FAMILY: ph2d_app_host::AppFamily = ph2d_app_host::AppFamily {
    key: "code",
    routers: &[],
};
