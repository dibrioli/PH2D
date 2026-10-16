//! A pintura do editor de scripts (CodeEditor Onda 1, E4/E12 — `docs/CodeEditor/02_onda1.md`):
//! linhas visíveis com cache por linha, calha, seleção e cursores, barra de estado, acessibilidade
//! por linha. Sem estado da cena: a família (`ph2d-app-code`) entrega o buffer e as decorações
//! ([`CodeFrame`]) e guarda um [`CodeView`] por documento.

mod a11y;
mod chrome;
pub mod console;
mod display;
pub mod find_bar;
pub mod frame;
mod geom;
mod glyphs;
pub mod line_cache;
mod metrics;
mod paint;
mod palette;
pub mod script_list;
mod status;
pub mod style;
mod view;

pub use a11y::RUN_CHARS;
pub use console::{
    ConsoleHit, ConsoleRow, ConsoleSource, ConsoleView, title_text as console_title_text,
};
pub use find_bar::{FindBar, FindBarView, FindCount, FindField, FindHit, count_text};
pub use frame::{Badge, CodeFrame, FoldMark, Problem, Severity};
pub use geom::Column;
pub use glyphs::{PaintWork, paint_work};
pub use line_cache::LineCache;
pub use metrics::Metrics;
pub use script_list::{ListField, ListHit, ListRow, ScriptList, ScriptListView};
pub use status::{StatusLine, Tone};
pub use style::Ink;
pub use view::{CodeView, Hit, Underline};
