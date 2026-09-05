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

use ph2d_a11y::NodeId;

use super::super::hash_node_id;
use super::painter::fnv_node_id_runtime;

/// O cabeçalho da seção **Appearance**.
pub const VECTOR_SECTION_APPEARANCE: NodeId = hash_node_id("vector.section.appearance");

/// **Opacity** — o slider da opacidade do objecto (0..100 %).
///
/// ⚠️ **Não confundir com os dois `VECTOR_*_OPACITY` da seção de estilo:** aqueles são o alfa da
/// TINTA que a ferramenta tem na mão (eles semeiam a forma seguinte e re-vestem a selecção), e
/// este é uma propriedade da forma SELECIONADA, que viaja no documento.
pub const VECTOR_OBJ_OPACITY: NodeId = hash_node_id("vector.obj.opacity");

/// O chip numérico do slider acima — o par que todo slider desta casa tem.
pub const VECTOR_OBJ_OPACITY_NUM: NodeId = hash_node_id("vector.obj.opacity.num");

/// **Blend** — o chip que abre a lista de modos de mistura do objecto.
///
/// ⚠️ Ele é um `Dropdown` e não uma segmentada pela mesma razão do ícone do widget: são dezanove
/// modos, e uma fileira de chips cortaria a lista num teto de tabela de ids.
pub const VECTOR_OBJ_BLEND: NodeId = hash_node_id("vector.obj.blend");

/// A linha `i` da lista de modos aberta.
///
/// ⚠️ **`i` indexa a lista OFERECIDA** (`ph2d_vec_render::blend::offered`), que é derivada da
/// tradução para o Vello — nunca o código do modo no documento. Um índice de runtime vive um
/// frame; o código viaja no ficheiro, e misturá-los é como um id de widget passa a depender de um
/// valor gravado.
#[must_use]
pub fn vector_obj_blend_option_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.obj.blendopt.{i}"))
}
