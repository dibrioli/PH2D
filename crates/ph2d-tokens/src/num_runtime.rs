//! **A FORMA DE RUNTIME** — a escala achatada num vector, resolvida uma vez por quadro
//! (plano UI/UX W4c.2).
//!
//! # A arquitetura que o plano enuncia, e por que ela é a resposta certa AQUI
//!
//! O [plano](../../../docs/Vector%20Module/Estudos/PLANO_UI_UX_padrao_figma.md) §(b) diz, sobre
//! variáveis: *a tabela achatada por modo é a forma de RUNTIME; o grafo de autoria vive no editor*.
//! Este módulo **é** essa tabela. O grafo — a camada de override, os aliases, e um dia a math —
//! continua onde está ([`crate::num_overrides`]); o que muda é que ele passa a ser **projectado**
//! para um vector plano, e é o vector que os widgets lêem.
//!
//! ⚠️ **O que isso compra, medido, é a ausência de churn.** A escala é lida em **~1200 sítios**
//! (`Spacing::Sm.px()` e irmãos), e só **13** deles são `const` items. Enfiar o modo em cada
//! chamada seria mil e duzentas edições para responder, mil e duzentas vezes, uma pergunta que o
//! app responde **UMA** vez por quadro: *qual é o modo vigente?* Com a tabela, cada um desses
//! sítios fica vivo **sem ser tocado**, e os treze `const` quebram na compilação — o compilador
//! enumera-os, em vez de uma lista escrita à mão que envelhece.
//!
//! # ⚠️ Existe UM modo activo, e é um facto medido — não uma suposição
//!
//! `AppGfx.theme` é um campo, entregue ao `HeroScreen` por quadro; a tecla `M` e o menu de tema
//! escrevem nele; o painel e todo widget lêem `ctx.host.theme()`, que devolve o mesmo. **Não há
//! dois hosts com temas diferentes ao mesmo tempo**, então perguntar *"em que modo?"* uma vez por
//! quadro não perde informação nenhuma.
//!
//! # O TETO foi MEDIDO, e a resposta é que não há teto a escrever
//!
//! A W4c.1 deixou a dívida: *o painel de Tokens desenha-se a si mesmo com estes tokens, então um
//! valor absurdo pode empurrar para fora da tela o botão que o desfaria*. Medido
//! (`ph2d-panel-tokens/tests/scale_ceiling.rs`): com `spacing.* = 1024 px` o *Reset This Mode*
//! pousa em `y = 2206` numa viewport de 900 — **e a rolagem alcança-o**, em toda a escala testada
//! até `65536 px`. O escape não é um número, é o corpo rolável que o painel já tem.
//!
//! ⚠️ **Um cap constante seria um palpite:** o penhasco de posição é `y ≈ 158 + 2·px`, função da
//! ALTURA DA JANELA — qualquer literal estaria errado para metade dos monitores. A porta continua
//! a recusar o que não é um comprimento ([`crate::num_overrides::is_a_length`]) e mais nada.
//!
//! # O modo de falha é LENTO-e-CERTO, nunca ERRADO
//!
//! [`publish`] é a única porta que enche a tabela. Se ninguém a chamar, [`live`] nunca é
//! consultada e **todo token vale a fábrica** — exactamente o comportamento de antes desta wave.
//! É por isso que a bandeira é local a este módulo em vez de ser o `ANY` da camada: *"há coisa
//! autorada"* e *"a tabela está cheia"* são perguntas diferentes, e é a segunda que a leitura
//! precisa.

use std::cell::Cell;

use crate::num::NumToken;
use crate::theme::Theme;

/// Quantos tokens numéricos existem — a régua da tabela.
pub const COUNT: usize = NumToken::ALL.len();

/// A escala de FÁBRICA, achatada em `const`.
///
/// ⚠️ Construída pelo laço `while` de uma `const fn` a partir do próprio `NumToken::ALL`: uma
/// segunda lista escrita à mão seria a que envelhece quando um degrau novo entrar na macro.
const FACTORY: [f32; COUNT] = {
    let mut t = [0.0; COUNT];
    let mut i = 0;
    while i < COUNT {
        t[i] = NumToken::ALL[i].factory_px();
        i += 1;
    }
    t
};

thread_local! {
    /// A escala vigente. ⚠️ `[Cell<f32>; N]` e não `Cell<[f32; N]>`: o segundo copiaria o vector
    /// inteiro a cada leitura, e uma leitura acontece por cada medida de cada widget de cada quadro.
    static TABLE: [Cell<f32>; COUNT] = FACTORY.map(Cell::new);
    /// *A tabela está cheia?* — só [`publish`] responde `true`.
    static FILLED: Cell<bool> = const { Cell::new(false) };
    /// O modo da última publicação, para quem precise de o saber sem o ter em mãos.
    static MODE: Cell<Theme> = const { Cell::new(Theme::Forge) };
}

/// **Resolve o grafo de autoria para `theme` e enche a tabela.** Uma vez por quadro.
///
/// ⚠️ Corre **DEPOIS** de a camada ter absorvido as edições deste quadro e **ANTES** do paint —
/// senão o quadro em que o artista digita um número pinta com o valor anterior, e o chip pisca de
/// volta (a mesma ordem, e o mesmo motivo, do read-back do picker de cor).
///
/// ⚠️ Sem nada autorado a tabela **não é enchida**: `FILLED` cai para `false`, [`live`] deixa de
/// ser consultada e a leitura devolve a fábrica bit a bit. É isto que torna a wave gratuita para
/// quem nunca abriu o painel — e é isto que faz um *Reset This Mode* voltar ao início sem que
/// ninguém precise de limpar o vector.
pub fn publish(theme: Theme) {
    MODE.with(|m| m.set(theme));
    if !crate::num_overrides::any_authored() {
        FILLED.with(|f| f.set(false));
        return;
    }
    TABLE.with(|t| {
        for (i, tok) in NumToken::ALL.iter().enumerate() {
            t[i].set(tok.px(theme));
        }
    });
    FILLED.with(|f| f.set(true));
}

/// O valor VIVO de um token, ou `None` se a tabela não foi enchida.
///
/// ⚠️ `pub(crate)`: quem pergunta é o `px()` de cada família, e mais ninguém. Um consumidor de fora
/// que a chamasse estaria a escolher entre duas portas para *"quanto mede isto?"*.
#[inline]
#[must_use]
pub(crate) fn live(token: NumToken) -> Option<f32> {
    if !FILLED.with(Cell::get) {
        return None;
    }
    Some(TABLE.with(|t| t[token.index()].get()))
}

/// O modo da última [`publish`].
#[must_use]
pub fn published_mode() -> Theme {
    MODE.with(Cell::get)
}

/// *A tabela está a valer?* — para gates, e para quem quiser saber se a escala foi re-vestida.
#[must_use]
pub fn is_filled() -> bool {
    FILLED.with(Cell::get)
}

#[cfg(test)]
#[path = "num_runtime_tests.rs"]
mod tests;
