//! **Os ids da secção PARTICLES** (TOP-20 #18, W3).
//!
//! ⚠️ **As tabelas indexam pela ORDEM do modelo** (`ph2d_editor_core::particles_edits::
//! PARTICLES_NUMBERS` / `PARTICLES_TEXTS`): a linha `i` mostra o campo `i` e edita o campo `i`, e
//! há gate a atar os comprimentos. Uma segunda lista escrita à mão é o defeito que este painel já
//! pagou.

use ph2d_a11y::NodeId;

use ph2d_tool_registry::hash_node_id;

/// Os NÚMEROS da secção, na ordem da `PARTICLES_NUMBERS`.
pub const INSP_PART_NUM: [NodeId; 19] = [
    hash_node_id("insp_part_num_0"),
    hash_node_id("insp_part_num_1"),
    hash_node_id("insp_part_num_2"),
    hash_node_id("insp_part_num_3"),
    hash_node_id("insp_part_num_4"),
    hash_node_id("insp_part_num_5"),
    hash_node_id("insp_part_num_6"),
    hash_node_id("insp_part_num_7"),
    hash_node_id("insp_part_num_8"),
    hash_node_id("insp_part_num_9"),
    hash_node_id("insp_part_num_10"),
    hash_node_id("insp_part_num_11"),
    hash_node_id("insp_part_num_12"),
    hash_node_id("insp_part_num_13"),
    hash_node_id("insp_part_num_14"),
    hash_node_id("insp_part_num_15"),
    hash_node_id("insp_part_num_16"),
    hash_node_id("insp_part_num_17"),
    hash_node_id("insp_part_num_18"),
];

/// Os TEXTOS (os quatro sinais), na ordem da `PARTICLES_TEXTS`.
pub const INSP_PART_TEXT: [NodeId; 4] = [
    hash_node_id("insp_part_text_0"),
    hash_node_id("insp_part_text_1"),
    hash_node_id("insp_part_text_2"),
    hash_node_id("insp_part_text_3"),
];

/// A caixa **Emitting** — começa a emitir com a corrida?
pub const INSP_PART_EMITTING: NodeId = hash_node_id("insp_part_emitting");
/// A caixa **One Shot** — uma rajada só.
pub const INSP_PART_ONE_SHOT: NodeId = hash_node_id("insp_part_one_shot");
/// O segmentado da FORMA de nascimento (as quatro do `EmissionShape`, pela ordem do índice).
pub const INSP_PART_SHAPE: [NodeId; 4] = [
    hash_node_id("insp_part_shape_0"),
    hash_node_id("insp_part_shape_1"),
    hash_node_id("insp_part_shape_2"),
    hash_node_id("insp_part_shape_3"),
];
/// O segmentado do ESPAÇO (`World` · `Local`).
pub const INSP_PART_SPACE: [NodeId; 2] = [
    hash_node_id("insp_part_space_0"),
    hash_node_id("insp_part_space_1"),
];
/// A amostra da cor ao NASCER.
pub const INSP_PART_COLOR: NodeId = hash_node_id("insp_part_color");
/// A amostra da cor ao MORRER.
pub const INSP_PART_COLOR_END: NodeId = hash_node_id("insp_part_color_end");
