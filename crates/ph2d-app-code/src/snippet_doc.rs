//! O modelo com buracos NA ABA (I12): inserir, o Tab/Shift+Tab/Esc da sessão e o fim dela (a lei está
//! em [`crate::snippet`]).

use crate::snippet::{Expanded, Jump, Tabstops};
use ph2d_code_buffer::{Change, ChangeSet, EditKind, Range as Cursor, Selection};
use ph2d_editor_core::doc_surface::Key;
use ph2d_host::Modifiers;

impl crate::doc::CodeDoc {
    /// Troca cada `(início, fim, primário)` por `exp` num passo de undo; com buracos, abre a sessão e
    /// selecciona o 1.º; sem eles, o cursor fica no `$0` (ou no fim).
    pub(crate) fn insert_snippet(&mut self, spots: &[(usize, usize, bool)], exp: &Expanded) {
        let t = self.buffer.text();
        let unit = self.auto.unit();
        let adapted: Vec<Expanded> = spots
            .iter()
            .map(|&(a, _, _)| {
                let ls = t.byte_of_line(t.line_of_byte(a));
                let indent: String = t
                    .slice(ls..a)
                    .chars()
                    .take_while(|c| *c == ' ' || *c == '\t')
                    .collect();
                exp.adapt(&indent, &unit)
            })
            .collect();
        let changes = spots
            .iter()
            .zip(&adapted)
            .map(|(&(a, b, _), e)| Change {
                range: a..b,
                insert: e.text.clone(),
            })
            .collect();
        let Some(cs) = ChangeSet::new(changes) else {
            return;
        };
        let mut delta = 0isize;
        let mut bases = Vec::with_capacity(spots.len());
        let mut main = 0;
        for (&(a, b, primary), e) in spots.iter().zip(&adapted) {
            if primary {
                main = bases.len();
            }
            bases.push((a as isize + delta) as usize);
            delta += e.text.len() as isize - (b - a) as isize;
        }
        if bases.is_empty() {
            return;
        }
        let placed: Vec<(usize, &Expanded)> = bases.iter().copied().zip(&adapted).collect();
        let session = Tabstops::new(&placed, main);
        let after = match &session {
            Some(s) => s.selection(),
            None => Selection::new(
                placed
                    .iter()
                    .map(|(b, e)| Cursor::cursor(b + e.end.unwrap_or(e.text.len())))
                    .collect(),
                main,
            ),
        };
        let edits = self.buffer.apply(cs, Some(after), EditKind::Other);
        self.edited(&edits, true);
        // Uma inserção SEM buracos dentro de um buraco (o campo escolhido da lista) deixa a sessão de
        // fora viva — o `edited` já a deslocou; acabá-la tornava o Tab seguinte em recuo (smoke I15).
        // COM buracos dentro de um buraco, as duas fundem-se ([`Tabstops::merge`]); fora, a nova manda.
        if let Some(inner) = session {
            match self.snippet.as_mut() {
                Some(outer) if outer.nests(&inner) => outer.merge(inner),
                _ => self.snippet = Some(inner),
            }
        }
    }

    /// Uma tecla com a sessão viva: `true` = era dela (Tab, Shift+Tab). O Esc acaba-a e segue para o
    /// Esc de sempre (um cursor só); sair do modelo acaba-a antes de a tecla contar.
    pub(crate) fn snippet_key(&mut self, key: Key, mods: Modifiers) -> bool {
        self.snippet_follow();
        let Some(s) = self.snippet.as_mut() else {
            return false;
        };
        if mods.ctrl || mods.meta || mods.alt {
            return false;
        }
        match key {
            Key::Tab if !mods.shift => {
                match s.next() {
                    Jump::Stop(sel) => self.buffer.jump_selection(sel),
                    Jump::End(sel) => {
                        self.buffer.jump_selection(sel);
                        self.snippet = None;
                    }
                }
                true
            }
            Key::Tab => {
                let sel = s.prev();
                self.buffer.jump_selection(sel);
                true
            }
            Key::Escape => {
                self.snippet = None;
                false
            }
            _ => false,
        }
    }

    /// A sessão acaba quando o cursor primário sai de todas as inserções.
    pub(crate) fn snippet_follow(&mut self) {
        let head = self.buffer.selection().primary().head;
        if self.snippet.as_ref().is_some_and(|s| !s.holds(head)) {
            self.snippet = None;
        }
    }
}
