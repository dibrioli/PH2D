//! **Os ids do catálogo de PERFIS de largura** (plano 25 §5, W2b) — irmão de `vector` pelo teto
//! de 700 LOC, e o corte é por responsabilidade: aqui mora a lista de FORMAS que a largura pode
//! ter, e não os controles do comando Expand que a consome.
//!
//! ⚠️ Os quatro sliders (`VECTOR_EXPAND_W_*`) ficam no irmão, com o resto da seção Expand: eles
//! REFINAM o perfil, e a seção é uma coisa só. O que muda aqui é como se ESCOLHE um.

use ph2d_a11y::NodeId;

use super::fnv_node_id_runtime;

/// Teto de perfis nomeados que o painel oferece (`ph2d_stroke_width::PRESETS`). O `populate`
/// registra os `MAX` botões de uma vez e o `paint` desenha só os que a tabela publica — assim
/// acrescentar um perfil é **uma linha na tabela** e nenhum sítio de UI (o idioma dos presets de
/// gaiola do envelope, e o da rack de áudio que se popula de `KINDS`).
pub const MAX_WIDTH_PRESETS: usize = 8;

/// [`NodeId`] do botão do perfil `index`. Runtime `format!` (a lista é dado), gêmeo FNV no mesmo
/// espaço de ids — espelho exato da fábrica dos presets de gaiola.
#[must_use]
pub fn vector_width_preset_id(index: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.width.preset.{index}"))
}
