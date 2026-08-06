//! **OS PARES QUE PRECISAM DE CONTRASTE** — a lei WCAG do design system, como DADO
//! (plano UI/UX W4b).
//!
//! # Por que uma tabela, e não quatro testes
//!
//! Antes disto a lei vivia em **quatro cópias do mesmo laço** em `color_tests.rs`, cada uma com o
//! par e a barra escritos à mão. Uma lista escrita à mão é uma lista que apodrece: o quinto par
//! entra num dos consumidores e não no outro, e ninguém descobre até alguém olhar.
//!
//! Agora há **uma lista e dois consumidores** — o gate de compilação (que mede a tabela de
//! FÁBRICA) e o readout do painel (que mede o que o artista **autorou**). Um par novo nasce
//! gateado *e* visível, sem que ninguém toque no painel.
//!
//! # ⚠️ O gate de compilação é CEGO ao que o artista escreve, e é essa a razão desta wave
//!
//! Um teste de unidade corre com a camada de override **vazia**, então ele mede sempre a tabela
//! gerada — que o `tokens.json` já garante conforme. No instante em que o artista autora uma cor
//! (ou faz um token **SEGUIR** outro, W4b.1), o valor efetivo muda **em runtime** e o gate não tem
//! como o ver: ele já correu, noutra máquina, sobre outros números.
//!
//! O readout é a **outra metade da mesma lei** — a que corre onde a escolha é feita. Sem ele o
//! design system valida acessibilidade só até o momento em que alguém o usa.
//!
//! # As barras não foram escolhidas
//!
//! **4.5:1** é o mínimo de TEXTO da WCAG 2.2 AA (SC 1.4.3) e **3.0:1** o de componentes de
//! interface não-textuais (SC 1.4.11). São números da especificação, com o critério nomeado ao
//! lado de cada par — um limite legítimo diz de que recurso ele é.

use crate::color::ColorToken;
use crate::theme::Theme;

/// Um par que a WCAG obriga a ter contraste: **o que se vê SOBRE o quê, e quanto no mínimo**.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContrastPair {
    /// O que está por cima (texto, borda, acento).
    pub fg: ColorToken,
    /// O fundo sobre o qual ele é lido.
    pub bg: ColorToken,
    /// A razão mínima da WCAG 2.2.
    pub min_ratio: f64,
    /// O critério de sucesso que exige esta razão — para a mensagem poder ser procurada.
    pub criterion: &'static str,
}

impl ContrastPair {
    /// A razão que este par tem **AGORA**, neste modo.
    ///
    /// ⚠️ Sai de [`ColorToken::resolve`], a porta única — então ela já conta o valor autorado **e
    /// segue a cadeia de aliases**. Ler a tabela gerada aqui daria um número que descreve um app
    /// que o artista já não está a ver.
    #[must_use]
    pub fn ratio(self, theme: Theme) -> f64 {
        self.bg
            .resolve(theme)
            .contrast_ratio(&self.fg.resolve(theme))
    }

    /// *Este par ainda cumpre a WCAG neste modo?*
    #[must_use]
    pub fn passes(self, theme: Theme) -> bool {
        self.ratio(theme) >= self.min_ratio
    }

    /// *Este token participa deste par?* — os DOIS lados, porque os dois o causam.
    ///
    /// ⚠️ Marcar só o de cima seria dizer que o texto está errado quando o artista escureceu o
    /// FUNDO; o par é uma relação, e uma relação não tem culpado.
    #[must_use]
    pub fn involves(self, token: ColorToken) -> bool {
        self.fg == token || self.bg == token
    }
}

/// **A LISTA.** Um par novo entra aqui e nasce gateado *e* visível no painel.
///
/// ⚠️ Ela é deliberadamente CURTA: cada linha é uma promessa que o design system faz e que o
/// `ship.sh` vai cobrar. Pares que ninguém consegue quebrar não pertencem aqui — eles só ensinam
/// a ignorar o readout.
pub const CONTRAST_PAIRS: &[ContrastPair] = &[
    ContrastPair {
        fg: ColorToken::Text1,
        bg: ColorToken::Bg1,
        min_ratio: 4.5,
        criterion: "WCAG 2.2 AA 1.4.3",
    },
    ContrastPair {
        fg: ColorToken::Text2,
        bg: ColorToken::Bg1,
        min_ratio: 4.5,
        criterion: "WCAG 2.2 AA 1.4.3",
    },
    ContrastPair {
        fg: ColorToken::BorderEmph,
        bg: ColorToken::Bg1,
        min_ratio: 3.0,
        criterion: "WCAG 2.2 AA 1.4.11",
    },
    ContrastPair {
        fg: ColorToken::Accent,
        bg: ColorToken::Bg1,
        min_ratio: 3.0,
        criterion: "WCAG 2.2 AA 1.4.11",
    },
];

/// Os pares que **NÃO cumprem** a WCAG neste modo, agora.
///
/// ⚠️ **Deste modo, e só dele** — a mesma decisão de escopo do *Reset This Mode*: um override é do
/// par `(modo, token)`, o artista vê um modo de cada vez, e o painel já nomeia qual. Um readout
/// que somasse os quatro diria *"há um problema"* sobre uma tela onde não há nada a consertar.
#[must_use]
pub fn failing_pairs(theme: Theme) -> Vec<ContrastPair> {
    CONTRAST_PAIRS
        .iter()
        .copied()
        .filter(|p| !p.passes(theme))
        .collect()
}

/// *Este token participa de algum par que está a falhar?* — a pergunta que a LINHA faz.
///
/// ⚠️ Ela e a [`failing_pairs`] são a **mesma** resposta vista de dois lados (o resumo no topo e a
/// marca na linha), e por isso a linha pergunta à lista em vez de re-medir: um resumo que dissesse
/// *dois pares* com uma marca só é a divergência que ninguém percebe até estar a olhar para ela.
#[must_use]
pub fn token_is_in_a_failing_pair(theme: Theme, token: ColorToken) -> bool {
    CONTRAST_PAIRS
        .iter()
        .any(|p| p.involves(token) && !p.passes(theme))
}

#[cfg(test)]
#[path = "contrast_tests.rs"]
mod tests;
