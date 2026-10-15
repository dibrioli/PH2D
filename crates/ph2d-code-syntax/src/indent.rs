//! Indentação pela query `indents.scm` (as capturas `@indent.*` são lidas AQUI, ver o cabeçalho).
//!
//! Lei: o nível de uma posição é o número de LINHAS distintas onde abre um bloco que a contém e
//! ainda não fechou antes dela. Bloco = nó `@indent.begin` antepassado da posição, ou — no texto
//! meio escrito, que o tree-sitter devolve como ERROR com tokens soltos — token `@indent.open` de
//! um ERROR sem o seu `@indent.end`. Uma linha que COMEÇA por `@indent.end` (`end`, `}`…) ou
//! `@indent.branch` (`else`, `elseif`) recua um nível.

use crate::Syntax;
use crate::queries::{self, source};
use ph2d_code_buffer::Text;
use std::collections::BTreeSet;
use tree_sitter::{Node, QueryCursor, StreamingIterator};

/// O nó tem a captura `cap` num padrão enraizado NELE (não num descendente).
fn has_capture(text: &Text, node: Node, cap: u32) -> bool {
    let q = queries::get();
    let mut cursor = QueryCursor::new();
    cursor.set_max_start_depth(Some(0));
    let mut caps = cursor.captures(&q.indents, node, source(text));
    while let Some((m, i)) = caps.next() {
        let c = m.captures()[*i];
        if c.index == cap && c.node.id() == node.id() {
            return true;
        }
    }
    false
}

/// O nó fecha-se com um token real (não MISSING).
fn really_closed(n: Node) -> bool {
    let mut leaf = n;
    while let Some(c) = leaf.child(leaf.child_count().saturating_sub(1)) {
        leaf = c;
    }
    !leaf.is_missing()
}

/// Linhas dos `@indent.open` de um ERROR que ficam sem par antes de `byte`.
fn unclosed_in_error(text: &Text, err: Node, byte: usize, rows: &mut Vec<usize>) {
    let q = queries::get();
    let mut cursor = QueryCursor::new();
    cursor
        .set_byte_range(err.start_byte()..byte)
        .set_max_start_depth(Some(1));
    let mut caps = cursor.captures(&q.indents, err, source(text));
    let mut stack: Vec<usize> = Vec::new();
    let mut last: Option<(usize, u32)> = None;
    while let Some((m, i)) = caps.next() {
        let c = m.captures()[*i];
        let n = c.node;
        if n.parent().map(|p| p.id()) != Some(err.id()) || n.start_byte() >= byte || n.is_missing()
        {
            continue;
        }
        if last == Some((n.id(), c.index)) {
            continue;
        }
        last = Some((n.id(), c.index));
        if c.index == q.indent_end || c.index == q.indent_branch {
            stack.pop();
        } else if c.index == q.indent_open {
            stack.push(n.start_position().row);
        }
    }
    rows.extend(stack);
}

impl Syntax {
    /// Quantas linhas distintas abrem um bloco que contém `byte`, vistas da linha `row`.
    fn open_rows(&self, text: &Text, byte: usize, row: usize) -> usize {
        let q = queries::get();
        let root = self.tree().root_node();
        let mut rows: Vec<usize> = Vec::new();
        let mut node = root.descendant_for_byte_range(byte, byte);
        while let Some(a) = node {
            let open = a.start_byte() < byte && (byte < a.end_byte() || !really_closed(a));
            if open && !a.is_error() && has_capture(text, a, q.indent_begin) {
                rows.push(a.start_position().row);
            }
            if a.is_error() && a.start_byte() < byte {
                unclosed_in_error(text, a, byte, &mut rows);
            }
            // ERRORs irmãos que acabaram antes de `byte`: os seus blocos abertos continuam.
            let mut c = a.walk();
            for k in a
                .children(&mut c)
                .filter(|k| k.is_error() && k.end_byte() <= byte)
            {
                unclosed_in_error(text, k, byte, &mut rows);
            }
            node = a.parent();
        }
        rows.into_iter()
            .filter(|r| *r < row)
            .collect::<BTreeSet<_>>()
            .len()
    }

    /// O token que começa em `byte` fecha ou ramifica um bloco (a linha que ele abre recua).
    pub fn dedents_at(&self, text: &Text, byte: usize) -> bool {
        let q = queries::get();
        let Some(leaf) = self
            .tree()
            .root_node()
            .descendant_for_byte_range(byte, byte.saturating_add(1))
        else {
            return false;
        };
        leaf.start_byte() == byte
            && !leaf.is_missing()
            && (has_capture(text, leaf, q.indent_end) || has_capture(text, leaf, q.indent_branch))
    }

    /// Quantos níveis o Enter em `byte` põe na linha NOVA (vazia): o corpo do bloco em curso.
    /// Depois de `function f()`, `then`, `do`, `{`, `repeat` → +1; dentro de um bloco mantém.
    /// Se o texto que desce para a linha nova começa por `end`/`}`…, chame depois
    /// [`Syntax::line_indent_level`] nessa linha.
    pub fn indent_level_after(&self, text: &Text, byte: usize) -> usize {
        let byte = byte.min(text.len());
        self.open_rows(text, byte, text.line_of_byte(byte) + 1)
    }

    /// O nível CERTO da linha `line` tal como está — o editor chama-o depois de o utilizador
    /// escrever `end`, `else`, `until`, `}` no início de uma linha, e re-indenta se diferir.
    pub fn line_indent_level(&self, text: &Text, line: usize) -> usize {
        let start = text.byte_of_line(line);
        let s = text.line_string(line);
        let first = start + (s.len() - s.trim_start_matches([' ', '\t']).len());
        let level = self.open_rows(text, first, line);
        if first < text.line_end(line) && self.dedents_at(text, first) {
            level.saturating_sub(1)
        } else {
            level
        }
    }
}
