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
/// The panel's scrollbar.
pub const WET_TUNING_SCROLL: NodeId = hash_node_id("wet_tuning.scroll");
/// The panel's close button — forwards the SAME Tuning toggle the basic
/// section owns (visibility is the tool's authored fact).
pub const WET_TUNING_CLOSE: NodeId = hash_node_id("wet_tuning.close");

/// Collapsible group headers, in display order:
/// Paint · Water · Physics · Tools · Paper · Experimental.
pub const WET_TUNING_GROUP_HEADERS: [NodeId; 6] = [
    hash_node_id("wet_tuning.group_paint"),
    hash_node_id("wet_tuning.group_water"),
    hash_node_id("wet_tuning.group_physics"),
    hash_node_id("wet_tuning.group_tools"),
    hash_node_id("wet_tuning.group_paper"),
    hash_node_id("wet_tuning.group_experimental"),
];

/// Per-group reset buttons (the model's header `resetGroup`), same order as
/// the first five headers (Experimental has no reset — two checkboxes).
pub const WET_TUNING_GROUP_RESETS: [NodeId; 5] = [
    hash_node_id("wet_tuning.reset_paint"),
    hash_node_id("wet_tuning.reset_water"),
    hash_node_id("wet_tuning.reset_physics"),
    hash_node_id("wet_tuning.reset_tools"),
    hash_node_id("wet_tuning.reset_paper"),
];

/// The PAPER group's eye toggle — the `paperVisibility` master's on/off face
/// (the same authored fact as the basic section's Paper checkbox).
pub const WET_TUNING_PAPER_EYE: NodeId = hash_node_id("wet_tuning.paper_eye");

/// Experimental: Kubelka–Munk pigment mixing (`sim.km_mixing`).
pub const WET_TUNING_KM_MIXING: NodeId = hash_node_id("wet_tuning.km_mixing");
/// Experimental: Kubelka–Munk glaze layering (composite-side).
pub const WET_TUNING_KM_GLAZE: NodeId = hash_node_id("wet_tuning.km_glaze");

/// A knob row's SLIDER id, derived from the engine knob key (the runtime FNV
/// twin of [`hash_node_id`], same hash — `fnv_node_id_runtime` is gated to
/// agree with the const fn).
#[must_use]
pub fn wet_tuning_slider_id(key: &str) -> NodeId {
    super::painter::fnv_node_id_runtime(&format!("wet_tuning.slider.{key}"))
}

/// A knob row's number-CHIP id.
#[must_use]
pub fn wet_tuning_chip_id(key: &str) -> NodeId {
    super::painter::fnv_node_id_runtime(&format!("wet_tuning.chip.{key}"))
}

/// A knob row's per-knob RESET id.
#[must_use]
pub fn wet_tuning_reset_id(key: &str) -> NodeId {
    super::painter::fnv_node_id_runtime(&format!("wet_tuning.reset.{key}"))
}
