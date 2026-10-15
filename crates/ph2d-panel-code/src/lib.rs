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
pub mod line_cache;
mod metrics;
mod paint;
mod palette;
mod status;
pub mod style;
mod view;

pub use a11y::RUN_CHARS;
pub use console::{ConsoleHit, ConsoleRow, ConsoleView};
pub use find_bar::{count_text, FindBar, FindBarView, FindCount, FindField, FindHit};
pub use frame::{Badge, CodeFrame, FoldMark, Problem, Severity};
pub use geom::Column;
pub use line_cache::LineCache;
pub use metrics::Metrics;
pub use status::{StatusLine, Tone};
pub use style::Ink;
pub use view::{CodeView, Hit, Underline};
