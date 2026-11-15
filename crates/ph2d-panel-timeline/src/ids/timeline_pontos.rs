//! Os pontos da curva Custom no gráfico (dono 15/11): o alvo de cada ponto/alça, os pedaços da
//! linha da curva que criam um ponto, e os três tipos de alça na coluna da esquerda.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

use super::timeline::dynamic_id;

/// O alvo do nó `node` do trecho que sai de `key` em `target`; `part` `0` = o ponto, `1`/`2` =
/// as alças de chegada/saída.
#[must_use]
pub fn timeline_curve_point_hit_id(target: u64, key: u64, node: u8, part: u8) -> NodeId {
    dynamic_id(
        "timeline.curve_point",
        &[target, key, u64::from(node), u64::from(part)],
    )
}

/// O pedaço `i` da linha da curva do trecho que sai de `key` em `target`.
#[must_use]
pub fn timeline_curve_line_hit_id(target: u64, key: u64, i: u64) -> NodeId {
    dynamic_id("timeline.curve_line", &[target, key, i])
}

/// Corner: as duas alças do ponto escolhido andam soltas.
pub const TIMELINE_POINT_CORNER: NodeId = hash_node_id("timeline.point_kind.corner");
/// Smooth: a outra alça alinha, cada uma com o seu comprimento.
pub const TIMELINE_POINT_SMOOTH: NodeId = hash_node_id("timeline.point_kind.smooth");
/// Symmetric: a outra alça espelha, com o mesmo comprimento.
pub const TIMELINE_POINT_SYMM: NodeId = hash_node_id("timeline.point_kind.symm");
/// O grupo dos três.
pub const TIMELINE_POINT_KIND: NodeId = hash_node_id("timeline.point_kind");
