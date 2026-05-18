//! Example tools that prove the [`crate::tool::Tool`] trait shape.
//!
//! These are the seed two — Brush + Move — wired up enough for the
//! shell to switch between them and render their panels. Real
//! controls (sliders, color swatches, axis radios) lazy-bind once
//! the widget-primitives PR lands.

pub mod bgremoval;
pub mod brush;
pub mod make_square;
pub mod move_tool;
pub mod trim_transparency;
// `registry_init` moved to crate `ph2d-tool-registry-init` (PR 6.0
// retrofit). See `docs/Migracao/2026-05-convention-by-discovery.md`.

pub use brush::BrushTool;
pub use make_square::{MakeSquareResult, make_square, square_bezpath};
pub use move_tool::MoveTool;
pub use trim_transparency::{Bounds, TrimResult, crop_bezpath, trim_transparency};
