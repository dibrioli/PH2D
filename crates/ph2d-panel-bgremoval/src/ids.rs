//! Widget `NodeId`s for the Background-Removal panel.
//!
//! Like the other panel crates, the IDs themselves stay defined in
//! editor-core (`ph2d_editor_core::ids`) — the layout + z-order walk +
//! `node_id_collisions` arch test all reference them, and re-defining
//! them in a panel crate would fork the source of truth. This module is
//! a convenience re-export so the panel's internal modules can write
//! `crate::ids::BGR_*`.

pub use ph2d_editor_core::ids::{
    BGR_APPLY, BGR_CANCEL, BGR_EYEDROPPER, BGR_FEATHER, BGR_FEATHER_NUM, BGR_GROW, BGR_GROW_NUM,
    BGR_MODE_CHROMA, BGR_MODE_GRABCUT, BGR_PANEL, BGR_PROTECT, BGR_PROTECT_CLEAR, BGR_REFINE,
    BGR_REFINE_NUM, BGR_SWATCHES, BGR_TOLERANCE, BGR_TOLERANCE_NUM, bgr_swatch_index,
};
