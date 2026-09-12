//! Widget `NodeId`s for the Flip Style panel.
//!
//! Like the other panel crates, the ids stay defined in editor-core
//! (`ph2d_editor_core::ids`) — the layout + z-order walk + `node_id_collisions`
//! arch test all reference them, and re-defining them here would fork the
//! source of truth. This module is a convenience re-export so the panel's
//! internal modules can write `crate::ids::FLIP_*`.

pub use ph2d_editor_core::ids::{FLIP_COLORIZE_APPLY, FLIP_COLORIZE_CLEAR, FLIP_PANEL};

mod flip;
pub use flip::*;
