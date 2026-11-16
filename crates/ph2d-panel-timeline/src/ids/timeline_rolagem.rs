//! A barra de rolagem do tempo com alças de zoom (dono 16/11): uma alça por parte.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// As quatro partes da barra, pela ordem da `TimelineHitKind::TimeBar { part }`: a ponta esquerda, a
/// direita, o meio do polegar e a calha.
pub const TIMELINE_TIME_BAR: [NodeId; 4] = [
    hash_node_id("timeline.time_bar.start"),
    hash_node_id("timeline.time_bar.end"),
    hash_node_id("timeline.time_bar.thumb"),
    hash_node_id("timeline.time_bar.track"),
];
