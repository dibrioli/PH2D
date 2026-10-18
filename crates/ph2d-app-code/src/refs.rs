//! ⭐ **I10 — as referências**: com o cursor num nome, as outras ocorrências dele ficam realçadas.
//! Sem pausa de interface: o VS Code 1.142.0 realça em ≤ 125 ms depois do clique
//! (`ferramentas/medicoes_onda2/I10_rename_e_undo.md` §4), e o servidor responde em `0,1` ms a 7 500
//! linhas, fora do quadro. Os nomes do código vêm do `luau-lsp`; os campos do `self` e a
//! `ph2d.property` deles são nossos (o servidor não os liga). Um membro nunca se pergunta (o servidor
//! morre, §1).

use crate::doc::CodeDoc;
use crate::rename::{field_sites, hook_names};
use ph2d_code_intel::{Answer, Ticket};
use ph2d_code_syntax::NameAt;
use std::ops::Range;

#[derive(Default)]
pub(crate) struct Occurrences {
    /// (revisão do texto, cursor, forma da árvore) da última pergunta.
    at: Option<(u64, usize, u64)>,
    /// Os sítios realçados (bytes do buffer).
    pub ranges: Vec<Range<usize>>,
    /// Um nome do código à espera de ser perguntado (o byte dele).
    pub want: Option<usize>,
    pub ticket: Option<Ticket>,
    /// O Shift+F12 espera que as ocorrências assentem ([`CodeDoc::refs_try_open`]).
    pub list_wanted: bool,
}

impl Occurrences {
    pub(crate) fn edited(&mut self) {
        *self = Self::default();
    }

    fn clear(&mut self) {
        self.ranges.clear();
        self.want = None;
        self.ticket = None;
    }
}

impl CodeDoc {
    /// Um quadro: o cursor mudou de sítio (ou o texto, ou a árvore) ⇒ as ocorrências do nome nele.
    pub(crate) fn occurrences_tick(&mut self) {
        let sel = self.buffer.selection().primary();
        let key = (self.buffer.revision(), sel.head, self.syntax.generation());
        let o = &mut self.occ;
        if o.at == Some(key) {
            return;
        }
        o.at = Some(key);
        if !sel.is_empty() {
            o.clear();
            return;
        }
        let t = self.buffer.text();
        match self.syntax.name_at(t, sel.head, &hook_names()) {
            Some(NameAt::SelfField(name, _)) => {
                o.want = None;
                o.ticket = None;
                o.ranges = field_sites(&self.syntax, t, &name)
                    .map(|(s, _)| s)
                    .unwrap_or_default();
            }
            Some(NameAt::Name(r) | NameAt::Hook(r)) => {
                // Dentro de um dos sítios já realçados o realce fica até à resposta (sem piscar).
                if !o
                    .ranges
                    .iter()
                    .any(|x| x.start <= sel.head && sel.head <= x.end)
                {
                    o.ranges.clear();
                }
                o.want = Some(r.start);
                o.ticket = None;
            }
            Some(NameAt::Member(_)) | None => o.clear(),
        }
        self.refs_try_open();
    }

    /// A resposta às referências. Uma edição largou o bilhete ([`Occurrences::edited`]): a resposta
    /// que chega é sempre da revisão de agora.
    pub(crate) fn occurrences_answer(&mut self, answer: Answer) {
        let o = &mut self.occ;
        o.ticket = None;
        match answer {
            Answer::References(r) => o.ranges = r,
            _ => o.ranges.clear(),
        }
        self.refs_try_open();
    }
}

/// O Shift+F12 (I10): a lista dos sítios do nome sob o cursor, na janelinha da completação — cada
/// linha é «<linha>: <o código dela>» com o nome marcado; Enter, Tab ou clique levam o cursor lá.
pub(crate) struct RefsList {
    pub anchor: usize,
    pub sites: Vec<Range<usize>>,
    pub selected: usize,
    pub top: usize,
}

impl CodeDoc {
    /// Shift+F12: a lista abre quando as ocorrências do cursor estão assentes (as do servidor chegam
    /// depois da tecla).
    pub(crate) fn open_refs(&mut self) {
        self.complete.close();
        self.actions = None;
        self.refs = None;
        self.occ.list_wanted = true;
        self.refs_try_open();
    }

