#![forbid(unsafe_code)]
//! ph2d-text — text layout via parley (M11).
//!
//! parley is Linebender's 2D text layout engine. It handles
//! line-breaking, BiDi, fallback font selection, and produces a
//! [`parley::Layout`] that downstream renderers (Vello, raster) can
//! consume to emit glyphs.
//!
//! M11 ships a thin wrapper:
//! - [`TextSystem`] owns the [`parley::FontContext`] +
//!   [`parley::LayoutContext`] (heavy state — created once per app).
//! - [`TextSystem::layout`] takes a `&str` + size and returns a
//!   ready `Layout<()>` with default-style choices appropriate for
//!   editor UI (system fallback chain, no italics, no weight
//!   override).
//!
//! Editor visual gate (Hello PH2D + CJK + emoji color fallback) is
//! deferred to M12 — it requires the editor's surface, not this
//! pure-data crate. The integration test here proves parley
//! produces a non-empty glyph layout for ASCII + CJK + emoji input.

pub mod system;

pub use parley::{FontContext, FontWeight, Layout, LayoutContext, PositionedLayoutItem};
pub use system::{TextSystem, active_text_rendering, set_active_text_rendering};
