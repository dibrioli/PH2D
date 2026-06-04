//! Widget `NodeId`s for the Geometry-Graph panel.
//!
//! Like the other panel crates, the IDs themselves stay defined in
//! editor-core (`ph2d_editor_core::ids`) — the layout + z-order walk +
//! `node_id_collisions` arch test all reference them, and re-defining
//! them here would fork the source of truth. This module is a
//! convenience re-export so the panel's internal modules can write
//! `crate::ids::VGRAPH_*`.

pub use ph2d_editor_core::ids::{
    VGRAPH_HEIGHT, VGRAPH_INNER_RATIO, VGRAPH_KIND, VGRAPH_PANEL, VGRAPH_ROTATION,
    VGRAPH_SAMPLES_PER_TURN, VGRAPH_SIDES, VGRAPH_TURNS, VGRAPH_WIDTH,
};
