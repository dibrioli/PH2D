//! ⭐⭐⭐ **DE QUE FERRAMENTA É CADA SEÇÃO** — a tabela única do painel Vector.
//!
//! # O defeito que esta tabela existe para matar
//!
//! Report do Enio (2026-08-31, com foto): *"Mesmo com outras ferramentas selecionadas, as Shapes
//! ficam expostas e as propriedades das shapes também. Melhor deixar no painel apenas o que é útil
//! para a ferramenta em uso."*
//!
//! ⚠️ **Medido antes de mexer: das 39 seções do orquestrador, UMA consultava a ferramenta na mão**
//! (a do Lápis). Todo o resto ou pintava sempre, ou se escondia por SELEÇÃO — que é outra pergunta.
//!
//! ⭐⭐ **E a lei já estava escrita e honrada — uma fileira abaixo.** Dentro da seção TOOL, a fileira
//! do Marquee só aparece no modo Node e os botões da linha de corte só no modo Cut, com o
//! doc-comment a dizer *"fora do modo Cut os dois seriam controles de uma ferramenta que não está
//! na mão"* (Enio, 2026-07-31). *A regra nunca atravessou a fronteira da seção* — e o que a fazia
//! não atravessar era ela viver escrita à mão dentro de cada uma, onde 38 podem esquecê-la.
//!
//! # As três coisas de que uma seção pode falar
//!
//! 1. **O GESTO** — o que a ferramenta na mão vai PRODUZIR (a grade de formas, os knobs do lápis).
//!    ⛔ **Desde a D10 (07/10) já não mora no painel**: a fila ② e a coluna ① são a casa dele
//!    (`crate::bars`), e com ele saiu o escopo por modo.
//! 2. **A SELEÇÃO** — o que já está na tela. A seleção SOBREVIVE à troca de ferramenta, então estas
//!    valem em todo modo — e quase todas já se escondem sozinhas quando a seleção não as serve
//!    (`state::current_*` devolve `None`). [`Scope::Always`].
//! 3. **O DOCUMENTO** — guias, encaixe, tokens. Também [`Scope::Always`].
//!
//! ⛔ **Não confunda (1) com (2).** Os parâmetros de forma são os dois: com uma forma viva
//! selecionada eles a EDITAM (o ciclo Live Shape, que vale em qualquer ferramenta), e sem alvo eles
//! são o default do próximo traço (que só faz sentido onde se desenha uma forma). Por isso aquela
//! seção é `Always` aqui e resolve o resto na própria lei de foco (`crate::shape_focus`).

use ph2d_tool_vector::params::DrawMode;

/// Em que ferramentas uma seção do corpo aparece.
///
/// ⚠️ **Não há `Default`, e a ausência é a decisão**: uma seção nova tem de dizer de quem é. O
/// `Always` é uma escolha que se escreve, não um esquecimento que se herda — foi um `Always`
/// implícito, repetido 38 vezes, que produziu o report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scope {
    /// Fala da SELEÇÃO ou do DOCUMENTO — vale em toda ferramenta. A seção continua livre para se
    /// esconder sozinha quando a seleção não a serve; esta tabela não opina sobre isso.
    Always,
    /// ⭐⭐ **Fala de COMANDOS sobre a seleção** — e TODO controle dela morre numa guarda de
    /// seleção. Sem nada selecionado ela é um cabeçalho com botões que só sabem recusar.
    ///
    /// ⚠️ **Isto é um FATO medido por seção, nunca um palpite**: cada id foi seguido do `paint` até
    /// ao consumidor, e a etiqueta só se aplica quando *todos* param num
    /// `let Some(sel) = pen.selected() else { return }` ou equivalente. ⛔ Duas seções que pareciam
    /// iguais **não** entraram — ver [`WHEN_SELECTED`].
    WhenSelected,
}

/// **A seção fala da SELEÇÃO ou do DOCUMENTO** — vale em toda ferramenta.
///
/// ⚠️ **Escrito, e não herdado.** Toda seção do corpo declara o escopo dela no orquestrador; este
/// é o valor que a maioria escolhe, e escolhê-lo é um ato. *Foi um `Always` implícito, repetido 38
/// vezes, que pôs a grade de formas na ferramenta Select.*
pub(crate) const ALWAYS: Scope = Scope::Always;

