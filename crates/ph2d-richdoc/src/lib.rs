//! `ph2d-richdoc` — documento rico em blocos (parágrafo, títulos 1–3, listas, checklist, citação,
//! código, régua) com marcas inline (negrito, itálico, riscado, código, link, menção), e as
//! operações de edição puras que o editor do Kanban conduz (W3, ADR-0195). Zero dependências.
//! O formato gravado é markdown, em `ph2d-kanban-md`.

mod doc;
mod edit;
mod editor;
mod inline;
mod shortcuts;

pub use doc::{Block, BlockKind, Caret, RichDoc, Selection};
pub use editor::Editor;
pub use inline::{Inline, Mark, MarkSpan, Run};
pub use shortcuts::Shortcut;
