//! As operações de TEXTO da aba (escrever, apagar, indentar, copiar/cortar/colar, desfazer) e o campo
//! de uma linha da barra de busca. Toda mutação passa pelo `Buffer` e a gramática segue-a
//! ([`CodeDoc::edited`]); o `AutoEdit` avança a gramática sozinho.

use crate::doc::CodeDoc;
use ph2d_code_buffer::{Buffer, Change, ChangeSet, EditKind, Motion, word_left, word_right};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;

/// Uma tecla para um campo de uma linha.
#[derive(Clone, Copy)]
pub(crate) struct FieldKey<'a> {
    pub key: Key,
    pub mods: Modifiers,
    pub text: Option<&'a str>,
    pub paste: Option<&'a str>,
}

#[derive(Default)]
pub(crate) struct FieldOut {
    pub changed: bool,
    pub enter: bool,
    pub escape: bool,
    pub tab: bool,
    pub copy: Option<String>,
}

/// Os caracteres que se escrevem (sem controlos: Enter, Tab e Backspace têm tecla própria).
pub(crate) fn printable(s: &str) -> impl Iterator<Item = char> + '_ {
    s.chars().filter(|c| !c.is_control())
}

/// Uma tecla num campo de uma linha.
pub(crate) fn field_key(buf: &mut Buffer, k: FieldKey<'_>) -> FieldOut {
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

impl CodeDoc {
    /// Escreve `s` em todos os cursores, carácter a carácter pelo `AutoEdit` (pares, recuo do `end`).
    pub fn type_text(&mut self, s: &str) {
        let mut last = None;
        for ch in printable(s) {
            let edits = self.auto.on_char(&mut self.buffer, &mut self.syntax, ch);
            self.edited(&edits, false);
            last = Some(ch);
        }
        if let Some(ch) = last {
            let head = self.buffer.selection().primary().head;
            let code = self.syntax.in_code(self.buffer.text(), head);
            self.complete
                .typed(self.buffer.text(), head, ch, code, self.last_input_ns);
            self.hints.typed(ch, code);
        }
    }

    pub fn enter(&mut self) {
        let edits = self.auto.on_enter(&mut self.buffer, &mut self.syntax);
        self.edited(&edits, false);
    }

    /// Backspace (`word` = Ctrl: até ao início da palavra).
    pub fn backspace(&mut self, word: bool) {
        if word {
            let edits = self.buffer.delete_with(|t, r| word_left(t, r.head)..r.head);
            self.edited(&edits, true);
        } else {
            let edits = self.auto.on_backspace(&mut self.buffer, &mut self.syntax);
            self.edited(&edits, false);
        }
    }

    /// Delete (`word` = Ctrl: até ao fim da palavra).
    pub fn delete(&mut self, word: bool) {
        let edits = if word {
            self.buffer
                .delete_with(|t, r| r.head..word_right(t, r.head))
        } else {
            self.buffer.delete_forward()
        };
        self.edited(&edits, true);
    }

    /// As linhas tocadas pelas seleções (uma seleção que acaba no início de uma linha não a toca).
    fn touched_lines(&self) -> Vec<usize> {
        let t = self.buffer.text();
        let mut lines: Vec<usize> = Vec::new();
        for r in self.buffer.selection().ranges() {
            let a = t.line_of_byte(r.from());
            let mut b = t.line_of_byte(r.to());
            if b > a && t.byte_of_line(b) == r.to() {
                b -= 1;
            }
            lines.extend(a..=b);
        }
        lines.sort_unstable();
        lines.dedup();
        lines
    }

    /// Tab / Shift+Tab: sem seleção o Tab escreve um nível; com seleção (ou `back`) indenta/recua
    /// cada linha tocada — UM passo de undo.
    pub fn indent(&mut self, back: bool) {
        let unit = self.auto.unit();
        let all_empty = self
            .buffer
            .selection()
            .ranges()
            .iter()
            .all(|r| r.is_empty());
        if !back && all_empty {
            let edits = self.buffer.insert_each(|_| unit.clone(), EditKind::Other);
            self.edited(&edits, true);
            return;
        }
        let t = self.buffer.text();
        let changes: Vec<Change> = self
            .touched_lines()
            .into_iter()
            .filter_map(|l| {
                let s = t.byte_of_line(l);
                if !back {
                    return Some(Change {
                        range: s..s,
                        insert: unit.clone(),
                    });
                }
                let line = t.line_string(l);
                let n = if line.starts_with('\t') {
                    1
                } else {
                    line.chars()
                        .take(unit.len())
                        .take_while(|c| *c == ' ')
                        .count()
                };
                (n > 0).then(|| Change {
                    range: s..s + n,
                    insert: String::new(),
                })
            })
            .collect();
        if let Some(cs) = ChangeSet::new(changes) {
            let edits = self.buffer.apply(cs, None, EditKind::Other);
            self.edited(&edits, true);
        }
    }

    /// O texto a copiar: as seleções, uma por linha; sem seleção nenhuma, as linhas dos cursores.
    pub fn copy(&self) -> Option<String> {
        let t = self.buffer.text();
        let ranges = self.buffer.selection().ranges();
        if ranges.iter().any(|r| !r.is_empty()) {
            let parts: Vec<String> = ranges
                .iter()
                .filter(|r| !r.is_empty())
                .map(|r| t.slice(r.span()))
                .collect();
            return Some(parts.join("\n"));
        }
        let lines = self.touched_lines();
        Some(
            lines
                .iter()
                .map(|&l| format!("{}\n", t.line_string(l)))
                .collect(),
        )
    }

    pub fn cut(&mut self) -> Option<String> {
        let copied = self.copy();
        let t = self.buffer.text();
        let edits = if self
            .buffer
            .selection()
            .ranges()
            .iter()
            .any(|r| !r.is_empty())
        {
            self.buffer.delete_with(|_, r| r.head..r.head)
        } else {
            let changes: Vec<Change> = self
                .touched_lines()
                .into_iter()
                .map(|l| {
                    let s = t.byte_of_line(l);
                    let e = if l + 1 < t.line_count() {
                        t.byte_of_line(l + 1)
                    } else {
                        t.len()
                    };
                    Change {
                        range: s..e,
                        insert: String::new(),
                    }
                })
                .collect();
            match ChangeSet::new(changes) {
                Some(cs) => self.buffer.apply(cs, None, EditKind::Other),
                None => Vec::new(),
            }
        };
        self.edited(&edits, true);
        copied
    }

    /// Colar: com tantas linhas quantos cursores, uma por cursor; senão o texto todo em cada um.
    pub fn paste(&mut self, s: &str) {
        let n = self.buffer.selection().len();
        let lines: Vec<&str> = s.lines().collect();
        let edits = if n > 1 && lines.len() == n {
            let mut i = 0;
            self.buffer.insert_each(
                |_| {
                    i += 1;
                    lines[i - 1].to_owned()
                },
                EditKind::Other,
            )
        } else {
            self.buffer.insert(s)
        };
        self.edited(&edits, true);
    }

    pub fn undo(&mut self, redo: bool) -> bool {
        let edits = if redo {
            self.buffer.redo()
        } else {
            self.buffer.undo()
        };
        let did = !edits.is_empty();
        self.edited(&edits, true);
        did
    }
}
