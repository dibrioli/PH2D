//! Texto inline → markdown: marcas como delimitadores bem aninhados (código sempre por dentro),
//! texto escapado, quebras de linha como `\` + fim de linha.

use crate::render::longest_run;
use ph2d_richdoc::{Inline, Mark};

/// Escreve `inline` em `out`. `cont` = prefixo das linhas de continuação; num título as
/// quebras viram espaço.
pub(crate) fn emit_inline(out: &mut String, inline: &Inline, cont: &str, heading: bool) {
    let stable = stable(clean(inline, heading), heading);
    write(out, &stable, cont, heading);
}

/// O leitor é o juiz: escreve-se, relê-se, e só se grava o que volta IGUAL. Marcas sobrepostas
/// partem-se em delimitadores que o CommonMark pode recusar (`x*"q"*y`): se a releitura muda só
/// as marcas, fica a dela; se deixaria `*` soltos no texto, larga-se a ênfase culpada. Cada volta
/// tira uma marca ou adopta a do leitor; no fim, só com código nunca falha.
fn stable(first: Inline, heading: bool) -> Inline {
    let mut cur = first;
    for _ in 0..=cur.marks().len() + 1 {
        let Some(back) = reread(&written(&cur, heading)) else {
            break;
        };
        if back == cur {
            return cur;
        }
        if back.text() == cur.text() {
            cur = clean(&back, heading);
            continue;
        }
        let culprit = (0..cur.marks().len())
            .filter(|&k| cur.marks()[k].mark != Mark::Code)
            .map(|k| without(&cur, k))
            .find(|c| reread(&written(c, heading)).is_some_and(|b| b.text() == c.text()));
        match culprit {
            Some(c) => cur = c,
            None => break,
        }
    }
    let mut plain = Inline::plain(cur.text());
    for s in cur.marks().iter().filter(|s| s.mark == Mark::Code) {
        plain.add_mark(s.start..s.end, s.mark.clone());
    }
    plain
}

fn written(i: &Inline, heading: bool) -> String {
    let mut md = String::new();
    write(&mut md, i, "", heading);
    md
}

/// `i` sem a marca `k`.
fn without(i: &Inline, k: usize) -> Inline {
    let mut out = Inline::plain(i.text());
    for (_, s) in i.marks().iter().enumerate().filter(|&(n, _)| n != k) {
        out.add_mark(s.start..s.end, s.mark.clone());
    }
    out
}

fn reread(md: &str) -> Option<Inline> {
    let doc = crate::parse(md);
    let [b] = doc.blocks() else { return None };
    let mut back = Inline::plain(b.inline.text());
    for s in b.inline.marks().iter().filter(|s| s.mark != Mark::Mention) {
        back.add_mark(s.start..s.end, s.mark.clone());
    }
    Some(back)
}

fn write(out: &mut String, clean: &Inline, cont: &str, heading: bool) {
    let text = clean.text();
    let mut e = Emitter {
        out,
        cont,
        heading,
        line_start: true,
        prev: None,
    };
    let mut stack: Vec<Mark> = Vec::new();
    for run in clean.runs() {
        let code = run.marks.contains(&Mark::Code);
        let target: Vec<Mark> = run
            .marks
            .iter()
            .filter(|m| **m != Mark::Code)
            .cloned()
            .collect();
        let keep = stack.iter().take_while(|m| target.contains(m)).count();
        while stack.len() > keep {
            let m = stack.pop().expect("pilha");
            e.close(&m);
        }
        let mut open: Vec<(usize, Mark)> = target
            .into_iter()
            .filter(|m| !stack.contains(m))
            .map(|m| (extent(clean, run.range.start, &m), m))
            .collect();
        open.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        for (_, m) in open {
            e.open(&m);
            stack.push(m);
        }
        let piece = &text[run.range.clone()];
        if code {
            e.code(piece);
        } else {
            e.text(piece);
        }
    }
    while let Some(m) = stack.pop() {
        e.close(&m);
    }
}

/// Tira os espaços (fora de código) das pontas de cada linha e as linhas vazias das pontas: o
/// leitor de markdown tirá-los-ia.
fn trim_lines(inline: &Inline) -> Inline {
    let t = inline.text();
    let soft = |i: usize, ch: char| {
        matches!(ch, ' ' | '\t') && !inline.marks_on_byte(i).contains(&Mark::Code)
    };
    let mut keep = vec![false; t.len()];
    let mut line_start = 0;
    for line in t.split('\n') {
        let idx: Vec<(usize, char)> = line
            .char_indices()
            .map(|(i, ch)| (line_start + i, ch))
            .collect();
        let first = idx.iter().position(|&(i, ch)| !soft(i, ch));
        let last = idx.iter().rposition(|&(i, ch)| !soft(i, ch));
        if let (Some(f), Some(l)) = (first, last) {
            for &(i, _) in &idx[f..=l] {
                keep[i] = true;
            }
        }
        line_start += line.len() + 1;
    }
    let content: Vec<usize> = (0..t.len()).filter(|&i| keep[i]).collect();
    if let (Some(&lo), Some(&hi)) = (content.first(), content.last()) {
        for (i, ch) in t.char_indices() {
            if ch == '\n' && i > lo && i < hi {
                keep[i] = true;
            }
        }
    }
    let mut out = inline.clone();
    for (p, ch) in t.char_indices().rev() {
        if !keep[p] {
            out.delete(p..p + ch.len_utf8());
        }
    }
    out
}

