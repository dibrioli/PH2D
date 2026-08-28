//! **OS TOKENS da seleção** — que propriedade dela segue um token, e qual (plano UI/UX §4/W4).
//!
//! Irmão do [`crate::state_frame`], com a mesma divisão de donos: a verdade mora no ECS
//! (`ph2d_ecs::VecBindings`) e isto é a projeção que a shell publica por frame. O painel não
//! alcança o mundo — se alcançasse, haveria duas respostas para *"esta forma segue um token?"*.
//!
//! ⚠️⚠️ **O `stroke_exists` SAIU daqui em 2026-08-27 (plano 34).** Ele respondia a *"esta forma tem
//! traço?"*, que passou a ter uma porta própria — [`crate::state::stroke_present`] — porque a
//! caixa de marcar da secção *Stroke* faz a MESMA pergunta. ⛔ Duas respostas à mesma pergunta
//! divergem no dia em que uma ganha uma condição, e aqui a divergência seria visível de imediato:
//! a caixa a dizer *"tem traço"* e a row de token a não aparecer. A razão de a pergunta existir
//! não mudou: o token de traço colore o traço que existe e nunca inventa largura (ver
//! `VecPath::painted`), então sem ela o painel oferece uma row que não muda um pixel.
//!
//! O `flows` (W4c.4) fica: ele é o gêmeo para os vãos (sem auto layout não há vão a espaçar), e
//! **não** tem um segundo leitor.

use std::cell::RefCell;

/// O que a seleção tem preso, e o que ela tem para prender.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TokenBindings {
    /// Chave do token do preenchimento (`None` = literal).
    pub fill: Option<String>,
    /// Chave do token do traço (`None` = literal).
    pub stroke: Option<String>,
    /// Chave do token da ESPESSURA do traço (W4c.4).
    pub width: Option<String>,
    /// Chave do token do vão PRINCIPAL do auto layout (W4c.4).
    pub gap_main: Option<String>,
    /// Chave do token do vão TRANSVERSAL (W4c.4).
    pub gap_cross: Option<String>,
    /// A seleção é uma moldura que FLUI (`VecLayout`) — idem, para as rows de vão.
    pub flows: bool,
}

impl TokenBindings {
    /// **A chave presa neste slot** — pelo mesmo código que a UI usa para nomear as opções.
    ///
    /// ⚠️ Porta ÚNICA das duas perguntas que TÊM de concordar: que rótulo o chip mostra, e que
    /// linha o popover destaca. Um `if prop == 0 { fill } else { stroke }` a cada sítio de leitura
    /// é o `match` que envelhece — e o sintoma é o chip a dizer um token e o picker a destacar
    /// outro, que este painel já viu uma vez.
    #[must_use]
    pub fn of_slot(&self, prop: u16) -> Option<String> {
        match prop {
            0 => self.fill.clone(),
            1 => self.stroke.clone(),
            2 => self.width.clone(),
            3 => self.gap_main.clone(),
            4 => self.gap_cross.clone(),
            _ => None,
        }
    }
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
