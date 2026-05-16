#![forbid(unsafe_code)]
//! ph2d-tool-registry — convention-by-discovery infrastructure.
//!
//! See `docs/Migracao/2026-05-convention-by-discovery.md` for the full
//! migration plan + rationale. This crate ships:
//!
//! - [`ToolManifest`] — declarative description of one tool.
//! - [`Registry`] — collects manifests, builds lookup indices.
//! - [`ActionInvocation`] — type-erased queued click / panel event.
//! - [`hash_node_id`] / [`detect_collisions`] — FNV-1a 64-bit
//!   `const fn` + collision detection (HR-5 cross-platform stable).
//! - [`IconHandle`] — typed wrapper for icon `fn` lookup.
//! - [`Zone`] — canvas zone enum (re-exported by
//!   `ph2d-editor::zones`).
//!
//! ### Why a separate crate
//!
//! Tool crates (`crates/ph2d-tool-<slug>/`) declare their
//! `ToolManifest` const at compile time. Each tool crate needs to
//! `import` `ToolManifest` / `Registry` / `Zone` / `Role`. If those
//! types lived in `ph2d-editor`, every tool crate would depend on
//! `ph2d-editor`, and `ph2d-editor` would depend on tool crates (to
//! call `register_all()`), producing a cycle Cargo refuses. Pulling
//! the registry into this tiny dep-light crate breaks the cycle.
//!
//! Per HR-1, this crate is platform-agnostic. Per ADR-0022, it uses
//! `BTreeMap` (not `HashMap`) for cross-platform deterministic
//! iteration ordering.

mod action;
mod icon_handle;
mod manifest;
mod node_id;
mod runtime;
mod zone;

pub use action::ActionInvocation;
pub use icon_handle::IconHandle;
pub use manifest::{
    BezPath, ClusterId, HandlerFn, LabelKey, McpExposure, Role, ToolHandler, ToolId, ToolManifest,
};
pub use node_id::{CollisionError, detect_collisions, hash_node_id};
pub use runtime::{BuildError, Registry};
pub use zone::Zone;
