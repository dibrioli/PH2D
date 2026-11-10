//! O estado da barra de buscar/substituir e ir para linha (E9), sobre o núcleo puro
//! `ph2d_code_buffer::find`: os campos são `Buffer`s de uma linha. A aba (`ph2d-app-code`) guarda um
//! por documento e liga-o ao texto.

use crate::{FindBar, FindBarView, FindCount, FindField};
use ph2d_code_buffer::{Buffer, FindOptions, TextEdit, find_all, find_update};
use std::ops::Range as Span;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FindMode {
    #[default]
    Find,
    Replace,
    GoTo,
}

#[derive(Default)]
pub struct FindState {
    pub open: bool,
    pub mode: FindMode,
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
        self.mode = if replace {
            FindMode::Replace
        } else {
            FindMode::Find
        };
        self.focus = Some(FindField::Query);
        self.key = None;
    }

    /// Ctrl+G.
    pub fn open_goto(&mut self) {
        self.goto = Buffer::default();
        self.open = true;
        self.mode = FindMode::GoTo;
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
        let goto = self.mode == FindMode::GoTo;
        FindBar {
            query: &self.query,
            replace: (self.mode == FindMode::Replace).then_some(&self.replace),
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
        self.current =
            (!self.matches.is_empty()).then_some(if i < self.matches.len() { i } else { 0 });
    }

    /// O texto mudou por `edits`: com a busca aberta, as ocorrências seguem-nas sem reler o
    /// ficheiro (`find_update`); fechada, nada.
    pub fn follow(&mut self, buf: &Buffer, edits: &[TextEdit]) {
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
