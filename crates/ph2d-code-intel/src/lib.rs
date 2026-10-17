//! `ph2d-code-intel` — o trabalho do editor de scripts que NÃO cabe na tecla (CodeEditor Onda 2,
//! `docs/CodeEditor/03_onda2.md`): a sessão assíncrona do `luau-lsp` sobre instantâneos da rope.
//! Sem UI; o quadro só envia e drena.

pub mod lsp;
pub mod pos;

pub use lsp::{
    Answer, Completion, Diag, Event, Hover, LspSession, Req, ServerState, Ticket, uri_of,
};
pub use pos::{Dirty, byte_of, bytes_of, position_of, range_of};
