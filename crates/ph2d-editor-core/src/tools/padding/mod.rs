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

// ADR-0040 T3: `PaddingTool` moved to crate `ph2d-tool-padding` (which
// already owned the `add_padding` algorithm). What stays here is the
// UI/action vocabulary the editor-core action bus and the
// `ph2d-panel-padding` crate read (foundation ⊥ tools); the tool crate
// re-exports this `params` module so its moved `tool.rs` resolves
// `super::params` unchanged.
pub mod params;

pub use params::{
    PAD_SLIDER_FULL_SCALE, PaddingUiEdit, PaddingUiSnapshot, px_to_slider, slider_to_px,
};
