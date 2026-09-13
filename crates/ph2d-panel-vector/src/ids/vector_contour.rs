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
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_contour.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── Contour: N anéis concêntricos, do original até uma cor-alvo ─────────────────
// O efeito é o componente `ph2d_ecs::VecContour` na entidade da forma; presença = tem contour,
// ausência = forma nua. Estes ids são a única porta do PRODUTO para ele — sem eles o motor
// existiria, gateado e smokado, e não existiria para o artista.
/// Seção **CONTOUR** — a forma ganha N anéis concêntricos com uma rampa de cor.
pub const VECTOR_SECTION_CONTOUR: NodeId = hash_node_id("vector.section.contour");

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
