//! O estado do painel — e ele é **quase vazio**, de propósito.
//!
//! # O painel não guarda cópia de token nenhum
//!
//! A lista de linhas é [`ph2d_tokens::ColorToken::ALL`] (tabela de compile-time), o valor de cada
//! uma sai de `ColorToken::resolve(theme)` — a porta única — e *"este está autorado?"* sai de
//! `ph2d_tokens::overrides::color_override`. O modo vigente vem do próprio host
//! (`ctx.host.theme()`), que é a fonte que a tecla `M` cicla.
//!
//! ⚠️ Um `snapshot` publicado pela shell (o padrão do painel de física) seria uma **segunda cópia
//! da tabela de cor** viajando por frame — e a pergunta *"de que cor é este token?"* passaria a ter
//! duas respostas, num painel cuja razão de existir é ser a autoridade sobre ela.
//!
//! # O que sobra é a FILA DE INTENTS, e a razão dela é haver UM escritor
//!
//! O painel **lê** a camada de override e **nunca a escreve**: quem escreve é a shell, que já é
//! obrigada a fazê-lo no read-back do picker (o estado do picker mora no store que ela possui).
//! Deixar o painel escrever também daria dois escritores para a mesma tabela — e o segundo é
//! sempre o que esquece de marcar o projeto como sujo.

use std::cell::{Cell, RefCell};

/// **De que LISTA é uma linha** — a família de cor, ou a numérica (plano UI/UX W4c.1).
///
/// ⚠️ O painel passou a ter DUAS listas, e um índice cru deixou de identificar uma linha. Um par
/// `(família, índice)` torna impossível o modo de falha barato: um clique numa linha de
/// espaçamento a escrever no token de COR de mesmo índice, em silêncio.
///
/// ⚠️ `pub(crate)`: ele descreve o GESTO dentro do painel, e nenhum consumidor de fora o pergunta
/// (os intents nomeiam a família no próprio variant — `Reset` contra `NumReset`). Exportá-lo seria
/// superfície pública à espera de um chamador, e a próxima linha a olhar para ela teria de
/// adivinhar para que serve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TokenFamily {
    /// `ColorToken::ALL` — a swatch e o picker.
    Colour,
    /// `NumToken::ALL` — o chip de px.
    Num,
}

/// O que um clique neste painel PEDE. Os índices são posições em `ColorToken::ALL`.
///
/// ⚠️ Sem `Copy` desde a math (W4c.3): uma fórmula é TEXTO, e interná-la num índice poria uma
/// segunda tabela ao lado da que já existe para poupar clones num caminho que corre uma vez por
/// CLIQUE.
#[derive(Clone, Debug, PartialEq)]
pub enum TokensIntent {
    /// Devolve UM token do modo vigente à fábrica.
    Reset(usize),
    /// Devolve UM token NUMÉRICO do modo vigente à fábrica. Índice em `NumToken::ALL`.
    NumReset(usize),
    /// **O artista digitou/arrastou um número.** Índice em `NumToken::ALL`, valor em px.
    ///
    /// ⚠️ O painel NÃO escreve a camada — ele diz o que o chip passou a mostrar, e a shell decide
    /// se isso muda alguma coisa. É o mesmo motivo que mantém o read-back do picker na shell: um
    /// segundo escritor é sempre o que esquece de marcar o projeto como sujo.
    NumSet { row: usize, px: f32 },
    /// **`from` passa a SEGUIR `to`**, na família numérica.
    NumLink { from: usize, to: usize },
    /// Devolve o MODO VIGENTE inteiro à fábrica.
    ///
    /// ⚠️ Só o vigente: o artista vê um modo de cada vez, e apagar trabalho que ele não está a
    /// olhar é a forma mais barata de perder uma re-vestida.
    ResetAll,
    /// **O artista escreveu uma FÓRMULA** (plano UI/UX W4c.3). Índice em `NumToken::ALL`.
    ///
    /// ⚠️ Texto CRU, e a admissão é da shell: quem sabe se `{spacing.md} * 2` serve é a porta de
    /// escrita (sintaxe, laço, comprimento), e o painel a duplicar essa pergunta seria a segunda
    /// resposta que diverge no dia em que a linguagem ganhar um operador.
    ///
    /// ⚠️ Texto VAZIO significa *retire a fórmula* — é o gesto que o artista faz (apagar o campo),
    /// e não um valor à espera de um caso especial.
    NumFormula { row: usize, src: String },
    /// **`from` passa a SEGUIR `to`** (plano UI/UX W4b).
    ///
    /// ⚠️ O ARMAR não é intent, e a assimetria é deliberada: armar não muda o documento — é o
    /// painel a lembrar-se de onde o gesto começou —, então ele vive no estado do painel. Só o que
    /// ESCREVE atravessa a fila, e a shell continua o único escritor da tabela.
    Link { from: usize, to: usize },
}

thread_local! {
    static INTENTS: RefCell<Vec<TokensIntent>> = const { RefCell::new(Vec::new()) };
    static LAST_CONTENT_H: Cell<f32> = const { Cell::new(0.0) };
    static LAST_VISIBLE_H: Cell<f32> = const { Cell::new(0.0) };
}

