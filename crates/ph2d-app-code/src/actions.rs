//! I7 na aba: abrir, andar e aplicar as acções de contexto (Alt+Enter). A lista e as leis moram em
//! `ph2d_code_assist::actions`.

use crate::doc::CodeDoc;
pub(crate) use ph2d_code_assist::actions::*;
use ph2d_code_buffer::{Change, ChangeSet, EditKind, Range as Cursor, Selection, word_at};
use ph2d_code_intel::{Answer, Diag, Req, Ticket};
use ph2d_i18n::tr;
use std::ops::Range;

impl CodeDoc {
    /// Alt+Enter: as acções para o cursor primário. As nossas já; as do servidor quando ele responder.
    pub(crate) fn open_actions(&mut self) {
        self.complete.close();
        self.hints.hover = None;
        self.syntax.fresh(self.buffer.text());
        let head = self.buffer.selection().primary().head;
        self.actions = Some(self.actions_at(&[head]));
    }

    /// Uma lista de escolhas que não pergunta nada ao servidor (a soltura, item 15), alinhada em
    /// `anchor`, com `note` na caixa ao lado.
    pub(crate) fn offer(&mut self, anchor: usize, list: Vec<Action>, note: String) {
        self.complete.close();
        self.hints.hover = None;
        self.actions = Some(Actions {
            revision: self.buffer.revision(),
            anchor,
            wants: Vec::new(),
            asked: Vec::new(),
            groups: [list, Vec::new(), Vec::new()],
            selected: 0,
            top: 0,
            note: Some(note),
        });
    }

    /// As acções que o Alt+Enter daria em cada um dos `heads` (a janelinha alinha-se no 1.º), sem
    /// repetir títulos. A porta única: o Alt+Enter e os botões do balão (I8b) vêm daqui.
    pub(crate) fn actions_at(&self, heads: &[usize]) -> Actions {
        let t = self.buffer.text();
        let word_of = |h: usize| word_at(t, h).filter(|w| self.syntax.in_code(t, w.start));
        let first = heads.first().copied().unwrap_or(0);
        let mut a = Actions {
            revision: self.buffer.revision(),
            anchor: word_of(first).map_or(first, |w| w.start),
            wants: Vec::new(),
            asked: Vec::new(),
            groups: Default::default(),
            selected: 0,
            top: 0,
            note: None,
        };
        if let Some(p) = self.syntax.promote_number(t, first) {
            a.groups[1].push(Action {
                title: tr("code.action.make_property").to_owned(),
                edits: p.edits,
                then: Then::Rename(p.name_at),
            });
        }
        for &head in heads {
            let touches = |r: &Range<usize>| r.start <= head && head <= r.end;
            let shown: Vec<Diag> = self
                .assist
                .list
                .iter()
                .filter(|d| touches(&d.bytes) && !crate::problems::engine_hook(t, d, &d.bytes))
                .cloned()
                .collect();
            for l in self.lint.list.iter().filter(|l| touches(&l.bytes)) {
                a.did_you_mean(&l.bytes, l.near.clone());
            }
            if let Some(typo) = shown.iter().find_map(|d| typo_of(t, d)) {
                a.wants
                    .push((Req::Candidates(typo.start), Purpose::Candidates(typo)));
            }
            if !shown.is_empty() {
                a.wants
                    .push((Req::Fixes(head..head, shown), Purpose::Fixes));
            }
            if let Some(w) = word_of(head) {
                a.wants.push((Req::TypeHints, Purpose::TypeHints(w)));
            }
        }
        a
    }

    /// A resposta `ticket` às acções abertas.
    pub(crate) fn actions_answer(&mut self, ticket: Ticket, answer: Answer) {
        let text = self.buffer.text().clone();
        if let Some(a) = self.actions.as_mut() {
            a.answer(ticket, answer, &text);
        }
    }

