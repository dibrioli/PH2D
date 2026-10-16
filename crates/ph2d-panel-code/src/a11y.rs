//! ⭐ **E12 — a árvore de acessibilidade por LINHA**: um nó `Document` com a seleção primária e um
//! `TextRun` por linha VISÍVEL (no máximo [`RUN_CHARS`] caracteres). Os ids derivam da linha, por
//! isso rolar troca os nós e uma linha que fica à vista mantém o seu. A porta pública
//! (`CodeView::a11y_nodes`) mora com a pintura, que é dona das filas; aqui ficam as contas.

use crate::view::CodeView;
use accesskit::{TextPosition, TextSelection};
use ph2d_a11y::NodeId;
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

/// A posição de `byte` nos nós visíveis; fora da vista prende-se à primeira/última linha.
fn position(buffer: &Buffer, lines: &[usize], base: NodeId, byte: usize) -> Option<TextPosition> {
    let t = buffer.text();
    let (&first, &last) = (lines.first()?, lines.last()?);
    let line = t.line_of_byte(byte);
    let (line, index) = if line < first {
        (first, 0)
    } else if line > last || !lines.contains(&line) {
        let l = if line > last { last } else { *lines.iter().rev().find(|&&l| l < line).unwrap_or(&first) };
        (l, units(&t.line_string(l)).take(RUN_CHARS).count())
    } else {
        let start = t.byte_of_line(line);
        (line, units(&t.slice(start..byte)).take(RUN_CHARS).count())
    };
    Some(TextPosition { node: CodeView::a11y_run_id(base, line).into(), character_index: index })
}

pub(crate) fn selection(buffer: &Buffer, lines: &[usize], base: NodeId) -> Option<TextSelection> {
    let p = buffer.selection().primary();
    Some(TextSelection { anchor: position(buffer, lines, base, p.anchor)?, focus: position(buffer, lines, base, p.head)? })
}
