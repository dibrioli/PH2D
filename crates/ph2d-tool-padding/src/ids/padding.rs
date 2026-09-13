//! Padding panel widget NodeIds (PAD_*).
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/padding.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

pub const PAD_TOP: NodeId = hash_node_id("pad_top");

pub const PAD_RIGHT: NodeId = hash_node_id("pad_right");

pub const PAD_BOTTOM: NodeId = hash_node_id("pad_bottom");

pub const PAD_LEFT: NodeId = hash_node_id("pad_left");

/// Per-edge px chips paired with the `PAD_*` sliders (real-time link).
pub const PAD_TOP_NUM: NodeId = hash_node_id("pad_top_num");

pub const PAD_RIGHT_NUM: NodeId = hash_node_id("pad_right_num");

pub const PAD_BOTTOM_NUM: NodeId = hash_node_id("pad_bottom_num");

pub const PAD_LEFT_NUM: NodeId = hash_node_id("pad_left_num");

/// Pivot-mode toggle: ON = recenter (recalculate translation so the
/// original content stays world-fixed); OFF = keep the pivot unchanged.
pub const PAD_PIVOT_RECENTER: NodeId = hash_node_id("pad_pivot_recenter");

pub const PAD_APPLY: NodeId = hash_node_id("pad_apply");

/// Reset-all button — returns every per-edge padding back to 0 and
/// the pivot-recenter toggle to its default. Also fires from the
/// tool's `on_activate`.
pub const PAD_RESET: NodeId = hash_node_id("pad_reset");
