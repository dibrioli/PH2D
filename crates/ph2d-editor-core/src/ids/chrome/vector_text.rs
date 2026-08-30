//! Os ids da seção **TEXT** do painel do vetor — irmão de `vector` pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE, e ele é o mesmo que os outros `vector_*` já fazem: aqui vive
//! *o que um texto DIZ e como ele se dispõe* (tamanho, peso, fonte, alinhamento, entrelinha,
//! tracking, o refluxo) — e no pai fica *o que uma forma É*. `VECTOR_MODE_TEXT` **não** vem
//! junto de propósito: ele é um MODO, e mora com os outros três.

use super::painter::fnv_node_id_runtime;
use super::{NodeId, hash_node_id};

/// Text "Size" slider (world units) — shown only in Text mode; drives the glyph
/// size of the active session + the size a new session starts at.
pub const VECTOR_TEXT_SIZE: NodeId = hash_node_id("vector.text.size");
/// Value chip paired with [`VECTOR_TEXT_SIZE`].
pub const VECTOR_TEXT_SIZE_NUM: NodeId = hash_node_id("vector.text.size_num");
/// Text "Weight" slider (`wght` axis 100..900) — shown only in Text mode; drives the
/// variable-font weight of the active session + the weight a new session starts at.
pub const VECTOR_TEXT_WEIGHT: NodeId = hash_node_id("vector.text.weight");
/// Value chip paired with [`VECTOR_TEXT_WEIGHT`].
pub const VECTOR_TEXT_WEIGHT_NUM: NodeId = hash_node_id("vector.text.weight_num");
/// Text font-family picker prev / next buttons (`<` / `>`) — shown only in Text mode;
/// cycle the chosen system font family (or the bundled default) of the text.
pub const VECTOR_TEXT_FONT_PREV: NodeId = hash_node_id("vector.text.font_prev");
pub const VECTOR_TEXT_FONT_NEXT: NodeId = hash_node_id("vector.text.font_next");
/// Text "Import Font…" button — opens a native file picker for a `.ttf`/`.otf`,
/// loads it as the current text font (and adds it to the cycle).
pub const VECTOR_TEXT_FONT_IMPORT: NodeId = hash_node_id("vector.text.font_import");
/// Text font **dropdown** chip (between the `<` / `>` arrows) — a `Dropdown` whose
/// open popover lists every pickable family rendered **in its own outline** (real
/// style preview). Option clicks route by [`vector_text_font_option_id`].
pub const VECTOR_TEXT_FONT_DD: NodeId = hash_node_id("vector.text.font_dd");
/// Paragraph section (Text mode): horizontal alignment L / C / R (segmented, sets
/// `VecTextEdit::align`), line height (leading, × size) + its chip, and tracking
/// (letter-spacing, em fraction) + its chip.
pub const VECTOR_TEXT_ALIGN_LEFT: NodeId = hash_node_id("vector.text.align_left");
pub const VECTOR_TEXT_ALIGN_CENTER: NodeId = hash_node_id("vector.text.align_center");
pub const VECTOR_TEXT_ALIGN_RIGHT: NodeId = hash_node_id("vector.text.align_right");
pub const VECTOR_TEXT_LINE_HEIGHT: NodeId = hash_node_id("vector.text.line_height");
pub const VECTOR_TEXT_LINE_HEIGHT_NUM: NodeId = hash_node_id("vector.text.line_height_num");
pub const VECTOR_TEXT_TRACKING: NodeId = hash_node_id("vector.text.tracking");
pub const VECTOR_TEXT_TRACKING_NUM: NodeId = hash_node_id("vector.text.tracking_num");
/// **Width: Auto | Fixed** — o par que edita o `Option<f64>` do refluxo (`wrap_width`).
///
/// ⚠️ Dois chips e um slider que só vive num deles, e não um slider com um zero mágico: a
/// grandeza tem *presença* (reflui?) E *valor* (a que largura), e um `0` a significar "sem
/// caixa" seria um número que quer dizer duas coisas. É o par `Mass: Auto | Manual` do editor
/// de áudio — **só UMA row viva de cada vez**, porque as duas responderiam à mesma pergunta.
pub const VECTOR_TEXT_WRAP_AUTO: NodeId = hash_node_id("vector.text.wrap_auto");
pub const VECTOR_TEXT_WRAP_FIXED: NodeId = hash_node_id("vector.text.wrap_fixed");
pub const VECTOR_TEXT_WRAP_W: NodeId = hash_node_id("vector.text.wrap_w");
pub const VECTOR_TEXT_WRAP_W_NUM: NodeId = hash_node_id("vector.text.wrap_w_num");

/// Stable [`NodeId`] for the `index`-th family row in the open font dropdown
/// (index into the shell's pickable list `[bundled] ++ imported ++ system`). Runtime
/// `format!` (the family count is only known at runtime); the FNV twin keeps it in
/// the same id space as the `hash_node_id` consts. Mirrors the Painter option-id
/// fatories (`painter_brush_*_option_id`).
#[must_use]
pub fn vector_text_font_option_id(index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.text.fontopt.{index}"))
}

/// Max variation-axis number fields the Text panel shows (besides the dedicated
/// Weight slider) — one per non-`wght` axis the current font exposes.
///
/// ⚠️ **Este número é o TECTO DE TODOS OS CONSUMIDORES, e o painel tem de o honrar.** Até
/// 2026-08-30 ele era `6` e o pintor não o consultava: ele desenhava uma linha por eixo que a
/// fonte publicasse, **sem tecto**. Do 7.º em diante o campo saía com o **nome real do eixo** ao
/// lado e o valor `0` — porque o registo (`populate`), o mapa id→índice (`state`) e a publicação
/// da shell param todos aqui. *Um campo com o nome certo e nenhum leitor é a pior forma deste
/// defeito: ele convence.* Alcançável com a Roboto Flex, que publica ~12 eixos além do `wght`.
///
/// **De que recurso ele é:** não dos ids — eles são hasheados em runtime
/// (`fnv_node_id_runtime`) e um slot a mais custa uma iteração. É do **orçamento de linhas do
/// painel**, a mesma grandeza que já governa o resto do chrome.
///
/// **Por que 16:** o OpenType regista exactamente **cinco** tags de eixo (`ital`, `opsz`, `slnt`,
/// `wdth`, `wght`) — quatro depois de tirar o `wght`, que tem slider próprio. O resto são eixos
/// personalizados, e `fvar.axisCount` é `uint16`, logo o formato não dá tecto nenhum. `16` = os 4
/// registados + 12 personalizados, que cobre com folga a fonte variável mais rica que se envia
/// hoje. ⏳ **A perda que fica, nomeada:** uma fonte com mais de 16 eixos além do `wght` perde os
/// excedentes — mas perde-os **em silêncio e sem mentir**, em vez de os pintar mortos.
pub const MAX_TEXT_VARIATION_AXES: usize = 16;

/// NodeId for the `index`-th variation-axis field in the Text panel — bound to the
/// `index`-th non-`wght` axis of the current font (its name/range/value published by
/// the shell). Runtime `format!` (the axis set is per-font). Mirrors the font-option
/// factory.
#[must_use]
pub fn vector_text_axis_id(index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.text.axis.{index}"))
}