    pub(crate) fn refs_try_open(&mut self) {
        let sel = self.buffer.selection().primary();
        let key = (self.buffer.revision(), sel.head, self.syntax.generation());
        let o = &mut self.occ;
        if !o.list_wanted || o.at != Some(key) || o.want.is_some() || o.ticket.is_some() {
            return;
        }
        o.list_wanted = false;
        if o.ranges.is_empty() {
            return;
        }
        let selected = o
            .ranges
            .iter()
            .position(|r| r.start <= sel.head && sel.head <= r.end)
            .unwrap_or(0);
        let rows = ph2d_panel_code::COMPLETION_ROWS;
        self.refs = Some(RefsList {
            anchor: o.ranges[selected].start,
            sites: o.ranges.clone(),
            selected,
            top: selected.saturating_sub(rows - 1),
        });
    }

    pub(crate) fn refs_shown(&self) -> Option<crate::complete_doc::Shown> {
        let r = self.refs.as_ref()?;
        let rows = self.refs_rows(r);
        let width_chars = ph2d_panel_code::CompletionView::width_of(
            rows.iter().map(|x| (x.label.as_str(), None)),
        );
        Some(crate::complete_doc::Shown {
            anchor: r.anchor,
            total: rows.len(),
            rows: rows
                .into_iter()
                .skip(r.top)
                .take(ph2d_panel_code::COMPLETION_ROWS)
                .collect(),
            top: r.top,
            selected: r.selected,
            width_chars,
            doc: None,
            preview: Vec::new(),
            kind: ph2d_panel_code::ListKind::References,
        })
    }

    /// Todas as linhas da lista.
    pub(crate) fn refs_rows(&self, r: &RefsList) -> Vec<ph2d_panel_code::CompletionRow> {
        let t = self.buffer.text();
        r.sites
            .iter()
            .map(|s| {
                let line = t.line_of_byte(s.start);
                let ls = t.byte_of_line(line);
                let raw = t.slice(ls..t.line_end(line));
                let lead = raw.len() - raw.trim_start().len();
                let code = raw.trim();
                let label = ph2d_i18n::tr("code.refs.row")
                    .replace("{line}", &(line + 1).to_string())
                    .replace("{code}", code);
                let at = label.len() - code.len() + (s.start - ls).saturating_sub(lead);
                ph2d_panel_code::CompletionRow {
                    matched: vec![at..(at + s.len()).min(label.len())],
                    ink: ph2d_panel_code::Ink::Plain,
                    detail: None,
                    label,
                }
            })
            .collect()
    }

    /// Uma tecla com a lista aberta: `true` = era dela.
    pub(crate) fn refs_key(
        &mut self,
        key: ph2d_editor_core::doc_surface::Key,
        mods: ph2d_host::Modifiers,
    ) -> bool {
        use ph2d_editor_core::doc_surface::Key;
        let Some(r) = self.refs.as_mut() else {
            return false;
        };
        if mods.ctrl || mods.meta || mods.alt {
            return false;
        }
        let rows = ph2d_panel_code::COMPLETION_ROWS;
        let len = r.sites.len();
        let step = |r: &mut RefsList, d: isize| {
            (r.selected, r.top) = crate::actions::step_in(r.selected, r.top, d, len, rows);
        };
        match key {
            Key::Up => step(r, -1),
            Key::Down => step(r, 1),
            Key::PageUp => step(r, -(rows as isize)),
            Key::PageDown => step(r, rows as isize),
            Key::Enter | Key::Tab => {
                let i = r.selected;
                self.refs_go(i);
            }
            Key::Escape => self.refs = None,
            _ => return false,
        }
        true
    }

    /// Leva o cursor ao sítio `i` da lista e fecha-a.
    pub(crate) fn refs_go(&mut self, i: usize) {
        let Some(s) = self.refs.take().and_then(|r| r.sites.get(i).cloned()) else {
            return;
        };
        self.buffer
            .set_selection(ph2d_code_buffer::Selection::single(
                ph2d_code_buffer::Range::cursor(s.start),
            ));
        self.buffer.seal_history();
        self.reveal();
    }

    /// A roda sobre a lista.
    pub(crate) fn refs_scroll(&mut self, lines: isize) {
        if let Some(r) = self.refs.as_mut() {
            let rows = ph2d_panel_code::COMPLETION_ROWS;
            let max_top = r.sites.len().saturating_sub(rows) as isize;
            r.top = (r.top as isize + lines).clamp(0, max_top) as usize;
        }
    }
}
