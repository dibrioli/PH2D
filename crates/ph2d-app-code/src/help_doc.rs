//! ⭐ **Uma [`Page`] do catálogo → o [`HelpDoc`] que o painel pinta** (`docs/CodeEditor/04_pagina_de_ajuda.md`
//! §3.3): o título, a descrição, a tabela das irmãs (retorno · `nome(args)`, o nome é link) e, por
//! membro, a assinatura realçada, o texto, os argumentos, o retorno e o exemplo. Cada link do texto
//! (`[lib.membro]`, `[nome]`) resolve-se pela MESMA porta do Ctrl+clique
//! ([`ph2d_code_help::resolve::resolve`]); a família guarda o destino de cada número ([`Target`]).

use ph2d_code_buffer::Text;
use ph2d_code_help::resolve::{Query, resolve};
use ph2d_code_help::{Anchor, Kind, Member, Page};
use ph2d_code_syntax::Syntax;
use ph2d_i18n::tr;
use ph2d_panel_code::Ink;
use ph2d_panel_code::help::{Block, HelpDoc, Span, SpanKind};

/// Para onde leva o número de um link (ou de um botão de copiar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Target {
    Anchor(Anchor),
    /// O botão de copiar o exemplo do membro `n` da página.
    Example(usize),
}

/// A página, e o destino de cada número.
pub(crate) struct Built {
    pub doc: HelpDoc,
    pub targets: Vec<Target>,
}

/// Um `[…]` do texto que não resolve (o gate do catálogo).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeadLink {
    pub page: &'static str,
    pub member: Option<&'static str>,
    pub link: String,
}

struct Builder {
    blocks: Vec<Block>,
    targets: Vec<Target>,
    dead: Vec<String>,
}

impl Builder {
    fn number(&mut self, t: Target) -> u32 {
        if let Some(i) = self.targets.iter().position(|x| *x == t) {
            return i as u32;
        }
        self.targets.push(t);
        (self.targets.len() - 1) as u32
    }

    fn heading(&mut self, key: &str) {
        self.blocks.push(Block::Heading(tr(key).to_owned()));
    }
}

/// O destino de `[x.y]` / `[nome]`: a porta do Ctrl+clique.
fn link_anchor(inner: &str) -> Option<Anchor> {
    let (qualifier, word) = match inner.rsplit_once('.') {
        Some((q, w)) => (Some(q), w),
        None => (None, inner),
    };
    resolve(&Query {
        word,
        qualifier,
        colon: false,
        hover: None,
    })
}

fn is_link_text(s: &str) -> bool {
    !s.is_empty()
        && s.split('.').all(|p| {
            !p.is_empty()
                && p.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !p.starts_with(|c: char| c.is_ascii_digit())
        })
}

/// Um parágrafo do texto: `` `código` `` → mono; `[x.y]` → link; «Note:» no início a negrito.
fn markup(b: &mut Builder, para: &str, base: SpanKind) -> Vec<Span> {
    let mut out = Vec::new();
    let mut rest = para;
    // «Note:» (o texto do catálogo é inglês, decisão de 17/10) a negrito, como no Godot.
    let note = "Note:".len();
    if para.starts_with("Note:") {
        out.push(Span::new(&para[..note], SpanKind::Strong));
        rest = &para[note..];
    }
    let mut prose = String::new();
    let flush = |prose: &mut String, out: &mut Vec<Span>| {
        if !prose.is_empty() {
            out.push(Span::new(std::mem::take(prose), base));
        }
    };
    while let Some(c) = rest.chars().next() {
        if c == '`'
            && let Some(end) = rest[1..].find('`')
        {
            flush(&mut prose, &mut out);
            out.push(Span::new(&rest[1..=end], SpanKind::Code(Ink::Plain)));
            rest = &rest[end + 2..];
            continue;
        }
        if c == '['
            && let Some(end) = rest[1..].find(']')
            && is_link_text(&rest[1..=end])
        {
            let inner = &rest[1..=end];
            flush(&mut prose, &mut out);
            let span = Span::new(inner, SpanKind::Code(Ink::Function));
            out.push(match link_anchor(inner) {
                Some(a) => span.link(b.number(Target::Anchor(a))),
                None => {
                    b.dead.push(inner.to_owned());
                    span
                }
            });
            rest = &rest[end + 2..];
            continue;
        }
        prose.push(c);
        rest = &rest[c.len_utf8()..];
    }
    flush(&mut prose, &mut out);
    out
}

