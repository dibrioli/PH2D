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
use ph2d_tool_vector::ids::{
    VECTOR_BONE_ACT_CREATE, VECTOR_BONE_ACT_TRANSFORM, VECTOR_BONE_DEFORM_FAST,
    VECTOR_BONE_DEFORM_SMOOTH,
};

/// Os dois segmentos, **índice-alinhados** com [`ph2d_tool_vector::BoneAction::ALL`]. ⚠️ Alinhar
/// por índice é o que impede a lista do painel e a do vocabulário de divergirem em silêncio.
pub const VECTOR_BONE_ACTION_IDS: [NodeId; 2] = [VECTOR_BONE_ACT_CREATE, VECTOR_BONE_ACT_TRANSFORM];

/// Os dois segmentos, **índice-alinhados** com [`ph2d_tool_vector::SkinDeform::ALL`].
pub const VECTOR_BONE_DEFORM_IDS: [NodeId; 2] =
    [VECTOR_BONE_DEFORM_FAST, VECTOR_BONE_DEFORM_SMOOTH];

/// ⭐⭐⭐ **Action** — QUAL acção este osso percorre. O chip que abre a lista das acções da timeline.
///
/// ⚠️ **É o READOUT e o gesto ao mesmo tempo** (o idioma da tecla de uma forma do Morph): o rótulo
/// do chip é o nome da acção ligada, então *«qual é?»* responde-se sem abrir nada. Um rótulo fixo
/// tipo *"Choose…"* obrigaria a abrir a lista para saber o que lá está — e foi precisamente a
/// AUSÊNCIA desta linha que fez o dono ler o osso inteligente como avariado.
///
/// ⚠️ **Registado como `Dropdown`, pintado como botão** — abrir/fechar é do dispatch genérico, e
/// registá-lo como `Button` faria o clique acender e nunca abrir lista nenhuma.
pub const VECTOR_BONE_SMART_CLIP: NodeId = hash_node_id("vector.bone.smart.clip");
