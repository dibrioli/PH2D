//! Movimento dos cursores e comandos de seleção (Ctrl+D, cursor extra). Não mudam o texto.

use crate::buffer::Buffer;
use crate::selection::{Range, Selection};
use crate::text::Text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
    /// Início «esperto»: o 1.º não-espaço da linha e, se já está lá, a coluna 0.
    LineStart,
    LineEnd,
    DocStart,
    DocEnd,
    /// ↑/↓ de `n` linhas (a página da vista).
    Lines(isize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Space,
    Word,
    Punct,
    Newline,
}

fn class(c: char) -> Class {
    if c == '\n' {
        Class::Newline
    } else if c.is_whitespace() {
        Class::Space
    } else if c.is_alphanumeric() || c == '_' {
        Class::Word
    } else {
        Class::Punct
    }
}

pub fn is_word_char(c: char) -> bool {
    class(c) == Class::Word
}

fn char_at(t: &Text, b: usize) -> Option<char> {
    (b < t.len())
        .then(|| t.slice(b..t.next_grapheme(b)).chars().next())
        .flatten()
}

fn char_before(t: &Text, b: usize) -> Option<char> {
    (b > 0)
        .then(|| t.slice(t.prev_grapheme(b)..b).chars().next())
        .flatten()
}

/// Ctrl+→: salta os espaços e depois um bloco da mesma classe.
pub fn word_right(t: &Text, mut b: usize) -> usize {
    if char_at(t, b) == Some('\n') {
        return b + 1;
    }
    while char_at(t, b).is_some_and(|c| class(c) == Class::Space) {
        b = t.next_grapheme(b);
    }
    if let Some(k) = char_at(t, b).map(class).filter(|k| *k != Class::Newline) {
        while char_at(t, b).is_some_and(|c| class(c) == k) {
            b = t.next_grapheme(b);
        }
    }
    b
}

pub fn word_left(t: &Text, mut b: usize) -> usize {
    if char_before(t, b) == Some('\n') {
        return b - 1;
    }
    while char_before(t, b).is_some_and(|c| class(c) == Class::Space) {
        b = t.prev_grapheme(b);
    }
    if let Some(k) = char_before(t, b)
        .map(class)
        .filter(|k| *k != Class::Newline)
    {
        while char_before(t, b).is_some_and(|c| class(c) == k) {
            b = t.prev_grapheme(b);
        }
    }
    b
}

/// A palavra que contém `b` (ou que acaba nele); `None` fora de palavra.
pub fn word_at(t: &Text, b: usize) -> Option<std::ops::Range<usize>> {
    let inside =
        char_at(t, b).is_some_and(is_word_char) || char_before(t, b).is_some_and(is_word_char);
    if !inside {
        return None;
    }
    let mut s = b;
    while char_before(t, s).is_some_and(is_word_char) {
        s = t.prev_grapheme(s);
    }
    let mut e = b;
    while char_at(t, e).is_some_and(is_word_char) {
        e = t.next_grapheme(e);
    }
    Some(s..e)
}

pub(crate) fn first_non_space(t: &Text, line: usize) -> usize {
    let start = t.byte_of_line(line);
    let s = t.line_string(line);
    start + s.len() - s.trim_start_matches([' ', '\t']).len()
}