fn paragraphs(text: &str) -> impl Iterator<Item = &str> {
    text.split("\n\n").map(str::trim).filter(|p| !p.is_empty())
}

/// `(args)` e o retorno de uma assinatura (`<T>(a: T): R` → `("<T>(a: T)", "R")`); sem retorno = `""`.
pub(crate) fn split_sig(sig: &str) -> (&str, &str) {
    let mut depth = 0i32;
    let bytes = sig.as_bytes();
    let mut opened = false;
    for (i, c) in sig.char_indices() {
        match c {
            '(' | '{' | '[' => {
                depth += 1;
                opened |= c == '(';
            }
            '<' => depth += 1,
            '>' if i > 0 && bytes[i - 1] == b'-' => {}
            ')' | '}' | ']' | '>' => {
                depth -= 1;
                if depth == 0 && c == ')' && opened {
                    let ret = sig[i + 1..].trim_start();
                    let ret = ret.strip_prefix(':').map_or("", str::trim);
                    return (&sig[..=i], ret);
                }
            }
            _ => {}
        }
    }
    (sig, "")
}

/// O texto Luau `src` realçado pelo MESMO `ph2d-code-syntax` do editor, só a faixa `keep`
/// (deslocada para começar em 0), com os nomes de tipo que têm página como links.
fn highlight(b: &mut Builder, src: &str, keep: std::ops::Range<usize>) -> Vec<Span> {
    let text = Text::new(src);
    let spans = Syntax::new(&text).highlights(&text, keep.clone());
    let mut out = Vec::new();
    let mut at = keep.start;
    let mut push = |out: &mut Vec<Span>, r: std::ops::Range<usize>, ink: Ink| {
        if r.is_empty() {
            return;
        }
        let s = &src[r];
        let mut span = Span::new(s, SpanKind::Code(ink));
        if ink == Ink::Type
            && let Some(a) = type_anchor(s)
        {
            span = span.link(b.number(Target::Anchor(a)));
        }
        out.push(span);
    };
    for h in spans {
        let r = h.range.start.max(keep.start)..h.range.end.min(keep.end);
        if r.start < at || r.is_empty() {
            continue;
        }
        push(&mut out, at..r.start, Ink::Plain);
        push(&mut out, r.clone(), crate::problems::ink_of(h.kind));
        at = r.end;
    }
    push(&mut out, at..keep.end, Ink::Plain);
    out
}

/// O nome de tipo `s` tem página (`PoseField`, `Entity`) ou é uma biblioteca.
fn type_anchor(s: &str) -> Option<Anchor> {
    ph2d_code_help::anchor("types", Some(s)).or_else(|| ph2d_code_help::anchor(s, None))
}

/// Luau de uma linha: a assinatura do membro tal como o script a escreve.
fn sig_spans(b: &mut Builder, page: &Page, m: &Member, sig: &str) -> Vec<Span> {
    let lib = if matches!(page.id, "globals" | "types" | "hooks") {
        String::new()
    } else {
        format!("{}.", page.id)
    };
    match m.kind {
        Kind::Function | Kind::Hook => {
            let head = format!("function {lib}{}", m.name);
            let src = format!("{head}{sig}\nend");
            let keep = "function ".len()..head.len() + sig.len();
            highlight(b, &src, keep)
        }
        Kind::Type => {
            let src = format!("type {} = {sig}", m.name);
            let len = src.len();
            highlight(b, &src, 0..len)
        }
        Kind::Constant => {
            let mut out = vec![Span::new(
                format!("{lib}{}", m.name),
                SpanKind::Code(Ink::Constant),
            )];
            out.push(Span::new(": ", SpanKind::Code(Ink::Punctuation)));
            let src = format!("type _ = {sig}");
            let keep = "type _ = ".len()..src.len();
            out.extend(highlight(b, &src, keep));
            out
        }
    }
}

