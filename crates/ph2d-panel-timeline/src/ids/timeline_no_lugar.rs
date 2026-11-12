//! As lanes de dentro do container aberto no lugar (doc 17 item 10).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

use super::timeline::dynamic_id;

/// O olho das lanes de DENTRO do container aberto no lugar (doc 17 item 10). Mesmo tecto.
pub const TIMELINE_NO_LUGAR_MUTE: [NodeId; 8] = [
    hash_node_id("timeline.no_lugar_mute_0"),
    hash_node_id("timeline.no_lugar_mute_1"),
    hash_node_id("timeline.no_lugar_mute_2"),
    hash_node_id("timeline.no_lugar_mute_3"),
    hash_node_id("timeline.no_lugar_mute_4"),
    hash_node_id("timeline.no_lugar_mute_5"),
    hash_node_id("timeline.no_lugar_mute_6"),
    hash_node_id("timeline.no_lugar_mute_7"),
];

/// …e o peso delas.
pub const TIMELINE_NO_LUGAR_WEIGHT: [NodeId; 8] = [
    hash_node_id("timeline.no_lugar_weight_0"),
    hash_node_id("timeline.no_lugar_weight_1"),
    hash_node_id("timeline.no_lugar_weight_2"),
    hash_node_id("timeline.no_lugar_weight_3"),
    hash_node_id("timeline.no_lugar_weight_4"),
    hash_node_id("timeline.no_lugar_weight_5"),
    hash_node_id("timeline.no_lugar_weight_6"),
    hash_node_id("timeline.no_lugar_weight_7"),
];

/// A cópia irmã esmaecida (só a dica), por lane de dentro e strip.
#[must_use]
pub fn timeline_no_lugar_irma_id(lane: u64, strip: u64, volta: u64) -> NodeId {
    dynamic_id("timeline.no_lugar_irma", &[lane, strip, volta])
}
