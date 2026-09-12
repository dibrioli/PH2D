//! **Wet Tuning** side-panel NodeIds (doc 22) — the full knob table of the
//! `ph2d-wet-paint` engine, shown beside the painter panel while the Wet
//! Paint section's Tuning checkbox is on.
//!
//! The knob rows are a DYNAMIC id family (the `painter_dynamic_ids` pattern):
//! the panel builds itself from the engine's `KNOB_DEFS` table, so the ids
//! derive from the knob KEY at runtime — no per-knob const to drift from the
//! table. Uniqueness across the family and against the static chrome ids is
//! gated in the panel crate (which sees the real keys) via
//! `wet_tuning_ids_dont_collide`.

use super::{NodeId, hash_node_id};

/// The panel surface itself (rect publish + z-order + scroll owner).
pub const WET_TUNING_PANEL: NodeId = hash_node_id("wet_tuning.panel");
