//! Stable [`NodeId`] constants for the Motion Nodes docked panels (Motion Nodes
//! M0.T9). The graph-editor panel occupies the `motion_graph` layout region; the
//! params panel takes over the Inspector slot (both visible only while the
//! `motion` tool is active — the shell's `motion_bridge` drives `panel_visible`).
//!
//! Re-exported through `ids/chrome/mod.rs` so the whole workspace reaches them as
//! `ph2d_editor_core::ids::MOTION_*` (FNV-1a slug hashing convention, `ids/mod.rs`).

use super::{NodeId, hash_node_id};

/// Motion Nodes graph-editor panel outer rect (docked in the `motion_graph`
/// region of [`crate::screens::layout::HeroLayout`]).
pub const MOTION_GRAPH_PANEL: NodeId = hash_node_id("motion.graph.panel");
/// Motion Nodes node-params panel outer rect (Inspector-slot takeover).
pub const MOTION_PARAMS_PANEL: NodeId = hash_node_id("motion.params.panel");