/// A linha da tabela das irmãs: o retorno (ou o tipo) e o nome, que leva à descrição.
fn summary(b: &mut Builder, page: &Page, m: &Member) -> Block {
    let sig = m.sigs.first().map_or("", String::as_str);
    let to = Target::Anchor(Anchor {
        page: page.id,
        member: Some(m.name),
    });
    let name = Span::new(m.name, SpanKind::Code(Ink::Function)).link(b.number(to));
    let (ret, call) = match m.kind {
        Kind::Function | Kind::Hook => {
            let (args, ret) = split_sig(sig);
            let ret = if ret.is_empty() {
                Vec::new()
            } else {
                let src = format!("type _ = {ret}");
                highlight(b, &src, "type _ = ".len()..src.len())
            };
            (ret, vec![name, Span::new(args, SpanKind::Code(Ink::Plain))])
        }
        Kind::Constant => {
            let src = format!("type _ = {sig}");
            (highlight(b, &src, "type _ = ".len()..src.len()), vec![name])
        }
        Kind::Type => (
            vec![Span::new("type", SpanKind::Code(Ink::Keyword))],
            vec![name],
        ),
    };
    Block::Summary { ret, call }
}

/// O corpo de um membro: a assinatura, o texto, os argumentos, o retorno, o exemplo.
fn describe(b: &mut Builder, page: &Page, i: usize, m: &Member, first: bool) {
    for (k, sig) in m.sigs.iter().enumerate() {
        let spans = sig_spans(b, page, m, sig);
        b.blocks.push(Block::Sig {
            spans,
            member: i as u32,
            first: first || k > 0,
        });
    }
    for p in paragraphs(&m.text) {
        let spans = markup(b, p, SpanKind::Prose);
        b.blocks.push(Block::Body(spans));
    }
    if !m.params.is_empty() {
        b.blocks.push(Block::Body(vec![Span::new(
            tr("code.help.page.parameters"),
            SpanKind::Strong,
        )]));
    }
    for p in &m.params {
        let mut spans = vec![
            Span::new(p.name.as_str(), SpanKind::Code(Ink::Parameter)),
            Span::new(tr("code.help.page.dash"), SpanKind::Muted),
        ];
        spans.extend(markup(b, &p.text, SpanKind::Prose));
        b.blocks.push(Block::Body(spans));
    }
    if let Some(r) = &m.returns {
        let mut spans = vec![
            Span::new(tr("code.help.page.returns"), SpanKind::Strong),
            Span::new(tr("code.help.page.dash"), SpanKind::Muted),
        ];
        spans.extend(markup(b, r, SpanKind::Prose));
        b.blocks.push(Block::Body(spans));
    }
    if let Some(ex) = m.example {
        b.blocks.push(Block::Body(vec![Span::new(
            tr("code.help.page.example"),
            SpanKind::Strong,
        )]));
        let copy = b.number(Target::Example(i));
        let all = highlight(b, ex, 0..ex.len());
        b.blocks.push(Block::Code {
            lines: split_lines(all),
            copy,
        });
    }
}

/// Os trechos de um texto de várias linhas, partidos em linhas (um trecho que atravessa `\n` parte-se).
fn split_lines(spans: Vec<Span>) -> Vec<Vec<Span>> {
    let mut lines = vec![Vec::new()];
    for s in spans {
        let mut parts = s.text.split('\n').peekable();
        while let Some(part) = parts.next() {
            if !part.is_empty() {
                let mut piece = Span::new(part, s.kind);
                piece.link = s.link;
                lines.last_mut().expect("uma linha").push(piece);
            }
            if parts.peek().is_some() {
                lines.push(Vec::new());
            }
        }
    }
    lines
}

/// As secções de membros: (espécie, rótulo da tabela, rótulo das descrições).
const SECTIONS: [(Kind, &str, &str); 4] = [
    (
        Kind::Function,
        "code.help.page.functions",
        "code.help.page.function_descriptions",
    ),
    (
        Kind::Hook,
        "code.help.page.hooks",
        "code.help.page.hook_descriptions",
    ),
    (
        Kind::Type,
        "code.help.page.types",
        "code.help.page.type_descriptions",
    ),
    (
        Kind::Constant,
        "code.help.page.constants",
        "code.help.page.constant_descriptions",
    ),
];

