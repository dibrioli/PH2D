//! **Os ids da secção TWEEN** (suplente #22).
//!
//! ⚠️ **Irmão do [`super::inspector_timer`] por CAP de LOC**, e o molde é o dele pela mesma razão:
//! um `Tweens` guarda até [`ph2d_ecs::TWEENS_MAX`] tweens de **cinco** campos, e desenhar os cinco
//! em cada linha custaria dezenas de ids e uma coluna que não cabe na largura do Inspector. ⇒ a
//! **lista** escolhe qual tween está aberto, e um editor só, abaixo dela, mostra os campos desse.
//!
//! ⚠️ **A escolha da linha NÃO vai ao barramento** — é um facto da UI e vive no `InspectorState`,
//! como no irmão: *um `Tweens` não tem «o tween actual», os N correm todos ao mesmo tempo.*

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// `+ Add Tween`.
pub const INSP_TWEEN_ADD: NodeId = hash_node_id("insp_tween_add");

/// `x Remove Tween` — apaga o que está aberto.
pub const INSP_TWEEN_REMOVE: NodeId = hash_node_id("insp_tween_remove");

/// **As linhas da lista** — uma por tween, até ao cap de [`ph2d_ecs::TWEENS_MAX`].
///
/// ⚠️ O comprimento deste array **é** o cap do modelo, e há gate a prendê-los: *um modelo que
/// aceita o que o painel não mostra produz estado inalcançável por gesto nenhum.*
pub const INSP_TWEEN_ROW: [NodeId; 16] = [
    hash_node_id("insp_tween_row_00"),
    hash_node_id("insp_tween_row_01"),
    hash_node_id("insp_tween_row_02"),
    hash_node_id("insp_tween_row_03"),
    hash_node_id("insp_tween_row_04"),
    hash_node_id("insp_tween_row_05"),
    hash_node_id("insp_tween_row_06"),
    hash_node_id("insp_tween_row_07"),
    hash_node_id("insp_tween_row_08"),
    hash_node_id("insp_tween_row_09"),
    hash_node_id("insp_tween_row_10"),
    hash_node_id("insp_tween_row_11"),
    hash_node_id("insp_tween_row_12"),
    hash_node_id("insp_tween_row_13"),
    hash_node_id("insp_tween_row_14"),
    hash_node_id("insp_tween_row_15"),
];

/// **Os oito canais**, um chip cada. ⚠️ O comprimento é o do `ph2d_tween::Canal::ALL`, e há gate:
/// *um canal sem chip existe, tem lei, tem gates — e o artista não lhe chega.*
pub const INSP_TWEEN_CANAL: [NodeId; 8] = [
    hash_node_id("insp_tween_canal_0"),
    hash_node_id("insp_tween_canal_1"),
    hash_node_id("insp_tween_canal_2"),
    hash_node_id("insp_tween_canal_3"),
    hash_node_id("insp_tween_canal_4"),
    hash_node_id("insp_tween_canal_5"),
    hash_node_id("insp_tween_canal_6"),
    hash_node_id("insp_tween_canal_7"),
];

/// **As QUATRO componentes de `From`** — o canal diz quantas contam
/// ([`ph2d_tween::Canal::aridade`]), e o painel pinta uma ou quatro. ⛔ Dois conjuntos de ids —
/// um escalar e um de cor — seriam duas respostas a *«de onde?»*.
pub const INSP_TWEEN_DE: [NodeId; 4] = [
    hash_node_id("insp_tween_de_0"),
    hash_node_id("insp_tween_de_1"),
    hash_node_id("insp_tween_de_2"),
    hash_node_id("insp_tween_de_3"),
];

/// Idem para `To`.
pub const INSP_TWEEN_PARA: [NodeId; 4] = [
    hash_node_id("insp_tween_para_0"),
    hash_node_id("insp_tween_para_1"),
    hash_node_id("insp_tween_para_2"),
    hash_node_id("insp_tween_para_3"),
];

/// **As onze famílias de curva** — o comprimento é o do `ph2d_anim::EasingFamily::ALL`, com gate.
///
/// ⚠️ **Elas são pintadas em TRÊS fileiras de quatro**, e não numa de onze: a coluna do Inspector
/// tem ~300 px, e onze botões numa fileira dão ~25 px cada — um rótulo que não cabe é um chip que
/// o artista não lê. *O cap não é do modelo, é da largura.*
pub const INSP_TWEEN_FAMILIA: [NodeId; 11] = [
    hash_node_id("insp_tween_familia_00"),
    hash_node_id("insp_tween_familia_01"),
    hash_node_id("insp_tween_familia_02"),
    hash_node_id("insp_tween_familia_03"),
    hash_node_id("insp_tween_familia_04"),
    hash_node_id("insp_tween_familia_05"),
    hash_node_id("insp_tween_familia_06"),
    hash_node_id("insp_tween_familia_07"),
    hash_node_id("insp_tween_familia_08"),
    hash_node_id("insp_tween_familia_09"),
    hash_node_id("insp_tween_familia_10"),
];

/// **Os três modos** (`In` · `Out` · `In-Out`).
pub const INSP_TWEEN_MODO: [NodeId; 3] = [
    hash_node_id("insp_tween_modo_0"),
    hash_node_id("insp_tween_modo_1"),
    hash_node_id("insp_tween_modo_2"),
];

/// **O que acontece no fim** (`Hold` · `Rewind`).
pub const INSP_TWEEN_AO_ACABAR: [NodeId; 2] = [
    hash_node_id("insp_tween_ao_acabar_0"),
    hash_node_id("insp_tween_ao_acabar_1"),
];
