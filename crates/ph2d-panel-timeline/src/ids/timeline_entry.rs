//! O interruptor «Autoplay» do grupo de clipes (onda 4, item 1) — irmão de [`super::timeline`], que
//! está no tecto de LOC.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// ⭐ «Autoplay»: o clipe em edição é a ENTRADA da animação dele (o que toca sozinho no Play).
pub const TIMELINE_CLIP_AUTOPLAY: NodeId = hash_node_id("timeline.clip_autoplay");

/// The gap after marker `a` on the ruler (play that stretch in a loop; doc 17 item 2).
#[must_use]
pub fn timeline_marker_gap_hit_id(a: usize) -> NodeId {
    super::timeline::dynamic_id("timeline.marker_gap", &[a as u64])
}

/// The bar chip «takeoff → top ✕»: a click plays the whole clip again.
pub const TIMELINE_STRETCH_CLEAR: NodeId = hash_node_id("timeline.stretch_clear");
