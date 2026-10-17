//! Bytes da rope ↔ posições LSP (linha, unidades UTF-16), e a faixa suja de um lote de edições.
//!
//! Tudo O(log n) pelas métricas do `crop`: o worker converte posições de milhares de diagnósticos ou
//! tokens sem varrer o texto.

use ph2d_code_buffer::{Text, TextEdit};
use ph2d_code_lsp::{Position, Range};

pub fn position_of(text: &Text, byte: usize) -> Position {
    let byte = text.floor_char(byte.min(text.len()));
    let line = text.line_of_byte(byte);
    let start = text.byte_of_line(line);
    let character = text.utf16_of_byte(byte) - text.utf16_of_byte(start);
    Position::new(line as u32, character as u32)
}

/// Uma linha além do fim dá o fim do texto; uma coluna além do fim da linha dá o fim da linha.
pub fn byte_of(text: &Text, pos: Position) -> usize {
    let line = pos.line as usize;
    if line >= text.line_count() {
        return text.len();
    }
    let start = text.byte_of_line(line);
    let end = text.line_end(line);
    let unit = text.utf16_of_byte(start) + pos.character as usize;
    text.byte_of_utf16(unit).min(end)
}

pub fn range_of(text: &Text, bytes: std::ops::Range<usize>) -> Range {
    Range {
        start: position_of(text, bytes.start),
        end: position_of(text, bytes.end),
    }
}

pub fn bytes_of(text: &Text, r: Range) -> std::ops::Range<usize> {
    let a = byte_of(text, r.start);
    a..byte_of(text, r.end).max(a)
}

/// A troca ÚNICA que leva o texto de antes ao de depois de um lote de edições SEQUENCIAIS (as do
/// `Buffer`/`AutoEdit`, cada uma nas coordenadas do texto em que é aplicada): a faixa `old` do texto
/// de antes vira a faixa `new` do texto de depois. Fora dela os dois textos são iguais.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dirty {
    pub old: std::ops::Range<usize>,
    pub new: std::ops::Range<usize>,
}

impl Dirty {
    /// `None` = lote vazio. `old_len` é o comprimento do texto antes do lote.
    pub fn of(edits: &[TextEdit], old_len: usize) -> Option<Self> {
        let first = edits.first()?;
        let (mut a, mut b) = (first.start_byte, first.start_byte);
        let mut len = old_len;
        // Invariante: o texto de agora = o de antes até `a`, e o de antes a partir do mesmo número
        // de bytes do fim (`len - b`); só `a..b` difere.
        for e in edits {
            let delta = e.new_end_byte as isize - e.old_end_byte as isize;
            a = a.min(e.start_byte);
            b = (b.max(e.old_end_byte) as isize + delta) as usize;
            len = (len as isize + delta) as usize;
        }
        let tail = len - b;
        Some(Self {
            old: a..old_len - tail,
            new: a..b,
        })
    }
}
