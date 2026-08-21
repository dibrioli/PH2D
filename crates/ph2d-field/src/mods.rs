//! ⭐ **Os modificadores de um nó** — a casca e o afastamento.
//!
//! # Por que estes dois, e por que aqui
//!
//! São a **tese do módulo** dita numa linha de aritmética, como a booleana e o filete:
//!
//! | verbo | a conta | por que ela não pode falhar |
//! |---|---|---|
//! | **casca** | `\|f\| − t` | o valor absoluto de uma distância **é** a distância à mesma superfície, vista dos dois lados. Não há costura a fechar, não há auto-intersecção a resolver, não há espessura mínima |
//! | **afastamento** | `f − d` | deslocar a superfície por uma distância é o que uma distância assinada **é**. Cresce com `d > 0`, encolhe com `d < 0` |
//!
//! ⚠️ Numa malha, **a casca é a operação que falha**: ela pede um offset da superfície, e um offset
//! de malha auto-intersecta em toda concavidade mais apertada do que a espessura. É por isso que
//! todo modelador de malha tem um botão de casca com uma lista de exceções ao lado. Aqui a lista
//! não existe, e é essa a razão de o módulo ser um campo.
//!
//! # ⚠️ O afastamento ARREDONDA a quina convexa, e é de propósito
//!
//! `f − d` com `d > 0` transforma cada aresta convexa num arco de raio exatamente `d`, e deixa a
//! côncava viva — é o mesmo operador que a [`crate::Primitive`] usa para o `round` dela
//! ([`ph2d_field_eval::ops::offset`]). Quem quiser crescer **sem** arredondar quer outra operação,
//! e ela não existe neste módulo: é a receita canônica que o campo entrega, não um defeito.
//!
//! # A PILHA, e de onde ela vem
//!
//! Os modificadores de um nó são uma **lista ordenada**, e não um grafo: encascar-e-afastar não é
//! o mesmo que afastar-e-encascar, e a ordem tem de ser dita. É a mesma forma que os *Live Path
//! Effects* do vetorial escolheram e mediram ([ADR-0132]: *"uma pilha por path, não um grafo de
//! nós"*) — e pela mesma razão: um grafo paga um editor de grafo para exprimir uma sequência.
//!
//! # ⚠️ Os números são LOCAIS, como as dimensões
//!
//! A pilha corre **antes** da pose ([`ph2d_field_eval`] aplica `place` por cima), então uma
//! espessura de `0,02` num nó escalado 2× dá parede de `0,04` no mundo — exatamente como a largura
//! de uma caixa dentro de um grupo escalado. *Uma regra para todo número deste módulo*, em vez de
//! uma exceção que só aparece quando alguém agrupa.
//!
//! [ADR-0132]: ../../../docs/architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md

use crate::{FieldError, Span};
use serde::{Deserialize, Serialize};

/// Um modificador aplicado ao campo de um nó, **depois** do que ele é e **antes** da pose dele.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Unary {
    /// **Casca**: esvazia o sólido e deixa uma parede de espessura `thickness`, centrada na
    /// superfície que lá estava.
    ///
    /// ⚠️ A parede é **centrada**, e é o que `|f| − t` entrega: metade para dentro, metade para
    /// fora. Uma casca *"só para dentro"* é `|f + t| − t`, e é outra decisão — de produto, com o
    /// número na mão de quem a pedir.
    Shell { thickness: f32 },
    /// **Afastamento**: move a superfície por `distance`. Positivo cresce (e arredonda a quina
    /// convexa); negativo encolhe.
    Offset { distance: f32 },
}

