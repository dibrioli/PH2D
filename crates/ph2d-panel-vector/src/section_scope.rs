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
//!    Fora do modo dela não há gesto nenhum a configurar: [`Scope::Modes`].
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
    /// Fala do GESTO — só nestes modos.
    Modes(&'static [DrawMode]),
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

/// ⭐⭐ **A SIMETRIA — e a razão de ela não ser uma constante.**
///
/// Ela é uma opção de DESENHO (`symmetry_live`: *"funciona apenas para formas que serão desenhadas
/// com a tool ligada"*, Enio 2026-08-01), logo pertence aos modos que autoram geometria.
///
/// ⛔⛔ **MAS o efeito dela SOBREVIVE à troca de ferramenta**, e o interruptor que a desliga é o
/// único que existe: escondê-la por modo com ela LIGADA deixaria o artista a ver o eixo na tela e
/// sem nada para o apagar — *um controlo cujo efeito sobrevive ao modo não pode ter o único
/// interruptor escondido pelo modo*. Ligada, ela aparece em toda ferramenta.
#[must_use]
pub(crate) fn symmetry(snap: &ph2d_tool_vector::VectorStyleSnapshot) -> Scope {
    if snap.symmetry.on {
        Scope::Always
    } else {
        Scope::Modes(AUTHORS_GEOMETRY)
    }
}

impl Scope {
    /// A seção aparece com esta ferramenta na mão e esta seleção?
    #[must_use]
    pub(crate) fn covers(self, mode: DrawMode, selecionadas: usize) -> bool {
        match self {
            Self::Always => true,
            Self::Modes(ms) => ms.contains(&mode),
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
/// os dois botões correm sobre o `vec_blend_picks` — uma lista que o `blend_pick_at` coleta e que
/// **nunca toca no `PenTool`**. Uma regra de *"esconde com a seleção vazia"* aplicada a olho teria
/// escondido justamente os dois controles que ainda funcionam.
/// *Uma etiqueta de escopo é um fato medido por controle, e duas seções vizinhas podem discordar.*
pub(crate) const WHEN_SELECTED: Scope = Scope::WhenSelected;

/// **Os modos em que o gesto DESENHA UMA FORMA do vocabulário paramétrico.**
///
/// ⚠️ A Moldura entra: ela desenha um `RoundRect` (`DrawMode::shape_kind`), e é por isso que os
/// campos de parâmetro fazem sentido nela mesmo sem nada selecionado. ⛔ A grade de tipos NÃO —
/// a Moldura não escolhe do catálogo, e a grade dela é [`CATALOG`].
///
/// ⚠️⚠️ **`#[cfg(test)]`, e a razão é que ele é um ORÁCULO INDEPENDENTE, não uma fonte.** Quem
/// responde no produto é o `DrawMode::shape_kind`, e o censo
/// `the_modes_that_arm_a_shape_are_exactly_the_ones_the_scope_table_names` confronta os dois. Pô-lo
/// no caminho do produto daria **duas** respostas à mesma pergunta, e um oráculo que partilha a lei
/// do que julga é um espelho: os dois concordariam no errado.
#[cfg(test)]
pub(crate) const DRAWS_A_SHAPE: &[DrawMode] = &[DrawMode::Shape, DrawMode::Frame];

/// **A grade de TIPOS de forma** — o report do Enio. Ela ARMA a forma que o próximo arrasto
/// desenha; fora do modo Forma não há arrasto nenhum a armar, e o caminho de volta é o pill
/// *Shape* da fileira TOOL, que nunca sai.
pub(crate) const CATALOG: Scope = Scope::Modes(&[DrawMode::Shape]);

/// **Os knobs do LÁPIS.** Já se escondiam sozinhos (a única das 39 que o fazia); a guarda mudou-se
/// para cá para haver **uma** resposta à pergunta, e não 39 sítios onde ela pode faltar.
pub(crate) const PENCIL: Scope = Scope::Modes(&[DrawMode::Pencil]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_covers_every_mode_and_modes_covers_only_its_own() {
        assert!(Scope::Always.covers(DrawMode::Select, 0));
        assert!(Scope::Always.covers(DrawMode::Cut, 0));
        let so_forma = Scope::Modes(&[DrawMode::Shape]);
        assert!(so_forma.covers(DrawMode::Shape, 0));
        assert!(!so_forma.covers(DrawMode::Select, 0));
    }

    /// ⚠️ As duas perguntas são INDEPENDENTES: um comando sobre a seleção aparece em toda
    /// ferramenta — desde que haja o que comandar.
    #[test]
    fn a_selection_command_ignores_the_tool_and_only_asks_for_a_target() {
        assert!(!Scope::WhenSelected.covers(DrawMode::Select, 0));
        assert!(Scope::WhenSelected.covers(DrawMode::Select, 1));
        assert!(Scope::WhenSelected.covers(DrawMode::Cut, 3));
    }

    /// ⚠️ **A lista vazia esconde a seção em TODA ferramenta** — é um estado alcançável e absurdo,
    /// e o gate que o proíbe é o censo da tabela (`the_panel_shows_what_the_tool_is_for`), que
    /// exige que toda seção seja alcançável por pelo menos um modo.
    #[test]
    fn an_empty_mode_list_reaches_nobody() {
        assert!(!Scope::Modes(&[]).covers(DrawMode::Shape, 9));
    }
}
