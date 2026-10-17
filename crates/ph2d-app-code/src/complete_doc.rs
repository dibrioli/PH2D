//! A lista de completação NA ABA (I4): o que se pinta neste quadro, as teclas dela e a aceitação
//! (a lei do filtro, da ordem e da sessão está em [`crate::complete`]).

use crate::complete::{insertion, replaced, word_at};
use std::ops::Range;

/// A tinta do rótulo pelo `kind` do LSP (as cores do realce do próprio código).
fn ink_of_kind(kind: Option<u32>) -> ph2d_panel_code::Ink {
    use ph2d_panel_code::Ink;
    match kind {
        Some(2..=4) => Ink::Function,
        Some(5 | 10) => Ink::Property,
        Some(6) => Ink::Variable,
        Some(7 | 8 | 9 | 22 | 25) => Ink::Type,
        Some(14) => Ink::Keyword,
        Some(12 | 20 | 21) => Ink::String,
        _ => Ink::Plain,
    }
}

/// As faixas de bytes do rótulo que casaram (os índices de carácter agrupados em trechos).
fn matched_bytes(label: &str, chars: &[usize]) -> Vec<Range<usize>> {
    let offs: Vec<(usize, usize)> = label
        .char_indices()
        .map(|(b, c)| (b, b + c.len_utf8()))
        .collect();
    let mut out: Vec<Range<usize>> = Vec::new();
    for &i in chars {
        let Some(&(a, b)) = offs.get(i) else { continue };
        match out.last_mut() {
            Some(r) if r.end == a => r.end = b,
            _ => out.push(a..b),
        }
    }
    out
}

/// O que a aba pinta da lista neste quadro.
pub(crate) struct Shown {
    pub anchor: usize,
    pub rows: Vec<ph2d_panel_code::CompletionRow>,
    pub total: usize,
    pub top: usize,
    pub selected: usize,
    pub width_chars: usize,
    pub doc: Option<String>,
    /// A prévia da acção seleccionada (só as acções do I7 a têm).
    pub preview: Vec<ph2d_panel_code::PreviewLine>,
    pub actions: bool,
}

impl crate::doc::CodeDoc {
    /// A lista aberta deste quadro (só a janela das linhas à vista).
    pub(crate) fn completion_shown(&mut self) -> Option<Shown> {
        let head = self.buffer.selection().primary().head;
        let list = self.complete.shown(self.buffer.text(), head)?;
        let s = self.complete.session.as_ref()?;
        let items = s.items.as_ref()?;
        let rows = ph2d_panel_code::COMPLETION_ROWS;
        let width_chars = ph2d_panel_code::CompletionView::width_of(
            list.iter()
                .map(|(i, _)| (items[*i].label.as_str(), items[*i].detail.as_deref())),
        );
        let window = list
            .iter()
            .skip(s.top)
            .take(rows)
            .map(|(i, m)| {
                let it = &items[*i];
                ph2d_panel_code::CompletionRow {
                    matched: matched_bytes(&it.label, &m.chars),
                    ink: ink_of_kind(it.kind),
                    detail: it.detail.clone(),
                    label: it.label.clone(),
                }
            })
            .collect();
        let sel = &items[list[s.selected].0];
        Some(Shown {
            anchor: s.anchor,
            rows: window,
            total: list.len(),
            top: s.top,
            selected: s.selected,
            width_chars,
            doc: sel.documentation.clone().or_else(|| sel.detail.clone()),
            preview: Vec::new(),
            actions: false,
        })
    }

    /// Uma tecla com a lista aberta: `true` = era dela (setas, páginas, Tab/Enter aceitam, Esc fecha).
    pub(crate) fn completion_key(
        &mut self,
        key: ph2d_editor_core::doc_surface::Key,
        mods: ph2d_host::Modifiers,
    ) -> bool {
        use ph2d_editor_core::doc_surface::Key;
        if mods.ctrl || mods.meta || mods.alt {
            return false;
        }
        let head = self.buffer.selection().primary().head;
        let Some(len) = self
            .complete
            .shown(self.buffer.text(), head)
            .map(|l| l.len())
        else {
            return false;
        };
        let rows = ph2d_panel_code::COMPLETION_ROWS;
        match key {
            Key::Up if !mods.shift => self.complete.step(-1, len, rows),
            Key::Down if !mods.shift => self.complete.step(1, len, rows),
            Key::PageUp if !mods.shift => self.complete.step(-(rows as isize), len, rows),
            Key::PageDown if !mods.shift => self.complete.step(rows as isize, len, rows),
            Key::Tab | Key::Enter if !mods.shift => {
                let i = self.complete.session.as_ref().map_or(0, |s| s.selected);
                self.accept_completion(i);
            }
            Key::Escape => self.complete.close(),
            _ => return false,
        }
        true
    }

    /// Aceita a linha `index` da lista mostrada: troca a palavra escrita em CADA cursor que a tem
    /// pela proposta, num passo de undo, com o cursor onde o snippet o põe.
    pub(crate) fn accept_completion(&mut self, index: usize) {
        use ph2d_code_buffer::{Change, ChangeSet, EditKind, Range as Cursor, Selection};
        let head = self.buffer.selection().primary().head;
        let Some(list) = self.complete.shown(self.buffer.text(), head) else {
            return;
        };
        let (Some((i, _)), Some(s)) = (list.get(index), self.complete.session.as_ref()) else {
            return;
        };
        let Some(item) = s.items.as_ref().and_then(|v| v.get(*i)) else {
            return;
        };
        let (insert, caret) = insertion(item);
        let signature = item.signature;
        let t = self.buffer.text();
        let typed = t.slice(replaced(s, head));
        let n = typed.len();
        let sel = self.buffer.selection();
        let primary = sel.primary();
        let mut spots: Vec<(usize, usize, bool)> = sel
            .ranges()
            .iter()
            .filter(|r| r.is_empty() && r.head >= n && t.slice(r.head - n..r.head) == typed)
            .filter(|r| word_at(t, r.head).is_some_and(|w| w.start == r.head - n))
            .map(|r| (r.head - n, r.head, *r == primary))
            .collect();
        spots.sort_by_key(|s| s.0);
        let changes = spots
            .iter()
            .map(|&(a, b, _)| Change {
                range: a..b,
                insert: insert.clone(),
            })
            .collect();
        let Some(cs) = ChangeSet::new(changes) else {
            return;
        };
        let mut delta = 0isize;
        let mut cursors = Vec::with_capacity(spots.len());
        let mut main = 0;
        for &(a, b, is_primary) in &spots {
            let at = (a as isize + delta) as usize + caret;
            if is_primary {
                main = cursors.len();
            }
            cursors.push(Cursor::cursor(at));
            delta += insert.len() as isize - (b - a) as isize;
        }
        if cursors.is_empty() {
            return;
        }
        let edits = self
            .buffer
            .apply(cs, Some(Selection::new(cursors, main)), EditKind::Other);
        self.edited(&edits, true);
        self.complete.close();
        if signature {
            self.hints.open_signature();
        }
    }
}
