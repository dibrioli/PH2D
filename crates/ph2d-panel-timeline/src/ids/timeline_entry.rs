//! O interruptor «Autoplay» do grupo de clipes (onda 4, item 1) — irmão de [`super::timeline`], que
//! está no tecto de LOC.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// ⭐ «Autoplay»: o clipe em edição é a ENTRADA da animação dele (o que toca sozinho no Play).
pub const TIMELINE_CLIP_AUTOPLAY: NodeId = hash_node_id("timeline.clip_autoplay");