fn build_into(page: &Page) -> Builder {
    let mut b = Builder {
        blocks: vec![Block::Title(page.title.clone())],
        targets: Vec::new(),
        dead: Vec::new(),
    };
    b.heading("code.help.page.description");
    let strong = markup(&mut b, &page.summary, SpanKind::Strong);
    b.blocks.push(Block::Strong(strong));
    for p in paragraphs(&page.about) {
        let spans = markup(&mut b, p, SpanKind::Prose);
        b.blocks.push(Block::Para(spans));
    }
    let of = |k: Kind| {
        page.members
            .iter()
            .enumerate()
            .filter(move |m| m.1.kind == k)
    };
    for (kind, table, _) in SECTIONS {
        if of(kind).next().is_none() {
            continue;
        }
        b.heading(table);
        for (_, m) in of(kind) {
            let row = summary(&mut b, page, m);
            b.blocks.push(row);
        }
    }
    for (kind, _, descriptions) in SECTIONS {
        if of(kind).next().is_none() {
            continue;
        }
        b.heading(descriptions);
        for (n, (i, m)) in of(kind).enumerate() {
            describe(&mut b, page, i, m, n == 0);
        }
    }
    b
}

/// A página `page` pronta a pintar.
pub(crate) fn build(page: &Page) -> Built {
    let b = build_into(page);
    let sections = b
        .blocks
        .iter()
        .filter_map(|x| match x {
            Block::Heading(s) => Some(s.clone()),
            _ => None,
        })
        .collect();
    let mut h = std::hash::DefaultHasher::new();
    std::hash::Hash::hash(&(page.id, format!("{:?}", ph2d_i18n::idioma())), &mut h);
    Built {
        doc: HelpDoc {
            key: std::hash::Hasher::finish(&h),
            title: page.title.clone(),
            blocks: b.blocks,
            sections,
        },
        targets: b.targets,
    }
}

/// ⚠️ GATE: cada `[…]` de cada texto do catálogo que NÃO resolve pela porta do Ctrl+clique.
#[must_use]
pub fn dead_links() -> Vec<DeadLink> {
    let mut out = Vec::new();
    for page in ph2d_code_help::catalog() {
        let mut b = Builder {
            blocks: Vec::new(),
            targets: Vec::new(),
            dead: Vec::new(),
        };
        let _ = markup(&mut b, &page.summary, SpanKind::Prose);
        let _ = markup(&mut b, &page.about, SpanKind::Prose);
        out.extend(b.dead.drain(..).map(|link| DeadLink {
            page: page.id,
            member: None,
            link,
        }));
        for m in &page.members {
            let texts = std::iter::once(m.text.as_str())
                .chain(m.params.iter().map(|p| p.text.as_str()))
                .chain(m.returns.as_deref());
            for t in texts {
                let _ = markup(&mut b, t, SpanKind::Prose);
            }
            out.extend(b.dead.drain(..).map(|link| DeadLink {
                page: page.id,
                member: Some(m.name),
                link,
            }));
        }
    }
    out
}

/// Quantos `[…]` o catálogo tem ao todo (o controlo do gate: ele não mede o nada).
#[must_use]
pub fn link_count() -> usize {
    ph2d_code_help::catalog()
        .iter()
        .map(|p| build(p).doc.blocks)
        .flat_map(|blocks| blocks.into_iter())
        .map(|b| match b {
            Block::Para(s) | Block::Strong(s) | Block::Body(s) => {
                s.iter().filter(|x| x.link.is_some()).count()
            }
            _ => 0,
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::split_sig;

    #[test]
    fn a_assinatura_parte_nos_argumentos_e_no_retorno() {
        assert_eq!(split_sig("(n: number): number"), ("(n: number)", "number"));
        assert_eq!(
            split_sig("<A..., R...>(f: (A...) -> (R...)): thread"),
            ("<A..., R...>(f: (A...) -> (R...))", "thread")
        );
        assert_eq!(
            split_sig("(entity: Entity, field: PoseField, value: number)"),
            ("(entity: Entity, field: PoseField, value: number)", "")
        );
        assert_eq!(
            split_sig("(co: thread): (boolean, any)"),
            ("(co: thread)", "(boolean, any)")
        );
    }
}
