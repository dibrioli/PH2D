//! Os ids do painel que não vêm da tabela de campos.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

pub const SEC_SOUND: NodeId = hash_node_id("sfxlab_sec_sound");
pub const SEC_LAYER: NodeId = hash_node_id("sfxlab_sec_layer");
pub const SEC_SOURCE: NodeId = hash_node_id("sfxlab_sec_source");
pub const SEC_SHAPE: NodeId = hash_node_id("sfxlab_sec_shape");

/// A caixa da lista de materiais.
pub const MATERIAL: NodeId = hash_node_id("sfxlab_material");
/// «Use Attack, Decay and Close»: troca uma curva sem esses três por uma que os tem.
pub const FORMA_HIT: NodeId = hash_node_id("sfxlab_forma_hit");

const BASE_DAS_OPCOES: NodeId = hash_node_id("sfxlab_material_opcao");

/// A opção `i` da lista de materiais. XOR com o índice: bijeção, não cria colisão.
#[must_use]
pub const fn opcao_de_material(i: usize) -> NodeId {
    NodeId(BASE_DAS_OPCOES.0 ^ (i as u64 + 1))
}
