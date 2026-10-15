//! ⭐ **A pré-edição de IME** — a composição em curso (o `´` antes do `a`) que o campo focado
//! MOSTRA, sublinhada no caret, e que **nunca** entra no valor antes do `Commit`.
//!
//! O dono do estado é o [`WidgetStore`] ([`WidgetStore::composition`]); este módulo é a ponte de
//! PINTURA (o padrão do `published`: *o hero publica, a folha lê*): o hero publica a composição no
//! início do quadro, o pintor do campo lê-a e devolve o rect do caret, e a shell pergunta-o depois
//! da pintura para pôr a janela de candidatos do sistema no caret.
//!
//! # O que o winit 0.30.13 entrega (medido 15/10 sob `Xvfb` + `xdotool`, layout `us intl`)
//!
//! | caminho | `´` | `a` depois |
//! |---|---|---|
//! | X11 com XIM (o de omissão) | nada (filtrado) | `Preedit("")` + `Commit("á")`, sem `KeyEvent` |
//! | xkb sem IME (= Wayland sem IME, a sessão do dono) | `Key::Dead('´')`, `text: None` | `KeyEvent.text = "á"` |
//! | Wayland/macOS com IME | `Preedit` | `Preedit("")` + `Commit`, o `KeyEvent` suprimido |
//!
//! ⇒ cada carácter chega por UM caminho, nunca pelos dois. Sem IME não há `Preedit`, e por isso a
//! tecla morta é composta aqui ([`ComposeKey`]). `´`+`q` cancela (o `q` chega sem texto); `´´` = `´`.

use crate::interaction::WidgetStore;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use std::cell::RefCell;

/// Uma composição em curso no campo `id`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Composition {
    pub id: NodeId,
    pub text: String,
    /// Cursor dentro de `text` em bytes `(início, fim)`, como o IME o dá; `None` = no fim.
    pub cursor: Option<(usize, usize)>,
    /// Nasceu de uma TECLA MORTA sem IME (o xkb compõe e o winit só entrega `Key::Dead`): a
    /// próxima tecla que não é modificador desfá-la. Uma composição do IME é do IME.
    pub dead_key: bool,
}

/// O que uma tecla premida é para a composição — a shell traduz a tecla lógica do winit nisto.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComposeKey {
    /// Tecla morta com o seu acento (`´` `` ` `` `^` `~` `¨`).
    Dead(char),
    /// Shift, Ctrl, AltGr… — não desfazem a tecla morta (`´` + Shift + `a` = `Á`).
    Modifier,
    /// Desfaz a tecla morta SEM sair do campo.
    Escape,
    Other,
}

#[derive(Default)]
struct Frame {
    focus: Option<NodeId>,
    composition: Option<Composition>,
    caret: Option<Rect>,
    sent: Option<Rect>,
}

thread_local! {
    static FRAME: RefCell<Frame> = RefCell::new(Frame::default());
}

/// Início do quadro: o hero publica o foco e a composição antes de qualquer pintor.
pub fn begin_frame(store: &WidgetStore) {
    FRAME.with(|f| {
        let mut f = f.borrow_mut();
        f.focus = store.focus_id();
        f.composition = store.composition().cloned();
        f.caret = None;
    });
}

/// A composição a mostrar no campo `id` — só o campo focado a tem.
#[must_use]
pub fn composition_for(id: NodeId) -> Option<Composition> {
    FRAME.with(|f| {
        let f = f.borrow();
        f.composition
            .clone()
            .filter(|c| c.id == id && f.focus == Some(id))
    })
}

/// O pintor do campo `id` diz onde pôs o caret (lógico); só o do campo focado conta.
pub fn publish_caret(id: NodeId, caret: Rect) {
    FRAME.with(|f| {
        let mut f = f.borrow_mut();
        if f.focus == Some(id) {
            f.caret = Some(caret);
        }
    });
}

/// O rect do caret do campo focado neste quadro (lógico).
#[must_use]
pub fn caret_area() -> Option<Rect> {
    FRAME.with(|f| f.borrow().caret)
}

/// O [`caret_area`], **só quando mudou** desde a última pergunta — a shell chama o
/// `set_ime_cursor_area` com ele.
#[must_use]
pub fn caret_area_changed() -> Option<Rect> {
    FRAME.with(|f| {
        let mut f = f.borrow_mut();
        let c = f.caret?;
        (f.sent != Some(c)).then(|| {
            f.sent = Some(c);
            c
        })
    })
}
