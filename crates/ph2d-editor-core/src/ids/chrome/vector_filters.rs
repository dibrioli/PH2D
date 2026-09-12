//! **Os ids da seção Filters (a PILHA de FX raster)** — módulo irmão de [`super`] pelo teto de
//! 700 LOC.
//!
//! O corte é por RESPONSABILIDADE, como o do `vector_contour` / `vector_textpath`: estes são os
//! controles do [`ph2d_ecs::VecFilter`] — o FX RASTER por-forma (Blur / Glow / Drop Shadow, plano
//! 24). É deliberadamente distinto da seção **Effects** (`VECTOR_SECTION_EFFECTS`, ADR-0132), que
//! é a pilha de deformadores VETORIAIS (`VecPath -> VecPath`); um filtro produz PIXELS, não
//! geometria, e colapsar os dois nomes esconderia a diferença que decide a arquitetura inteira.
//!
//! # Por-LINHA, como a pilha de geometria
//!
//! A W1 era **um** filtro por forma e os ids eram `const`. A W2 é uma PILHA ordenada (o modelo
//! AE/Photoshop/Figma), então os ids passam a ser derivados por linha — exatamente o bloco
//! `vector_fx_*` do ADR-0132, cujo `populate` regista o TETO de linhas e cujo `paint` desenha só
//! as que existem. Um id derivado em laço é invisível ao `architecture_panel_wiring_parity`, e é
//! por isso que a costura desta seção depende do seam que CLICA cada controle.
//!
//! ⚠️ **Bloco APPEND-ONLY**: um id é o hash de uma STRING, então reordenar não quebra nada — mas
//! renomear uma string quebra tudo o que a referencia por nome, e é assim que um widget fica órfão
//! em silêncio.

use ph2d_a11y::NodeId;

use ph2d_tool_registry::hash_node_id_runtime;

/// O teto de degraus numa pilha de filtros — o painel regista este número de blocos de linha,
/// sempre, e pinta só os que a pilha de facto tem.
///
/// ⚠️ Espelha o `ph2d_ecs::VecFilter::MAX_OPS`, que o painel não alcança (ele vive de snapshots);
/// há gate a exigir que os dois lados concordem.
pub const MAX_FILTER_ROWS: usize = 6;

/// **O trilho da rampa** da linha `row` — o PAI dos arrastos de stop.
///
/// ⚠️ Ele não é um widget clicável: é o alvo que o `InteractiveState::CurvePoint` de cada punho
/// carrega, e é por ele que o dispatch de 2D sabe a que rampa o gesto pertence. O primitivo é o
/// MESMO que o editor de falloff do Painter e a curva do motion-params já usam.
#[must_use]
pub fn filter_ramp_id(row: usize) -> NodeId {
    hash_node_id_runtime(&format!("vector.filter.ramp.{row}"))
}