/// ⛔ **D10 (07/10): o GESTO saiu do painel.** As secções que falavam do gesto (`Tool`, o catálogo
/// `Shape`, `Pencil`, `Symmetry`) mudaram-se para a fila ② e a coluna ① — o lugar cuja pergunta
/// elas respondem (*como o gesto actua*) —, e o escopo por modo foi com elas: o painel só pinta o
/// que a SELEÇÃO é. Esta lista continua aqui porque é a resposta a *quem desenha*, e a fila ②
/// (`bars.rs`) pergunta-a para oferecer a simetria.
///
/// **Os modos que AUTORAM geometria nova** — onde uma opção de desenho (a simetria) tem sobre o
/// que agir. ⛔ O Conector e o Corte também produzem caminhos, e ficam de fora de propósito: a
/// geometria de um conector é DERIVADA de a quem as pontas se prendem, e uma lâmina de corte
/// espelhada não é uma lâmina.
pub(crate) const AUTHORS_GEOMETRY: &[DrawMode] = &[
    DrawMode::Pen,
    DrawMode::Pencil,
    DrawMode::Shape,
    DrawMode::Frame,
];

impl Scope {
    /// A seção aparece com esta seleção? (A ferramenta na mão já não decide — o gesto mora na ②.)
    #[must_use]
    pub(crate) fn covers(self, selecionadas: usize) -> bool {
        match self {
            Self::Always => true,
            Self::WhenSelected => selecionadas > 0,
        }
    }
}

/// ⭐⭐ **As cinco seções que são COMANDOS sobre a seleção** — Boolean · Expand · Envelope ·
/// Arrange · Path. Cada controle delas foi seguido do `paint` até ao consumidor, e todos param
/// numa guarda de seleção (`input_dispatch.rs`, `vec_expand.rs`, `envelope_live.rs`,
/// `node_ops.rs`): com a seleção vazia elas são cabeçalho e botões que só sabem recusar.
///
/// ⛔⛔ **DUAS irmãs ficaram de fora, e a razão é uma SEGUNDA SELEÇÃO INVISÍVEL.** O *Blend* e o
/// *Morph* parecem iguais e não são: o `Pick Shapes` troca o `DrawMode` **sem olhar a seleção**, e
/// os dois botões correm sobre o `VecState::blend_picks` — uma lista que o `blend_pick_at` coleta e que
/// **nunca toca no `PenTool`**. Uma regra de *"esconde com a seleção vazia"* aplicada a olho teria
/// escondido justamente os dois controles que ainda funcionam.
/// *Uma etiqueta de escopo é um fato medido por controle, e duas seções vizinhas podem discordar.*
pub(crate) const WHEN_SELECTED: Scope = Scope::WhenSelected;

/// **Os modos em que o gesto DESENHA UMA FORMA do vocabulário paramétrico.**
///
/// ⚠️ A Moldura entra: ela desenha um `RoundRect` (`DrawMode::shape_kind`), e é por isso que os
/// campos de parâmetro fazem sentido nela mesmo sem nada selecionado. ⛔ O catálogo NÃO — a
/// Moldura não escolhe do catálogo (o flyout da Forma é só do modo Forma, `crate::bars`).
///
/// ⚠️⚠️ **`#[cfg(test)]`, e a razão é que ele é um ORÁCULO INDEPENDENTE, não uma fonte.** Quem
/// responde no produto é o `DrawMode::shape_kind`, e o censo
/// `the_modes_that_arm_a_shape_are_exactly_the_ones_the_scope_table_names` confronta os dois. Pô-lo
/// no caminho do produto daria **duas** respostas à mesma pergunta, e um oráculo que partilha a lei
/// do que julga é um espelho: os dois concordariam no errado.
#[cfg(test)]
pub(crate) const DRAWS_A_SHAPE: &[DrawMode] = &[DrawMode::Shape, DrawMode::Frame];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_covers_with_or_without_a_selection() {
        assert!(Scope::Always.covers(0));
        assert!(Scope::Always.covers(5));
    }

    /// ⚠️ As duas perguntas são INDEPENDENTES: um comando sobre a seleção aparece em toda
    /// ferramenta — desde que haja o que comandar.
    #[test]
    fn a_selection_command_ignores_the_tool_and_only_asks_for_a_target() {
        assert!(!Scope::WhenSelected.covers(0));
        assert!(Scope::WhenSelected.covers(1));
        assert!(Scope::WhenSelected.covers(3));
    }
}
