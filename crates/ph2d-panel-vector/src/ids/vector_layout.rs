//! **Os ids do AUTO LAYOUT** (plano UI/UX W2, ADR-0153) — irmão de [`super::vector_frame`] pelo
//! teto de 700 LOC, e o corte é o mesmo: aqui mora tudo o que a moldura que EMPILHA precisa.
//!
//! # Duas metades, e elas são perguntas independentes
//!
//! A seção *Layout* pinta dois blocos, e cada um responde a uma pergunta que o outro não faz:
//!
//! - **a moldura DISPÕE** (direção · vão · recuo · alinhamento) — o `VecLayout` do pai;
//! - **o filho se COMPORTA** (Grow/Shrink) — o `VecLayoutItem`.
//!
//! ⚠️ Elas **coexistem**, e é isso que uma moldura ANINHADA torna visível: ela empilha os próprios
//! filhos *e* é um item no fluxo do pai. Colapsá-las num bloco só faria a metade do item
//! desaparecer exactamente onde ela é mais útil.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_layout.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// O cabeçalho da seção **Layout** (só com uma moldura ou um filho de fluxo selecionado).
pub const VECTOR_SECTION_LAYOUT: NodeId = hash_node_id("vector.section.layout");

/// **Padding: All** — um campo só, que escreve os quatro lados.
///
/// ⚠️ O par All/Each **troca os campos pintados** em vez de acender um cadeado sobre quatro que
/// se movem juntos: quatro campos que espelham o mesmo número não dizem em qual se digita, e o
/// artista descobre por tentativa. É a forma do Figma.
pub const VECTOR_LAYOUT_PAD_ALL_MODE: NodeId = hash_node_id("vector.layout.pad.all_mode");

/// Ver [`VECTOR_LAYOUT_PAD_ALL_MODE`] — os quatro lados, cada um por si.
pub const VECTOR_LAYOUT_PAD_EACH_MODE: NodeId = hash_node_id("vector.layout.pad.each_mode");
