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
    VECTOR_BONE_ACT_CREATE, VECTOR_BONE_ACT_TRANSFORM, VECTOR_BONE_ACT_WEIGHT,
    VECTOR_BONE_WEIGHT_ABS, VECTOR_BONE_WEIGHT_ADD, VECTOR_BONE_WEIGHT_CUMUL,
    VECTOR_BONE_WEIGHT_SUB,
};

/// Os TRÊS segmentos, **índice-alinhados** com [`ph2d_tool_vector::BoneAction::ALL`]. ⚠️ Alinhar
/// por índice é o que impede a lista do painel e a do vocabulário de divergirem em silêncio — e há
/// gate a compará-las (`a_fileira_de_verbos_do_osso_tem_um_segmento_por_accao`).
pub const VECTOR_BONE_ACTION_IDS: [NodeId; 3] = [
    VECTOR_BONE_ACT_CREATE,
    VECTOR_BONE_ACT_TRANSFORM,
    VECTOR_BONE_ACT_WEIGHT,
];

/// ⭐⭐⭐ **Os DOIS segmentos da direcção do pincel de peso**, **índice-alinhados** com
/// [`ph2d_tool_vector::WeightDirection::ALL`] (ordem do dono, 2026-09-19: *«no lugar de valores
/// negativos em Brush Strength prefiro botões Add e Subtract»*).
///
/// ⚠️ **Alinhar por índice é o que impede a lista do painel e a do vocabulário de divergirem em
/// silêncio** — a mesma lei da [`VECTOR_BONE_ACTION_IDS`], com o gate irmão a compará-las
/// (`a_fileira_da_direccao_do_peso_tem_um_segmento_por_lado`).
pub const VECTOR_BONE_WEIGHT_DIR_IDS: [NodeId; 2] =
    [VECTOR_BONE_WEIGHT_ADD, VECTOR_BONE_WEIGHT_SUB];

/// ⭐⭐⭐ **Os DOIS segmentos do MODO de atribuir peso**, **índice-alinhados** com
/// [`ph2d_tool_vector::WeightMode::ALL`] (ordem do dono, 2026-09-19: *«precisamos de 2 modos de
/// atribuir peso aos pontos»*).
///
/// ⚠️ **A mesma lei das duas listas acima** — alinhar por índice, com gate a compará-las
/// (`a_fileira_do_modo_do_peso_tem_um_segmento_por_modo`).
pub const VECTOR_BONE_WEIGHT_MODE_IDS: [NodeId; 2] =
    [VECTOR_BONE_WEIGHT_CUMUL, VECTOR_BONE_WEIGHT_ABS];

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

/// ⭐⭐⭐ **Curve Tip** — o chip que abre *quem manda na ponta da curva* (o *custom handle*, ordem
/// do dono de 2026-09-16).
///
/// ⚠️ **É o READOUT e o gesto**, como o selector de acção ao lado: o rótulo é a escolha ligada, e
/// um rótulo fixo obrigaria a abrir a lista para saber o que lá está.
pub const VECTOR_BONE_TIP: NodeId = hash_node_id("vector.bone.tip");
