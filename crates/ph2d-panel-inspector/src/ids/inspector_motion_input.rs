//! **Os ids da secção MOTION INPUT** (ciclo 19 do Motion).

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O campo do NOME da entrada.
pub const INSP_MI_NAME: NodeId = hash_node_id("insp_mi_name");
/// O segmentado da FONTE, pela ordem de `MotionSource::ALL` (gate de comprimento no teste da secção).
pub const INSP_MI_SOURCE: [NodeId; 10] = [
    hash_node_id("insp_mi_source_0"),
    hash_node_id("insp_mi_source_1"),
    hash_node_id("insp_mi_source_2"),
    hash_node_id("insp_mi_source_3"),
    hash_node_id("insp_mi_source_4"),
    hash_node_id("insp_mi_source_5"),
    hash_node_id("insp_mi_source_6"),
    hash_node_id("insp_mi_source_7"),
    hash_node_id("insp_mi_source_8"),
    hash_node_id("insp_mi_source_9"),
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

/// O chip «New…» no fim dos nomes do grafo: esvazia o nome e abre a caixa para um nome novo.
pub const INSP_MI_NEW: NodeId = hash_node_id("insp_mi_new");
/// A saída do aviso «sem Health»: anexa a `Health` (com o corpo que ela exige), com Undo.
pub const INSP_MI_ADD_HEALTH: NodeId = hash_node_id("insp_mi_add_health");

/// O From pelo NOME (fontes que leem um objecto) e o conta-gotas dele.
pub const INSP_MI_FROM: NodeId = hash_node_id("insp_mi_from");
/// O conta-gotas do From (a porta `ph2d-app-components::conta_gotas`).
pub const INSP_MI_FROM_PICK: NodeId = hash_node_id("insp_mi_from_pick");
/// A lista da acção do Input Map (fonte Action); as opções são o [`super::INSP_MI_ACTION_OPT`].
pub const INSP_MI_ACTION: NodeId = hash_node_id("insp_mi_action");
/// A lista do campo lido (fonte Property); as opções são o [`super::INSP_MI_PROPERTY_OPT`].
pub const INSP_MI_PROPERTY: NodeId = hash_node_id("insp_mi_property");
/// O grupo dobrável «Shape» (Remap, Curve, Smooth) — só com uma fonte que publica número.
pub const INSP_MI_FOLD_SHAPE: NodeId = hash_node_id("insp_mi_fold_shape");
/// A caixa «Remap».
pub const INSP_MI_REMAP: NodeId = hash_node_id("insp_mi_remap");
/// O número que vira `0` e o que vira `1` (a mesma linha).
pub const INSP_MI_REMAP_FROM: NodeId = hash_node_id("insp_mi_remap_from");
pub const INSP_MI_REMAP_TO: NodeId = hash_node_id("insp_mi_remap_to");
/// As predefinições da Curve, pela ordem de `CURVE_PRESETS`.
pub const INSP_MI_CURVE: [NodeId; 4] = [
    hash_node_id("insp_mi_curve_0"),
    hash_node_id("insp_mi_curve_1"),
    hash_node_id("insp_mi_curve_2"),
    hash_node_id("insp_mi_curve_3"),
];
/// A meia-vida da suavização, em segundos.
pub const INSP_MI_SMOOTH: NodeId = hash_node_id("insp_mi_smooth");
/// A saída da queixa «sem corpo» com a fonte Speed: anexa o Rigid Body, com Undo.
pub const INSP_MI_ADD_BODY: NodeId = hash_node_id("insp_mi_add_body");
/// A saída da queixa «From não acha ninguém»: arma o conta-gotas do From.
pub const INSP_MI_PICK_OBJECT: NodeId = hash_node_id("insp_mi_pick_object");
/// A saída da queixa «sem acções move_*» (fonte Move): abre a aba do Input Map.
pub const INSP_MI_OPEN_INPUT_MAP: NodeId = hash_node_id("insp_mi_open_input_map");
