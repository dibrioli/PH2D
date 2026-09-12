//! Background-Removal panel chrome NodeIds (BGR_*).
use super::{NodeId, hash_node_id};

/// Background-Removal panel container — the typed `ph2d-panel-bgremoval`
/// outer rect. Right-docked (same geometry slot as the Inspector) and
/// only visible while the `bgremoval` tool is active.
pub const BGR_PANEL: NodeId = hash_node_id("bgr_panel");

/// Extra-colour swatch hit slots 0..11. Painted only when the
/// corresponding extra colour exists (a fixed pool, like the Blender
/// palette's `BLENDER_SWATCH_*`). Capacity matches
/// `ph2d_tool_bgremoval::params::MAX_EXTRA_BG_COLORS`. Right-clicking a
/// painted slot removes that colour.
pub const BGR_SWATCH_0: NodeId = hash_node_id("bgr_swatch_0");
pub const BGR_SWATCH_1: NodeId = hash_node_id("bgr_swatch_1");
pub const BGR_SWATCH_2: NodeId = hash_node_id("bgr_swatch_2");
pub const BGR_SWATCH_3: NodeId = hash_node_id("bgr_swatch_3");
pub const BGR_SWATCH_4: NodeId = hash_node_id("bgr_swatch_4");
pub const BGR_SWATCH_5: NodeId = hash_node_id("bgr_swatch_5");
pub const BGR_SWATCH_6: NodeId = hash_node_id("bgr_swatch_6");
pub const BGR_SWATCH_7: NodeId = hash_node_id("bgr_swatch_7");
pub const BGR_SWATCH_8: NodeId = hash_node_id("bgr_swatch_8");
pub const BGR_SWATCH_9: NodeId = hash_node_id("bgr_swatch_9");
pub const BGR_SWATCH_10: NodeId = hash_node_id("bgr_swatch_10");
pub const BGR_SWATCH_11: NodeId = hash_node_id("bgr_swatch_11");

/// Fixed-pool extra-colour swatch ids, indexed 0..11.
pub const BGR_SWATCHES: [NodeId; 12] = [
    BGR_SWATCH_0,
    BGR_SWATCH_1,
    BGR_SWATCH_2,
    BGR_SWATCH_3,
    BGR_SWATCH_4,
    BGR_SWATCH_5,
    BGR_SWATCH_6,
    BGR_SWATCH_7,
    BGR_SWATCH_8,
    BGR_SWATCH_9,
    BGR_SWATCH_10,
    BGR_SWATCH_11,
];

/// Recover the extra-colour swatch index `0..12` from a `NodeId` when
/// it matches one of the [`BGR_SWATCHES`] pool consts. Used by the
/// shell's right-click-delete dispatch to map a hit id → list index.
pub fn bgr_swatch_index(id: NodeId) -> Option<usize> {
    BGR_SWATCHES.iter().position(|&s| s == id)
}