fn move_one(t: &Text, r: &Range, m: Motion, extend: bool, tab: usize) -> Range {
    // Seta lateral sem Shift sobre uma seleção: cai na ponta dela, sem andar.
    if !extend && !r.is_empty() {
        match m {
            Motion::Left => return Range::cursor(r.from()),
            Motion::Right => return Range::cursor(r.to()),
            _ => {}
        }
    }
    let h = r.head;
    let (head, goal) = match m {
        Motion::Left => (t.prev_grapheme(h), None),
        Motion::Right => (t.next_grapheme(h), None),
        Motion::WordLeft => (word_left(t, h), None),
        Motion::WordRight => (word_right(t, h), None),
        Motion::LineEnd => (t.line_end(t.line_of_byte(h)), None),
        Motion::LineStart => {
            let line = t.line_of_byte(h);
            let fns = first_non_space(t, line);
            (if h == fns { t.byte_of_line(line) } else { fns }, None)
        }
        Motion::DocStart => (0, None),
        Motion::DocEnd => (t.len(), None),
        Motion::Up | Motion::Down | Motion::Lines(_) => {
            let n = match m {
                Motion::Up => -1,
                Motion::Down => 1,
                Motion::Lines(n) => n,
                _ => 0,
            };
            let goal = r.goal.unwrap_or_else(|| t.display_col(h, tab));
            let line = t.line_of_byte(h) as isize + n;
            if line < 0 {
                (0, Some(goal))
            } else if line as usize >= t.line_count() {
                (t.len(), Some(goal))
            } else {
                (t.byte_at_display_col(line as usize, goal, tab), Some(goal))
            }
        }
    };
    Range {
        anchor: if extend { r.anchor } else { head },
        head,
        goal,
    }
}

impl Buffer {
    pub fn move_cursors(&mut self, m: Motion, extend: bool, tab: usize) {
        let t = self.text();
        let ranges = self
            .selection()
            .ranges()
            .iter()
            .map(|r| move_one(t, r, m, extend, tab))
            .collect();
        let sel = self.selection().with_ranges(ranges);
        self.set_selection(sel);
        self.seal_history();
    }

    pub fn select_all(&mut self) {
        let n = self.text().len();
        self.set_selection(Selection::single(Range::new(0, n)));
        self.seal_history();
    }

    /// Alt+clique: um cursor a mais (ou tira o que já está lá, se não for o único).
    pub fn toggle_cursor(&mut self, at: usize) {
        let sel = self.selection();
        let hit: Vec<Range> = sel
            .ranges()
            .iter()
            .copied()
            .filter(|r| !(r.is_empty() && r.head == at))
            .collect();
        let next = if hit.len() < sel.len() && !hit.is_empty() {
            sel.with_ranges(hit)
        } else {
            sel.push(Range::cursor(at))
        };
        self.set_selection(next);
        self.seal_history();
    }

    /// Esc: fica só o cursor primário.
    pub fn single_cursor(&mut self) {
        let p = self.selection().primary();
        self.set_selection(Selection::single(p));
    }

    /// Ctrl+D. Cursor vazio: selecciona a palavra. Com seleção: junta a próxima ocorrência do
    /// texto da primária (dá a volta ao fim); devolve `false` se não há outra.
    pub fn select_next_occurrence(&mut self) -> bool {
        let t = self.text();
        let p = self.selection().primary();
        if p.is_empty() {
            let Some(w) = word_at(t, p.head) else {
                return false;
            };
            let ranges: Vec<Range> = self
                .selection()
                .ranges()
                .iter()
                .map(|r| {
                    if *r == p {
                        Range::new(w.start, w.end)
                    } else {
                        *r
                    }
                })
                .collect();
            let sel = self.selection().with_ranges(ranges);
            self.set_selection(sel);
            return true;
        }
        let needle = t.slice(p.span());
        let hay = t.to_string();
        let last = self
            .selection()
            .ranges()
            .iter()
            .map(Range::to)
            .max()
            .unwrap_or(0);
        let taken = |s: usize| self.selection().ranges().iter().any(|r| r.from() == s);
        let found = hay[last..]
            .match_indices(needle.as_str())
            .map(|(i, _)| last + i)
            .chain(hay[..last].match_indices(needle.as_str()).map(|(i, _)| i))
            .find(|s| !taken(*s));
        let Some(s) = found else { return false };
        let sel = self.selection().push(Range::new(s, s + needle.len()));
        self.set_selection(sel);
        self.seal_history();
        true
    }
}
