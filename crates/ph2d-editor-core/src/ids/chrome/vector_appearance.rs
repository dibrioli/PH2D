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

// ⭐⭐⭐ **A PILHA DE APARÊNCIA** (estudo 42 item 4, v20) — N preenchimentos e N contornos numa
// forma. Os ids de LINHA são de runtime (a lista tem tamanho variável), pela mesma lei das opções
// do dropdown acima: o índice vive um frame, e a resolução varre o espaço FIXO
// (`ph2d_vec_scene::MAX_PAINT_LAYERS`) para não depender de quantas camadas a forma tem hoje.

/// **+ Fill** — acrescenta um preenchimento no TOPO da pilha.
pub const VECTOR_PAINT_ADD_FILL: NodeId = hash_node_id("vector.paint.add.fill");

/// **+ Stroke** — acrescenta um contorno no TOPO da pilha.
pub const VECTOR_PAINT_ADD_STROKE: NodeId = hash_node_id("vector.paint.add.stroke");

/// O olho da camada `i` — desarma sem perder os parâmetros.
#[must_use]
pub fn vector_paint_eye_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.paint.eye.{i}"))
}

/// A swatch da camada `i` — abre o selector de cor DELA.
#[must_use]
pub fn vector_paint_swatch_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.paint.swatch.{i}"))
}

/// A linha da camada `i` — clicar ABRE-a (as propriedades dela aparecem por baixo).
#[must_use]
pub fn vector_paint_row_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.paint.row.{i}"))
}

/// Sobe a camada `i` uma posição na pilha.
#[must_use]
pub fn vector_paint_up_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.paint.up.{i}"))
}

/// Desce a camada `i` uma posição.
#[must_use]
pub fn vector_paint_down_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.paint.down.{i}"))
}

/// Apaga a camada `i`.
#[must_use]
pub fn vector_paint_del_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.paint.del.{i}"))
}

/// A largura do contorno da camada ABERTA.
pub const VECTOR_PAINT_WIDTH: NodeId = hash_node_id("vector.paint.width");

/// **ONDE a camada ABERTA desenha** — o deslocamento em `x`, relativo à forma.
pub const VECTOR_PAINT_DX: NodeId = hash_node_id("vector.paint.dx");

/// O gémeo em `y`. ⚠️ Dois campos e não um: a casa escreve um par de coordenadas como duas caixas
/// (`X`/`Y` do Transform, do Vertex), e um campo só obrigaria o artista a digitar uma sintaxe.
pub const VECTOR_PAINT_DY: NodeId = hash_node_id("vector.paint.dy");

/// ⭐⭐⭐ **O OFFSET DE CAD da camada ABERTA** — a silhueta cresce (`>0`) ou encolhe (`<0`).
///
/// ⛔ **Não confundir com o [`VECTOR_PAINT_DX`]/[`VECTOR_PAINT_DY`]**, que MOVEM a camada sem lhe
/// mudar a forma. São duas grandezas, e o painel chama-lhes `X`/`Y` e `Offset` — os nomes que o
/// artista já conhece do Illustrator e de um CAD.
pub const VECTOR_PAINT_DILATE: NodeId = hash_node_id("vector.paint.dilate");

/// A QUINA desse offset — `Miter`.
pub const VECTOR_PAINT_JOIN_MITER: NodeId = hash_node_id("vector.paint.join.miter");
/// `Round` — o default, pelo motivo que o `VecContour` já escreveu.
pub const VECTOR_PAINT_JOIN_ROUND: NodeId = hash_node_id("vector.paint.join.round");
/// `Bevel`.
pub const VECTOR_PAINT_JOIN_BEVEL: NodeId = hash_node_id("vector.paint.join.bevel");

/// A opacidade da camada ABERTA (0..100 %).
pub const VECTOR_PAINT_OPACITY: NodeId = hash_node_id("vector.paint.opacity");

/// O chip numérico do slider acima.
pub const VECTOR_PAINT_OPACITY_NUM: NodeId = hash_node_id("vector.paint.opacity.num");

/// O modo de mistura da camada ABERTA.
pub const VECTOR_PAINT_BLEND: NodeId = hash_node_id("vector.paint.blend");

/// A linha `i` da lista de modos da CAMADA.
///
/// ⚠️ Espaço de ids próprio, e não o do objecto: os dois popovers podem existir no mesmo frame, e
/// partilhar os ids faria um clique num deles resolver no outro.
#[must_use]
pub fn vector_paint_blend_option_id(i: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.paint.blendopt.{i}"))
}
