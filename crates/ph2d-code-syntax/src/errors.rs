//! Erros de sintaxe: nós ERROR e MISSING, na posição do símbolo culpado (o E7 sublinha-o).

use crate::Syntax;
use std::ops::Range;
use tree_sitter::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    /// Bytes do símbolo culpado; vazia (`a..a`) para um símbolo EM FALTA, onde ele devia estar.
    pub range: Range<usize>,
    /// O símbolo que falta (`"end"`, `")"`…) quando o erro é de ausência.
    pub missing: Option<String>,
}

/// O culpado de um ERROR: o seu primeiro TOKEN (o resto é o que o parser engoliu a seguir).
fn culprit(n: Node) -> Range<usize> {
    let mut leaf = n;
    while let Some(c) = leaf.child(0) {
        leaf = c;
    }
    if leaf.byte_range().is_empty() {
        n.byte_range()
    } else {
        leaf.byte_range()
    }
}

impl Syntax {
    pub fn errors(&self) -> Vec<SyntaxError> {
        let mut out = Vec::new();
        let mut stack = vec![self.tree().root_node()];
        while let Some(n) = stack.pop() {
            if n.is_missing() {
                out.push(SyntaxError {
                    range: n.start_byte()..n.start_byte(),
                    missing: Some(n.kind().to_string()),
                });
                continue;
            }
            if n.is_error() {
                out.push(SyntaxError {
                    range: culprit(n),
                    missing: None,
                });
            }
            if n.has_error() {
                let mut c = n.walk();
                stack.extend(
                    n.children(&mut c)
                        .filter(|k| k.has_error() || k.is_missing())
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev(),
                );
            }
        }
        out.sort_by_key(|e| (e.range.start, e.range.end));
        out
    }
}
