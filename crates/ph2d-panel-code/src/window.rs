//! ⭐ **A janela pintada de uma linha longa** — uma linha maior que o tecto do quadro
//! ([`crate::view::LINE_RENDER_BYTES_MAX`]) mostra ≤ tecto bytes À VOLTA da faixa horizontal à
//! vista, nunca os primeiros: o cursor no fim de uma linha colada de 64 KB vê o fim dela.
//!
//! A janela começa num degrau de colunas (`cap / 8`): rolar dentro do degrau não volta a moldar.
//! ⛔ O tecto é do QUADRO, mas a janela cobre SEMPRE as colunas à vista e o degrau: numa linha densa
//! (CJK 3 B, emoji 4 B por coluna) 2 000 B são 500–666 colunas, e o degrau (250) comia metade — a
//! vista larga ficava com um troço por pintar e o fim não se pintava. O tecto efectivo é então
//! `max(cap, bytes de degrau + vista)`, o custo de pintar o que está à vista (que não se corta).
//! O x do início da janela é a coluna de ecrã dele vezes o avanço da mono — a mesma conta do
//! cursor fora da cache (`CodeView::x_in_line`), sem moldar o prefixo.

use ph2d_code_buffer::Text;
use unicode_segmentation::UnicodeSegmentation;

/// O troço de uma linha que a pintura mostra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Window {
    /// Bytes do buffer do início e do fim do troço.
    pub start: usize,
    pub end: usize,
    /// A coluna de ecrã (da linha) onde o troço começa.
    pub col: usize,
}

/// As janelas já resolvidas (o passeio da linha até à coluna custa a linha inteira).
#[derive(Default)]
pub(crate) struct WindowMemo {
    rev: u64,
    seen: Vec<((usize, usize, usize, usize, usize), Window)>,
}

/// Quantas janelas a memória guarda (as filas cortadas à vista de um quadro cabem folgadas).
const MEMO_MAX: usize = 256;

impl WindowMemo {
    /// A janela da linha `line` (`ls..le`) para a vista `(left, cols)`: a coluna esquerda e quantas
    /// colunas se vêem. Uma linha que cabe no tecto mostra-se inteira.
    pub fn window(
        &mut self,
        t: &Text,
        rev: u64,
        line: usize,
        (left, cols): (usize, usize),
        cap: usize,
        tab: usize,
    ) -> Window {
        let (ls, le) = (t.byte_of_line(line), t.line_end(line));
        if le - ls <= cap {
            return Window {
                start: ls,
                end: le,
                col: 0,
            };
        }
        let step = (cap / 8).max(1);
        let key = (line, left / step * step, cols, cap, tab);
        if self.rev != rev {
            self.rev = rev;
            self.seen.clear();
        }
        if let Some(w) = self.seen.iter().find(|s| s.0 == key).map(|s| s.1) {
            return w;
        }
        let w = resolve(t, ls, le, key.1, step + cols, cap, tab);
        if self.seen.len() >= MEMO_MAX {
            self.seen.clear();
        }
        self.seen.push((key, w));
        w
    }
}

fn resolve(
    t: &Text,
    ls: usize,
    le: usize,
    col0: usize,
    span: usize,
    cap: usize,
    tab: usize,
) -> Window {
    let line = t.line_of_byte(ls);
    let first = t.byte_at_display_col(line, col0, tab);
    // O tecto cobre o degrau e a vista inteira (a vista no fim leva-o até ao fim da linha).
    let cap = cap.max(t.byte_at_display_col(line, col0 + span, tab) - first);
    // Perto do fim a janela recua para levar o tecto inteiro, a acabar no fim da linha.
    let start = first.min(grapheme_at_or_after(
        t,
        ls,
        le,
        le.saturating_sub(cap).max(ls),
    ));
    let end = if le - start > cap {
        grapheme_at_or_before(t, start, start + cap)
    } else {
        le
    };
    Window {
        start,
        end,
        col: t.display_col(start, tab),
    }
}

/// Quantos bytes de contexto se olham à volta de um corte (um grafema maior fica partido no «…»).
const GRAPHEME_CONTEXT: usize = 64;

/// A fronteira de grafema em `b` ou depois dele (na linha `ls..le`).
fn grapheme_at_or_after(t: &Text, ls: usize, le: usize, b: usize) -> usize {
    let b = t.ceil_char(b);
    let a = t.floor_char(b.saturating_sub(GRAPHEME_CONTEXT).max(ls));
    let s = t.slice(a..t.floor_char((b + GRAPHEME_CONTEXT).min(le)));
    s.grapheme_indices(true)
        .map(|(i, _)| a + i)
        .chain(std::iter::once(a + s.len()))
        .find(|&i| i >= b)
        .unwrap_or(b)
}

/// A fronteira de grafema em `b` ou antes dele, depois de `from`.
fn grapheme_at_or_before(t: &Text, from: usize, b: usize) -> usize {
    let b = t.floor_char(b);
    let a = t.floor_char(b.saturating_sub(GRAPHEME_CONTEXT).max(from));
    let s = t.slice(a..t.ceil_char(b + GRAPHEME_CONTEXT));
    s.grapheme_indices(true)
        .map(|(i, _)| a + i)
        .take_while(|&i| i <= b)
        .last()
        .filter(|&i| i > from)
        .unwrap_or(b)
}
