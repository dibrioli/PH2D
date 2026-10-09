//! O filtro ▾ por componente (doc ObjectsAndComponents 01 §13, F6): o chip e as linhas da lista.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O chip ▾ ao lado da busca.
pub const HIER_FILTER_DD: NodeId = hash_node_id("hier_filter_dd");

/// Quantas linhas a lista pode ter: «All», cada componente autorado do catálogo e a Animação. O
/// recurso é o catálogo — o gate `the_filter_list_fits_every_component` prova que cabem todos.
pub const HIER_FILTER_ROWS: usize = 64;

/// A linha `i` da lista (0 = «All»), derivada do chip. ⚠️ Nos 58 bits BAIXOS, como as sub-linhas de
/// conteúdo: os altos são os companheiros de uma linha (olho, cadeado, grupo…) e o despacho lê-os.
#[must_use]
pub const fn hier_filter_opt(i: usize) -> NodeId {
    let h = super::rows::mix(HIER_FILTER_DD.0 ^ super::rows::mix(i as u64 + 0xF1));
    let low = h & super::rows::ROW_ID_MASK;
    NodeId(if low == 0 { 1 } else { low })
}
