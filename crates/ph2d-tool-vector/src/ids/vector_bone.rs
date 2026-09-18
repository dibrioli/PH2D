//! **Os ids do ESQUELETO** (estudo 42 item 5, doc 47) — módulo irmão de [`super`] pelo teto de 700
//! LOC, com o corte por RESPONSABILIDADE: aqui vive a família que faz um desenho **dobrar** — o
//! modo que autora ossos e a seção que os liga à forma.
//!
//! ⚠️ **Bloco APPEND-ONLY**: um id é o hash de uma STRING, então reordenar não quebra nada — mas
//! renomear uma string quebra tudo o que a referencia por nome, e é assim que um widget fica órfão
//! em silêncio.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_bone.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **Criar** — arrastar faz um osso; carregar num osso apenas o selecciona (é assim que se escolhe
/// onde ramificar).
pub const VECTOR_BONE_ACT_CREATE: NodeId = hash_node_id("vector.bone.action.create");

/// **Transformar** — arrastar posa o que está sob o cursor (girar · deslocar · força · IK), e
/// ⛔ nunca cria.
pub const VECTOR_BONE_ACT_TRANSFORM: NodeId = hash_node_id("vector.bone.action.transform");
