//! Widget `NodeId`s for the Vector Style panel.
//!
//! Like the other panel crates, the ids stay defined in editor-core
//! (`ph2d_editor_core::ids`) — the layout + z-order walk + `node_id_collisions`
//! arch test all reference them, and re-defining them here would fork the
//! source of truth. This module is a convenience re-export so the panel's
//! internal modules (and the tool, via `handle_panel_event`) can write
//! `crate::ids::VECTOR_*`.

pub use ph2d_editor_core::ids::{
    VECTOR_CLOSE, VECTOR_FILL_NONE, VECTOR_FILL_SWATCH, VECTOR_PANEL, VECTOR_STROKE_SWATCH,
    VECTOR_WIDTH, VECTOR_WIDTH_NUM,
};
