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

/// O que um clique neste painel PEDE. Os índices são posições em `ColorToken::ALL`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokensIntent {
    /// Devolve UM token do modo vigente à fábrica.
    Reset(usize),
    /// Devolve o MODO VIGENTE inteiro à fábrica.
    ///
    /// ⚠️ Só o vigente: o artista vê um modo de cada vez, e apagar trabalho que ele não está a
    /// olhar é a forma mais barata de perder uma re-vestida.
    ResetAll,
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

/// Estado por-painel exigido pelo trait `Panel`. Vazio: ver o § do cabeçalho.
#[derive(Default)]
pub struct TokensPanelState;
