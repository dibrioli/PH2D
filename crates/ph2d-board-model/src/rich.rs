//! **Texto com estilo por trecho** (W3): negrito, itálico, sublinhado, riscado e cor num pedaço do
//! texto de uma forma ou nota. O texto e os trechos vivem JUNTOS — um índice de byte que não aponta
//! para este texto não chega a existir.
//!
//! Invariante (reposta por [`RichText::normalize`] a cada mudança): os trechos estão por ordem, não
//! se sobrepõem, não são vazios, caem em fronteiras de carácter, nunca têm as marcas de nascença e
//! dois vizinhos com as mesmas marcas são um só.

use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::Rgba;

/// O estilo de um trecho. `Default` = texto simples na tinta da forma.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Marks {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    /// `None` = a tinta da forma (`Style::text_color`).
    pub color: Option<Rgba>,
}

/// Um trecho `[start, end)` (bytes do texto) com as suas marcas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub marks: Marks,
}

impl Span {
    #[must_use]
    pub fn range(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}

/// Uma marca que a barra liga e desliga.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Bold,
    Italic,
    Underline,
    Strike,
}

impl Mark {
    #[must_use]
    pub fn get(self, m: &Marks) -> bool {
        match self {
            Mark::Bold => m.bold,
            Mark::Italic => m.italic,
            Mark::Underline => m.underline,
            Mark::Strike => m.strike,
        }
    }

    pub fn set(self, m: &mut Marks, on: bool) {
        match self {
            Mark::Bold => m.bold = on,
            Mark::Italic => m.italic = on,
            Mark::Underline => m.underline = on,
            Mark::Strike => m.strike = on,
        }
    }
}

/// O texto de uma forma, com os trechos de estilo.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RichText {
    text: String,
    spans: Vec<Span>,
}

impl From<&str> for RichText {
    fn from(s: &str) -> Self {
        Self::plain(s)
    }
}

impl From<String> for RichText {
    fn from(text: String) -> Self {
        Self {
            text,
            spans: Vec::new(),
        }
    }
}

impl RichText {
    /// Texto sem estilo nenhum.
    #[must_use]
    pub fn plain(s: &str) -> Self {
        Self::from(s.to_owned())
    }

