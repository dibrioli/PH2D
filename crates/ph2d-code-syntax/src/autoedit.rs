//! Fechamento automático e auto-indent (Onda 1 do CodeEditor, E8): discretos e apagáveis.
//!
//! A lei é a do `CodeEdit` do Godot 4.7.2, CORRIDO como oráculo sobre entradas nossas
//! (`docs/CodeEditor/ferramentas/godot_oraculo/`, gate `tests/it/autoedit.rs`), com duas trocas:
//! o «abre bloco» vem da árvore Luau ([`Syntax::indent_level_after`]) em vez do prefixo `:`, e
//! escrever `end`/`else`/`elseif`/`until`/`}`/`)` no início de uma linha recua-a (o GDScript não
//! tem fechos de bloco). Cada gesto é UM passo de undo e vale em todos os cursores.

use crate::Syntax;
use crate::indent::may_dedent;
use ph2d_code_buffer::{
    Buffer, Change, ChangeSet, EditKind, Edits, Range, Selection, Text, is_word_char,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    Tabs,
    Spaces(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoEdit {
    pub indent: IndentStyle,
    /// `false` desliga os pares (escrever, saltar o fecho, apagar o par); o auto-indent fica.
    pub close_pairs: bool,
}

const PAIRS: [(char, char); 5] = [('(', ')'), ('[', ']'), ('{', '}'), ('"', '"'), ('\'', '\'')];

pub fn closer_of(open: char) -> Option<char> {
    PAIRS.iter().find(|p| p.0 == open).map(|p| p.1)
}

fn is_closer(c: char) -> bool {
    PAIRS.iter().any(|p| p.1 == c)
}

fn is_quote(c: char) -> bool {
    c == '"' || c == '\''
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ctx {
    Code,
    String,
    Comment,
}

/// Uma troca por cursor: `caret` é o deslocamento do cursor a partir de `range.start` no texto NOVO.
struct Plan {
    range: std::ops::Range<usize>,
    insert: String,
    caret: usize,
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

/// Fim da indentação da linha `line` (o primeiro byte que não é espaço nem tab).
fn indent_end(t: &Text, line: usize) -> usize {
    let s = t.line_string(line);
    t.byte_of_line(line) + s.len() - s.trim_start_matches([' ', '\t']).len()
}

/// Corrida léxica da linha até `byte` (o caminho do Godot), para quando a árvore está em ERROR.
fn lexical_ctx(t: &Text, byte: usize) -> Ctx {
    let start = t.byte_of_line(t.line_of_byte(byte));
    let s = t.slice(start..byte);
    let mut chars = s.chars().peekable();
    let mut quote: Option<char> = None;
    while let Some(c) = chars.next() {
        match quote {
            Some(q) => {
                if c == '\\' {
                    chars.next();
                } else if c == q {
                    quote = None;
                }
            }
            None if is_quote(c) => quote = Some(c),
            None if c == '-' && chars.peek() == Some(&'-') => return Ctx::Comment,
            None => {}
        }
    }
    if quote.is_some() {
        Ctx::String
    } else {
        Ctx::Code
    }
}

fn context_at(t: &Text, syn: &Syntax, byte: usize) -> Ctx {
    let root = syn.tree().root_node();
    let mut node = root.descendant_for_byte_range(byte.saturating_sub(1), byte);
    let mut in_error = false;
    while let Some(n) = node {
        let (s, e) = (n.start_byte(), n.end_byte());
        match n.kind() {
            "comment" => {
                // Bytes, nunca `s + 4` cortado: o 4.º byte de `-- é` cai dentro do `é`.
                let block = ["--[[", "--[="]
                    .iter()
                    .any(|p| s + p.len() <= e && t.starts_with_at(s, p));
                if s < byte && (byte < e || !block) {
                    return Ctx::Comment;
                }
            }
            "string" => {
                let body = t.slice(s..e);
                let closed = match body.chars().next() {
                    Some(q) if is_quote(q) => body.len() >= 2 && body.ends_with(q),
                    _ => body.ends_with(']'),
                };
                if s < byte && (byte < e || !closed) {
                    return Ctx::String;
                }
            }
            _ => {}
        }
        in_error |= n.is_error();
        node = n.parent();
    }
    if in_error {
        lexical_ctx(t, byte)
    } else {
        Ctx::Code
    }
}

impl AutoEdit {
    pub fn new(indent: IndentStyle) -> Self {
        Self {
            indent,
            close_pairs: true,
        }
    }

    /// Um nível de indentação.
    pub fn unit(&self) -> String {
        match self.indent {
            IndentStyle::Tabs => "\t".into(),
            IndentStyle::Spaces(n) => " ".repeat(n),
        }
    }

    fn level(&self, n: usize) -> String {
        self.unit().repeat(n)
    }

    /// `base` com `delta` níveis a mais (ou a menos, tirando do fim).
    fn shift(&self, base: &str, delta: isize) -> String {
        if delta >= 0 {
            return format!("{base}{}", self.level(delta as usize));
        }
        let width = match self.indent {
            IndentStyle::Tabs => 4,
            IndentStyle::Spaces(n) => n,
        };
        let mut s = base.to_string();
        for _ in 0..delta.unsigned_abs() {
            if s.ends_with('\t') {
                s.pop();
            } else {
                let keep = s
                    .trim_end_matches(' ')
                    .len()
                    .max(s.len().saturating_sub(width));
                s.truncate(keep);
            }
        }
        s
    }

    fn apply(buf: &mut Buffer, syn: &mut Syntax, plans: Vec<Plan>, kind: EditKind) -> Edits {
        let mut delta: isize = 0;
        let mut afters = Vec::with_capacity(plans.len());
        let mut changes = Vec::with_capacity(plans.len());
        for p in plans {
            let start = (p.range.start as isize + delta) as usize;
            delta += p.insert.len() as isize - p.range.len() as isize;
            afters.push(Range::cursor(start + p.caret));
            changes.push(Change {
                range: p.range,
                insert: p.insert,
            });
        }
        let Some(cs) = ChangeSet::new(changes) else {
            return Vec::new();
        };
        let after = Selection::new(afters, buf.selection().primary_index());
        let edits = buf.apply(cs, Some(after), kind);
        syn.edit_deferred(&edits);
        edits
    }

    /// Escreve `ch` em todos os cursores: abre o par, salta o fecho, envolve a seleção e recua a
    /// linha que passa a começar por um fecho de bloco. Devolve TODAS as edições (vazio = nada).
    pub fn on_char(&self, buf: &mut Buffer, syn: &mut Syntax, ch: char) -> Edits {
        if ch == '\n' {
            return self.on_enter(buf, syn);
        }
        let t = buf.text();
        let mut was_dedent = Vec::new();
        let mut plans = Vec::new();
        for r in buf.selection().ranges() {
            let (from, to) = (r.from(), r.to());
            let line = t.line_of_byte(from);
            let first = indent_end(t, line);
            was_dedent.push(
                first < t.line_end(line) && may_dedent(t, first) && {
                    syn.fresh(t);
                    syn.dedents_at(t, first)
                },
            );
            let open = closer_of(ch).filter(|_| self.close_pairs);
            let plan = if !r.is_empty() {
                match open {
                    Some(close) => {
                        let sel = t.slice(from..to);
                        let caret = ch.len_utf8() + sel.len();
                        Plan {
                            range: from..to,
                            insert: format!("{ch}{sel}{close}"),
                            caret,
                        }
                    }
                    None => Plan {
                        range: from..to,
                        insert: ch.to_string(),
                        caret: ch.len_utf8(),
                    },
                }
            } else if self.close_pairs && is_closer(ch) && char_at(t, from) == Some(ch) {
                Plan {
                    range: from..from,
                    insert: String::new(),
                    caret: ch.len_utf8(),
                }
            } else if let Some(close) = open.filter(|_| self.closes_here(t, syn, from, ch)) {
                Plan {
                    range: from..from,
                    insert: format!("{ch}{close}"),
                    caret: ch.len_utf8(),
                }
            } else {
                Plan {
                    range: from..from,
                    insert: ch.to_string(),
                    caret: ch.len_utf8(),
                }
            };
            plans.push(plan);
        }
        let mut edits = Self::apply(buf, syn, plans, EditKind::Typing);
        edits.extend(self.reindent_closers(buf, syn, &was_dedent));
        edits
    }

    /// A lei do Godot para abrir o par: nunca em comentário; nunca antes de letra/dígito/`_`; as
    /// aspas também não dentro de string nem logo depois de letra/dígito (`don't`).
    fn closes_here(&self, t: &Text, syn: &mut Syntax, at: usize, ch: char) -> bool {
        if char_at(t, at).is_some_and(is_word_char) {
            return false;
        }
        syn.fresh(t);
        let ctx = context_at(t, syn, at);
        if ctx == Ctx::Comment {
            return false;
        }
        !(is_quote(ch) && (ctx == Ctx::String || char_before(t, at).is_some_and(is_word_char)))
    }

    /// Depois de escrever no 1.º token de uma linha: se ele fecha um bloco (ou fechava antes desta
    /// tecla, `end` → `endx`), a linha vai para o nível que a árvore lhe dá. A troca leva o token até
    /// ao cursor, para nunca ser «só espaço» e juntar-se ao passo de undo da escrita.
    fn reindent_closers(&self, buf: &mut Buffer, syn: &mut Syntax, was_dedent: &[bool]) -> Edits {
        let t = buf.text();
        let mut plans: Vec<Plan> = Vec::new();
        let mut last_line = None;
        for (i, r) in buf.selection().ranges().iter().enumerate() {
            let c = r.head;
            let line = t.line_of_byte(c);
            let (start, first) = (t.byte_of_line(line), indent_end(t, line));
            if !r.is_empty() || last_line == Some(line) || first >= c {
                continue;
            }
            let token = t.slice(first..c);
            if token.contains(char::is_whitespace) {
                continue;
            }
            let was = was_dedent.get(i).copied().unwrap_or(false);
            if !(was
                || (may_dedent(t, first) && {
                    syn.fresh(t);
                    syn.dedents_at(t, first)
                }))
            {
                continue;
            }
            syn.fresh(t);
            let want = self.level(syn.line_indent_level(t, line));
            if t.slice(start..first) == want {
                continue;
            }
            last_line = Some(line);
            let insert = format!("{want}{token}");
            plans.push(Plan {
                range: start..c,
                caret: insert.len(),
                insert,
            });
        }
        if plans.is_empty() {
            return Vec::new();
        }
        let changes = plans
            .into_iter()
            .map(|p| Change {
                range: p.range,
                insert: p.insert,
            })
            .collect();
        let Some(cs) = ChangeSet::new(changes) else {
            return Vec::new();
        };
        let edits = buf.apply(cs, None, EditKind::Typing);
        syn.edit_deferred(&edits);
        edits
    }

    /// Enter em todos os cursores: a linha nova herda a indentação da linha, +1 depois de quem abre
    /// um bloco; entre `(`/`[`/`{` e o seu fecho abre três linhas com o meio indentado.
    pub fn on_enter(&self, buf: &mut Buffer, syn: &mut Syntax) -> Edits {
        let t = buf.text();
        syn.fresh(t);
        let mut plans = Vec::new();
        for r in buf.selection().ranges() {
            let (from, to) = (r.from(), r.to());
            let line = t.line_of_byte(from);
            let ws_end = indent_end(t, line);
            let base = t.slice(t.byte_of_line(line)..from.min(ws_end));
            let (insert, caret) = if from <= ws_end {
                let s = format!("\n{base}");
                let n = s.len();
                (s, n)
            } else {
                let rest_line = t.line_of_byte(to);
                let rest_end = t.line_end(rest_line);
                let rest = t.slice(to..rest_end);
                let rest_first = to + rest.len() - rest.trim_start_matches([' ', '\t']).len();
                // Só os fechos-PALAVRA (`end`, `until`, `else`…): os parênteses seguem a lei do par adjacente.
                let rest_dedents = rest_first < rest_end
                    && char_at(t, rest_first).is_some_and(is_word_char)
                    && syn.dedents_at(t, rest_first);
                let delta = syn.indent_level_after(t, from) as isize
                    - syn.line_indent_level(t, line) as isize;
                let prev = char_before(t, from);
                let pair = prev.is_some_and(|p| {
                    matches!(p, '(' | '[' | '{') && char_at(t, to) == closer_of(p)
                });
                if pair || (rest_dedents && delta > 0) {
                    let d = if pair { 1 } else { delta };
                    let mid = format!("\n{}", self.shift(&base, d));
                    let n = mid.len();
                    (format!("{mid}\n{}", self.shift(&base, d - 1)), n)
                } else {
                    let s = format!("\n{}", self.shift(&base, delta - isize::from(rest_dedents)));
                    let n = s.len();
                    (s, n)
                }
            };
            plans.push(Plan {
                range: from..to,
                insert,
                caret,
            });
        }
        Self::apply(buf, syn, plans, EditKind::Other)
    }

    /// Backspace em todos os cursores: entre um par vazio apaga os dois; na indentação feita de
    /// espaços volta à paragem anterior; senão apaga um grafema.
    pub fn on_backspace(&self, buf: &mut Buffer, syn: &mut Syntax) -> Edits {
        let t = buf.text();
        let mut spans: Vec<std::ops::Range<usize>> = Vec::new();
        for r in buf.selection().ranges() {
            let h = r.head;
            let span = if !r.is_empty() {
                r.span()
            } else if self.close_pairs
                && char_before(t, h)
                    .and_then(closer_of)
                    .is_some_and(|c| char_at(t, h) == Some(c))
            {
                t.prev_grapheme(h)..t.next_grapheme(h)
            } else {
                let start = t.byte_of_line(t.line_of_byte(h));
                let col = h - start;
                match self.indent {
                    IndentStyle::Spaces(n)
                        if n > 0 && col > 0 && t.slice(start..h).bytes().all(|b| b == b' ') =>
                    {
                        start + (col - 1) / n * n..h
                    }
                    _ => t.prev_grapheme(h)..h,
                }
            };
            match spans.last_mut() {
                Some(last) if span.start <= last.end => last.end = last.end.max(span.end),
                _ => spans.push(span),
            }
        }
        let changes = spans
            .into_iter()
            .map(|range| Change {
                range,
                insert: String::new(),
            })
            .collect();
        let Some(cs) = ChangeSet::new(changes) else {
            return Vec::new();
        };
        let edits = buf.apply(cs, None, EditKind::Deleting);
        syn.edit_deferred(&edits);
        edits
    }
}
