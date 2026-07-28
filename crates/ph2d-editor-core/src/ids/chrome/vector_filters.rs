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

use super::super::hash_node_id;
use super::painter::fnv_node_id_runtime;

// ── Filters: a pilha de FX raster por-forma ─────────────────────────────────────
// A pilha é o componente `ph2d_ecs::VecFilter` na entidade da forma; presença = tem filtros,
// ausência = forma nua. Estes ids são a única porta do PRODUTO para ela.
/// Seção **FILTERS** — a forma ganha uma pilha de FX raster (Blur / Glow / Drop Shadow).
pub const VECTOR_SECTION_FILTERS: NodeId = hash_node_id("vector.section.filters");

/// O teto de degraus numa pilha de filtros — o painel regista este número de blocos de linha,
/// sempre, e pinta só os que a pilha de facto tem.
///
/// ⚠️ Espelha o `ph2d_ecs::VecFilter::MAX_OPS`, que o painel não alcança (ele vive de snapshots);
/// há gate a exigir que os dois lados concordem.
pub const MAX_FILTER_ROWS: usize = 6;

/// O teto de TIPOS que o menu "Add" oferece. Espelha o `ph2d_ecs::FxOp::KINDS`.
///
/// ⚠️ O painel não alcança o `ph2d-ecs`; há gate na shell (o único lugar que vê os dois lados) a
/// exigir que os números concordem. Um teto MENOR aqui deixaria os últimos tipos sem botão — sem
/// erro nenhum, porque o `paint` faz `.take(MAX_FILTER_KINDS)`.
pub const MAX_FILTER_KINDS: usize = 9;

/// O teto de MODOS que um tipo pode oferecer (hoje: Proximity | Contour, dos degraus de dentro).
/// Espelha o maior `FxKindSpec::modes` — o painel registra este número por linha, sempre, e pinta
/// só os que a tabela publicada de fato oferece.
pub const MAX_FILTER_MODES: usize = 4;

/// **O MODO `m` da linha `row`** — a LEI do degrau, não a intensidade dele: um Inner Shadow em
/// `Proximity` mede quanto de FORA há por perto (uma parte fina escurece inteira), em `Contour`
/// mede a DISTÂNCIA à borda (uma banda de largura constante ao longo de todo o contorno).
#[must_use]
pub fn filter_mode_id(row: usize, mode: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.mode.{row}.{mode}"))
}

/// **Add \<tipo\>** — põe um degrau do tipo `kind` no TOPO da pilha (o fim da lista).
#[must_use]
pub fn filter_add_id(kind: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.add.{kind}"))
}

/// **O card** da linha `row` — a moldura. Id próprio, e não o do ✕: o card e o botão de apagar
/// são coisas diferentes para a a11y.
#[must_use]
pub fn filter_card_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.card.{row}"))
}

/// **Remove** o degrau da linha `row`. Removida a última linha, o componente inteiro sai da
/// entidade (a forma volta nua).
#[must_use]
pub fn filter_remove_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.remove.{row}"))
}

/// Sobe o degrau da linha `row`. **A ORDEM é a feature**: `Shadow → Blur` e `Blur → Shadow`
/// desenham coisas diferentes, e é isso que uma pilha entrega e um filtro único não.
#[must_use]
pub fn filter_up_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.up.{row}"))
}

/// Desce o degrau da linha `row`.
#[must_use]
pub fn filter_down_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.down.{row}"))
}

/// **O olho** da linha `row` — desarma o degrau sem o apagar; os parâmetros ficam.
#[must_use]
pub fn filter_hide_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.hide.{row}"))
}

/// **Radius** da linha `row` — o `stdDev` do borrão, em unidades de MUNDO (dar zoom aumenta o
/// borrão na tela).
#[must_use]
pub fn filter_radius_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.radius.{row}"))
}

/// O campo numérico gêmeo do [`filter_radius_id`].
#[must_use]
pub fn filter_radius_num_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.radius.{row}.num"))
}

/// **Offset X** da linha `row` (mundo). Só no Drop Shadow.
#[must_use]
pub fn filter_offx_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.offx.{row}"))
}

/// O campo numérico gêmeo do [`filter_offx_id`].
#[must_use]
pub fn filter_offx_num_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.offx.{row}.num"))
}

/// **Offset Y** da linha `row` (mundo). Só no Drop Shadow.
#[must_use]
pub fn filter_offy_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.offy.{row}"))
}

/// O campo numérico gêmeo do [`filter_offy_id`].
#[must_use]
pub fn filter_offy_num_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.offy.{row}.num"))
}

/// **Color** da linha `row` — a cor do halo (swatch, abre o picker OKLCH). O Blur a ignora.
#[must_use]
pub fn filter_color_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.color.{row}"))
}

/// O teto de LEIS DE MISTURA que um degrau oferece. Espelha o `ph2d_ecs::FxOp::BLEND_KINDS`.
///
/// ⚠️ **VINTE, e o `BlendMode` do Rust tem 22** — `Behind` e `Clear` são operações de COBERTURA,
/// não leis de cor, e um degrau de FX aplica a sua lei onde a cobertura já está decidida. O painel
/// não alcança nenhuma das duas crates; há gate na shell (o único lugar que vê os dois lados).
pub const MAX_FILTER_BLENDS: usize = 20;

/// **A LEI DE MISTURA da linha `row`** — o chip que abre a lista. *Como a cor deste degrau se
/// combina com a que já está ali*: um Inner Shadow em `Multiply` escurece em vez de lavar, um Color
/// Overlay em `Color` troca a matiz preservando a luminosidade.
#[must_use]
pub fn filter_blend_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.blend.{row}"))
}

/// A opção `mode` no popover de mistura da linha `row`.
#[must_use]
pub fn filter_blend_option_id(row: usize, mode: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.blend.{row}.{mode}"))
}

/// **Opacity** da linha `row` (0..1).
#[must_use]
pub fn filter_opacity_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.opacity.{row}"))
}

/// O campo numérico gêmeo do [`filter_opacity_id`].
#[must_use]
pub fn filter_opacity_num_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.opacity.{row}.num"))
}
