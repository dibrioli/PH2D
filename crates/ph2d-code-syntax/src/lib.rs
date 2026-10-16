//! A sintaxe do editor de scripts (Onda 1 do CodeEditor, `docs/CodeEditor/02_onda1.md` E2).
//!
//! Puro, sem UI nem relógio: tree-sitter + gramática Luau sobre o [`Text`] de `ph2d-code-buffer`.
//! [`Syntax`] segue o texto pelas [`TextEdit`]s (reparse INCREMENTAL, lido da rope por pedaços) e
//! responde ao editor: realce por faixa, dobras, indentação ao Enter, erros e o par de um parêntese.
//! As queries vivem em `queries/` (cabeçalho de origem em cada uma).
//!
//! [`Text`]: ph2d_code_buffer::Text
//! [`TextEdit`]: ph2d_code_buffer::TextEdit

mod autoedit;
mod brackets;
mod errors;
mod fold;
mod highlight;
mod indent;
mod queries;
mod syntax;

pub use autoedit::{AutoEdit, IndentStyle, closer_of};
pub use errors::SyntaxError;
pub use fold::{Fold, fold_captures_visited};
pub use highlight::{HighlightKind, HighlightSpan};
pub use queries::{
    CaptureUse, FOLDS_SCM, HIGHLIGHTS_SCM, INDENTS_SCM, capture_table, highlight_captures,
    unevaluated_predicates,
};
pub use syntax::Syntax;
