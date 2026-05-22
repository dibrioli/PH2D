//! Tool UI/action **vocabulary** that the foundation owns (ADR-0040).
//!
//! All tool *implementations* now live in satellite crates
//! (`ph2d-tool-*`). What remains here is only the vocabulary the
//! editor-core action bus and the `ph2d-panel-*` crates read for the
//! stateful image tools — kept here to avoid an `editor-core → tool`
//! cycle until the generic action channel lands (ADR-0040 T1.2/T1.3):
//!
//! - [`bgremoval`] — `BgRemovalUiEdit`/`UiSnapshot`/`BrushFalloff` + params
//! - [`padding`] — `PaddingUiEdit`/`UiSnapshot` + slider mapping
//!
//! Tool crates re-export these modules at their own root so their
//! moved `tool.rs` files resolve `super::params` unchanged. Everything
//! else (Brush, Move, BgRemoval, Padding, Trim, Make Square, Real Size)
//! is a `ph2d-tool-*` crate; `editor-core/src/tools/` holds no tool
//! impl (foundation ⊥ tools, gated by `architecture_cycle_prevention`).

pub mod bgremoval;
pub mod padding;
