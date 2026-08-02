//! **OS TOKENS da seleção** — que propriedade dela segue um token, e qual (plano UI/UX §4/W4).
//!
//! Irmão do [`crate::state_frame`], com a mesma divisão de donos: a verdade mora no ECS
//! (`ph2d_ecs::VecBindings`) e isto é a projeção que a shell publica por frame. O painel não
//! alcança o mundo — se alcançasse, haveria duas respostas para *"esta forma segue um token?"*.
//!
//! ⚠️ **O `stroke_exists` viaja junto, e não é redundante.** O token de traço colore o traço que
//! existe e nunca inventa largura (ver `VecPath::painted`); sem este fato o painel ofereceria uma
//! row que o artista escolhe e que não muda um pixel — que é a definição de controle morto.

use std::cell::RefCell;

/// O que a seleção tem preso, e se ela tem traço para prender.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TokenBindings {
    /// Chave do token do preenchimento (`None` = literal).
    pub fill: Option<String>,
    /// Chave do token do traço (`None` = literal).
    pub stroke: Option<String>,
    /// A seleção tem traço — é isto que decide se a row do traço é OFERECIDA.
    pub stroke_exists: bool,
}

thread_local! {
    static BINDINGS: RefCell<Option<TokenBindings>> = const { RefCell::new(None) };
}

/// Publica os bindings da seleção deste frame (shell → painel). `None` = não há UMA forma
/// selecionada, e as rows de token não são pintadas.
pub fn set_token_bindings(b: Option<TokenBindings>) {
    BINDINGS.with(|c| *c.borrow_mut() = b);
}

/// Os bindings da seleção — `None` quando não há forma única selecionada.
#[must_use]
pub(crate) fn token_bindings() -> Option<TokenBindings> {
    BINDINGS.with(|c| c.borrow().clone())
}

thread_local! {
    static PENDING_DD: std::cell::Cell<Option<(u16, ph2d_editor_core::zones::Rect)>> =
        const { std::cell::Cell::new(None) };
}

/// O popover de token que este frame tem de pintar no passe DIFERIDO (`prop`, retângulo do chip).
///
/// ⚠️ Mesmo canal que o popover de mistura dos filtros usa: a row sabe que está aberta enquanto
/// pinta a seção, mas o card tem de sair POR CIMA de todas elas — então ela deixa o recado aqui e
/// o passe diferido o consome.
pub(crate) fn set_pending_token_dd(v: Option<(u16, ph2d_editor_core::zones::Rect)>) {
    PENDING_DD.with(|c| c.set(v));
}

/// Consome o recado — e o LIMPA, para um frame sem chip aberto não repintar o card do anterior.
pub(crate) fn take_pending_token_dd() -> Option<(u16, ph2d_editor_core::zones::Rect)> {
    PENDING_DD.with(|c| c.take())
}
