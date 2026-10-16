//! As dobras FECHADAS de um documento — estado próprio, que segue o texto pelas edições.
//!
//! ⛔ Por tecla nada aqui pergunta pelo ficheiro inteiro (`docs/CodeEditor/02_onda1.md` §Medições
//! (E14): a query das dobras sobre a árvore toda custava `6.46` ms a 7 500 linhas). As fechadas
//! andam com o texto ([`ClosedFolds::edited`]) e só se re-conferem contra a gramática as que a edição
//! ou a mudança de estrutura tocou ([`ClosedFolds::check`]); a que deixou de existir abre-se.

use crate::doc::map_byte;
use ph2d_code_buffer::{Text, TextEdit};
use ph2d_code_syntax::{Fold, Syntax};
use ph2d_panel_code::FoldMark;
use std::ops::Range;

/// Uma dobra fechada: o byte do início da linha do cabeçalho e o do início da última linha escondida.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Closed {
    start: usize,
    end: usize,
}

#[derive(Default)]
pub(crate) struct ClosedFolds {
    /// Por `start`, sem repetidos.
    list: Vec<Closed>,
    /// Os bytes (do texto de agora) que as edições tocaram desde o último [`ClosedFolds::check`].
    touched: Option<Range<usize>>,
}

impl ClosedFolds {
    /// As edições do `Buffer` (já aplicadas: `t` é o texto depois delas).
    pub fn edited(&mut self, edits: &[TextEdit], t: &Text) {
        for e in edits {
            for c in &mut self.list {
                c.start = map_byte(c.start, e);
                c.end = map_byte(c.end, e);
            }
            let r = match self.touched.take() {
                Some(r) => {
                    map_byte(r.start, e).min(e.start_byte)..map_byte(r.end, e).max(e.new_end_byte)
                }
                None => e.start_byte..e.new_end_byte,
            };
            self.touched = Some(r);
        }
        for c in &mut self.list {
            c.start = t.byte_of_line(t.line_of_byte(c.start));
            c.end = t.byte_of_line(t.line_of_byte(c.end));
        }
        self.list.sort_unstable_by_key(|c| c.start);
        self.list.dedup_by_key(|c| c.start);
    }

    /// Re-confere contra a gramática as fechadas que a edição ou a mudança de estrutura tocou: a que
    /// ainda começa na sua linha acerta o fim; a que deixou de existir abre-se.
    pub fn check(&mut self, syntax: &mut Syntax, t: &Text) {
        let touched = self.touched.take();
        if self.list.is_empty() {
            syntax.disarm_changes();
            return;
        }
        let changed = syntax.changed_since_mark();
        let meets = |r: &Range<usize>, a: usize, b: usize| r.start <= b && a <= r.end;
        self.list.retain_mut(|c| {
            let b = t.line_end(t.line_of_byte(c.end));
            let hit = match &changed {
                None => true,
                Some(v) => touched.iter().chain(v).any(|r| meets(r, c.start, b)),
            };
            if !hit {
                return true;
            }
            match syntax.fold_at(t, t.line_of_byte(c.start)) {
                Some(f) => {
                    c.end = t.byte_of_line(f.end_line);
                    true
                }
                None => false,
            }
        });
    }

    /// Com fechadas, a gramática guarda a árvore para a comparação de estrutura do próximo
    /// [`ClosedFolds::check`]; sem nenhuma, ninguém paga por ela.
    pub fn arm(&self, syntax: &mut Syntax) {
        if self.list.is_empty() {
            syntax.disarm_changes();
        } else {
            syntax.arm_changes();
        }
    }

    /// As fechadas, em linhas (o que esconde linhas na vista).
    pub fn marks(&self, t: &Text) -> Vec<FoldMark> {
        self.list
            .iter()
            .map(|c| FoldMark {
                start: t.line_of_byte(c.start),
                end: t.line_of_byte(c.end),
                folded: true,
            })
            .collect()
    }

    /// A dobra que começa no byte `start` (início de linha) está fechada?
    pub fn is_closed(&self, start: usize) -> bool {
        self.list.binary_search_by_key(&start, |c| c.start).is_ok()
    }

    /// Abre/fecha `f`.
    pub fn toggle(&mut self, f: Fold, t: &Text) {
        let start = t.byte_of_line(f.start_line);
        match self.list.binary_search_by_key(&start, |c| c.start) {
            Ok(i) => {
                self.list.remove(i);
            }
            Err(i) => self.list.insert(
                i,
                Closed {
                    start,
                    end: t.byte_of_line(f.end_line),
                },
            ),
        }
    }

    /// Fecha exactamente `folds` (ordenadas por linha de início, uma por linha).
    pub fn set_all(&mut self, folds: &[Fold], t: &Text) {
        self.list = folds
            .iter()
            .map(|f| Closed {
                start: t.byte_of_line(f.start_line),
                end: t.byte_of_line(f.end_line),
            })
            .collect();
    }

    /// Abre as fechadas que escondem a linha `line`.
    pub fn unfold_line(&mut self, line: usize, t: &Text) {
        self.list.retain(|c| {
            let (a, b) = (t.line_of_byte(c.start), t.line_of_byte(c.end));
            !(a < line && line <= b)
        });
    }
}
