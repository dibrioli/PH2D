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

use super::super::hash_node_id;
use ph2d_a11y::NodeId;

// ── As duas reivindicações de POSIÇÃO (bloco APPEND-ONLY, plano 25 §9) ───────
// Elas são de espécie diferente do "Shapes" que já estava na seção: aquele ALINHA um eixo de
// cada vez, estes pousam o ponto num lugar. Por isso são interruptores próprios e não modos do
// mesmo — ligar "encaixar sobre a linha" não deve arrastar junto "encaixar no cruzamento".

/// **Snap to Path — desligado.** O ímã só olha pontos notáveis (âncoras, cantos de caixa).
pub const VECTOR_SNAP_PATH_OFF: NodeId = hash_node_id("vector.snap.path.off");
/// **Snap to Path — ligado.** O ponto pousa **sobre** a geometria, onde quer que ela passe.
pub const VECTOR_SNAP_PATH_ON: NodeId = hash_node_id("vector.snap.path.on");

/// **Snap to Intersections — desligado.**
pub const VECTOR_SNAP_CROSS_OFF: NodeId = hash_node_id("vector.snap.cross.off");
/// **Snap to Intersections — ligado.** O ponto pousa onde duas curvas se cruzam — um lugar que
/// o desenho produziu e que nenhuma âncora marca.
pub const VECTOR_SNAP_CROSS_ON: NodeId = hash_node_id("vector.snap.cross.on");
