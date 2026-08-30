//! **O AUTO LAYOUT da seleção** — a projeção que o painel lê (plano UI/UX W2, ADR-0153).
//!
//! Irmão do [`crate::state_frame`], e com a mesma divisão de donos: a verdade mora no ECS
//! (`ph2d_ecs::VecLayout` / `VecLayoutItem`) e isto é o que a shell publica por frame.
//!
//! # Por que o chip aceso viaja como `NodeId`
//!
//! ⚠️ Este painel **não alcança o `ph2d-ecs`** (nem deve — ele desenha, não conhece o documento), e
//! espelhar aqui os três enums do layout criaria um segundo vocabulário para o mesmo facto. O que
//! o painel de facto precisa saber é *qual chip está aceso*, e isso é um `NodeId` — que já vive na
//! `ph2d-editor-core`, que os DOIS lados leem.
//!
//! A tradução `enum ↔ chip` fica então numa **tabela única na shell**
//! (`vec_layout_edit::DIRS`/`ALIGNS`/`JUSTIFIES`), lida nas duas direções: para publicar o aceso e
//! para resolver o clique. Uma segunda tabela — uma para pintar, outra para honrar — divergiria no
//! dia em que uma variante nova entrasse só numa delas, e o sintoma seria um chip que acende e não
//! faz nada.
//!
//! # Três factos, e cada um responde a uma pergunta que os outros não fazem
//!
//! - **a moldura FLUI?** → [`layout_flow`] `None` = não empilha (só a fileira de direção é
//!   pintada; vão, recuo e alinhamento sobre uma moldura que não flui são cinco controlos que não
//!   mudam um pixel);
//! - **o filho selecionado está num fluxo?** → [`layout_item`];
//! - **o recuo é um número ou quatro?** → [`layout_pad_each`], que é **panel-local**: ele decide
//!   quais campos são pintados e mais nada, então uma ida à shell seria uma porta sem trabalho do
//!   outro lado.

use std::cell::Cell;

use ph2d_a11y::NodeId;

/// O fluxo de uma moldura que empilha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutFlow {
    /// O chip de direção ACESO (`VECTOR_LAYOUT_DIR_ROW` / `_COL` / `_WRAP`).
    pub dir: NodeId,
    /// Vão `[principal, transversal]`.
    pub gap: [f64; 2],
    /// Recuo `[topo, direita, base, esquerda]`.
    pub pad: [f64; 4],
    /// O chip de alinhamento transversal ACESO.
    pub align: NodeId,
    /// O chip de distribuição principal ACESO.
    pub justify: NodeId,
    /// O chip de tamanho ACESO por eixo `[w, h]` (`..._SIZE_W_FIXED` / `_HUG`).
    pub size: [NodeId; 2],
    /// Piso por eixo `[w, h]`; `0` = sem piso (ver `VECTOR_LAYOUT_MIN_W`).
    pub min: [f64; 2],
    /// Teto por eixo `[w, h]`; `0` = sem teto.
    pub max: [f64; 2],
    /// **Quantas colunas** a grade tem — pintado só com o chip *Grid* aceso.
    ///
    /// ⚠️ Ele viaja SEMPRE (não é `Option`), e o valor sobrevive a uma troca de direção: é o mesmo
    /// que o vão e o recuo já fazem, e é o que devolve a grade intacta quando o artista vai a
    /// `Row` e volta. Quem decide se ele é PINTADO é a direção, no `paint_layout`.
    pub columns: f64,
}

/// Como o filho selecionado se comporta dentro do fluxo do pai.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayoutItem {
    pub grow: f64,
    pub shrink: f64,
    /// Este filho saiu do fluxo (o *Absolute position*). ⚠️ Quando `true`, `grow`/`shrink` **não
    /// são pintados**: quem não está no fluxo não reparte sobra nenhuma.
    pub absolute: bool,
    /// **O pai DISPÕE os filhos?** ⚠️ `false` ⇒ nenhum destes controlos faz nada — e o painel
    /// **DIZ isso** em vez de os esconder em silêncio (o precedente do Falloff dos Motion Nodes).
    ///
    /// A ausência muda de significado com esta linha: antes, um filho de moldura parada publicava
    /// `None` e a seção **Layout não era pintada de todo** — o artista clicava na forma, não via
    /// nada, e não tinha como saber que faltava ligar o fluxo no PAI. Foi o report do Enio no
    /// smoke da cena `=66`.
    pub in_flow: bool,
    /// **O pai dispõe em GRADE?** ⚠️ `true` ⇒ `grow`/`shrink` **não são pintados**.
    ///
    /// ⛔ Eles vivem no trait de **flex** do `taffy` e a grade consome outro: medido em
    /// 2026-08-30, `flex_grow` aparece **zero** vezes em `taffy/src/compute/grid/` e **13** em
    /// `compute/flexbox.rs`. Numa moldura `Grid` os dois números viajam até ao motor e são
    /// descartados lá — a segunda espécie de knob morto (*o consumidor projecta o valor fora*),
    /// que nenhuma sonda de «quem lê este campo?» vê, porque ele **é** lido.
    ///
    /// ⚠️ A lei já estava escrita duas vezes neste módulo — para o filho `absolute` (*«quem não
    /// está no fluxo não reparte sobra nenhuma»*) e para o `columns` do `VecLayout` (*«o painel
    /// não o pinta onde ele não move um pixel»*). Esta é a terceira aplicação da mesma.
    pub parent_is_grid: bool,
}

thread_local! {
    static FLOW: Cell<Option<LayoutFlow>> = const { Cell::new(None) };
    static ITEM: Cell<Option<LayoutItem>> = const { Cell::new(None) };
    static PAD_EACH: Cell<bool> = const { Cell::new(false) };
}

/// Publica o fluxo da moldura selecionada (shell → painel). `None` = ela não empilha.
pub fn set_layout_flow(flow: Option<LayoutFlow>) {
    FLOW.with(|c| c.set(flow));
}

/// O fluxo da moldura selecionada — `None` quando ela não empilha.
#[must_use]
pub(crate) fn layout_flow() -> Option<LayoutFlow> {
    FLOW.with(Cell::get)
}

/// Publica o comportamento do filho selecionado. `None` = a seleção não é um filho de fluxo.
pub fn set_layout_item(item: Option<LayoutItem>) {
    ITEM.with(|c| c.set(item));
}

/// O comportamento do filho selecionado — `None` = não oferecer as duas linhas.
#[must_use]
pub(crate) fn layout_item() -> Option<LayoutItem> {
    ITEM.with(Cell::get)
}

/// O modo do recuo: `false` = um campo para os quatro lados, `true` = quatro campos.
#[must_use]
pub(crate) fn layout_pad_each() -> bool {
    PAD_EACH.with(Cell::get)
}

/// Escolhe o modo do recuo — **panel-local**, escrito pelo próprio `event.rs`.
pub(crate) fn set_layout_pad_each(each: bool) {
    PAD_EACH.with(|c| c.set(each));
}
