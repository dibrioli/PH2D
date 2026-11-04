//! ⭐ **O censo do layout antigo dentro de um cartão** (`docs/CartoesUI/05` §3): as portas que só
//! o painel de antes devia usar anotam-se aqui quando o corpo de um cartão as chama. Um portão corre
//! a pintura com [`grava`] e lê o que apareceu. Fora de [`grava`] custa só ler uma célula.

use std::cell::RefCell;

/// A peça de layout antigo que um corpo de cartão pintou.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Velho {
    /// Um nome na coluna à esquerda do controlo.
    NomeAEsquerda(String),
    /// A barra deslizante com chip.
    Barra(String),
    /// A caixa de número com setinhas.
    NumeroComSetas(String),
    /// A amostra de cor sem o código.
    AmostraSemCodigo(String),
}

thread_local! {
    static GRAVACAO: RefCell<Option<Vec<Velho>>> = const { RefCell::new(None) };
}

/// Corre `f` e devolve o que os corpos de cartão pintaram da forma antiga.
pub fn grava<R>(f: impl FnOnce() -> R) -> (R, Vec<Velho>) {
    GRAVACAO.with(|g| *g.borrow_mut() = Some(Vec::new()));
    let r = f();
    let visto = GRAVACAO.with(|g| g.borrow_mut().take()).unwrap_or_default();
    (r, visto)
}

/// A caixa de número com setinhas de `id` (uma linha na porta, no tecto de LOC dela).
pub(crate) fn nota_numero(id: ph2d_a11y::NodeId) {
    nota(|| Velho::NumeroComSetas(format!("{id:?}")));
}

/// Anota `v` se estamos dentro do corpo de um cartão e há gravação.
pub(crate) fn nota(v: impl FnOnce() -> Velho) {
    if crate::widget::card_ink::card_body().is_none() {
        return;
    }
    GRAVACAO.with(|g| {
        if let Some(lista) = g.borrow_mut().as_mut() {
            lista.push(v());
        }
    });
}