/// Enfileira um pedido (painel → shell).
pub(crate) fn push_intent(i: TokensIntent) {
    INTENTS.with(|q| q.borrow_mut().push(i));
}

/// **Drena** os pedidos — a shell chama uma vez por frame.
///
/// ⚠️ Drenar ESVAZIA: um segundo leitor veria a fila vazia e o clique do artista viraria no-op num
/// dos dois caminhos. É a mesma lei do `wash_diag` do Painter.
pub fn drain_intents() -> Vec<TokensIntent> {
    INTENTS.with(|q| std::mem::take(&mut *q.borrow_mut()))
}

/// **Enfileira um intent a partir de um GATE** — a porta que a suíte da shell usa para dirigir a
/// ponte sem um painel na tela.
///
/// ⚠️ `#[cfg(any(test, feature = "test-intents"))]` seria a forma pura, e ela **não serve**: quem
/// precisa desta porta é a suíte de OUTRA crate, e `cfg(test)` não atravessa a fronteira. Fica
/// `pub` e nomeada — um nome que diz *para quem é* vale mais que uma feature que ninguém liga.
pub fn push_intent_for_tests(i: TokensIntent) {
    push_intent(i);
}

pub(crate) fn set_last_content_h(h: f32) {
    LAST_CONTENT_H.with(|c| c.set(h));
}
pub(crate) fn set_last_visible_h(h: f32) {
    LAST_VISIBLE_H.with(|c| c.set(h));
}

/// Altura do conteúdo na última pintura — o que a barra de rolagem precisa.
#[must_use]
pub fn last_content_h() -> f32 {
    LAST_CONTENT_H.with(Cell::get)
}

/// Altura visível na última pintura.
#[must_use]
pub fn last_visible_h() -> f32 {
    LAST_VISIBLE_H.with(Cell::get)
}

/// Estado por-painel exigido pelo trait `Panel`.
///
/// ⚠️ Quase vazio, e o que sobrou é a metade TRANSIENTE do gesto de elo (§ do cabeçalho): o
/// documento continua a não ter cópia nenhuma aqui.
#[derive(Default)]
pub struct TokensPanelState {
    /// A linha que está à espera de um alvo, se alguma — **e de que lista ela é**.
    ///
    /// ⚠️ Estado de GESTO, não de documento — é por isso que ele não é serializado nem atravessa a
    /// fila de intents: fechar o painel no meio de um elo simplesmente desiste dele, que é o que
    /// desistir de um gesto significa.
    ///
    /// ⚠️ **UM slot para as duas famílias, e não um por família.** Dois slots deixariam as duas
    /// listas armadas ao mesmo tempo, com dois botões acesos e nada a dizer qual deles o próximo
    /// clique fecharia. Armar na outra família **abandona** o gesto anterior, que é o que
    /// abandoná-lo parece.
    pub(crate) armed: Option<(TokenFamily, usize)>,
    /// A linha numérica cujo **campo de fórmula** está aberto sem ela ainda ter uma (W4c.3).
    ///
    /// ⚠️ UM slot, pelo mesmo motivo do `armed` acima: dois campos abertos seriam duas fórmulas
    /// meio-escritas com nada a dizer qual delas o próximo Enter grava.
    ///
    /// ⚠️ E ele é **dominado pelo documento** — uma linha já autorada como fórmula mostra o campo
    /// haja o que houver aqui, porque o campo É o editor dela. Este bit só responde por *"o artista
    /// pediu um campo numa linha que ainda não tem fórmula"*.
    pub(crate) fx_open: Option<usize>,
}

impl TokensPanelState {
    /// Que linha de COR está à espera de um alvo, se alguma.
    ///
    /// ⚠️ **Leitura, e é `pub` para o gate de seam** — que vive noutra crate e precisa de ver que
    /// o primeiro clique ARMOU sem que ele tenha enfileirado uma edição. Escrever continua a ser
    /// só do `apply_event`: um segundo escritor daria duas respostas a *"onde este gesto começou?"*.
    #[must_use]
    pub fn armed(&self) -> Option<usize> {
        match self.armed {
            Some((TokenFamily::Colour, row)) => Some(row),
            _ => None,
        }
    }

    /// Que linha numérica tem o campo de fórmula ABERTO por gesto, se alguma.
    ///
    /// ⚠️ `pub` pelo mesmo motivo do `armed`: o gate de seam vive noutra crate e precisa de ver que
    /// o clique no `f(x)` abriu o campo sem ter enfileirado edição nenhuma.
    #[must_use]
    pub fn fx_open(&self) -> Option<usize> {
        self.fx_open
    }

    /// Que linha NUMÉRICA está à espera de um alvo, se alguma.
    #[must_use]
    pub fn armed_num(&self) -> Option<usize> {
        match self.armed {
            Some((TokenFamily::Num, row)) => Some(row),
            _ => None,
        }
    }
}
