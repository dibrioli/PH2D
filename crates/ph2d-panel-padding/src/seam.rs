//! Padding panel seam — `populate` + `apply_event` generated from ONE widget
//! table via `ph2d_editor_core::panel_seam!` (blindagem Fase 2).
//!
//! Replaces the hand-written `populate.rs` + `event.rs`. The declaration below
//! IS the seam at a glance: four bipolar edge sliders (each paired with a px
//! chip) forward `SetValue`; Apply / Reset / pivot-toggle forward `Click`;
//! Cancel emits `CancelActiveTool`. A registered widget cannot be missing its
//! forward arm — both are emitted together. Behavior is verified byte-for-byte
//! against the previous hand-written code by `tests/seam.rs`.

use crate::ids;
use crate::state::PaddingPanelState;

/// Bipolar slider mapping: track `0.0..=1.0` → `±FULL_SCALE` px around the
/// neutral centre (`px = track * 2*FULL_SCALE - FULL_SCALE`).
const PAD_SCALE: f32 = 2.0 * ph2d_tool_padding::params::PAD_SLIDER_FULL_SCALE as f32;
const PAD_OFFSET: f32 = -(ph2d_tool_padding::params::PAD_SLIDER_FULL_SCALE as f32);

ph2d_editor_core::panel_seam! {
    state: PaddingPanelState,
    sliders: [
        (ph2d_tool_padding::ids::PAD_TOP, ph2d_tool_padding::ids::PAD_TOP_NUM, PAD_SCALE, PAD_OFFSET),
        (ph2d_tool_padding::ids::PAD_RIGHT, ph2d_tool_padding::ids::PAD_RIGHT_NUM, PAD_SCALE, PAD_OFFSET),
        (ph2d_tool_padding::ids::PAD_BOTTOM, ph2d_tool_padding::ids::PAD_BOTTOM_NUM, PAD_SCALE, PAD_OFFSET),
        (ph2d_tool_padding::ids::PAD_LEFT, ph2d_tool_padding::ids::PAD_LEFT_NUM, PAD_SCALE, PAD_OFFSET),
    ],
    forward_buttons: [ph2d_tool_padding::ids::PAD_APPLY, ph2d_tool_padding::ids::PAD_RESET, ph2d_tool_padding::ids::PAD_PIVOT_RECENTER],
    cancel_buttons: [ids::PAD_CANCEL],
}
