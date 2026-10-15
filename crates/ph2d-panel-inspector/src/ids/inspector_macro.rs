//! **Os ids da secção MACRO** (ADR-0186, W5d).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O segmentado do MODO, pela ordem de `ModoDaMacro` (gate de comprimento na secção).
pub const INSP_MACRO_MODO: [NodeId; 2] = [
    hash_node_id("insp_macro_modo_0"),
    hash_node_id("insp_macro_modo_1"),
];
/// O botão «Edit steps…» — abre o painel MACROS, que lista as etapas.
pub const INSP_MACRO_EDITAR_ETAPAS: NodeId = hash_node_id("insp_macro_editar_etapas");