    /// `text` com `spans` (fora de ordem, sobrepostos ou a meio de um carácter: arrumam-se — o
    /// último por cima).
    #[must_use]
    pub fn with_spans(text: String, spans: &[Span]) -> Self {
        let mut r = Self::from(text);
        for s in spans {
            let marks = s.marks;
            r.restyle(s.range(), |m| *m = marks);
        }
        r
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// As marcas do carácter que começa em `byte` (as de nascença fora de qualquer trecho).
    #[must_use]
    pub fn marks_at(&self, byte: usize) -> Marks {
        self.spans
            .iter()
            .find(|s| s.range().contains(&byte))
            .map_or_else(Marks::default, |s| s.marks)
    }

    /// As marcas com que se escreve num cursor em `byte`: as do carácter ANTES dele (continua-se o
    /// que se estava a escrever), ou as do primeiro no início do texto.
    #[must_use]
    pub fn typing_marks(&self, byte: usize) -> Marks {
        match self.text[..byte.min(self.text.len())]
            .char_indices()
            .next_back()
        {
            Some((i, _)) => self.marks_at(i),
            None => self.marks_at(0),
        }
    }

    /// Troca `range` por `s`, que fica com `marks`. Os trechos à volta seguem o texto.
    pub fn replace(&mut self, range: Range<usize>, s: &str, marks: Marks) {
        let range = self.clamp(range);
        let (start, removed, added) = (range.start, range.len(), s.len());
        let mut segs = self.segments();
        let mut out: Vec<(Range<usize>, Marks)> = Vec::with_capacity(segs.len() + 2);
        let mut inserted = false;
        for (r, m) in segs.drain(..) {
            // A parte antes do corte fica; a parte depois desloca-se pelo que mudou de tamanho.
            if r.start < start {
                out.push((r.start..r.end.min(start), m));
            }
            if !inserted && r.end >= start {
                out.push((start..start + added, marks));
                inserted = true;
            }
            if r.end > start + removed {
                let a = r.start.max(start + removed);
                out.push((a - removed + added..r.end - removed + added, m));
            }
        }
        if !inserted {
            out.push((start..start + added, marks));
        }
        self.text.replace_range(range, s);
        self.spans = compact(out);
    }

    /// Muda as marcas de cada carácter em `range` com `f`.
    pub fn restyle(&mut self, range: Range<usize>, f: impl Fn(&mut Marks)) {
        let range = self.clamp(range);
        if range.is_empty() {
            return;
        }
        let mut out = Vec::new();
        for (r, m) in self.segments() {
            let (a, b) = (r.start.max(range.start), r.end.min(range.end));
            if a >= b {
                out.push((r, m));
                continue;
            }
            if r.start < a {
                out.push((r.start..a, m));
            }
            let mut inside = m;
            f(&mut inside);
            out.push((a..b, inside));
            if b < r.end {
                out.push((b..r.end, m));
            }
        }
        self.spans = compact(out);
    }

    /// `true` se TODO o texto em `range` tem `mark` (a barra mostra-a ligada). Um intervalo vazio
    /// pergunta pelas marcas de escrever ali.
    #[must_use]
    pub fn all(&self, range: Range<usize>, mark: Mark) -> bool {
        let range = self.clamp(range);
        if range.is_empty() {
            return mark.get(&self.typing_marks(range.start));
        }
        self.segments()
            .into_iter()
            .filter(|(r, _)| r.start < range.end && r.end > range.start)
            .all(|(_, m)| mark.get(&m))
    }

    /// Liga `mark` em `range` se algum carácter a não tem; senão desliga-a (o Ctrl+B dos editores).
    pub fn toggle(&mut self, range: Range<usize>, mark: Mark) {
        let on = !self.all(range.clone(), mark);
        self.restyle(range, |m| mark.set(m, on));
    }

    /// A cor única de `range`, se é uma só (`Some(None)` = toda na tinta da forma).
    #[must_use]
    pub fn color(&self, range: Range<usize>) -> Option<Option<Rgba>> {
        let range = self.clamp(range);
        let mut it = self
            .segments()
            .into_iter()
            .filter(|(r, _)| r.start < range.end && r.end > range.start)
            .map(|(_, m)| m.color);
        let first = it.next()?;
        it.all(|c| c == first).then_some(first)
    }

    /// O texto todo em segmentos contíguos, com as marcas de nascença onde não há trecho.
    fn segments(&self) -> Vec<(Range<usize>, Marks)> {
        let mut v = Vec::with_capacity(self.spans.len() * 2 + 1);
        let mut at = 0;
        for s in &self.spans {
            if s.range().start > at {
                v.push((at..s.range().start, Marks::default()));
            }
            v.push((s.range(), s.marks));
            at = s.range().end;
        }
        if at < self.text.len() {
            v.push((at..self.text.len(), Marks::default()));
        }
        v
    }

    /// `range` dentro do texto e em fronteiras de carácter.
    fn clamp(&self, range: Range<usize>) -> Range<usize> {
        let fix = |mut i: usize| {
            i = i.min(self.text.len());
            while !self.text.is_char_boundary(i) {
                i -= 1;
            }
            i
        };
        let (a, b) = (fix(range.start), fix(range.end));
        a.min(b)..b.max(a)
    }

    /// Repõe a invariante (útil depois de ler de fora).
    pub fn normalize(&mut self) {
        let segs = self.segments();
        self.spans = compact(segs);
    }
}

/// Segmentos contíguos → trechos: sem os de nascença nem os vazios, e vizinhos iguais unidos.
fn compact(segs: Vec<(Range<usize>, Marks)>) -> Vec<Span> {
    let mut out: Vec<Span> = Vec::new();
    for (r, m) in segs {
        if r.is_empty() || m == Marks::default() {
            continue;
        }
        let (start, end) = (r.start as u32, r.end as u32);
        match out.last_mut() {
            Some(last) if last.end == start && last.marks == m => last.end = end,
            _ => out.push(Span {
                start,
                end,
                marks: m,
            }),
        }
    }
    out
}

#[cfg(test)]
#[path = "rich_tests.rs"]
mod tests;
