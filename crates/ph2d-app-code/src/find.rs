//! ⭐ **E9 — buscar/substituir e ir para linha**, sobre o núcleo puro `ph2d_code_buffer::find`. Os
//! campos são `Buffer`s de uma linha; substituir tudo é UM passo de undo (o núcleo garante-o).

use crate::doc::CodeDoc;
use crate::edit::{self, FieldKey};
use ph2d_code_buffer::{
    Buffer, FindOptions, Range, Selection, find_all, goto_line, replace_all, replace_one,
};
use ph2d_panel_code::{FindBar, FindBarView, FindCount, FindField};
use std::ops::Range as Span;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Mode {
    #[default]
    Find,
    Replace,
    GoTo,
}

#[derive(Default)]
pub(crate) struct FindState {
    pub open: bool,
    pub mode: Mode,
    pub query: Buffer,
    pub replace: Buffer,
    pub goto: Buffer,
    /// O campo com o teclado; `None` = o texto do editor.
    pub focus: Option<FindField>,
    pub case: bool,
    pub word: bool,
    pub matches: Vec<Span<usize>>,
    pub current: Option<usize>,
    key: Option<(u64, String, bool, bool, usize)>,
}

impl FindState {
    pub fn opts(&self) -> FindOptions {
        FindOptions {
            case_sensitive: self.case,
            whole_word: self.word,
        }
    }

    /// Ctrl+F / Ctrl+H: abre com a seleção de uma linha como busca, e o campo com o teclado.
    pub fn open_find(&mut self, buf: &Buffer, replace: bool) {
        let p = buf.selection().primary();
        let picked = buf.text().slice(p.span());
        if !p.is_empty() && !picked.contains('\n') {
            self.query = Buffer::new(&picked);
        }
        self.query.select_all();
        self.open = true;
        self.mode = if replace { Mode::Replace } else { Mode::Find };
        self.focus = Some(FindField::Query);
        self.key = None;
    }

    /// Ctrl+G.
    pub fn open_goto(&mut self) {
        self.goto = Buffer::default();
        self.open = true;
        self.mode = Mode::GoTo;
        self.focus = Some(FindField::GoTo);
    }

    pub fn close(&mut self) {
        self.open = false;
        self.focus = None;
    }

    pub fn bar(&self) -> FindBar<'_> {
        let goto = self.mode == Mode::GoTo;
        FindBar {
            query: &self.query,
            replace: (self.mode == Mode::Replace).then_some(&self.replace),
            goto: goto.then_some(&self.goto),
            focus: self.focus,
            case_sensitive: self.case,
            whole_word: self.word,
            count: if goto { None } else { self.count() },
        }
    }

    pub fn bar_height(&self) -> f32 {
        if self.open {
            FindBarView::height(&self.bar())
        } else {
            0.0
        }
    }

    /// A contagem do ecrã; `None` com a busca vazia.
    pub fn count(&self) -> Option<FindCount> {
        (!self.query.text().is_empty()).then_some(FindCount {
            current: self.current,
            total: self.matches.len(),
        })
    }

    /// As ocorrências seguem o texto, a busca e as opções; a corrente é a primeira no cursor ou depois.
    pub fn refresh(&mut self, buf: &Buffer) {
        let needle = self.query.text().to_string();
        let from = buf.selection().primary().from();
        let key = (buf.revision(), needle.clone(), self.case, self.word, from);
        if self.key.as_ref() == Some(&key) {
            return;
        }
        if self
            .key
            .as_ref()
            .is_none_or(|k| (k.0, &k.1, k.2, k.3) != (key.0, &key.1, key.2, key.3))
        {
            self.matches = find_all(buf.text(), &needle, self.opts());
        }
        self.current = if self.matches.is_empty() {
            None
        } else {
            Some(
                self.matches
                    .iter()
                    .position(|m| m.start >= from)
                    .unwrap_or(0),
            )
        };
        self.key = Some(key);
    }

    pub fn field_mut(&mut self, f: FindField) -> &mut Buffer {
        match f {
            FindField::Query => &mut self.query,
            FindField::Replace => &mut self.replace,
            FindField::GoTo => &mut self.goto,
        }
    }
}

impl CodeDoc {
    fn select_match(&mut self, i: usize) {
        let m = self.find.matches[i].clone();
        self.buffer
            .set_selection(Selection::single(Range::new(m.start, m.end)));
        self.buffer.seal_history();
        self.find.refresh(&self.buffer);
        self.reveal();
    }

    /// A ocorrência seguinte (`back` = a anterior), dando a volta.
    pub fn find_step(&mut self, back: bool) {
        self.find.refresh(&self.buffer);
        let n = self.find.matches.len();
        let Some(cur) = self.find.current else { return };
        let on = self.buffer.selection().primary().span() == self.find.matches[cur];
        let i = match (back, on) {
            (false, true) => (cur + 1) % n,
            (false, false) => cur,
            (true, _) => (cur + n - 1) % n,
        };
        self.select_match(i);
    }

    /// Substitui a ocorrência seleccionada e passa à seguinte (sem uma seleccionada, só a selecciona).
    pub fn replace_current(&mut self) {
        self.find.refresh(&self.buffer);
        let Some(cur) = self.find.current else { return };
        let m = self.find.matches[cur].clone();
        if self.buffer.selection().primary().span() != m {
            self.select_match(cur);
            return;
        }
        let with = self.find.replace.text().to_string();
        let edits = replace_one(&mut self.buffer, m, &with);
        self.edited(&edits, true);
        self.find.refresh(&self.buffer);
        if self.find.current.is_some() {
            self.find_step(false);
        }
    }

    /// Substitui todas — UM passo de undo.
    pub fn replace_everything(&mut self) {
        self.find.refresh(&self.buffer);
        let with = self.find.replace.text().to_string();
        let matches = self.find.matches.clone();
        let edits = replace_all(&mut self.buffer, &matches, &with);
        self.edited(&edits, true);
        self.find.refresh(&self.buffer);
        self.reveal();
    }

    /// Enter no «Go to line»: o cursor na linha, à vista; a barra fecha.
    pub fn goto_commit(&mut self) {
        let Ok(n) = self.find.goto.text().to_string().trim().parse::<usize>() else {
            return;
        };
        goto_line(&mut self.buffer, n);
        self.find.close();
        self.reveal();
    }

    /// Uma tecla com um campo da barra focado.
    pub fn find_key(&mut self, f: FindField, k: FieldKey<'_>) -> Option<String> {
        let out = edit::field_key(self.find.field_mut(f), k);
        if out.escape {
            self.find.close();
        } else if out.tab && self.find.mode == Mode::Replace {
            self.find.focus = Some(if f == FindField::Query {
                FindField::Replace
            } else {
                FindField::Query
            });
        } else if out.enter {
            match f {
                FindField::GoTo => self.goto_commit(),
                FindField::Query => self.find_step(k.mods.shift),
                FindField::Replace if k.mods.ctrl || k.mods.meta => self.replace_everything(),
                FindField::Replace => self.replace_current(),
            }
        } else if out.changed && f == FindField::Query {
            // Escrever na busca salta para a primeira ocorrência a partir do cursor.
            self.find.refresh(&self.buffer);
            if let Some(i) = self.find.current {
                let m = self.find.matches[i].clone();
                self.view.ensure_visible(&self.buffer, m.start);
            }
        }
        out.copy
    }
}