    /// O que a janelinha mostra das acções (só depois de tudo respondido; vazia = «nenhuma aqui»).
    pub(crate) fn actions_shown(&self) -> Option<crate::complete_doc::Shown> {
        let a = self.actions.as_ref().filter(|a| a.ready())?;
        let mut titles: Vec<String> = a.list().map(|x| x.title.clone()).collect();
        let preview = match a.note {
            Some(_) => Vec::new(),
            None => a.preview(a.selected, self.buffer.text()),
        };
        if titles.is_empty() {
            titles.push(tr("code.action.none").to_owned());
        }
        let rows = ph2d_panel_code::COMPLETION_ROWS;
        let width_chars =
            ph2d_panel_code::CompletionView::width_of(titles.iter().map(|t| (t.as_str(), None)));
        Some(crate::complete_doc::Shown {
            anchor: a.anchor,
            total: titles.len(),
            rows: titles
                .into_iter()
                .skip(a.top)
                .take(rows)
                .map(|label| ph2d_panel_code::CompletionRow {
                    label,
                    matched: Vec::new(),
                    ink: ph2d_panel_code::Ink::Plain,
                    detail: None,
                })
                .collect(),
            top: a.top,
            selected: a.selected,
            width_chars,
            doc: a.note.clone(),
            preview,
            kind: ph2d_panel_code::ListKind::Actions,
        })
    }

    /// Uma tecla com as acções abertas: `true` = era delas (setas, páginas, Enter/Tab aplicam, Esc
    /// fecha; um modificador sozinho não conta). `false` = quem chama fecha-as e segue.
    pub(crate) fn actions_key(
        &mut self,
        key: ph2d_editor_core::doc_surface::Key,
        mods: ph2d_host::Modifiers,
        text: Option<&str>,
    ) -> bool {
        use ph2d_editor_core::doc_surface::Key;
        let Some(a) = self.actions.as_mut() else {
            return false;
        };
        if key == Key::Other && text.is_none() {
            return true;
        }
        if key == Key::Escape {
            self.actions = None;
            return true;
        }
        if !a.ready() || mods.ctrl || mods.meta || mods.shift {
            return false;
        }
        let len = a.list().count().max(1);
        let rows = ph2d_panel_code::COMPLETION_ROWS;
        match key {
            Key::Up => step(a, -1, len, rows),
            Key::Down => step(a, 1, len, rows),
            Key::PageUp => step(a, -(rows as isize), len, rows),
            Key::PageDown => step(a, rows as isize, len, rows),
            Key::Enter | Key::Tab => {
                let i = a.selected;
                self.apply_action(i);
            }
            _ => return false,
        }
        true
    }

    /// Aplica a acção `index` da janelinha (Enter ou clique).
    pub(crate) fn apply_action(&mut self, index: usize) {
        if let Some(a) = self.actions.take() {
            self.apply(&a, index);
        }
    }

    /// Aplica a acção `index` de `a`, num passo de undo, com o cursor no fim da 1.ª troca. Sobre um
    /// texto que já não é o delas, nada.
    pub(crate) fn apply(&mut self, a: &Actions, index: usize) {
        let Some(act) = a
            .list()
            .nth(index)
            .filter(|_| a.revision == self.buffer.revision())
        else {
            return;
        };
        let Some((r0, s0)) = act.edits.first() else {
            return;
        };
        let at = match act.then {
            Then::EndOfFirst => r0.start + s0.len(),
            Then::Caret(b) | Then::Rename(b) => b,
        };
        let changes = act
            .edits
            .iter()
            .map(|(range, insert)| Change {
                range: range.clone(),
                insert: insert.clone(),
            })
            .collect();
        let Some(cs) = ChangeSet::new(changes) else {
            return;
        };
        self.buffer.seal_history();
        let edits = self.buffer.apply(
            cs,
            Some(Selection::single(Cursor::cursor(at))),
            EditKind::Other,
        );
        self.buffer.seal_history();
        self.edited(&edits, true);
        if matches!(act.then, Then::Rename(_)) {
            self.start_rename();
        }
    }

    /// A roda sobre a janelinha das acções.
    pub(crate) fn actions_scroll(&mut self, lines: isize) {
        if let Some(a) = self.actions.as_mut().filter(|a| a.ready()) {
            let len = a.list().count().max(1);
            let rows = ph2d_panel_code::COMPLETION_ROWS;
            let max_top = len.saturating_sub(rows) as isize;
            a.top = (a.top as isize + lines).clamp(0, max_top) as usize;
        }
    }
}
