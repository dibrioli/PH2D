//! Widget `NodeId`s for the Padding panel.
//!
//! Like the other panel crates, the IDs themselves stay defined in
//! editor-core (`ph2d_editor_core::ids`) — the layout + z-order walk +
//! `node_id_collisions` arch test all reference them, and re-defining
//! them here would fork the source of truth. This module is a
//! convenience re-export so the panel's internal modules can write
//! `crate::ids::PAD_*`.

pub use ph2d_editor_core::ids::{
    PAD_APPLY, PAD_BOTTOM, PAD_CANCEL, PAD_LEFT, PAD_PANEL, PAD_RIGHT, PAD_TOP,
};
