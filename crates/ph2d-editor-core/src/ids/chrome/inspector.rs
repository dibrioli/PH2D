//! Inspector panel container chrome NodeIds (INSP_*).
use super::{NodeId, hash_node_id};

/// Inspector panel container — used as the wheel-scroll key.
pub const INSP_PANEL: NodeId = hash_node_id("insp_panel");
/// Close (X) button at the top-right of the Inspector — toggles
/// `panel_visibility["inspector"]` same as the left-rail Inspector
/// pill. UI canon post-2026-05-24: every floating panel except
/// Hierarchy carries a close X.
pub const INSP_CLOSE: NodeId = hash_node_id("insp_close");
