//! **Os ids que a ferramenta de osso LÊ** — o verbo e o pincel de peso (a secção Bones do painel
//! emite-os). ⚠️ As STRINGS dos hashes ficam as de sempre (`vector.bone.*`): um id é o hash dela, e
//! renomeá-la orfanaria o widget em silêncio. Vieram da `ph2d-tool-vector` com o A14.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **Criar** — arrastar faz um osso; carregar num osso apenas o selecciona (é assim que se escolhe
/// onde ramificar).
pub const VECTOR_BONE_ACT_CREATE: NodeId = hash_node_id("vector.bone.action.create");

/// **Transformar** — arrastar posa o que está sob o cursor (girar · deslocar · força · IK), e
/// ⛔ nunca cria.
pub const VECTOR_BONE_ACT_TRANSFORM: NodeId = hash_node_id("vector.bone.action.transform");

/// ⭐⭐⭐ **Peso** — arrastar pinta a influência do osso aceso sobre a arte presa.
pub const VECTOR_BONE_ACT_WEIGHT: NodeId = hash_node_id("vector.bone.action.weight");

/// ⭐ **O RAIO do pincel de peso**, nas unidades do desenho.
pub const VECTOR_BONE_WEIGHT_RADIUS: NodeId = hash_node_id("vector.bone.weight.radius");

/// ⭐ **QUANTO cada pincelada empurra** — uma MAGNITUDE. ⛔ O sinal saiu daqui em 2026-09-19
/// (ordem do dono): para que lado é o [`VECTOR_BONE_WEIGHT_ADD`] / [`VECTOR_BONE_WEIGHT_SUB`].
pub const VECTOR_BONE_WEIGHT_AMOUNT: NodeId = hash_node_id("vector.bone.weight.amount");

/// ⭐⭐⭐ **Add** — a pincelada SOMA peso. Ver [`crate::params::WeightDirection`].
pub const VECTOR_BONE_WEIGHT_ADD: NodeId = hash_node_id("vector.bone.weight.add");

/// ⭐⭐⭐ **Subtract** — a pincelada TIRA peso. Ver [`crate::params::WeightDirection`].
pub const VECTOR_BONE_WEIGHT_SUB: NodeId = hash_node_id("vector.bone.weight.subtract");

/// ⭐⭐⭐ **Cumulative** — cada pincelada SOMA (ou tira) o *Brush Strength*. Ver
/// [`crate::params::WeightMode`].
pub const VECTOR_BONE_WEIGHT_CUMUL: NodeId = hash_node_id("vector.bone.weight.mode.cumulative");

/// ⭐⭐⭐ **Absolute** — a pincelada PÕE o *Brush Strength* no osso e reparte o resto pelos outros.
/// Ver [`crate::params::WeightMode`].
pub const VECTOR_BONE_WEIGHT_ABS: NodeId = hash_node_id("vector.bone.weight.mode.absolute");
