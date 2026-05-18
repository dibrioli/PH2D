//! ph2d-editor-core — shared types + widgets + paint primitives.
//!
//! Wave 6+7 Phase 2 foundation. Houses the engine-side primitives
//! that any panel crate (`ph2d-panel-*`) or downstream shell can
//! depend on without pulling in the full `ph2d-editor` orchestrator.
//!
//! Public surface is re-exported by `ph2d-editor::lib.rs` so existing
//! consumers (shells, tool crates) keep their import paths working
//! transparently.

#![forbid(unsafe_code)]

pub mod floating_panel;
pub mod gizmo;
pub mod grid;
pub mod icons;
pub mod ids;
pub mod interaction;
pub mod paint;
pub mod panel;
pub mod project;
pub mod screen;
pub mod toast;
pub mod widget;
pub mod zen;
pub mod zones;
