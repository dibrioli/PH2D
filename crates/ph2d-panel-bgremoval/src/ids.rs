//! Widget `NodeId`s for the Background-Removal panel.
//!
//! Most ids stay defined in editor-core (`ph2d_editor_core::ids`) — the
//! layout + z-order walk + `node_id_collisions` arch test all reference
//! them, and re-defining them in a panel crate would fork the source of
//! truth. This module re-exports them so the panel's internal modules
//! can write `crate::ids::BGR_*`.
//!
//! Ids added after the initial centralization may live in the tool crate
//! instead (`ph2d_tool_bgremoval::ids`) so a single feature lands without
//! editing the shared ids file — useful when a parallel agent is already
//! mutating it. They use the same `hash_node_id` mechanism, so the boot-
//! time collision check covers both namespaces.

pub use ph2d_editor_core::ids::{
    BGR_APPLY, BGR_BRUSH_SIZE, BGR_BRUSH_SIZE_NUM, BGR_CANCEL, BGR_EYEDROPPER,
    BGR_FALLOFF_CONSTANT, BGR_FALLOFF_SHARP, BGR_FALLOFF_SMOOTH, BGR_FALLOFF_SPHERE, BGR_FEATHER,
    BGR_FEATHER_NUM, BGR_GROW, BGR_GROW_NUM, BGR_MODE_CHROMA, BGR_MODE_GRABCUT, BGR_PANEL,
    BGR_PROTECT, BGR_PROTECT_CLEAR, BGR_REFINE, BGR_REFINE_NUM, BGR_RESET, BGR_SHOW_MASK,
    BGR_SWATCHES, BGR_TOLERANCE, BGR_TOLERANCE_NUM, bgr_swatch_index,
};

pub use ph2d_tool_bgremoval::ids::{
    BGR_MIN_ISLAND_PX, BGR_MIN_ISLAND_PX_NUM, BGR_SEPARATE_ISLANDS,
};
