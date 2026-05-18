//! ph2d-panel-hierarchy — Wave 7 Stage 2 panel-as-crate (alias form).
//!
//! Re-exports the in-tree [`ph2d_editor::screens::hero::hierarchy::PANEL_MANIFEST`]
//! so [`ph2d-panel-registry-init`] can register this panel under
//! its feature flag without coupling to ph2d-editor's internal
//! module path. Wave 8 will physically migrate the panel
//! implementation into this crate; for now the alias ensures the
//! external API path is stable.

#![forbid(unsafe_code)]

pub use ph2d_editor::screens::hero::hierarchy::PANEL_MANIFEST;
