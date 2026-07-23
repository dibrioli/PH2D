//! **Os ids da seção Text on Path** — módulo irmão de [`super`] pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE e não por tamanho: estes são os controles do vínculo
//! texto ↔ caminho (plano 22), uma feature inteira e fechada. O irmão fica com os ids de
//! desenho, forma, estilo e das outras seções.
//!
//! ⚠️ **Bloco APPEND-ONLY**, como os do Conector / Blend / Envelope / Effects: um id é o hash
//! de uma STRING, então reordenar não quebra nada — mas renomear uma string quebra tudo o que
//! a referencia por nome, e é assim que um widget fica órfão em silêncio.

use ph2d_a11y::NodeId;

use super::super::hash_node_id;

// ── Text on Path: o texto CAVALGA uma curva (plano 22) ──────────────────────────
// O vínculo é o componente `ph2d_ecs::VecTextPath` na entidade do texto; presença = cavalga,
// ausência = texto reto. Estes ids são a única porta do PRODUTO para ele — sem eles a feature
// existiria no motor, gateada e smokada, e não existiria para o artista (a mesma frase que o
// envelope teve de escrever sobre si).
/// Seção **TEXT ON PATH** — o texto corre ao longo de uma curva, com os glyphs rígidos.
pub const VECTOR_SECTION_TEXTPATH: NodeId = hash_node_id("vector.section.textpath");
/// **Text on Path** — prende o texto selecionado à outra forma selecionada. Só é oferecido com a
/// seleção que o gesto exige (um texto + um caminho), fato que só a shell enxerga.
pub const VECTOR_TEXTPATH_LINK: NodeId = hash_node_id("vector.textpath.link");
/// **Pick Path** — a porta EXPLÍCITA do vínculo (Enio 2026-07-23): com o TEXTO em foco, apertar isto
/// arma o pick e o clique seguinte no canvas escolhe o caminho-guia, sem a seleção de dois que o
/// [`VECTOR_TEXTPATH_LINK`] exige.
pub const VECTOR_TEXTPATH_PICK: NodeId = hash_node_id("vector.textpath.pick");
/// **Detach** — solta o texto do caminho; ele volta a ser texto reto e o caminho fica.
///
/// Sem isto, prender seria porta de mão única — a mesma razão pela qual o envelope tem Release.
pub const VECTOR_TEXTPATH_DETACH: NodeId = hash_node_id("vector.textpath.detach");
/// **Other side** — põe o texto do outro lado da curva, a ler no sentido oposto.
pub const VECTOR_TEXTPATH_FLIP: NodeId = hash_node_id("vector.textpath.flip");
/// **This side** — o par exclusivo do [`VECTOR_TEXTPATH_FLIP`]. O lado é uma escolha entre DOIS,
/// e um segmentado a mostra sem o artista ter de descobrir o que "desmarcado" significa.
pub const VECTOR_TEXTPATH_FLIP_OFF: NodeId = hash_node_id("vector.textpath.flip.off");
/// **Offset** — onde a 1ª linha começa, em FRAÇÃO do comprimento do caminho (`startOffset` do SVG).
pub const VECTOR_TEXTPATH_OFFSET: NodeId = hash_node_id("vector.textpath.offset");
/// O campo numérico gêmeo do [`VECTOR_TEXTPATH_OFFSET`].
pub const VECTOR_TEXTPATH_OFFSET_NUM: NodeId = hash_node_id("vector.textpath.offset.num");
