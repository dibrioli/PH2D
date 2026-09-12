//! **Os ids da seção Pattern on Path** — módulo irmão de [`super`] pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE, como o do `vector_textpath`: estes são os controles do vínculo
//! MOTIVO ↔ caminho-guia (plano 23) — um motivo copiado, rígido, ao longo de uma curva. O irmão
//! fica com os ids das outras seções.
//!
//! ⚠️ **Bloco APPEND-ONLY**, como os do Conector / Blend / Envelope / Text on Path: um id é o hash
//! de uma STRING, então reordenar não quebra nada — mas renomear uma string quebra tudo o que a
//! referencia por nome, e é assim que um widget fica órfão em silêncio.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_patternpath.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 43 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── Pattern on Path: um MOTIVO copiado ao longo de uma curva (plano 23) ──────────
// O vínculo é o componente `ph2d_ecs::VecPatternPath` na entidade do motivo; presença = cavalga,
// ausência = forma solta. Estes ids são a única porta do PRODUTO para ele — sem eles a feature
// existiria no motor, gateada e smokada, e não existiria para o artista.
/// Seção **PATTERN ON PATH** — o motivo se repete, rígido, ao longo de uma curva.
pub const VECTOR_SECTION_PATTERNPATH: NodeId = hash_node_id("vector.section.patternpath");

/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_SPACING`].
pub const VECTOR_PATTERNPATH_SPACING_NUM: NodeId = hash_node_id("vector.patternpath.spacing.num");

/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_START`].
pub const VECTOR_PATTERNPATH_START_NUM: NodeId = hash_node_id("vector.patternpath.start.num");

/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_END`].
pub const VECTOR_PATTERNPATH_END_NUM: NodeId = hash_node_id("vector.patternpath.end.num");

/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_SLIDE`].
pub const VECTOR_PATTERNPATH_SLIDE_NUM: NodeId = hash_node_id("vector.patternpath.slide.num");

/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_OFFSET`].
pub const VECTOR_PATTERNPATH_OFFSET_NUM: NodeId = hash_node_id("vector.patternpath.offset.num");

/// O campo numérico gêmeo do [`VECTOR_PATTERNPATH_ROTATION`].
pub const VECTOR_PATTERNPATH_ROTATION_NUM: NodeId = hash_node_id("vector.patternpath.rotation.num");
