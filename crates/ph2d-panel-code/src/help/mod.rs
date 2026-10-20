//! ⭐ **A página de ajuda** (CodeEditor Onda 2, item 5; `docs/CodeEditor/04_pagina_de_ajuda.md`): a
//! referência de uma biblioteca numa aba do editor de scripts, como a do Godot 4.7.2.

mod a11y;
pub(crate) mod layout;
pub mod model;
mod paint;
mod query;
mod view;

pub use model::{Block, HelpDoc, Span, SpanKind};
pub use query::HelpProbe;
pub use view::{HelpFrame, HelpHit, HelpView};
