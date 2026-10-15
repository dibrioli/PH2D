//! ⭐ **A entrada de um documento de FAMÍLIA** ([`crate::doc_surface`]) — chamada no TOPO das portas
//! por onde o quadro recebe rato, roda, teclado e composição: com uma aba de família activa, ela vem
//! primeiro e a cena nunca vê o evento que ela tomou. `None` = não há aba de família activa (segue a
//! porta do quadro como antes).

use super::board_keys::{BoardKey, KeyOutcome as BoardOutcome};
use super::board_view::Input;
use super::HeroScreen;
use crate::doc_surface::{Compose, Key, PointerInput};
use ph2d_host::{Modifiers, PointerButton, PointerKind};
use ph2d_text::TextSystem;

fn key_of(k: BoardKey) -> Key {
    match k {
        BoardKey::Char(c) => Key::Char(c),
        BoardKey::Enter => Key::Enter,
        BoardKey::Escape => Key::Escape,
        BoardKey::Backspace => Key::Backspace,
        BoardKey::Delete => Key::Delete,
        BoardKey::Left => Key::Left,
        BoardKey::Right => Key::Right,
        BoardKey::Up => Key::Up,
        BoardKey::Down => Key::Down,
        BoardKey::Home => Key::Home,
        BoardKey::End => Key::End,
        BoardKey::Space => Key::Space,
        BoardKey::Tab => Key::Tab,
        BoardKey::PageUp => Key::PageUp,
        BoardKey::PageDown => Key::PageDown,
        BoardKey::Other => Key::Other,
    }
}

pub(super) fn wheel(hero: &mut HeroScreen, x: f32, y: f32, dy: f32) -> Option<bool> {
    let (f, id) = hero.documents.active_surface_mut()?;
    Some(f.wheel(id, x, y, dy))
}

pub(super) fn pointer(
    hero: &mut HeroScreen,
    input: Input<'_>,
    kind: PointerKind,
    button: PointerButton,
    x: f32,
    y: f32,
) -> Option<bool> {
    let (f, id) = hero.documents.active_surface_mut()?;
    let pi = PointerInput { text: input.text, mods: input.mods, now_ns: input.now_ns };
    let taken = f.pointer(id, pi, kind, button, x, y);
    // Carregar no documento tira o foco a um campo da interface: senão o texto iria para lá.
    if taken && matches!(kind, PointerKind::Down) && hero.store.focus_id().is_some() {
        hero.store.set_focus(None);
    }
    Some(taken)
}

pub(super) fn pointer_move(hero: &mut HeroScreen, input: Input<'_>, x: f32, y: f32) -> Option<bool> {
    let (f, id) = hero.documents.active_surface_mut()?;
    let pi = PointerInput { text: input.text, mods: input.mods, now_ns: input.now_ns };
    Some(f.pointer_move(id, pi, x, y))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn key(
    hero: &mut HeroScreen,
    ts: &mut TextSystem,
    k: BoardKey,
    mods: Modifiers,
    pressed: bool,
    text: Option<&str>,
    paste: Option<String>,
) -> Option<BoardOutcome> {
    let (f, id) = hero.documents.active_surface_mut()?;
    let out = f.key(id, ts, key_of(k), mods, pressed, text, paste);
    Some(BoardOutcome { consumed: out.consumed, copy: out.copy })
}

/// Composição (IME / tecla morta / carácter confirmado) — só sem campo da interface focado.
pub(super) fn compose(hero: &mut HeroScreen, c: Compose<'_>) -> Option<bool> {
    if hero.store.focus_id().is_some() {
        return None;
    }
    let (f, id) = hero.documents.active_surface_mut()?;
    Some(f.compose(id, c))
}

/// Desfazer/refazer do menu Edit com a aba de família activa.
pub fn undo(hero: &mut HeroScreen, redo: bool) -> Option<bool> {
    let (f, id) = hero.documents.active_surface_mut()?;
    Some(f.undo(id, redo))
}
