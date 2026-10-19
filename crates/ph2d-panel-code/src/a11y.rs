//! ⭐ **E12 — a árvore de acessibilidade por LINHA**: um nó `Document` com a seleção primária e um
//! `TextRun` por linha VISÍVEL (no máximo [`RUN_CHARS`] caracteres). Os ids derivam da linha, por
//! isso rolar troca os nós e uma linha que fica à vista mantém o seu. A porta pública
//! ([`CodeView::a11y_nodes`]) lê as filas do último quadro (a pintura é dona delas) e junta os nós
//! dos balões, da lista de completação, da caixinha do F2 e do contador.

use crate::view::CodeView;
use accesskit::{TextPosition, TextSelection};
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_code_buffer::Buffer;
use unicode_segmentation::UnicodeSegmentation;

/// O tecto de caracteres de um `TextRun` (o resto da linha não entra no nó).
pub const RUN_CHARS: usize = 255;

/// As unidades de um `TextRun`: os grafemas, e um grafema com mais de 255 bytes (o `u8` do
/// AccessKit) partido por fronteira de carácter em pedaços que cabem. O texto e a posição do cursor
/// contam ESTAS unidades, por isso a soma dos comprimentos é sempre o comprimento do texto.
fn units(s: &str) -> impl Iterator<Item = &str> {
    s.graphemes(true).flat_map(|g| {
        let mut rest = g;
        std::iter::from_fn(move || {
            if rest.is_empty() {
                return None;
            }
            let mut cut = rest.len().min(usize::from(u8::MAX));
            while !rest.is_char_boundary(cut) {
                cut -= 1;
            }
            let (head, tail) = rest.split_at(cut);
            rest = tail;
            Some(head)
        })
    })
}

/// O texto do nó (até [`RUN_CHARS`] unidades) e o comprimento em bytes de cada uma.
pub(crate) fn clipped(line: &str) -> (String, Vec<u8>) {
    let mut value = String::new();
    let mut lens = Vec::new();
    for u in units(line).take(RUN_CHARS) {
        value.push_str(u);
        lens.push(u8::try_from(u.len()).unwrap_or(u8::MAX));
    }
    (value, lens)
}

/// O começo da linha `line` que basta para [`RUN_CHARS`] unidades — nunca a linha inteira (uma
/// linha de 1 MB custaria 1 MB por quadro).
pub(crate) fn run_text(t: &ph2d_code_buffer::Text, line: usize) -> String {
    let (ls, le) = (t.byte_of_line(line), t.line_end(line));
    let mut take = RUN_CHARS * 4;
    loop {
        let end = if le - ls > take {
            t.floor_char(ls + take)
        } else {
            le
        };
        let s = t.slice(ls..end);
        if end == le || units(&s).take(RUN_CHARS).count() >= RUN_CHARS {
            return s;
        }
        take *= 2;
    }
}

/// A posição de `byte` nos nós visíveis; fora da vista prende-se à primeira/última linha.
fn position(buffer: &Buffer, lines: &[usize], base: NodeId, byte: usize) -> Option<TextPosition> {
    let t = buffer.text();
    let (&first, &last) = (lines.first()?, lines.last()?);
    let line = t.line_of_byte(byte);
    let (line, index) = if line < first {
        (first, 0)
    } else if line > last || !lines.contains(&line) {
        let l = if line > last {
            last
        } else {
            *lines.iter().rev().find(|&&l| l < line).unwrap_or(&first)
        };
        (l, units(&run_text(t, l)).take(RUN_CHARS).count())
    } else {
        let start = t.byte_of_line(line);
        let run = run_text(t, line);
        let upto = (byte - start).min(run.len());
        (line, units(&run[..upto]).take(RUN_CHARS).count())
    };
    Some(TextPosition {
        node: CodeView::a11y_run_id(base, line).into(),
        character_index: index,
    })
}

pub(crate) fn selection(buffer: &Buffer, lines: &[usize], base: NodeId) -> Option<TextSelection> {
    let p = buffer.selection().primary();
    Some(TextSelection {
        anchor: position(buffer, lines, base, p.anchor)?,
        focus: position(buffer, lines, base, p.head)?,
    })
}

impl CodeView {
    /// O id do `TextRun` da linha `line`, com `base` o id do `Document`.
    #[must_use]
    pub fn a11y_run_id(base: NodeId, line: usize) -> NodeId {
        NodeId(base.0 + 1 + line as u64)
    }

    /// Os nós do último quadro: o `Document` primeiro (filhos = as linhas visíveis), depois um
    /// `TextRun` por linha. A família reserva os ids `base ..= base + nº de linhas`.
    #[must_use]
    pub fn a11y_nodes(&self, buffer: &Buffer, base: NodeId) -> Vec<(NodeId, Node)> {
        let t = buffer.text();
        let tr = self.text_rect();
        let lines: Vec<usize> = self
            .rows
            .iter()
            .map(|r| r.line)
            .filter(|&l| l < t.line_count())
            .collect();
        let mut out = Vec::with_capacity(lines.len() + 1);
        let mut runs = Vec::with_capacity(lines.len());
        for r in self.rows.iter().filter(|r| r.line < t.line_count()) {
            let (value, lens) = clipped(&run_text(t, r.line));
            let id = Self::a11y_run_id(base, r.line);
            let mut node = NodeBuilder::new(Role::TextRun)
                .bounds(
                    f64::from(tr.x),
                    f64::from(r.y),
                    f64::from(tr.w),
                    f64::from(self.m.line_h),
                )
                .build();
            node.set_value(value);
            node.set_character_lengths(lens);
            runs.push((id, node));
        }
        let mut doc = NodeBuilder::new(Role::Document)
            .label(ph2d_i18n::tr("code.a11y.document"))
            .bounds(
                f64::from(tr.x),
                f64::from(tr.y),
                f64::from(tr.w),
                f64::from(tr.h),
            )
            .children(lines.iter().map(|&l| Self::a11y_run_id(base, l)))
            .build();
        if let Some(sel) = selection(buffer, &lines, base) {
            doc.set_text_selection(sel);
        }
        let list = self.completion_a11y(base);
        if let Some((id, active, _)) = &list {
            doc.push_child((*id).into());
            if let Some(a) = active {
                doc.set_active_descendant((*a).into());
            }
        }
        let (roots, hints) = self.hints_a11y(base);
        for id in roots {
            doc.push_child(id.into());
        }
        let rename = self.rename_a11y(base);
        let counter = self.counter_a11y(base);
        for (id, _) in rename.iter().chain(&counter) {
            doc.push_child((*id).into());
        }
        out.push((base, doc));
        out.extend(runs);
        out.extend(list.into_iter().flat_map(|l| l.2));
        out.extend(hints);
        out.extend(rename);
        out.extend(counter);
        out
    }
}
