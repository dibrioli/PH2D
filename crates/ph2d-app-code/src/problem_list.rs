//! ⭐ **A lista de problemas** (acabamento, item 4 de `docs/CodeEditor/03_onda2.md`; oráculo em
//! `docs/CodeEditor/ferramentas/oraculo_acabamento/`): o 2.º separador da faixa de baixo. Lê o MESMO
//! `Arc<[Problem]>` que a calha pinta ([`CodeDoc::problems`]) a cada quadro, e ordena ÍNDICES — a
//! calha exige o `Arc` por linha. A ordem é a do VS Code 1.142: gravidade (erro, aviso, informação),
//! depois a linha, depois o início do culpado.
//!
//! Clicar numa linha (ou o F8 / Shift+F8, pela ordem da lista, dando a volta) selecciona o culpado
//! e revela-o; o teclado fica no texto.

use crate::doc::CodeDoc;
use ph2d_code_buffer::{Range, Selection};
use ph2d_editor_core::doc_surface::{self, KeyOutcome};
use ph2d_host::Modifiers;
use ph2d_panel_code::{Problem, ProblemRow, Severity};
use ph2d_script::ScriptId;
use std::cmp::Reverse;

/// Um problema na ordem da lista (o F8 segue-o entre quadros, mesmo que a lista mude).
pub(crate) type Key = (Reverse<Severity>, usize, Option<usize>, String);

fn sort_key(p: &Problem) -> (Reverse<Severity>, usize, Option<usize>, &str) {
    (
        Reverse(p.severity),
        p.line,
        p.culprit.as_ref().map(|c| c.start),
        p.message.as_str(),
    )
}

fn key(p: &Problem) -> Key {
    let (s, l, c, m) = sort_key(p);
    (s, l, c, m.to_owned())
}

fn key_ref(k: &Key) -> (Reverse<Severity>, usize, Option<usize>, &str) {
    (k.0, k.1, k.2, k.3.as_str())
}

/// Os índices de `ps` pela ordem da lista.
pub(crate) fn order(ps: &[Problem]) -> Vec<usize> {
    let mut v: Vec<usize> = (0..ps.len()).collect();
    v.sort_by(|&a, &b| sort_key(&ps[a]).cmp(&sort_key(&ps[b])));
    v
}

/// O texto de uma linha: o título da explicação (I8) quando a há, senão a mensagem crua (a 1.ª
/// linha dela).
pub(crate) fn text(p: &Problem) -> &str {
    p.explanation
        .as_ref()
        .map_or(p.message.as_str(), |e| e.title.as_str())
        .lines()
        .next()
        .unwrap_or_default()
}

pub(crate) fn row(p: &Problem) -> ProblemRow<'_> {
    ProblemRow {
        severity: p.severity,
        text: text(p),
        line: p.line + 1,
    }
}

impl CodeDoc {
    /// A posição na ordem `order` de `ps` do problema onde o F8 (ou o clique) está.
    pub(crate) fn problem_current(&self, ps: &[Problem], order: &[usize]) -> Option<usize> {
        let cur = key_ref(self.problem_at.as_ref()?);
        order.iter().position(|&i| sort_key(&ps[i]) == cur)
    }

    /// O clique na linha `k` da lista.
    pub(crate) fn problem_pick(&mut self, k: usize) {
        let ps = self.problems();
        if let Some(&i) = order(&ps).get(k) {
            self.problem_go(&ps[i]);
        }
    }

    /// F8 (`forward`) / Shift+F8: o problema seguinte/anterior ao actual pela ordem da lista, dando a
    /// volta; sem actual, o primeiro/último.
    pub(crate) fn problem_step(&mut self, forward: bool) {
        let ps = self.problems();
        let order = order(&ps);
        let Some(last) = order.len().checked_sub(1) else {
            return;
        };
        let k = match self.problem_at.as_ref().map(key_ref) {
            None if forward => 0,
            None => last,
            Some(cur) if forward => order
                .iter()
                .position(|&i| sort_key(&ps[i]) > cur)
                .unwrap_or(0),
            Some(cur) => order
                .iter()
                .rposition(|&i| sort_key(&ps[i]) < cur)
                .unwrap_or(last),
        };
        self.problem_go(&ps[order[k]]);
    }

    /// Selecciona o culpado de `p` (sem culpado, o cursor no início da linha), revela-o e deixa o
    /// teclado no texto.
    fn problem_go(&mut self, p: &Problem) {
        let t = self.buffer.text();
        let sel = match &p.culprit {
            Some(c) => Range::new(c.start.min(t.len()), c.end.min(t.len())),
            None => Range::cursor(t.byte_of_line(p.line.min(t.line_count().saturating_sub(1)))),
        };
        self.buffer.set_selection(Selection::single(sel));
        self.buffer.seal_history();
        self.find.focus = None;
        self.problem_at = Some(key(p));
        self.reveal();
    }
}

impl crate::docs::CodeDocs {
    /// Os problemas que a aba do script `id` mostra (E7).
    #[must_use]
    pub fn problems_of(&mut self, id: ScriptId) -> Vec<Problem> {
        self.doc(id.0)
            .map(|d| d.problems().to_vec())
            .unwrap_or_default()
    }

    /// Ctrl+Shift+M: a lista de problemas, com o teclado em qualquer sítio da aba `id` (o soltar
    /// também é tomado: nenhum atalho da cena o vê). `None` = outra tecla (ou nenhuma aba `id`).
    pub(crate) fn problems_key(
        &mut self,
        id: u64,
        key: doc_surface::Key,
        mods: Modifiers,
        pressed: bool,
    ) -> Option<KeyOutcome> {
        if !crate::keys::problems_chord(key, mods) {
            return None;
        }
        let (_, store) = self.split(id)?;
        if pressed {
            store.toggle_problems();
        }
        Some(KeyOutcome {
            consumed: true,
            copy: None,
        })
    }
}
