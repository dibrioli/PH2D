//! A escolha da vista de velocidade (plano 18, dono 15/11): Override · Retime, e o «Reset timing».

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// A escolha Override · Retime (a pílula de duas células).
pub const TIMELINE_SPEED_MODE: NodeId = hash_node_id("timeline.speed_mode");
/// «Override»: a alça de velocidade reescreve o easing (como sempre foi).
pub const TIMELINE_SPEED_OVERRIDE: NodeId = hash_node_id("timeline.speed_mode.override");
/// «Retime»: a alça muda só o RITMO do trecho; o easing fica.
pub const TIMELINE_SPEED_RETIME: NodeId = hash_node_id("timeline.speed_mode.retime");
/// «Reset timing»: as chaves seleccionadas voltam ao ritmo do easing.
pub const TIMELINE_RESET_TIMING: NodeId = hash_node_id("timeline.reset_timing");
