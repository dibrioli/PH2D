//! Convenience re-export of `ph2d_editor_core::grid_snap::ids` so the
//! panel crate's internal modules can write `crate::ids::GS_*` without
//! qualifying with the full editor-core path. The IDs themselves stay
//! defined in editor-core (the canvas renderer + topbar both reference
//! them, and moving them to a panel crate would create a cycle).

pub use ph2d_editor_core::grid_snap::ids::*;
