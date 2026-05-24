//! Widget `NodeId`s for the Equalize Sizes panel.
//!
//! Convention-mirror of `ph2d-panel-padding`: the IDs themselves are
//! defined in the **tool crate** (`ph2d_tool_equalize_sizes::ids`) so the
//! tool and the panel share a single source of truth — the tool's
//! `handle_panel_event` routes by these consts, the panel paint / event
//! / populate modules register them in the `WidgetStore`. This module is
//! a convenience re-export so the panel's internal modules can write
//! `crate::ids::EQS_*`.

pub use ph2d_editor_core::ids::{EQS_TITLE_COLOR, INSP_BLENDER_PICKER};

pub use ph2d_tool_equalize_sizes::ids::{
    EQS_ALG_LANCZOS, EQS_ALG_NEAREST, EQS_ALG_XBR, EQS_APPLY, EQS_ARRANGE_ON_GRID, EQS_CANCEL,
    EQS_FIXED_H, EQS_FIXED_W, EQS_GRID_OFFSET, EQS_GRID_OFFSET_NUM, EQS_MODE_FIXED, EQS_MODE_GRID,
    EQS_MODE_MAX, EQS_PANEL, EQS_RASTERIZE_AFTER, EQS_RESET, EQS_UPSCALE_IF_SMALLER,
};
