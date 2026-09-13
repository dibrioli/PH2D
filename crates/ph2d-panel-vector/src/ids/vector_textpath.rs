//! **Os ids da seção Text on Path** — módulo irmão de [`super`] pelo teto de 700 LOC.
//!
//! O corte é por RESPONSABILIDADE e não por tamanho: estes são os controles do vínculo
//! texto ↔ caminho (plano 22), uma feature inteira e fechada. O irmão fica com os ids de
//! desenho, forma, estilo e das outras seções.
//!
//! ⚠️ **Bloco APPEND-ONLY**, como os do Conector / Blend / Envelope / Effects: um id é o hash
//! de uma STRING, então reordenar não quebra nada — mas renomear uma string quebra tudo o que
//! a referencia por nome, e é assim que um widget fica órfão em silêncio.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_textpath.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── Text on Path: o texto CAVALGA uma curva (plano 22) ──────────────────────────
// O vínculo é o componente `ph2d_ecs::VecTextPath` na entidade do texto; presença = cavalga,
// ausência = texto reto. Estes ids são a única porta do PRODUTO para ele — sem eles a feature
// existiria no motor, gateada e smokada, e não existiria para o artista (a mesma frase que o
// envelope teve de escrever sobre si).
/// Seção **TEXT ON PATH** — o texto corre ao longo de uma curva, com os glyphs rígidos.
pub const VECTOR_SECTION_TEXTPATH: NodeId = hash_node_id("vector.section.textpath");

/// O campo numérico gêmeo do [`VECTOR_TEXTPATH_OFFSET`].
pub const VECTOR_TEXTPATH_OFFSET_NUM: NodeId = hash_node_id("vector.textpath.offset.num");
