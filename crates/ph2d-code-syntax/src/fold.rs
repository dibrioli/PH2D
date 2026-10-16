//! Dobras: os nós de `folds.scm` com >= 2 linhas, UMA por linha de início (a maior).
//!
//! ⛔ Por tecla só se pergunta por uma FAIXA de linhas ([`Syntax::folds_in`], a vista) ou por uma
//! linha ([`Syntax::fold_at`], uma dobra fechada): o custo é o da faixa, não o do ficheiro
//! (`docs/CodeEditor/02_onda1.md` §Medições (E14)). [`Syntax::folds`] (o ficheiro inteiro) é do
//! «Fold All».

use crate::{Syntax, queries};
use ph2d_code_buffer::Text;
use std::cell::Cell;
use std::ops::Range;
use tree_sitter::{Point, QueryCursor, StreamingIterator};

/// Linhas (base 0) de um bloco dobrável: `start_line` fica visível, o resto até `end_line` esconde.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fold {
    pub start_line: usize,
    pub end_line: usize,
}

thread_local! {
    static CAPTURES: Cell<u64> = const { Cell::new(0) };
}

/// Quantas capturas a query das dobras devolveu nesta thread desde o início: o trabalho das
/// dobras, sem relógio (o gate de custo por tecla conta-o).
#[must_use]
pub fn fold_captures_visited() -> u64 {
    CAPTURES.with(Cell::get)
}

impl Syntax {
    /// Todas as dobras do texto (o «Fold All»; custa o ficheiro inteiro).
    pub fn folds(&self, text: &Text) -> Vec<Fold> {
        self.query_folds(text, None)
    }

    /// As dobras que COMEÇAM nas linhas `lines` (a query só desce pelos nós que tocam a faixa).
    pub fn folds_in(&self, text: &Text, lines: Range<usize>) -> Vec<Fold> {
        if lines.is_empty() {
            return Vec::new();
        }
        self.query_folds(text, Some(lines))
    }

    /// A dobra que começa na linha `line` (a maior), se a há.
    pub fn fold_at(&self, text: &Text, line: usize) -> Option<Fold> {
        self.folds_in(text, line..line + 1).into_iter().next()
    }

    fn query_folds(&self, text: &Text, lines: Option<Range<usize>>) -> Vec<Fold> {
        let q = queries::get();
        let mut cursor = QueryCursor::new();
        if let Some(r) = &lines {
            cursor.set_point_range(
                Point {
                    row: r.start,
                    column: 0,
                }..Point {
                    row: r.end,
                    column: 0,
                },
            );
        }
        let mut caps = cursor.captures(&q.folds, self.tree().root_node(), queries::source(text));
        let mut out: Vec<Fold> = Vec::new();
        let mut visited = 0u64;
        while let Some((m, i)) = caps.next() {
            visited += 1;
            let n = m.captures()[*i].node;
            let start_line = n.start_position().row;
            if lines.as_ref().is_some_and(|r| !r.contains(&start_line)) {
                continue;
            }
            let end = n.end_position();
            // Um nó que acaba na coluna 0 terminou na quebra da linha anterior.
            let end_line = if end.column == 0 && end.row > start_line {
                end.row - 1
            } else {
                end.row
            };
            if end_line > start_line {
                out.push(Fold {
                    start_line,
                    end_line,
                });
            }
        }
        CAPTURES.with(|c| c.set(c.get() + visited));
        out.sort_by(|a, b| {
            a.start_line
                .cmp(&b.start_line)
                .then(b.end_line.cmp(&a.end_line))
        });
        out.dedup_by_key(|f| f.start_line);
        out
    }
}
