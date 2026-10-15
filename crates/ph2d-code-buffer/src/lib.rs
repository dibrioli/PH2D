//! O texto editável do editor de scripts (Onda 1 do CodeEditor, `docs/CodeEditor/02_onda1.md` E1).
//!
//! Puro, sem UI: rope (`crop`) indexada por byte UTF-8, `ChangeSet` invertível, seleção
//! multicursor, histórico por grupos. Toda mutação passa por [`Buffer::apply`] e devolve as
//! [`TextEdit`]s para quem segue o texto (sintaxe, LSP).

mod buffer;
mod change;
mod find;
mod history;
mod motion;
mod selection;
mod text;

pub use buffer::{Buffer, Edits};
pub use change::{Assoc, Change, ChangeSet, TextEdit};
pub use find::{find_all, goto_line, replace_all, replace_one, FindOptions};
pub use history::{EditKind, History};
pub use motion::{is_word_char, word_at, word_left, word_right, Motion};
pub use selection::{Range, Selection};
pub use text::{display_width, normalize_newlines, Point, Text};
