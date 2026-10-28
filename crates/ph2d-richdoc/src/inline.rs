//! Texto inline: uma `String` e marcas por intervalo de bytes, sempre normalizadas
//! (ordenadas, sem vazias, iguais adjacentes fundidas, um só link por byte).

use std::ops::Range;

/// Marca inline. `Link` leva o url; `Mention` cobre o texto `@nome`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mark {
    Bold,
    Italic,
    Strike,
    Code,
    Link(String),
    Mention,
}

impl Mark {
    /// Mesmo tipo (dois links com urls diferentes são do mesmo tipo).
    pub fn same_kind(&self, other: &Mark) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    /// Marcas que se estendem ao escrever logo depois delas.
    pub fn is_inclusive(&self) -> bool {
        matches!(self, Mark::Bold | Mark::Italic | Mark::Strike)
    }
}

/// Uma marca sobre `start..end` (bytes do texto).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MarkSpan {
    pub start: usize,
    pub end: usize,
    pub mark: Mark,
}

/// Um troço contíguo com o mesmo conjunto de marcas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub range: Range<usize>,
    pub marks: Vec<Mark>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inline {
    text: String,
    marks: Vec<MarkSpan>,
}

impl Inline {
    pub fn plain(text: impl Into<String>) -> Inline {
        Inline {
            text: text.into(),
            marks: Vec::new(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn marks(&self) -> &[MarkSpan] {
        &self.marks
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Acrescenta `text` no fim com `marks`.
    pub fn push(&mut self, text: &str, marks: &[Mark]) {
        let at = self.text.len();
        self.insert(at, text, marks);
    }

    /// Insere `text` em `at` (fronteira de char); o texto novo leva EXACTAMENTE `marks`.
    pub fn insert(&mut self, at: usize, text: &str, marks: &[Mark]) {
        assert!(self.text.is_char_boundary(at), "insert fora de fronteira");
        let n = text.len();
        if n == 0 {
            return;
        }
        let mut out = Vec::with_capacity(self.marks.len() + marks.len() + 1);
        for s in self.marks.drain(..) {
            if s.end <= at {
                out.push(s);
            } else if s.start >= at {
                out.push(MarkSpan {
                    start: s.start + n,
                    end: s.end + n,
                    mark: s.mark,
                });
            } else if s.mark == Mark::Mention {
                // Escrever dentro de uma menção desfá-la.
            } else {
                out.push(MarkSpan {
                    start: s.start,
                    end: at,
                    mark: s.mark.clone(),
                });
                out.push(MarkSpan {
                    start: at + n,
                    end: s.end + n,
                    mark: s.mark,
                });
            }
        }
        for m in marks {
            if let Mark::Link(_) = m {
                out.retain(|s| !(s.mark.same_kind(m) && s.start < at + n && s.end > at));
            }
            out.push(MarkSpan {
                start: at,
                end: at + n,
                mark: m.clone(),
            });
        }
        self.text.insert_str(at, text);
        self.marks = out;
        self.normalize();
    }

    /// Apaga `range`; uma menção cortada perde a marca.
    pub fn delete(&mut self, range: Range<usize>) {
        let (a, b) = (range.start, range.end.min(self.text.len()));
        if a >= b {
            return;
        }
        let n = b - a;
        let shift = |p: usize| {
            if p >= b {
                p - n
            } else if p > a {
                a
            } else {
                p
            }
        };
        let mut out = Vec::with_capacity(self.marks.len());
        for s in self.marks.drain(..) {
            let clipped = s.start < b && s.end > a;
            if s.mark == Mark::Mention && clipped {
                continue;
            }
            out.push(MarkSpan {
                start: shift(s.start),
                end: shift(s.end),
                mark: s.mark,
            });
        }
        self.text.replace_range(a..b, "");
        self.marks = out;
        self.normalize();
    }

    /// Parte em `at`: `self` fica com `..at`, devolve `at..`.
    pub fn split_off(&mut self, at: usize) -> Inline {
        let tail_text = self.text.split_off(at);
        let mut tail = Vec::new();
        let mut head = Vec::new();
        for s in self.marks.drain(..) {
            if s.end <= at {
                head.push(s);
            } else if s.start >= at {
                tail.push(MarkSpan {
                    start: s.start - at,
                    end: s.end - at,
                    mark: s.mark,
                });
            } else if s.mark != Mark::Mention {
                head.push(MarkSpan {
                    start: s.start,
                    end: at,
                    mark: s.mark.clone(),
                });
                tail.push(MarkSpan {
                    start: 0,
                    end: s.end - at,
                    mark: s.mark,
                });
            }
        }
        self.marks = head;
        self.normalize();
        let mut t = Inline {
            text: tail_text,
            marks: tail,
        };
        t.normalize();
        t
    }

    /// Junta `other` no fim.
    pub fn append(&mut self, other: Inline) {
        let off = self.text.len();
        self.text.push_str(&other.text);
        self.marks.extend(other.marks.into_iter().map(|s| MarkSpan {
            start: s.start + off,
            end: s.end + off,
            mark: s.mark,
        }));
        self.normalize();
    }

    pub fn clear_marks(&mut self) {
        self.marks.clear();
    }

    /// Aplica `mark` a `range` (um link substitui outros links no intervalo).
    pub fn add_mark(&mut self, range: Range<usize>, mark: Mark) {
        if range.start >= range.end {
            return;
        }
        if let Mark::Link(_) = mark {
            self.remove_mark(range.clone(), &mark);
        }
        self.marks.push(MarkSpan {
            start: range.start,
            end: range.end,
            mark,
        });
        self.normalize();
    }

    /// Tira as marcas do MESMO TIPO de `kind` dentro de `range`.
    pub fn remove_mark(&mut self, range: Range<usize>, kind: &Mark) {
        let (a, b) = (range.start, range.end);
        let mut out = Vec::with_capacity(self.marks.len() + 1);
        for s in self.marks.drain(..) {
            if !s.mark.same_kind(kind) || s.end <= a || s.start >= b {
                out.push(s);
                continue;
            }
            if s.mark == Mark::Mention {
                continue;
            }
            if s.start < a {
                out.push(MarkSpan {
                    start: s.start,
                    end: a,
                    mark: s.mark.clone(),
                });
            }
            if s.end > b {
                out.push(MarkSpan {
                    start: b,
                    end: s.end,
                    mark: s.mark,
                });
            }
        }
        self.marks = out;
        self.normalize();
    }

    /// Todo o `range` (não vazio) está coberto por marcas do tipo de `kind`?
    pub fn covers(&self, range: Range<usize>, kind: &Mark) -> bool {
        let mut at = range.start;
        let mut spans: Vec<&MarkSpan> = self
            .marks
            .iter()
            .filter(|s| s.mark.same_kind(kind))
            .collect();
        spans.sort_by_key(|s| s.start);
        for s in spans {
            if s.start <= at && s.end > at {
                at = s.end;
            }
        }
        range.start < range.end && at >= range.end
    }

    /// Marcas que cobrem o byte `at..at+1`.
    pub fn marks_on_byte(&self, at: usize) -> Vec<Mark> {
        let mut v: Vec<Mark> = self
            .marks
            .iter()
            .filter(|s| s.start <= at && at < s.end)
            .map(|s| s.mark.clone())
            .collect();
        v.sort();
        v
    }

    /// As marcas que um caractere escrito em `at` herda: as inclusivas do caractere anterior;
    /// código e link só por dentro; menção nunca.
    pub fn marks_for_typing(&self, at: usize) -> Vec<Mark> {
        if at == 0 {
            return Vec::new();
        }
        let before = self
            .marks
            .iter()
            .filter(|s| s.start < at && at <= s.end)
            .filter(|s| s.mark != Mark::Mention)
            .filter(|s| s.mark.is_inclusive() || at < s.end);
        let mut v: Vec<Mark> = before.map(|s| s.mark.clone()).collect();
        v.sort();
        v.dedup();
        v
    }

    /// O intervalo da menção que contém o byte `at`, se houver.
    pub fn mention_at(&self, at: usize) -> Option<Range<usize>> {
        self.marks
            .iter()
            .find(|s| s.mark == Mark::Mention && s.start <= at && at < s.end)
            .map(|s| s.start..s.end)
    }

    /// Troços com o mesmo conjunto de marcas, cobrindo o texto inteiro.
    pub fn runs(&self) -> Vec<Run> {
        let mut cuts = vec![0, self.text.len()];
        for s in &self.marks {
            cuts.push(s.start);
            cuts.push(s.end);
        }
        cuts.sort_unstable();
        cuts.dedup();
        cuts.windows(2)
            .filter(|w| w[0] < w[1])
            .map(|w| Run {
                range: w[0]..w[1],
                marks: self.marks_on_byte(w[0]),
            })
            .collect()
    }

    fn normalize(&mut self) {
        let len = self.text.len();
        self.marks.retain(|s| s.start < s.end && s.end <= len);
        self.marks
            .sort_by(|x, y| (&x.mark, x.start, x.end).cmp(&(&y.mark, y.start, y.end)));
        let mut out: Vec<MarkSpan> = Vec::with_capacity(self.marks.len());
        for s in self.marks.drain(..) {
            match out.last_mut() {
                // Menções adjacentes são pessoas distintas: nunca se fundem.
                Some(l) if l.mark == s.mark && s.start <= l.end && s.mark != Mark::Mention => {
                    l.end = l.end.max(s.end);
                }
                Some(l)
                    if l.mark == Mark::Mention && s.mark == Mark::Mention && s.start < l.end =>
                {
                    l.end = l.end.max(s.end);
                }
                _ => out.push(s),
            }
        }
        out.sort_by(|x, y| (x.start, x.end, &x.mark).cmp(&(y.start, y.end, &y.mark)));
        self.marks = out;
    }
}
