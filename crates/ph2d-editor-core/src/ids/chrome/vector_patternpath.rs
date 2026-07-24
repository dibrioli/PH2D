//! **Os ids da seção Pattern on Path** — módulo irmão de [`super`] pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE, como o do `vector_textpath`: estes são os controles do vínculo
//! MOTIVO ↔ caminho-guia (plano 23) — um motivo copiado, rígido, ao longo de uma curva. O irmão
//! fica com os ids das outras seções.
//!
//! ⚠️ **Bloco APPEND-ONLY**, como os do Conector / Blend / Envelope / Text on Path: um id é o hash
//! de uma STRING, então reordenar não quebra nada — mas renomear uma string quebra tudo o que a
//! referencia por nome, e é assim que um widget fica órfão em silêncio.

use ph2d_a11y::NodeId;

use super::super::hash_node_id;

// ── Pattern on Path: um MOTIVO copiado ao longo de uma curva (plano 23) ──────────
// O vínculo é o componente `ph2d_ecs::VecPatternPath` na entidade do motivo; presença = cavalga,
// ausência = forma solta. Estes ids são a única porta do PRODUTO para ele — sem eles a feature
// existiria no motor, gateada e smokada, e não existiria para o artista.
/// Seção **PATTERN ON PATH** — o motivo se repete, rígido, ao longo de uma curva.
pub const VECTOR_SECTION_PATTERNPATH: NodeId = hash_node_id("vector.section.patternpath");
/// **Pattern on Path** — prende o motivo (o primário) à outra forma selecionada (o guia). Só é
/// oferecido com a seleção que o gesto exige (dois caminhos), fato que só a shell enxerga.
pub const VECTOR_PATTERNPATH_LINK: NodeId = hash_node_id("vector.patternpath.link");
/// **Pick Path** — o gesto de duas mãos (Enio 2026-07-23): com o MOTIVO selecionado, apertar isto
/// arma o pick e o clique seguinte no canvas escolhe o caminho-guia. É a porta EXPLÍCITA — a fonte é
/// a selecionada, o guia é o clicado —, sem a adivinhação por bbox do [`VECTOR_PATTERNPATH_LINK`].
pub const VECTOR_PATTERNPATH_PICK: NodeId = hash_node_id("vector.patternpath.pick");
/// **Detach** — solta o motivo do caminho; ele volta a ser uma forma solta e o caminho fica.
pub const VECTOR_PATTERNPATH_DETACH: NodeId = hash_node_id("vector.patternpath.detach");
/// **Other side** — põe o padrão do outro lado da curva, a percorrê-la ao contrário.
pub const VECTOR_PATTERNPATH_FLIP: NodeId = hash_node_id("vector.patternpath.flip");
/// **This side** — o par exclusivo do [`VECTOR_PATTERNPATH_FLIP`]. O lado é uma escolha entre DOIS.
pub const VECTOR_PATTERNPATH_FLIP_OFF: NodeId = hash_node_id("vector.patternpath.flip.off");
/// **Spacing** — o avanço por cópia, em múltiplos da largura do motivo (`1` encaixa borda-a-borda).
/// É o controle-assinatura: quão densamente as cópias povoam a curva.
pub const VECTOR_PATTERNPATH_SPACING: NodeId = hash_node_id("vector.patternpath.spacing");
/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_SPACING`].
pub const VECTOR_PATTERNPATH_SPACING_NUM: NodeId = hash_node_id("vector.patternpath.spacing.num");
/// **Start** — onde a tilagem começa, em FRAÇÃO do comprimento do caminho (`startOffset` do SVG).
pub const VECTOR_PATTERNPATH_START: NodeId = hash_node_id("vector.patternpath.start");
/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_START`].
pub const VECTOR_PATTERNPATH_START_NUM: NodeId = hash_node_id("vector.patternpath.start.num");
/// **End** — onde a tilagem termina, em FRAÇÃO do comprimento. `[Start, End]` é o trecho tilado.
pub const VECTOR_PATTERNPATH_END: NodeId = hash_node_id("vector.patternpath.end");
/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_END`].
pub const VECTOR_PATTERNPATH_END_NUM: NodeId = hash_node_id("vector.patternpath.end.num");
/// **Slide** — o CENTRO do trecho `[Start, End]`. Arrastar desliza as DUAS âncoras juntas ao longo
/// do caminho (o comprimento do trecho se preserva) — mover o padrão sem mudar sua extensão.
pub const VECTOR_PATTERNPATH_SLIDE: NodeId = hash_node_id("vector.patternpath.slide");
/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_SLIDE`].
pub const VECTOR_PATTERNPATH_SLIDE_NUM: NodeId = hash_node_id("vector.patternpath.slide.num");
/// **Offset** — desvio das cópias ao longo da NORMAL (perpendicular à curva), bipolar em unidades
/// de mundo. Empurra o padrão para fora/dentro da curva.
pub const VECTOR_PATTERNPATH_OFFSET: NodeId = hash_node_id("vector.patternpath.offset");
/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_OFFSET`].
pub const VECTOR_PATTERNPATH_OFFSET_NUM: NodeId = hash_node_id("vector.patternpath.offset.num");
/// **Rotation** — a ORIENTAÇÃO do motivo em cima da guia, em graus, bipolar (`0` = deitado ao longo
/// da curva, `90` = de pé, atravessado). É constante em todas as cópias e relativa à TANGENTE, não
/// ao mundo: numa curva as cópias continuam a acompanhar a guia e só ganham a orientação por cima.
pub const VECTOR_PATTERNPATH_ROTATION: NodeId = hash_node_id("vector.patternpath.rotation");
/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_ROTATION`].
pub const VECTOR_PATTERNPATH_ROTATION_NUM: NodeId = hash_node_id("vector.patternpath.rotation.num");