impl Unary {
    /// A chave i18n do nome. ⚠️ Uma **chave**, nunca um rótulo pronto (HR-15).
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Unary::Shell { .. } => "field.mod.shell",
            Unary::Offset { .. } => "field.mod.offset",
        }
    }

    /// O número que o painel mostra e edita.
    #[must_use]
    pub fn value(self) -> f32 {
        match self {
            Unary::Shell { thickness } => thickness,
            Unary::Offset { distance } => distance,
        }
    }

    /// **O que este número admite.**
    ///
    /// ⚠️ São faixas **diferentes**, e a diferença é o significado: uma espessura negativa não quer
    /// dizer nada, e um afastamento negativo é metade da razão de ele existir (encolher).
    #[must_use]
    pub fn span(self) -> Span {
        match self {
            // Sem parede: uma casca mais grossa do que a peça deixa de ser oca, o que é uma forma
            // legítima e não um erro. O alcance útil é o da vista, como toda faixa aberta.
            Unary::Shell { .. } => Span::Positive,
            Unary::Offset { .. } => Span::Free,
        }
    }

    /// ⭐ **Escreve o número**, ou recusa — a porta única.
    ///
    /// # Errors
    /// [`FieldError::NonPositive`] para um valor não-finito, e para uma espessura `≤ 0` (uma casca
    /// sem parede não é uma casca — é o sólido de volta, por um caminho que ninguém pediu).
    pub fn set_value(&mut self, node: u32, value: f32) -> Result<(), FieldError> {
        if !value.is_finite() {
            return Err(FieldError::NonPositive { node, what: "mod" });
        }
        match self {
            Unary::Shell { thickness } => {
                if value <= 0.0 {
                    return Err(FieldError::NonPositive {
                        node,
                        what: "thickness",
                    });
                }
                *thickness = value;
            }
            // ⚠️ **Zero é legítimo aqui**: um afastamento de zero é o campo intacto, e é o ponto por
            // onde o número passa ao ir de encolher para crescer. Recusá-lo faria o slider ter um
            // buraco no meio.
            Unary::Offset { distance } => *distance = value,
        }
        Ok(())
    }

    /// **Um modificador novo, no ponto NEUTRO da sua natureza.**
    ///
    /// ⚠️ Neutro quer dizer coisas diferentes nos dois, e é por isso que não há um default só: um
    /// afastamento de zero é literalmente nada a acontecer, e é o sítio certo para começar a
    /// arrastar. Uma casca de zero seria **recusada** pela própria porta acima — então ela nasce
    /// numa fração da peça, e o número vem de fora ([`crate::characteristic_size`]), porque só quem
    /// vê a peça sabe o que é fino nela.
    #[must_use]
    pub fn born(kind: UnaryKind, scale: f32) -> Unary {
        match kind {
            UnaryKind::Shell => Unary::Shell {
                thickness: (scale * SHELL_BIRTH_FRACTION).max(f32::MIN_POSITIVE),
            },
            UnaryKind::Offset => Unary::Offset { distance: 0.0 },
        }
    }

    /// De que **natureza** este modificador é — o que o botão do painel escolhe.
    #[must_use]
    pub fn kind(self) -> UnaryKind {
        match self {
            Unary::Shell { .. } => UnaryKind::Shell,
            Unary::Offset { .. } => UnaryKind::Offset,
        }
    }
}

/// Que fração da menor peça uma casca nova mede.
///
/// ⚠️ **Um décimo, e o recurso é a VISIBILIDADE**: a parede tem de se ver no primeiro quadro (senão
/// o botão parece não ter feito nada) e tem de deixar buraco (senão a peça continua a parecer
/// maciça). Entre `1/20` — invisível a 480 px numa peça que ocupa meio quadro — e `1/4`, que quase
/// não deixa vazio, um décimo é o degrau que cumpre as duas.
const SHELL_BIRTH_FRACTION: f32 = 0.1;

/// A **natureza** de um modificador, sem o número dele — o que um botão nomeia.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryKind {
    Shell,
    Offset,
}

impl UnaryKind {
    /// ⭐ **A fonte da contagem.** O painel deriva os botões daqui, como já faz com `Mode::ALL` — um
    /// modificador novo acrescenta-se aqui e o painel segue sem uma linha de mudança.
    pub const ALL: [UnaryKind; 2] = [UnaryKind::Shell, UnaryKind::Offset];

    /// A chave i18n do botão que o acrescenta.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            UnaryKind::Shell => "panel.model3d.mod.shell",
            UnaryKind::Offset => "panel.model3d.mod.offset",
        }
    }
}
