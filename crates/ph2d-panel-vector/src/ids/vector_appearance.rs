//! **Os ids da APARÊNCIA DO OBJECTO** (estudo 42 item 2, v19 do schema) — irmão do
//! [`super::vector`] pelo teto de LOC.
//!
//! O corte é por ASSUNTO: aqui mora *quão opaca esta forma é, e como ela se mistura com o que está
//! por baixo* — o que o Illustrator põe no painel *Transparency* e o Figma na fileira de baixo do
//! *Fill*.
//!
//! ⚠️ **Nenhuma destas propriedades é a TINTA**, e os ids ficarem noutro ficheiro é o que mantém
//! isso legível: o alfa de uma cor descreve UMA marca, e estes dois descrevem o OBJECTO — a
//! diferença vê-se onde uma forma desenha mais de uma marca (traço sobre preenchimento).
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_appearance.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

/// O cabeçalho da seção **Appearance**.
pub const VECTOR_SECTION_APPEARANCE: NodeId = hash_node_id("vector.section.appearance");

/// O chip numérico do slider acima — o par que todo slider desta casa tem.
pub const VECTOR_OBJ_OPACITY_NUM: NodeId = hash_node_id("vector.obj.opacity.num");

/// A linha `i` da lista de modos aberta.
///
/// ⚠️ **`i` indexa a lista OFERECIDA** (`ph2d_vec_render::blend::offered`), que é derivada da
/// tradução para o Vello — nunca o código do modo no documento. Um índice de runtime vive um
/// frame; o código viaja no ficheiro, e misturá-los é como um id de widget passa a depender de um
/// valor gravado.
#[must_use]
pub fn vector_obj_blend_option_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.obj.blendopt.{i}"))
}

/// **+ Fill** — acrescenta um preenchimento no TOPO da pilha.
pub const VECTOR_PAINT_ADD_FILL: NodeId = hash_node_id("vector.paint.add.fill");

/// **+ Stroke** — acrescenta um contorno no TOPO da pilha.
pub const VECTOR_PAINT_ADD_STROKE: NodeId = hash_node_id("vector.paint.add.stroke");

/// O olho da camada `i` — desarma sem perder os parâmetros.
#[must_use]
pub fn vector_paint_eye_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.paint.eye.{i}"))
}

/// A swatch da camada `i` — abre o selector de cor DELA.
#[must_use]
pub fn vector_paint_swatch_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.paint.swatch.{i}"))
}

/// A linha da camada `i` — clicar ABRE-a (as propriedades dela aparecem por baixo).
#[must_use]
pub fn vector_paint_row_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.paint.row.{i}"))
}

/// Sobe a camada `i` uma posição na pilha.
#[must_use]
pub fn vector_paint_up_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.paint.up.{i}"))
}

/// Desce a camada `i` uma posição.
#[must_use]
pub fn vector_paint_down_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.paint.down.{i}"))
}

/// Apaga a camada `i`.
#[must_use]
pub fn vector_paint_del_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.paint.del.{i}"))
}

/// A QUINA desse offset — `Miter`.
pub const VECTOR_PAINT_JOIN_MITER: NodeId = hash_node_id("vector.paint.join.miter");

/// `Round` — o default, pelo motivo que o `VecContour` já escreveu.
pub const VECTOR_PAINT_JOIN_ROUND: NodeId = hash_node_id("vector.paint.join.round");

/// `Bevel`.
pub const VECTOR_PAINT_JOIN_BEVEL: NodeId = hash_node_id("vector.paint.join.bevel");

/// O chip numérico do slider acima.
pub const VECTOR_PAINT_OPACITY_NUM: NodeId = hash_node_id("vector.paint.opacity.num");

/// A linha `i` da lista de modos da CAMADA.
///
/// ⚠️ Espaço de ids próprio, e não o do objecto: os dois popovers podem existir no mesmo frame, e
/// partilhar os ids faria um clique num deles resolver no outro.
#[must_use]
pub fn vector_paint_blend_option_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.paint.blendopt.{i}"))
}
