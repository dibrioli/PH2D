//! Tool UI/action **vocabulary** the foundation still owns (ADR-0040).
//!
//! All tool *implementations* live in satellite crates (`ph2d-tool-*`).
//! After ADR-0040 TG-B the bgremoval vocabulary moved to
//! `ph2d_tool_bgremoval::params` — the generic `ToolPanelEvent` channel
//! and `BgRemovalTool::handle_panel_event` made the editor-core copy
//! unnecessary. Only padding still keeps a vocabulary module here until
//! TG-C completes the same migration:
//!
//! - [`padding`] — `PaddingUiEdit`/`UiSnapshot` + slider mapping
//!
//! The tool crate re-exports `padding::params` at its own root so its
//! moved `tool.rs` files resolve `super::params` unchanged. When TG-C
//! ships, `pub mod tools` disappears from `lib.rs` (ADR-0040 TG-D) and
//! this file is deleted.

pub mod padding;
