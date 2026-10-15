//! Dobras: os nós de `folds.scm` com >= 2 linhas, UMA por linha de início (a maior).

use crate::{Syntax, queries};
use ph2d_code_buffer::Text;
use tree_sitter::{QueryCursor, StreamingIterator};

/// Linhas (base 0) de um bloco dobrável: `start_line` fica visível, o resto até `end_line` esconde.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fold {
    pub start_line: usize,
    pub end_line: usize,
}

impl Syntax {
    pub fn folds(&self, text: &Text) -> Vec<Fold> {
        let q = queries::get();
        let mut cursor = QueryCursor::new();
        let mut caps = cursor.captures(&q.folds, self.tree().root_node(), queries::source(text));
        let mut out: Vec<Fold> = Vec::new();
        while let Some((m, i)) = caps.next() {
            let n = m.captures()[*i].node;
            let start_line = n.start_position().row;
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
        out.sort_by(|a, b| {
            a.start_line
                .cmp(&b.start_line)
                .then(b.end_line.cmp(&a.end_line))
        });
        out.dedup_by_key(|f| f.start_line);
        out
    }
}
