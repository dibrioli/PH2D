//! ph2d-editor — DEPRECATED shim. Use `ph2d-editor-core` directly.
//!
//! ADR-0029 Phase B.5 stripped this crate down to a re-export of
//! `ph2d-editor-core`. All substantive content (HeroScreen, panel
//! registry, action bus, grid_snap, tools, image_edit, screens/*)
//! moved to `ph2d-editor-core` in Phase B.2 so panel crates can
//! depend on a single foundation crate without pulling the legacy
//! orchestrator god-crate.
//!
//! Existing `ph2d_editor::*` import paths continue to resolve here
//! transparently via the wildcard re-export below. New code SHOULD
//! import from `ph2d_editor_core::*` directly; this crate is targeted
//! for deletion in ADR-0030 (~6 months) once the call-site sweep
//! confirms no consumers remain on the `ph2d_editor::*` path.

#![forbid(unsafe_code)]

pub use ph2d_editor_core::*;
