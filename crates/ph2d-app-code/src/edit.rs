//! As operações de TEXTO da aba (escrever, apagar, indentar, copiar/cortar/colar, desfazer) e o campo
//! de uma linha da barra de busca. Toda mutação passa pelo `Buffer` e a gramática segue-a
//! ([`CodeDoc::edited`]); o `AutoEdit` avança a gramática sozinho.

use crate::doc::CodeDoc;
use ph2d_code_buffer::{Change, ChangeSet, EditKind, word_left, word_right};
pub(crate) use ph2d_panel_code::field::{FieldKey, field_key, printable};

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
            // A assinatura pergunta NA tecla: o `(`/`,` pede a árvore exacta, como o Enter.
            if matches!(ch, '(' | ',') {
                self.syntax.fresh(self.buffer.text());
            }
            // A árvore velha não sabe se o que acabou de entrar é código (`Completer::typed`).
            let code =
                (!self.syntax.is_stale()).then(|| self.syntax.in_code(self.buffer.text(), head));
            self.complete
                .typed(self.buffer.text(), head, ch, code, self.last_input_ns);
            self.hints.typed(ch, code.unwrap_or(false));
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

    /// Ctrl+/: comenta ou descomenta (`-- `) as linhas tocadas por todos os cursores — UM passo de
    /// undo, o idioma do VS Code e do Godot. Todas as não-vazias já começam por `--` ⇒ tira o `--` e
    /// um espaço; senão comenta-as todas na menor indentação delas. As vazias ficam como estão.
    pub fn toggle_comment(&mut self) {
        let t = self.buffer.text();
        let lines: Vec<(usize, String)> = self
            .touched_lines()
            .into_iter()
            .map(|l| (l, t.line_string(l)))
            .filter(|(_, s)| !s.trim().is_empty())
            .collect();
        let indent = |s: &str| s.bytes().take_while(|b| *b == b' ' || *b == b'\t').count();
        let commented =
            !lines.is_empty() && lines.iter().all(|(_, s)| s[indent(s)..].starts_with("--"));
        let min = lines.iter().map(|(_, s)| indent(s)).min().unwrap_or(0);
        let changes: Vec<Change> = lines
            .iter()
            .map(|(l, s)| {
                let b = t.byte_of_line(*l);
                if commented {
                    let at = b + indent(s);
                    let n = if s[indent(s) + 2..].starts_with(' ') {
                        3
                    } else {
                        2
                    };
                    Change {
                        range: at..at + n,
                        insert: String::new(),
                    }
                } else {
                    Change {
                        range: b + min..b + min,
                        insert: "-- ".to_owned(),
                    }
                }
            })
            .collect();
        if let Some(cs) = ChangeSet::new(changes) {
            let edits = self.buffer.apply(cs, None, EditKind::Other);
            self.edited(&edits, true);
        }
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

    /// Desfazer/refazer na aba. Um grupo ligado à fila global (I10, [`crate::link`]) não anda aqui:
    /// o pedido vai a ela, e o texto anda quando ela andar.
    pub fn undo(&mut self, redo: bool) -> bool {
        if let Some(l) = self.buffer.history().top_link(redo) {
            self.link_ask = Some((l, redo));
            return true;
        }
        self.undo_text(redo)
    }

    pub(crate) fn undo_text(&mut self, redo: bool) -> bool {
        self.snippet = None;
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
