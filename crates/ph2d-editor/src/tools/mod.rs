//! Example tools that prove the [`crate::tool::Tool`] trait shape.
//!
//! These are the seed two — Brush + Move — wired up enough for the
//! shell to switch between them and render their panels. Real
//! controls (sliders, color swatches, axis radios) lazy-bind once
//! the widget-primitives PR lands.

pub mod brush;
pub mod move_tool;
pub mod trim_transparency;

pub use brush::BrushTool;
pub use move_tool::MoveTool;
pub use trim_transparency::{Bounds, TrimResult, crop_bezpath, trim_transparency};
