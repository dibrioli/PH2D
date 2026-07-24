//! **Os ids da seção Contour** — módulo irmão de [`super`] pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE, como o do `vector_textpath` e o do `vector_patternpath`: estes
//! são os controles do [`ph2d_ecs::VecContour`] — N anéis concêntricos com progressão de cor
//! (pesquisa `20_*` item #9, o efeito que a Corel publica como não tendo equivalente no
//! Illustrator). O irmão fica com os ids das outras seções.
//!
//! ⚠️ **Bloco APPEND-ONLY**, como os do Conector / Blend / Envelope / Text on Path / Pattern on
//! Path: um id é o hash de uma STRING, então reordenar não quebra nada — mas renomear uma string
//! quebra tudo o que a referencia por nome, e é assim que um widget fica órfão em silêncio.
//!
//! [`ph2d_ecs::VecContour`]: https://docs.rs/ph2d-ecs

use ph2d_a11y::NodeId;

use super::super::hash_node_id;

// ── Contour: N anéis concêntricos, do original até uma cor-alvo ─────────────────
// O efeito é o componente `ph2d_ecs::VecContour` na entidade da forma; presença = tem contour,
// ausência = forma nua. Estes ids são a única porta do PRODUTO para ele — sem eles o motor
// existiria, gateado e smokado, e não existiria para o artista.
/// Seção **CONTOUR** — a forma ganha N anéis concêntricos com uma rampa de cor.
pub const VECTOR_SECTION_CONTOUR: NodeId = hash_node_id("vector.section.contour");
/// **Add Contour** — arma o efeito na seleção. É a porta EXPLÍCITA, e é ela que resolve o
/// problema do swatch morto: sem contour armado não há para onde uma cor-alvo escrever, então
/// os controles (incluindo a swatch) só existem depois deste botão.
pub const VECTOR_CONTOUR_ADD: NodeId = hash_node_id("vector.contour.add");
/// **Remove Contour** — tira o efeito; a forma volta a desenhar-se sozinha e nada é materializado.
pub const VECTOR_CONTOUR_REMOVE: NodeId = hash_node_id("vector.contour.remove");
/// **Expand Contour** — materializa os anéis em formas REAIS na cena e descarta o efeito. É o
/// *Break Contour Apart* do Corel, e o irmão exato do Expand do Blend (ADR-0128 Fase D): o que
/// estava na tela passa a ser geometria editável ponto a ponto.
pub const VECTOR_CONTOUR_EXPAND: NodeId = hash_node_id("vector.contour.expand");
/// **Steps** — quantos anéis, além da forma. Inteiro (o track do slider é contínuo e arredonda).
pub const VECTOR_CONTOUR_STEPS: NodeId = hash_node_id("vector.contour.steps");
/// O campo numérico gêmeo do [`VECTOR_CONTOUR_STEPS`].
pub const VECTOR_CONTOUR_STEPS_NUM: NodeId = hash_node_id("vector.contour.steps.num");
/// **Offset** — a distância POR PASSO, bipolar, em percentual do tamanho da forma. Negativo
/// encolhe (o *Inside* do Corel) e cai da mesma aritmética, sem um segundo modo.
pub const VECTOR_CONTOUR_OFFSET: NodeId = hash_node_id("vector.contour.offset");
/// O campo numérico gêmeo do [`VECTOR_CONTOUR_OFFSET`].
pub const VECTOR_CONTOUR_OFFSET_NUM: NodeId = hash_node_id("vector.contour.offset.num");
/// **Accel** — a aceleração da progressão: `1` é linear, `>1` espalha os anéis para longe, `<1`
/// os amontoa perto da forma. É o knob que o Corel tem e o Illustrator não.
pub const VECTOR_CONTOUR_ACCEL: NodeId = hash_node_id("vector.contour.accel");
/// O campo numérico gêmeo do [`VECTOR_CONTOUR_ACCEL`].
pub const VECTOR_CONTOUR_ACCEL_NUM: NodeId = hash_node_id("vector.contour.accel.num");
/// **To** — a cor do ÚLTIMO anel. A swatch abre o picker OKLCH partilhado; o primeiro anel parte
/// da cor da FONTE, então a rampa tem os dois extremos sem o artista autorar o de partida.
pub const VECTOR_CONTOUR_TO: NodeId = hash_node_id("vector.contour.to");
/// **Corner: Miter** — a quina que o offset dos anéis produz. Mesmos códigos do Expand, resolvidos
/// pela MESMA porta (`vec_expand::join_of_code`).
pub const VECTOR_CONTOUR_JOIN_MITER: NodeId = hash_node_id("vector.contour.join.miter");
/// **Corner: Round** — ver [`VECTOR_CONTOUR_JOIN_MITER`]. É o default: a quina que faz um contour
/// parecer um contour.
pub const VECTOR_CONTOUR_JOIN_ROUND: NodeId = hash_node_id("vector.contour.join.round");
/// **Corner: Bevel** — ver [`VECTOR_CONTOUR_JOIN_MITER`].
pub const VECTOR_CONTOUR_JOIN_BEVEL: NodeId = hash_node_id("vector.contour.join.bevel");
/// **Side: Outer** — que contorno anda num compound (forma com furos). Mesmos códigos do Expand.
pub const VECTOR_CONTOUR_SIDE_OUTER: NodeId = hash_node_id("vector.contour.side.outer");
/// **Side: Inner** — ver [`VECTOR_CONTOUR_SIDE_OUTER`].
pub const VECTOR_CONTOUR_SIDE_INNER: NodeId = hash_node_id("vector.contour.side.inner");
/// **Side: Both** — ver [`VECTOR_CONTOUR_SIDE_OUTER`].
pub const VECTOR_CONTOUR_SIDE_BOTH: NodeId = hash_node_id("vector.contour.side.both");
