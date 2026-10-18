//! O `Buffer`: texto + seleção + histórico + revisão. Toda mutação passa por `apply`.

use crate::change::{Change, ChangeSet, TextEdit};
use crate::history::{EditKind, History};
use crate::selection::{Range, Selection};
use crate::text::{Text, normalize_newlines};
use unicode_segmentation::UnicodeSegmentation;

/// O que uma mutação devolve: as edições para quem segue o texto (sintaxe, LSP) em ordem de
/// aplicação sequencial. Vazio = nada mudou.
pub type Edits = Vec<TextEdit>;

#[derive(Debug, Clone, Default)]
pub struct Buffer {
    text: Text,
    selection: Selection,
    history: History,
    revision: u64,
}

impl Buffer {
    pub fn new(s: &str) -> Self {
        Self {
            text: Text::new(s),
            ..Self::default()
        }
    }

    pub fn text(&self) -> &Text {
        &self.text
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    /// Sobe a cada mutação do texto (inclui undo/redo); nunca desce.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn is_dirty(&self) -> bool {
        !self.history.at_saved()
    }

    pub fn mark_saved(&mut self) {
        self.history.mark_saved();
    }

    /// O tecto de bytes do histórico ([`crate::HISTORY_BYTES_MAX`] por omissão).
    pub fn set_history_cap(&mut self, bytes: usize) {
        self.history.set_cap(bytes);
    }

    pub fn seal_history(&mut self) {
        self.history.seal();
    }

    /// Liga o último grupo do histórico a um passo de outra história (ver [`History::top_link`]).
    pub fn link_last(&mut self, link: u64) {
        self.history.link_last(link);
    }

    pub fn unlink(&mut self, link: u64) {
        self.history.unlink(link);
    }

    pub fn set_selection(&mut self, sel: Selection) {
        let n = self.text.len();
        let clamped = sel
            .ranges()
            .iter()
            .map(|r| Range {
                // Uma posição vinda de fora no meio de um carácter desce ao início dele.
                anchor: self.text.floor_char(r.anchor.min(n)),
                head: self.text.floor_char(r.head.min(n)),
                goal: r.goal,
            })
            .collect();
        self.selection = sel.with_ranges(clamped);
    }

    /// A ÚNICA porta de mutação: aplica, mapeia a seleção (ou usa `after`) e regista no histórico.
    pub fn apply(&mut self, cs: ChangeSet, after: Option<Selection>, kind: EditKind) -> Edits {
        // As pontas de cada troca descem à fronteira de carácter (o `crop` entra em pânico no meio
        // de um); as edições, o inverso e a aplicação usam as MESMAS faixas.
        let cs = cs.snapped(&self.text);
        if cs.is_empty() {
            if let Some(sel) = after {
                self.set_selection(sel);
            }
            return Vec::new();
        }
        let edits = cs.text_edits(&self.text);
        let inverse = cs.invert(&self.text);
        let inserted: String = cs.changes().iter().map(|c| c.insert.as_str()).collect();
        let before = self.selection.clone();
        cs.apply(&mut self.text);
        let after = after.unwrap_or_else(|| before.map(&cs));
        self.history
            .record(cs, inverse, before, after.clone(), kind, &inserted);
        self.selection = after;
        self.revision += 1;
        edits
    }

    /// Substitui CADA faixa da seleção pelo texto (a escrita em todos os cursores).
    pub fn insert(&mut self, s: &str) -> Edits {
        let s = normalize_newlines(s);
        let kind = if s.graphemes(true).count() == 1 && !s.contains('\n') {
            EditKind::Typing
        } else {
            EditKind::Other
        };
        self.insert_each(|_| s.to_string(), kind)
    }

    /// Como `insert`, mas com um texto por cursor (Enter com a indentação de cada linha). O texto
    /// de cada cursor passa pela mesma lei de fins de linha (colar `a\rb` num cursor de cada vez).
    pub fn insert_each(&mut self, mut f: impl FnMut(&Range) -> String, kind: EditKind) -> Edits {
        let changes = self
            .selection
            .ranges()
            .iter()
            .map(|r| Change {
                range: r.span(),
                insert: normalize_newlines(&f(r)).into_owned(),
            })
            .collect();
        match ChangeSet::new(changes) {
            Some(cs) => {
                let after = self.collapsed_after(&cs);
                self.apply(cs, Some(after), kind)
            }
            None => Vec::new(),
        }
    }

    fn collapsed_after(&self, cs: &ChangeSet) -> Selection {
        let mut delta: isize = 0;
        let ends: Vec<Range> = cs
            .changes()
            .iter()
            .map(|c| {
                delta += c.insert.len() as isize - c.range.len() as isize;
                Range::cursor((c.range.end as isize + delta) as usize)
            })
            .collect();
        Selection::new(ends, self.selection.primary_index())
    }

    pub fn delete_backward(&mut self) -> Edits {
        self.delete_with(|t, r| t.prev_grapheme(r.head)..r.head)
    }

    pub fn delete_forward(&mut self) -> Edits {
        self.delete_with(|t, r| r.head..t.next_grapheme(r.head))
    }

    /// Apaga a seleção de cada cursor; o cursor vazio apaga a faixa que `span` lhe der.
    pub fn delete_with(&mut self, span: impl Fn(&Text, &Range) -> std::ops::Range<usize>) -> Edits {
        let changes = self
            .selection
            .ranges()
            .iter()
            .map(|r| Change {
                range: if r.is_empty() {
                    span(&self.text, r)
                } else {
                    r.span()
                },
                insert: String::new(),
            })
            .collect::<Vec<_>>();
        let mut merged: Vec<Change> = Vec::new();
        for c in changes {
            match merged.last_mut() {
                Some(last) if c.range.start <= last.range.end => {
                    last.range.end = last.range.end.max(c.range.end)
                }
                _ => merged.push(c),
            }
        }
        match ChangeSet::new(merged) {
            Some(cs) => self.apply(cs, None, EditKind::Deleting),
            None => Vec::new(),
        }
    }

    pub fn undo(&mut self) -> Edits {
        let Some((sets, sel)) = self.history.pop_undo() else {
            return Vec::new();
        };
        self.replay(sets, sel)
    }

    pub fn redo(&mut self) -> Edits {
        let Some((sets, sel)) = self.history.pop_redo() else {
            return Vec::new();
        };
        self.replay(sets, sel)
    }

    fn replay(&mut self, sets: Vec<ChangeSet>, sel: Selection) -> Edits {
        let mut edits = Vec::new();
        for cs in sets {
            edits.extend(cs.text_edits(&self.text));
            cs.apply(&mut self.text);
        }
        self.set_selection(sel);
        self.revision += 1;
        edits
    }

    /// Troca o texto inteiro (ficheiro importado de fora): um passo de undo como qualquer outro.
    pub fn replace_all(&mut self, s: &str) -> Edits {
        let cs = ChangeSet::single(0..self.text.len(), normalize_newlines(s).into_owned());
        self.apply(cs, Some(Selection::default()), EditKind::Other)
    }
}

impl std::fmt::Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.text.fmt(f)
    }
}
