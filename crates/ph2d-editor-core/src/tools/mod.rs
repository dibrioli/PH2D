//! Example tools that prove the [`crate::tool::Tool`] trait shape.
//!
//! These are the seed two — Brush + Move — wired up enough for the
//! shell to switch between them and render their panels. Real
//! controls (sliders, color swatches, axis radios) lazy-bind once
//! the widget-primitives PR lands.

pub mod bgremoval;
pub mod brush;
pub mod move_tool;
pub mod padding;
pub mod trim_transparency;
// `make_square` fully lives in crate `ph2d-tool-make-square` (ADR-0040
// T1.5): the editor-core facade + dep were removed so editor-core has
// zero `ph2d-tool-<slug>` dependency (foundation ⊥ tools, gated by
// `architecture_cycle_prevention`). Consumers call
// `ph2d_tool_make_square::{make_square, square_bezpath}` directly.
// `registry_init` moved to crate `ph2d-tool-registry-init` (PR 6.0
// retrofit). See `docs/Migracao/2026-05-convention-by-discovery.md`.

pub use brush::BrushTool;
pub use move_tool::MoveTool;
pub use padding::PaddingTool;
pub use trim_transparency::{Bounds, TrimResult, crop_bezpath, trim_transparency};
