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
pub const MAX_FILTER_KINDS: usize = 15;

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

/// **A SEGUNDA cor** da linha `row` — a ponta CLARA da rampa do Duotone (swatch, abre o MESMO
/// picker OKLCH). Só os tipos com `FxKindSpec::color_b_label` a oferecem.
#[must_use]
pub fn filter_color_b_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.colorb.{row}"))
}

/// O teto de STOPS de uma rampa. **Espelha o `ph2d_ecs::FxOp::MAX_GRADIENT_STOPS`** (há gate na
/// shell, o único lugar que vê as duas crates).
///
/// ⚠️ **O recurso de que este teto é: o TRILHO, não a memória.** Os punhos têm caixa de agarre, e
/// stops mais juntos que ela deixam de ser alcançáveis pelo ponteiro — o gate mede a largura REAL
/// do card. O uniform não aperta (medido no stride: 512 B custa 2,345 ms contra 2,332 do de 256).
pub const MAX_FILTER_STOPS: usize = 8;

/// **O trilho da rampa** da linha `row` — o PAI dos arrastos de stop.
///
/// ⚠️ Ele não é um widget clicável: é o alvo que o `InteractiveState::CurvePoint` de cada punho
/// carrega, e é por ele que o dispatch de 2D sabe a que rampa o gesto pertence. O primitivo é o
/// MESMO que o editor de falloff do Painter e a curva do motion-params já usam.
#[must_use]
pub fn filter_ramp_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.ramp.{row}"))
}

/// O punho do stop `stop` da rampa da linha `row` — arrastável na horizontal (a POSIÇÃO), e
/// clicável para selecionar (a cor do selecionado é o que a swatch edita).
#[must_use]
pub fn filter_stop_id(row: usize, stop: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.stop.{row}.{stop}"))
}

/// **A COR do stop SELECIONADO** da rampa da linha `row` (swatch, abre o MESMO picker OKLCH das
/// duas cores do Duotone).
///
/// ⚠️ **Uma swatch, e o alvo dela é a SELEÇÃO** — não uma swatch por stop. Oito swatches empilhadas
/// diriam ao artista que ele edita oito cores de uma vez, quando o gesto real é *escolha o punho,
/// depois a cor*; e o picker é modal, então ele só pode ter um alvo de qualquer forma.
#[must_use]
pub fn filter_stop_color_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.stop.color.{row}"))
}

/// **+** — acrescenta um stop na rampa da linha `row`.
#[must_use]
pub fn filter_stop_add_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.stop.add.{row}"))
}

/// **−** — remove o stop SELECIONADO da rampa da linha `row`. O piso é DOIS (uma rampa com menos
/// de duas pontas não é uma rampa, e o degenerado de zero stops é outra lei — ver o gate
/// `no_stops_is_the_painters_empty_ramp_which_is_not_the_two_stop_default`).
#[must_use]
pub fn filter_stop_remove_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.stop.remove.{row}"))
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

/// **Size** da linha `row` — o TAMANHO das ondulações do ruído, em unidades de MUNDO. É o
/// `baseFrequency` do SVG pelo avesso: ali é frequência, aqui é comprimento, porque o artista
/// pensa em *quão grandes são os caroços* e não em quantos cabem por unidade.
#[must_use]
pub fn filter_scale_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.scale.{row}"))
}

/// O campo numérico gêmeo do [`filter_scale_id`].
#[must_use]
pub fn filter_scale_num_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.scale.{row}.num"))
}

/// **Detail** da linha `row` — quantas OITAVAS o ruído soma (o `numOctaves`).
#[must_use]
pub fn filter_detail_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.detail.{row}"))
}

/// O campo numérico gêmeo do [`filter_detail_id`].
#[must_use]
pub fn filter_detail_num_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.detail.{row}.num"))
}

/// **Seed** da linha `row` — qual das infinitas realizações do ruído. Não muda a estatística do
/// campo, só qual desenho ele é.
#[must_use]
pub fn filter_seed_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.seed.{row}"))
}

/// O campo numérico gêmeo do [`filter_seed_id`].
#[must_use]
pub fn filter_seed_num_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.seed.{row}.num"))
}

/// **Amount** da linha `row` — quanto a silhueta ENGORDA, **com sinal** (o Grow / Shrink).
///
/// ⚠️ Id próprio, e não o do raio: a régua deste slider é BIPOLAR (`0,5` no meio = zero), e o mapa
/// track→valor é registado por LINHA, não por tipo. Uma linha muda de tipo em runtime, então
/// emprestar a régua do raio faria o Blur ler metade da faixa dele como negativa.
#[must_use]
pub fn filter_grow_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.grow.{row}"))
}

/// O campo numérico gêmeo do [`filter_grow_id`].
#[must_use]
pub fn filter_grow_num_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.grow.{row}.num"))
}

/// **Hue** da linha `row` — a matiz do Color Adjust, régua BIPOLAR (`0,5` no meio = sem rotação).
///
/// ⚠️ Os três do ajuste têm ids próprios pelo mesmo motivo do [`filter_grow_id`]: o mapa
/// track→valor de um slider é registado por LINHA, e emprestar a régua de outro knob faz a linha
/// ler metade da faixa dele ao contrário quando ela troca de tipo em runtime.
#[must_use]
pub fn filter_hue_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.hue.{row}"))
}

/// O campo numérico gêmeo do [`filter_hue_id`].
#[must_use]
pub fn filter_hue_num_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.hue.{row}.num"))
}

/// **Saturation** da linha `row` — bipolar: `-1` drena até o cinza, `+1` dobra o croma.
#[must_use]
pub fn filter_sat_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.sat.{row}"))
}

/// O campo numérico gêmeo do [`filter_sat_id`].
#[must_use]
pub fn filter_sat_num_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.sat.{row}.num"))
}

/// **Brightness** da linha `row` — bipolar: `-1` é preto exacto, `+1` é branco exacto.
#[must_use]
pub fn filter_bright_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.bright.{row}"))
}

/// O campo numérico gêmeo do [`filter_bright_id`].
#[must_use]
pub fn filter_bright_num_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("vector.filter.bright.{row}.num"))
}
