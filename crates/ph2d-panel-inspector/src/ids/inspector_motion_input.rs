//! **Os ids da secção MOTION INPUT** (ciclo 19 do Motion).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O campo do NOME da entrada.
pub const INSP_MI_NAME: NodeId = hash_node_id("insp_mi_name");
/// O segmentado da FONTE, pela ordem de `MotionSource::ALL` (gate de comprimento no teste da secção).
pub const INSP_MI_SOURCE: [NodeId; 4] = [
    hash_node_id("insp_mi_source_0"),
    hash_node_id("insp_mi_source_1"),
    hash_node_id("insp_mi_source_2"),
    hash_node_id("insp_mi_source_3"),
];
/// O número publicado (fonte `Value`).
pub const INSP_MI_VALUE: NodeId = hash_node_id("insp_mi_value");
/// A amostra da cor publicada (fonte `Color`).
pub const INSP_MI_COLOR: NodeId = hash_node_id("insp_mi_color");

/// **Os chips dos nomes que o grafo lê** — um por `value.input`.
///
/// ⚠️ Não é o cap de um modelo (o grafo não tem cap de nós Input): é o que a fileira mostra de uma
/// vez. A coluna de valor do dock leva ~4 chips por fileira, logo 16 são quatro fileiras; o que
/// passa daqui é DITO numa linha (`more_names`) e o campo do nome aceita qualquer texto.
pub const INSP_MI_GRAFO: [NodeId; 16] = [
    hash_node_id("insp_mi_grafo_00"),
    hash_node_id("insp_mi_grafo_01"),
    hash_node_id("insp_mi_grafo_02"),
    hash_node_id("insp_mi_grafo_03"),
    hash_node_id("insp_mi_grafo_04"),
    hash_node_id("insp_mi_grafo_05"),
    hash_node_id("insp_mi_grafo_06"),
    hash_node_id("insp_mi_grafo_07"),
    hash_node_id("insp_mi_grafo_08"),
    hash_node_id("insp_mi_grafo_09"),
    hash_node_id("insp_mi_grafo_10"),
    hash_node_id("insp_mi_grafo_11"),
    hash_node_id("insp_mi_grafo_12"),
    hash_node_id("insp_mi_grafo_13"),
    hash_node_id("insp_mi_grafo_14"),
    hash_node_id("insp_mi_grafo_15"),
];
