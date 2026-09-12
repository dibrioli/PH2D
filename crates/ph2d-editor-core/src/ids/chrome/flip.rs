//! Flip module chrome NodeIds (ADR-0114 W2 — docked `ph2d-panel-flip`).
//!
//! The `flip` tool's Brush / Color / Layers controls live in a right-docked
//! `Panel<State>` (the tool `FloatingPanel` is unpainted, mirror of the Vector
//! Style panel). Fixed chrome ids below (`FLIP_*`); the per-layer row widgets
//! use a runtime-hashed id family ([`flip_layer_widget_id`], mirror of the
//! Painter layers panel) since the layer count is only known at runtime.
use super::{NodeId, hash_node_id};

// ── Flip Style panel (docked `ph2d-panel-flip`) ──────────────────────────────
/// Flip panel outer rect id (for `z_order` + hit-barrier).
pub const FLIP_PANEL: NodeId = hash_node_id("flip.panel");

/// Apply: run the LazyBrush cut over the accumulated scribbles + the line-art, commit
/// each region as a filled stroke, and clear the scribble buffer.
pub const FLIP_COLORIZE_APPLY: NodeId = hash_node_id("flip.colorize.apply");
/// Clear: drop the accumulated scribbles without colouring.
pub const FLIP_COLORIZE_CLEAR: NodeId = hash_node_id("flip.colorize.clear");

// ── Frame strip (bottom-docked `ph2d-panel-flip-frames`, ADR-0114 W3) ────────
// The animator's inner loop: the cells of the active layer, the transport, Ghost
// Frames, autokey and the tween. Bottom dock (its own band — the global timeline
// is a separate, deferred integration, W6).

/// Frame-strip panel outer rect id (z-order + hit-barrier).
pub const FLIP_STRIP_PANEL: NodeId = hash_node_id("flip.strip.panel");
