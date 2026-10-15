//! Realce por faixa: só os bytes pedidos, spans ordenados e SEM sobreposição.
//!
//! Lei da sobreposição (a dos editores): a captura de faixa MENOR vence; na mesma faixa, a do
//! padrão que vem DEPOIS na query vence (ex.: `(identifier) @variable` cede a `@function.call`).

use crate::{Syntax, queries};
use ph2d_code_buffer::Text;
use std::ops::Range;
use tree_sitter::{QueryCursor, StreamingIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightKind {
    Keyword,
    Function,
    FunctionCall,
    Variable,
    Parameter,
    Property,
    Type,
    String,
    Number,
    Boolean,
    Constant,
    Builtin,
    Comment,
    Operator,
    Punctuation,
    Attribute,
}

impl HighlightKind {
    pub const ALL: [HighlightKind; 16] = [
        Self::Keyword,
        Self::Function,
        Self::FunctionCall,
        Self::Variable,
        Self::Parameter,
        Self::Property,
        Self::Type,
        Self::String,
        Self::Number,
        Self::Boolean,
        Self::Constant,
        Self::Builtin,
        Self::Comment,
        Self::Operator,
        Self::Punctuation,
        Self::Attribute,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    pub range: Range<usize>,
    pub kind: HighlightKind,
}

struct Hit {
    range: Range<usize>,
    pattern: usize,
    kind: HighlightKind,
}

impl Syntax {
    pub fn highlights(&self, text: &Text, bytes: Range<usize>) -> Vec<HighlightSpan> {
        let end = bytes.end.min(text.len());
        let start = bytes.start.min(end);
        if start == end {
            return Vec::new();
        }
        let q = queries::get();
        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(start..end);
        let mut matches = cursor.matches(
            &q.highlights,
            self.tree().root_node(),
            queries::source(text),
        );
        let mut hits = Vec::new();
        while let Some(m) = matches.next() {
            for c in m.captures() {
                let Some(kind) = q.paint.get(c.index as usize).copied().flatten() else {
                    continue;
                };
                let r = c.node.byte_range();
                let (a, b) = (r.start.max(start), r.end.min(end));
                if a < b {
                    hits.push(Hit {
                        range: r,
                        pattern: m.pattern_index,
                        kind,
                    });
                }
            }
        }
        // Pinta do maior para o menor, e na mesma faixa pela ordem da query: quem pinta por
        // último vence.
        hits.sort_by(|x, y| {
            y.range
                .len()
                .cmp(&x.range.len())
                .then(x.pattern.cmp(&y.pattern))
        });
        let mut paint: Vec<Option<HighlightKind>> = vec![None; end - start];
        for h in &hits {
            for slot in &mut paint[h.range.start.max(start) - start..h.range.end.min(end) - start] {
                *slot = Some(h.kind);
            }
        }
        let mut out: Vec<HighlightSpan> = Vec::new();
        for (i, k) in paint.into_iter().enumerate() {
            let Some(kind) = k else { continue };
            let b = start + i;
            match out.last_mut() {
                Some(last) if last.kind == kind && last.range.end == b => last.range.end = b + 1,
                _ => out.push(HighlightSpan {
                    range: b..b + 1,
                    kind,
                }),
            }
        }
        out
    }
}
