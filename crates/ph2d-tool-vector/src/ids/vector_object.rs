//! **A aparência do OBJECTO** (opacidade e mistura da forma selecionada) — desceram do
//! `ph2d-panel-vector` (CartoesUI C4): o Inspector em cartões também os pinta, e painel não depende
//! de painel.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::{hash_node_id, hash_node_id_runtime};

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

/// **Opacity** — o slider da opacidade do objecto (0..100 %).
///
/// ⚠️ **Não confundir com os dois `VECTOR_*_OPACITY` da seção de estilo:** aqueles são o alfa da
/// TINTA que a ferramenta tem na mão (eles semeiam a forma seguinte e re-vestem a selecção), e
/// este é uma propriedade da forma SELECIONADA, que viaja no documento.
pub const VECTOR_OBJ_OPACITY: NodeId = hash_node_id("vector.obj.opacity");

/// **Blend** — o chip que abre a lista de modos de mistura do objecto.
///
/// ⚠️ Ele é um `Dropdown` e não uma segmentada pela mesma razão do ícone do widget: são dezanove
/// modos, e uma fileira de chips cortaria a lista num teto de tabela de ids.
pub const VECTOR_OBJ_BLEND: NodeId = hash_node_id("vector.obj.blend");

/// O cabeçalho da seção **Appearance**.
pub const VECTOR_SECTION_APPEARANCE: NodeId = hash_node_id("vector.section.appearance");

/// **+ Fill** — acrescenta um preenchimento no TOPO da pilha.
pub const VECTOR_PAINT_ADD_FILL: NodeId = hash_node_id("vector.paint.add.fill");

/// **+ Stroke** — acrescenta um contorno no TOPO da pilha.
pub const VECTOR_PAINT_ADD_STROKE: NodeId = hash_node_id("vector.paint.add.stroke");

/// ⭐ F45 — **o arrasto pelo ⚙ largou uma linha**: o verbo que a shell resolve, com o par (linha,
/// lugar) guardado no estado do painel (`state::take_paint_move`).
pub const VECTOR_PAINT_MOVE: NodeId = hash_node_id("vector.paint.move");

/// ⭐ F45 — **o «+» de Fill & Stroke** (um `Dropdown`): as linhas são «Fill» e «Stroke».
pub const VECTOR_PAINT_ADD_MENU: NodeId = hash_node_id("vector.paint.add.menu");

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

/// ⭐ F44b — a POSIÇÃO do traço da camada aberta (o `StrokeSpec::align` dela): `Inside`.
pub const VECTOR_PAINT_ALIGN_INNER: NodeId = hash_node_id("vector.paint.align.inner");

/// `Center`.
pub const VECTOR_PAINT_ALIGN_CENTRE: NodeId = hash_node_id("vector.paint.align.centre");

/// `Outside`.
pub const VECTOR_PAINT_ALIGN_OUTER: NodeId = hash_node_id("vector.paint.align.outer");

/// ⭐ F47 — a caixa «Above Effects» do ⚙ do traço (o de base, ou o da camada aberta).
pub const VECTOR_STROKE_ABOVE_EFFECTS: NodeId = hash_node_id("vector.stroke.above_effects");

/// O chip numérico do slider acima.
pub const VECTOR_PAINT_OPACITY_NUM: NodeId = hash_node_id("vector.paint.opacity.num");

/// ⭐ F44 — a opacidade (%) na LINHA da camada `i` (Fill/Stroke no layout do Figma).
#[must_use]
pub fn vector_paint_row_opacity_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.paint.rowopacity.{i}"))
}

/// ⭐ F44c — o Weight na LINHA do contorno `i`.
#[must_use]
pub fn vector_paint_row_width_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.paint.rowwidth.{i}"))
}

/// ⭐ F44 — a linha do preenchimento de BASE: um `Dropdown` cujo painel abre à esquerda (tipo, cor,
/// degradê).
pub const VECTOR_FILL_ROW: NodeId = hash_node_id("vector.fill.row");

/// O X do painel do preenchimento de base.
pub const VECTOR_FILL_PROPS_CLOSE: NodeId = hash_node_id("vector.fill.props.close");

/// ⭐ F44 — o botão de ajustes do traço: um `Dropdown` cujo painel «Stroke settings» abre à esquerda.
pub const VECTOR_STROKE_SETTINGS: NodeId = hash_node_id("vector.stroke.settings");

/// O X do painel «Stroke settings».
pub const VECTOR_STROKE_PROPS_CLOSE: NodeId = hash_node_id("vector.stroke.props.close");

/// O X do painel de uma camada extra de Fill/Stroke.
pub const VECTOR_PAINT_PROPS_CLOSE: NodeId = hash_node_id("vector.paint.props.close");

/// A linha `i` da lista de modos da CAMADA.
///
/// ⚠️ Espaço de ids próprio, e não o do objecto: os dois popovers podem existir no mesmo frame, e
/// partilhar os ids faria um clique num deles resolver no outro.
#[must_use]
pub fn vector_paint_blend_option_id(i: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.paint.blendopt.{i}"))
}

// ── Desceu de `ph2d-editor-core/src/ids/chrome/vector_appearance.rs` em 2026-09-13 (2.ª passagem: a cerca com a
//    `line/render-loop` prendia-os na fundação até às duas linhas se integrarem).

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

/// A opacidade da camada ABERTA (0..100 %).
pub const VECTOR_PAINT_OPACITY: NodeId = hash_node_id("vector.paint.opacity");

/// O modo de mistura da camada ABERTA.
pub const VECTOR_PAINT_BLEND: NodeId = hash_node_id("vector.paint.blend");

/// ⭐ **Stroke (a caixa de marcar)** — *esta forma TEM traço?* (plano 34).
///
/// ⚠️ **Um checkbox e não um `segmented`**, pela lei que este painel já escreveu: *um `segmented` é
/// uma escolha entre MODOS nomeados; um checkbox é uma PROPRIEDADE que o objeto tem ou não tem*.
///
/// ⛔ **O rótulo NÃO é "Outline"**: já existe [`VECTOR_EXPAND_OUTLINE_STROKE`], que é outra coisa
/// (converter o traço numa forma preenchida). Duas palavras para um conceito e uma palavra para
/// dois é como um painel passa a mentir.
pub const VECTOR_STROKE_PRESENT: NodeId = hash_node_id("vector.stroke.present");

/// ⭐ **O «−» e o «+» do preenchimento de BASE** (F44b) — o par do [`VECTOR_STROKE_PRESENT`].
pub const VECTOR_FILL_PRESENT: NodeId = hash_node_id("vector.fill.present");
/// ⭐ **O olho do preenchimento de BASE** (F44b): esconde sem perder a tinta (`VecPath::fill_hidden`).
pub const VECTOR_FILL_EYE: NodeId = hash_node_id("vector.fill.eye");
/// O olho do traço de base (F44b).
pub const VECTOR_STROKE_EYE: NodeId = hash_node_id("vector.stroke.eye");
