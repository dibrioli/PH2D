//! Widget `NodeId`s for the Color Equalization panel — re-exports from
//! the tool crate's `ids` module (single source of truth in
//! `ph2d-tool-color-equalization/src/ids.rs`, derived via `hash_node_id`).
//!
//! Mirrors the `ph2d-panel-padding` convenience module — every panel
//! internal writes `crate::ids::CEQ_*`, and the tool's
//! `handle_panel_event` matches against the same hashes.

pub use ph2d_tool_color_equalization::ids::{
    CEQ_APPLY, CEQ_AUTO_WB, CEQ_BRIGHTNESS, CEQ_BRIGHTNESS_NUM, CEQ_CANCEL, CEQ_CLIP_LIMIT,
    CEQ_CLIP_LIMIT_NUM, CEQ_CONTRAST, CEQ_CONTRAST_NUM, CEQ_PANEL, CEQ_SATURATION,
    CEQ_SATURATION_NUM, CEQ_TILE_GRID, CEQ_TILE_GRID_NUM,
};
