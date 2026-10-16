//! ⭐ **E9 — buscar/substituir e ir para linha**, sobre o núcleo puro `ph2d_code_buffer::find`. Os
//! campos são `Buffer`s de uma linha; substituir tudo é UM passo de undo (o núcleo garante-o).

use crate::doc::CodeDoc;
use crate::edit::{self, FieldKey};
use ph2d_code_buffer::{
    Buffer, FindOptions, Range, Selection, find_all, find_update, goto_line, replace_all,
    replace_one,
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
    /// A busca e as opções das `matches`, e a revisão do texto que elas seguem.
    key: Option<(String, bool, bool)>,
    rev: u64,
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

    /// Fechada, a busca não custa nada: as ocorrências saem (e não seguem as edições).
    pub fn close(&mut self) {
        self.open = false;
        self.focus = None;
        self.matches.clear();
        self.current = None;
        self.key = None;
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

    /// As ocorrências seguem a busca e as opções (uma busca nova lê o texto inteiro, uma vez); a
    /// corrente é a primeira no cursor ou depois. As edições do texto chegam por [`Self::follow`].
    pub fn refresh(&mut self, buf: &Buffer) {
        let key = (self.query.text().to_string(), self.case, self.word);
        if self.key.as_ref() != Some(&key) || self.rev != buf.revision() {
            self.matches = find_all(buf.text(), &key.0, self.opts());
            self.key = Some(key);
            self.rev = buf.revision();
        }
        let from = buf.selection().primary().from();
        let i = self.matches.partition_point(|m| m.start < from);
        self.current = (!self.matches.is_empty()).then_some(if i < self.matches.len() { i } else { 0 });
    }

    /// O texto mudou por `edits`: com a busca aberta, as ocorrências seguem-nas sem reler o
    /// ficheiro (`find_update`); fechada, nada.
    pub fn follow(&mut self, buf: &Buffer, edits: &[ph2d_code_buffer::TextEdit]) {
        let Some((needle, ..)) = self.key.as_ref().filter(|_| self.open) else {
            return;
        };
        find_update(buf.text(), needle, self.opts(), &mut self.matches, edits);
        self.rev = buf.revision();
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
