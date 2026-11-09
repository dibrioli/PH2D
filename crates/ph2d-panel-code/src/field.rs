//! O campo de uma linha (a busca, a lista, a célula da Bancada): uma tecla mexe no `Buffer` e diz se
//! foi Enter, Esc ou Tab. Saiu do `ph2d-app-code` (`edit.rs`) para a célula da grelha o usar.

use ph2d_code_buffer::{Buffer, Motion};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;

/// Uma tecla para um campo de uma linha.
#[derive(Clone, Copy)]
pub struct FieldKey<'a> {
    pub key: Key,
    pub mods: Modifiers,
    pub text: Option<&'a str>,
    pub paste: Option<&'a str>,
}

#[derive(Default)]
pub struct FieldOut {
    pub changed: bool,
    pub enter: bool,
    pub escape: bool,
    pub tab: bool,
    pub copy: Option<String>,
}

/// Os caracteres que se escrevem (sem controlos: Enter, Tab e Backspace têm tecla própria).
pub fn printable(s: &str) -> impl Iterator<Item = char> + '_ {
    s.chars().filter(|c| !c.is_control())
}

/// Uma tecla num campo de uma linha.
pub fn field_key(buf: &mut Buffer, k: FieldKey<'_>) -> FieldOut {
    let mut out = FieldOut::default();
    let cmd = k.mods.ctrl || k.mods.meta;
    let rev = buf.revision();
    match k.key {
        Key::Enter => out.enter = true,
        Key::Escape => out.escape = true,
        Key::Tab => out.tab = true,
        Key::Char('a') if cmd => buf.select_all(),
        Key::Char('c') if cmd => {
            out.copy = Some(buf.text().slice(buf.selection().primary().span()))
        }
        Key::Char('x') if cmd => {
            out.copy = Some(buf.text().slice(buf.selection().primary().span()));
            buf.delete_with(|_, r| r.head..r.head);
        }
        Key::Char('v') if cmd => {
            if let Some(p) = k.paste {
                let one: String = p
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .chars()
                    .filter(|c| !c.is_control())
                    .collect();
                buf.insert(&one);
            }
        }
        Key::Char('z') if cmd && k.mods.shift => {
            buf.redo();
        }
        Key::Char('z') if cmd => {
            buf.undo();
        }
        Key::Char('y') if cmd => {
            buf.redo();
        }
        Key::Backspace => {
            buf.delete_backward();
        }
        Key::Delete => {
            buf.delete_forward();
        }
        Key::Left => buf.move_cursors(
            if cmd { Motion::WordLeft } else { Motion::Left },
            k.mods.shift,
            1,
        ),
        Key::Right => buf.move_cursors(
            if cmd {
                Motion::WordRight
            } else {
                Motion::Right
            },
            k.mods.shift,
            1,
        ),
        Key::Home | Key::Up => buf.move_cursors(Motion::DocStart, k.mods.shift, 1),
        Key::End | Key::Down => buf.move_cursors(Motion::DocEnd, k.mods.shift, 1),
        _ if !cmd => {
            if let Some(t) = k.text {
                let s: String = printable(t).collect();
                if !s.is_empty() {
                    buf.insert(&s);
                }
            }
        }
        _ => {}
    }
    out.changed = buf.revision() != rev;
    out
}
