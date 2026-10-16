//! O texto do buffer: uma `crop::Rope` com fins de linha `\n` e as métricas do editor.
//!
//! Lei das linhas: um texto com `k` quebras tem `k + 1` linhas — a linha vazia depois do último
//! `\n` EXISTE (o cursor vive lá). O `crop` não a conta (`line_len`), por isso nada aqui o usa.

use crop::Rope;
use unicode_segmentation::UnicodeSegmentation;

/// Posição em (linha, coluna em BYTES dentro da linha) — a unidade do tree-sitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Point {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Text {
    rope: Rope,
}

/// `\r\n` e `\r` soltos viram `\n`: o buffer só conhece um fim de linha.
pub fn normalize_newlines(s: &str) -> std::borrow::Cow<'_, str> {
    if s.contains('\r') {
        std::borrow::Cow::Owned(s.replace("\r\n", "\n").replace('\r', "\n"))
    } else {
        std::borrow::Cow::Borrowed(s)
    }
}

impl Text {
    pub fn new(s: &str) -> Self {
        Self { rope: Rope::from(normalize_newlines(s).as_ref()) }
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub(crate) fn replace(&mut self, range: std::ops::Range<usize>, text: &str) {
        if range.is_empty() {
            if !text.is_empty() {
                self.rope.insert(range.start, text);
            }
        } else {
            self.rope.replace(range, text);
        }
    }

    pub fn len(&self) -> usize {
        self.rope.byte_len()
    }

    pub fn is_empty(&self) -> bool {
        self.rope.is_empty()
    }

    pub fn line_count(&self) -> usize {
        if self.is_empty() { 1 } else { self.rope.line_len() + usize::from(self.ends_with_newline()) }
    }

    fn ends_with_newline(&self) -> bool {
        let n = self.len();
        n > 0 && self.rope.byte(n - 1) == b'\n'
    }

    pub fn line_of_byte(&self, byte: usize) -> usize {
        self.rope.line_of_byte(byte.min(self.len()))
    }

    /// Byte onde a linha começa; `line == line_count()` dá o fim do texto.
    pub fn byte_of_line(&self, line: usize) -> usize {
        if line == 0 {
            return 0;
        }
        let breaks = self.line_count() - 1;
        if line > breaks {
            return self.len();
        }
        self.rope.byte_of_line(line)
    }

    /// Byte do fim da linha, ANTES do `\n`.
    pub fn line_end(&self, line: usize) -> usize {
        let next = self.byte_of_line(line + 1);
        if line + 1 < self.line_count() { next - 1 } else { next }
    }

    pub fn line_string(&self, line: usize) -> String {
        self.slice(self.byte_of_line(line)..self.line_end(line))
    }

    /// ⛔ **Total: nunca entra em pânico.** As pontas descem à fronteira de carácter anterior e
    /// ficam dentro do texto (o `crop` entra em pânico num byte do meio de um carácter — o app caía
    /// ao escrever `(` depois de `-- é`, auditoria do fecho da Onda 1).
    pub fn slice(&self, range: std::ops::Range<usize>) -> String {
        // Nos testes um chamador com uma ponta cortada fica vermelho; no app, o texto segue.
        debug_assert!(
            self.floor_char(range.start) == range.start && self.floor_char(range.end) == range.end,
            "slice({range:?}) fora de uma fronteira de carácter"
        );
        let start = self.floor_char(range.start);
        let end = self.floor_char(range.end).max(start);
        self.rope.byte_slice(start..end).to_string()
    }

    /// A fronteira de carácter em `byte` ou antes dele (dentro do texto).
    pub fn floor_char(&self, byte: usize) -> usize {
        let mut b = byte.min(self.len());
        while !self.rope.is_char_boundary(b) {
            b -= 1;
        }
        b
    }

    /// A fronteira de carácter em `byte` ou depois dele (dentro do texto).
    pub fn ceil_char(&self, byte: usize) -> usize {
        let mut b = byte.min(self.len());
        while !self.rope.is_char_boundary(b) {
            b += 1;
        }
        b
    }

    /// O texto tem `pat` a começar em `at`? Compara BYTES (nunca corta um carácter).
    pub fn starts_with_at(&self, at: usize, pat: &str) -> bool {
        at.checked_add(pat.len()).is_some_and(|end| end <= self.len())
            && pat.bytes().enumerate().all(|(i, b)| self.rope.byte(at + i) == b)
    }

    pub fn point_of_byte(&self, byte: usize) -> Point {
        let byte = self.floor_char(byte);
        let row = self.line_of_byte(byte);
        Point { row, col: byte - self.byte_of_line(row) }
    }

    /// O byte de `p`; uma coluna no meio de um carácter desce ao início dele.
    pub fn byte_of_point(&self, p: Point) -> usize {
        let start = self.byte_of_line(p.row.min(self.line_count() - 1));
        let b = start.saturating_add(p.col).min(self.line_end(self.line_of_byte(start)));
        self.floor_char(b)
    }

    pub fn utf16_of_byte(&self, byte: usize) -> usize {
        self.rope.utf16_code_unit_of_byte(self.floor_char(byte))
    }

    /// O byte da unidade UTF-16 `unit` (fora do texto = o fim; no meio de um par substituto = o
    /// início do carácter).
    pub fn byte_of_utf16(&self, unit: usize) -> usize {
        let unit = unit.min(self.rope.utf16_len());
        self.floor_char(self.rope.byte_of_utf16_code_unit(unit))
    }

    /// Byte do grafema anterior (para Backspace e seta esquerda).
    pub fn prev_grapheme(&self, byte: usize) -> usize {
        if byte == 0 {
            return 0;
        }
        let line = self.line_of_byte(byte);
        let start = self.byte_of_line(line);
        if byte == start {
            return byte - 1;
        }
        let s = self.slice(start..byte);
        start + s.grapheme_indices(true).next_back().map_or(0, |(i, _)| i)
    }

    pub fn next_grapheme(&self, byte: usize) -> usize {
        if byte >= self.len() {
            return self.len();
        }
        let line = self.line_of_byte(byte);
        let end = self.line_end(line);
        if byte == end {
            return byte + 1;
        }
        let s = self.slice(byte..end);
        byte + s.graphemes(true).next().map_or(1, str::len)
    }

    /// Coluna de ECRÃ (grafemas, `\t` até à próxima paragem de `tab`).
    pub fn display_col(&self, byte: usize, tab: usize) -> usize {
        let line = self.line_of_byte(byte);
        display_width(&self.slice(self.byte_of_line(line)..byte), tab)
    }

    /// Byte do grafema que COBRE a coluna de ecrã `col` (o fim da linha se ela é mais curta).
    pub fn byte_at_display_col(&self, line: usize, col: usize, tab: usize) -> usize {
        let start = self.byte_of_line(line);
        let s = self.line_string(line);
        let mut w = 0;
        let tab = tab.max(1);
        for (i, g) in s.grapheme_indices(true) {
            let gw = if g == "\t" { tab - w % tab } else { 1 };
            if w + gw > col {
                return start + i;
            }
            w += gw;
        }
        start + s.len()
    }
}

pub fn display_width(s: &str, tab: usize) -> usize {
    let tab = tab.max(1);
    s.graphemes(true).fold(0, |w, g| if g == "\t" { w + tab - w % tab } else { w + 1 })
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.rope.chunks() {
            f.write_str(c)?;
        }
        Ok(())
    }
}
