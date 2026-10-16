//! **Os ids da PRECISÃO** (plano 25 §9, a W6) — módulo irmão de [`super`] pelo teto de 700 LOC
//! (`vector.rs` estava em 685 quando este nasceu).
//!
//! O corte é por RESPONSABILIDADE, como o do `vector_cut`: aqui moram os controles que decidem
//! **ONDE um ponto pousa** — o ímã e, mais adiante, as guias. Os ids de estilo, de forma e de
//! seção ficam no irmão. ⚠️ O `VECTOR_SNAP_OFF`/`_ON` de "Shapes" **fica onde está**: um id é o
//! hash de uma STRING, então movê-lo de arquivo é grátis mas renomeá-lo quebra tudo o que o
//! referencia — e não há motivo para mexer no que já shipa.
//!
//! ⚠️ **Bloco APPEND-ONLY.**
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/vector_snap.rs` em 2026-09-13** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

/// **Snap to Path — desligado.** O ímã só olha pontos notáveis (âncoras, cantos de caixa).
pub const VECTOR_SNAP_PATH_OFF: NodeId = hash_node_id("vector.snap.path.off");

/// **Snap to Path — ligado.** O ponto pousa **sobre** a geometria, onde quer que ela passe.
pub const VECTOR_SNAP_PATH_ON: NodeId = hash_node_id("vector.snap.path.on");

/// **Snap to Intersections — desligado.**
pub const VECTOR_SNAP_CROSS_OFF: NodeId = hash_node_id("vector.snap.cross.off");

/// **Snap to Intersections — ligado.** O ponto pousa onde duas curvas se cruzam — um lugar que
/// o desenho produziu e que nenhuma âncora marca.
pub const VECTOR_SNAP_CROSS_ON: NodeId = hash_node_id("vector.snap.cross.on");

/// **Snap to Guides — desligado.**
pub const VECTOR_SNAP_GUIDES_OFF: NodeId = hash_node_id("vector.snap.guides.off");

/// **Snap to Guides — ligado.** Nasce assim: num documento sem guias o ímã é inerte.
pub const VECTOR_SNAP_GUIDES_ON: NodeId = hash_node_id("vector.snap.guides.on");
