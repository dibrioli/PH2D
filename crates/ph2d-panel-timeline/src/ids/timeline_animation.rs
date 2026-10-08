//! The «Animation: Hero ▾» chip's NodeIds (`TIMELINE_ANIM_*`, doc ObjectsAndComponents 01 §6) —
//! sibling of [`super::timeline`], which sits at the panel LOC cap.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── Whose animation (ObjectsAndComponents 01 §6) ────────────────────────────
/// The «Animation: Hero ▾» chip — whose animation the panel shows.
pub const TIMELINE_ANIM_DD: NodeId = hash_node_id("timeline.anim_dd");

/// Its options: the Scene, each ancestor with an Animation, the selection, and the offer to
/// add one. **The array is the resource** — the chrome cannot mint a hit id at runtime.
pub const TIMELINE_ANIM_OPT: [NodeId; 16] = [
    hash_node_id("timeline.anim_opt_0"),
    hash_node_id("timeline.anim_opt_1"),
    hash_node_id("timeline.anim_opt_2"),
    hash_node_id("timeline.anim_opt_3"),
    hash_node_id("timeline.anim_opt_4"),
    hash_node_id("timeline.anim_opt_5"),
    hash_node_id("timeline.anim_opt_6"),
    hash_node_id("timeline.anim_opt_7"),
    hash_node_id("timeline.anim_opt_8"),
    hash_node_id("timeline.anim_opt_9"),
    hash_node_id("timeline.anim_opt_10"),
    hash_node_id("timeline.anim_opt_11"),
    hash_node_id("timeline.anim_opt_12"),
    hash_node_id("timeline.anim_opt_13"),
    hash_node_id("timeline.anim_opt_14"),
    hash_node_id("timeline.anim_opt_15"),
];
