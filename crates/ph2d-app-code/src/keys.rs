//! ⭐ **E5 — o teclado da aba**: com a aba de código activa, TODA tecla é do editor (nenhum atalho da
//! cena a vê). Escrever passa pelo `AutoEdit`; cada tecla deixa o cursor à vista.

use crate::doc::{CodeDoc, TAB};
use crate::docs::SaveKind;
use crate::edit::FieldKey;
use ph2d_code_buffer::Motion;
use ph2d_editor_core::doc_surface::{Key, KeyOutcome};
use ph2d_host::Modifiers;

/// As letras dos acordes Ctrl/Cmd que o editor usa (no texto e nos campos da busca).
const CHORDS: [char; 11] = ['a', 'c', 'd', 'f', 'g', 'h', 's', 'v', 'x', 'y', 'z'];

/// O acorde é do editor? Os outros (Ctrl+K a paleta, Ctrl+O abrir…) NÃO são tomados: a shell segue
/// para o atalho dela.
pub(crate) fn foreign_chord(key: Key, mods: Modifiers) -> bool {
    (mods.ctrl || mods.meta)
        && match key {
            Key::Char(c) => !CHORDS.contains(&c),
            Key::Other => true,
            _ => false,
        }
}

/// Ctrl+Espaço: a lista de completação na hora — só com o TEXTO do script em foco (decisão do dono,
/// 17/10); num campo (busca, lista de scripts) segue para a shell, onde é o modo foco.
pub(crate) fn suggest_chord(key: Key, mods: Modifiers) -> bool {
    (mods.ctrl || mods.meta) && key == Key::Space
}

impl CodeDoc {
    /// Uma tecla premida (ou solta: tomada sem efeito).
    pub fn key(
        &mut self,
        key: Key,
        mods: Modifiers,
        pressed: bool,
        text: Option<&str>,
        paste: Option<&str>,
    ) -> KeyOutcome {
        let mut out = KeyOutcome {
            consumed: true,
            copy: None,
        };
        if !pressed {
            return out;
        }
        if let Some(f) = self.find.focus.filter(|_| self.find.open) {
            let cmd = mods.ctrl || mods.meta;
            // Os atalhos da barra continuam vivos dentro dela.
            match key {
                Key::Char('f') if cmd => self.find.open_find(&self.buffer, false),
                Key::Char('h') if cmd => self.find.open_find(&self.buffer, true),
                Key::Char('g') if cmd => self.find.open_goto(),
                Key::Char('s') if cmd => {
                    self.save = Some(if mods.shift {
                        SaveKind::SaveAs
                    } else {
                        SaveKind::Save
                    })
                }
                _ => {
                    out.copy = self.find_key(
                        f,
                        FieldKey {
                            key,
                            mods,
                            text,
                            paste,
                        },
                    )
                }
            }
            return out;
        }
        // Uma tecla morta pendente desfaz-se com a tecla seguinte (o texto dela já vem composto).
        if self.preedit.as_ref().is_some_and(|p| p.1) {
            self.preedit = None;
        }
        if self.completion_key(key, mods) {
            self.reveal();
            return out;
        }
        let cmd = mods.ctrl || mods.meta;
        let shift = mods.shift;
        match key {
            Key::Space if cmd => self.complete.open_now(
                self.buffer.text(),
                self.buffer.selection().primary().head,
                self.last_input_ns,
            ),
            Key::Char(c) if cmd => out.copy = self.chord(c, shift, paste),
            Key::Enter => self.enter(),
            Key::Backspace => self.backspace(cmd),
            Key::Delete => self.delete(cmd),
            Key::Tab => self.indent(shift),
            Key::Escape => self.escape(),
            Key::Left => self.motion(if cmd { Motion::WordLeft } else { Motion::Left }, shift),
            Key::Right => self.motion(
                if cmd {
                    Motion::WordRight
                } else {
                    Motion::Right
                },
                shift,
            ),
            Key::Up => self.motion(Motion::Up, shift),
            Key::Down => self.motion(Motion::Down, shift),
            Key::Home => self.motion(
                if cmd {
                    Motion::DocStart
                } else {
                    Motion::LineStart
                },
                shift,
            ),
            Key::End => self.motion(if cmd { Motion::DocEnd } else { Motion::LineEnd }, shift),
            Key::PageUp => self.motion(Motion::Lines(-(self.view.page_rows() as isize)), shift),
            Key::PageDown => self.motion(Motion::Lines(self.view.page_rows() as isize), shift),
            Key::Char(_) | Key::Space | Key::Other if !cmd => {
                if let Some(t) = text {
                    self.type_text(t);
                }
            }
            _ => {}
        }
        // A lista segue a palavra: escrever já a pôs (`type_text`); apagar ou andar dentro dela
        // mantém-na; o resto fecha-a.
        let head = self.buffer.selection().primary().head;
        match key {
            Key::Char(_) | Key::Space | Key::Other if !cmd => {}
            Key::Space => {}
            Key::Backspace | Key::Delete | Key::Left | Key::Right if !cmd && !shift => {
                self.complete.moved(self.buffer.text(), head);
            }
            _ => self.complete.close(),
        }
        self.reveal();
        out
    }

    fn motion(&mut self, m: Motion, extend: bool) {
        self.buffer.move_cursors(m, extend, TAB);
    }

    /// Esc: fecha a busca; senão um cursor só; senão desfaz a seleção.
    fn escape(&mut self) {
        if self.find.open {
            self.find.close();
        } else if self.buffer.selection().len() > 1 {
            self.buffer.single_cursor();
        } else {
            let h = self.buffer.selection().primary().head;
            self.buffer
                .set_selection(ph2d_code_buffer::Selection::single(
                    ph2d_code_buffer::Range::cursor(h),
                ));
        }
    }

    /// Os acordes com Ctrl/Cmd. Devolve o texto para a área de transferência.
    fn chord(&mut self, c: char, shift: bool, paste: Option<&str>) -> Option<String> {
        match c {
            'a' => self.buffer.select_all(),
            'd' => {
                self.buffer.select_next_occurrence();
            }
            'z' => {
                self.undo(shift);
            }
            'y' => {
                self.undo(true);
            }
            'c' => return self.copy(),
            'x' => return self.cut(),
            'v' => {
                if let Some(p) = paste {
                    self.paste(p);
                }
            }
            'f' => self.find.open_find(&self.buffer, false),
            'h' => self.find.open_find(&self.buffer, true),
            'g' => self.find.open_goto(),
            's' => {
                self.save = Some(if shift {
                    SaveKind::SaveAs
                } else {
                    SaveKind::Save
                })
            }
            _ => {}
        }
        None
    }
}