/// Onde acaba a marca `m` que cobre `at`.
fn extent(inline: &Inline, at: usize, m: &Mark) -> usize {
    inline
        .marks()
        .iter()
        .find(|s| s.mark == *m && s.start <= at && at < s.end)
        .map_or(at, |s| s.end)
}

/// Cópia sem menções, sem espaços nas pontas das linhas, e só com a ênfase que o CommonMark lê.
fn clean(inline: &Inline, heading: bool) -> Inline {
    let mut c = trim_lines(inline);
    if heading {
        while let Some(p) = c.text().find('\n') {
            let marks = c.marks_on_byte(p);
            c.delete(p..p + 1);
            c.insert(p, " ", &marks);
        }
    }
    let text = c.text().to_owned();
    let spans = c.marks().to_vec();
    let mut out = Inline::plain(text.clone());
    for s in &spans {
        let (mut a, mut b) = (s.start, s.end);
        if is_emphasis(&s.mark) {
            // Os espaços das pontas saem da ênfase (os de código ficam: são conteúdo).
            let soft = |i: usize| {
                text[i..].starts_with(char::is_whitespace)
                    && !c.marks_on_byte(i).contains(&Mark::Code)
            };
            while a < b && soft(a) {
                a += text[a..].chars().next().map_or(1, char::len_utf8);
            }
            while a < b {
                let last = text[..b].char_indices().next_back().map_or(a, |(i, _)| i);
                if !soft(last) {
                    break;
                }
                b = last;
            }
            if a >= b {
                continue;
            }
        }
        if s.mark != Mark::Mention {
            out.add_mark(a..b, s.mark.clone());
        }
    }
    out
}

fn is_emphasis(m: &Mark) -> bool {
    matches!(m, Mark::Bold | Mark::Italic | Mark::Strike)
}

struct Emitter<'a> {
    out: &'a mut String,
    cont: &'a str,
    heading: bool,
    line_start: bool,
    prev: Option<char>,
}

impl Emitter<'_> {
    fn raw(&mut self, s: &str) {
        self.out.push_str(s);
        if let Some(c) = s.chars().last() {
            self.prev = Some(c);
            self.line_start = false;
        }
    }

    fn open(&mut self, m: &Mark) {
        match m {
            Mark::Bold => self.raw("**"),
            Mark::Italic => self.raw("*"),
            Mark::Strike => self.raw("~~"),
            Mark::Link(_) => self.raw("["),
            Mark::Code | Mark::Mention => {}
        }
    }

    fn close(&mut self, m: &Mark) {
        match m {
            Mark::Bold => self.raw("**"),
            Mark::Italic => self.raw("*"),
            Mark::Strike => self.raw("~~"),
            Mark::Link(url) => {
                let dest = link_dest(url);
                self.raw("](");
                self.raw(&dest);
                self.raw(")");
            }
            Mark::Code | Mark::Mention => {}
        }
    }

    fn code(&mut self, text: &str) {
        let text = text.replace('\n', " ");
        let fence = "`".repeat(longest_run(&text, '`') + 1);
        let pad = text.starts_with('`')
            || text.ends_with('`')
            || (text.starts_with(' ') && text.ends_with(' ') && !text.trim().is_empty());
        let pad = if pad { " " } else { "" };
        self.raw(&format!("{fence}{pad}{text}{pad}{fence}"));
    }

    fn text(&mut self, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        let mut escape_at: Option<usize> = None;
        for (i, &c) in chars.iter().enumerate() {
            let next = chars.get(i + 1).copied();
            if c == '\n' {
                if self.heading {
                    self.raw(" ");
                } else {
                    self.out.push_str("\\\n");
                    self.out.push_str(self.cont);
                    self.line_start = true;
                    self.prev = None;
                }
                continue;
            }
            if self.line_start && c.is_ascii_digit() {
                let digits = chars[i..].iter().take_while(|d| d.is_ascii_digit()).count();
                if matches!(chars.get(i + digits), Some('.' | ')')) {
                    escape_at = Some(i + digits);
                }
            }
            let esc = match c {
                '\\' | '`' | '*' | '[' | ']' | '<' | '~' => true,
                '#' | '>' | '-' | '+' | '=' => self.line_start || (self.heading && c == '#'),
                '_' => {
                    !(self.prev.is_some_and(char::is_alphanumeric)
                        && next.is_some_and(char::is_alphanumeric))
                }
                '&' => next.is_none_or(|n| n.is_alphanumeric() || n == '#'),
                // `!` colado a um link que abre a seguir faria uma imagem.
                '!' => next.is_none(),
                _ => escape_at == Some(i),
            };
            if esc {
                self.out.push('\\');
            }
            let mut buf = [0u8; 4];
            self.raw(c.encode_utf8(&mut buf));
        }
    }
}

fn link_dest(url: &str) -> String {
    let needs_angle = url.is_empty() || url.contains([' ', '(', ')', '<', '>']);
    let mut s = String::with_capacity(url.len() + 2);
    if needs_angle {
        s.push('<');
    }
    for c in url.chars() {
        if c == '\\' || (needs_angle && matches!(c, '<' | '>')) {
            s.push('\\');
        }
        s.push(c);
    }
    if needs_angle {
        s.push('>');
    }
    s
}
