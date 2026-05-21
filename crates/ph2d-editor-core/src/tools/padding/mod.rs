//! Stateful Padding tool — condenses the legacy *Image Padding* +
//! *Directional Expand* into one panel with four signed per-edge fields
//! (top / right / bottom / left) and Apply / Cancel.
//!
//! v1: panel + Apply bake (resize the canvas via
//! `ph2d_tool_padding::add_padding`, swap the texture, reproject the
//! pivot). No live preview and no on-canvas gizmo edge-drag — both are
//! v2 (the directional-expand gizmo is the canvas half of the feature).
//!
//! Shape mirrors [`crate::tools::bgremoval`] (stateful tool + typed
//! panel crate + shell bridge + bake), minus the preview / scratch /
//! algorithm machinery.

pub mod params;
pub mod tool;

pub use params::{
    PAD_SLIDER_FULL_SCALE, PaddingUiEdit, PaddingUiSnapshot, px_to_slider, slider_to_px,
};
pub use tool::PaddingTool;
