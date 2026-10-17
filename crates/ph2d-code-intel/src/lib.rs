//! `ph2d-code-intel` — o trabalho do editor de scripts que NÃO cabe na tecla (CodeEditor Onda 2,
//! `docs/CodeEditor/03_onda2.md`): o reparse do tree-sitter e a sessão assíncrona do `luau-lsp`,
//! sobre instantâneos da rope. Sem UI; o quadro só envia e drena.

pub mod grammar;
pub mod lsp;
pub mod pos;

pub use grammar::GrammarWorker;
pub use lsp::{
    Answer, Completion, Diag, Edit, Event, Fix, Hover, LspSession, Req, Sem, SemToken, ServerState,
    Signature, Target, Ticket, TypeHint, uri_of,
};
pub use pos::{Dirty, byte_of, bytes_of, position_of, range_of};
