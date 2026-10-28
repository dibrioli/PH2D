//! markdown → `RichDoc`. O que o modelo não tem (tabelas, HTML, imagens) entra como TEXTO literal.

use ph2d_richdoc::{Block, BlockKind, Inline, Mark, RichDoc};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// Lê markdown (CommonMark + tarefas + riscado).
pub fn parse(md: &str) -> RichDoc {
    let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut b = Builder::default();
    for ev in Parser::new_ext(md, opts) {
        b.event(ev);
    }
    b.flush();
    RichDoc::from_blocks(b.blocks)
}

#[derive(Default)]
struct Builder {
    blocks: Vec<Block>,
    cur: Option<Block>,
    marks: Vec<Mark>,
    /// Itens abertos: `true` enquanto o item ainda aceita o seu texto.
    items: Vec<bool>,
    /// Listas abertas: `true` = numerada.
    lists: Vec<bool>,
    quote: usize,
    images: Vec<String>,
    raw: bool,
}

impl Builder {
    fn event(&mut self, ev: Event<'_>) {
        match ev {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(t) => self.text(&t),
            Event::Code(t) => {
                let mut m = self.marks.clone();
                m.push(Mark::Code);
                self.ensure_cur().inline.push(&t, &m);
            }
            Event::Html(t) | Event::InlineHtml(t) => self.text(&t),
            Event::SoftBreak | Event::HardBreak => self.text("\n"),
            Event::Rule => {
                self.new_block(BlockKind::Rule);
                self.flush();
            }
            Event::TaskListMarker(done) => {
                if let Some(c) = self.cur.as_mut().filter(|c| c.kind.is_list_item()) {
                    c.kind = BlockKind::Check(done);
                }
            }
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {
                let item_slot = self.items.last() == Some(&true)
                    && self.cur.as_ref().is_some_and(|c| c.kind.is_list_item());
                if item_slot {
                    if let Some(s) = self.items.last_mut() {
                        *s = false;
                    }
                } else {
                    let kind = self.para_kind();
                    self.new_block(kind);
                }
            }
            Tag::Heading { level, .. } => self.new_block(BlockKind::heading(heading_level(level))),
            Tag::BlockQuote(_) => {
                self.close_block();
                self.quote += 1;
            }
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(info) => {
                        info.split_whitespace().next().unwrap_or("").to_owned()
                    }
                    CodeBlockKind::Indented => String::new(),
                };
                self.new_block(BlockKind::Code(lang));
                self.raw = true;
            }
            Tag::HtmlBlock => {
                let kind = self.para_kind();
                self.new_block(kind);
                self.raw = true;
            }
            Tag::List(start) => {
                self.close_block();
                self.lists.push(start.is_some());
            }
            Tag::Item => {
                let numbered = self.lists.last() == Some(&true);
                self.new_block(if numbered {
                    BlockKind::Numbered
                } else {
                    BlockKind::Bullet
                });
                self.items.push(true);
            }
            Tag::Emphasis => self.marks.push(Mark::Italic),
            Tag::Strong => self.marks.push(Mark::Bold),
            Tag::Strikethrough => self.marks.push(Mark::Strike),
            Tag::Link { dest_url, .. } => self.marks.push(Mark::Link(dest_url.into_string())),
            Tag::Image {
                dest_url, title, ..
            } => {
                self.text("![");
                let title = if title.is_empty() {
                    String::new()
                } else {
                    format!(" \"{title}\"")
                };
                self.images.push(format!("{dest_url}{title}"));
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph | TagEnd::Heading(_) => self.flush(),
            TagEnd::CodeBlock | TagEnd::HtmlBlock => {
                if let Some(c) = self.cur.as_mut() {
                    let t = c.inline.text();
                    // O código acaba sempre com UMA `\n` do leitor; o resto é do autor.
                    let keep = if c.kind.is_code() {
                        t.strip_suffix('\n').unwrap_or(t).len()
                    } else {
                        t.trim_end_matches('\n').len()
                    };
                    let len = t.len();
                    c.inline.delete(keep..len);
                }
                self.raw = false;
                self.flush();
            }
            TagEnd::BlockQuote(_) => {
                self.flush();
                self.quote = self.quote.saturating_sub(1);
            }
            TagEnd::List(_) => {
                self.flush();
                self.lists.pop();
            }
            TagEnd::Item => {
                self.flush();
                self.items.pop();
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link => {
                self.marks.pop();
            }
            TagEnd::Image => {
                let dest = self.images.pop().unwrap_or_default();
                self.text(&format!("]({dest})"));
            }
            _ => {}
        }
    }

    fn text(&mut self, t: &str) {
        let marks = if self.raw {
            Vec::new()
        } else {
            self.marks.clone()
        };
        self.ensure_cur().inline.push(t, &marks);
    }

    fn para_kind(&self) -> BlockKind {
        if self.quote > 0 {
            BlockKind::Quote
        } else {
            BlockKind::Paragraph
        }
    }

    fn ensure_cur(&mut self) -> &mut Block {
        if self.cur.is_none() {
            let kind = self.para_kind();
            self.new_block(kind);
        }
        self.cur.as_mut().expect("bloco corrente")
    }

    /// Um bloco novo fecha o anterior e tira ao item aberto a vaga do seu texto.
    fn new_block(&mut self, kind: BlockKind) {
        self.close_block();
        let indent = self.items.len();
        self.cur = Some(Block::new(kind, Inline::default()).with_indent(indent));
    }

    fn close_block(&mut self) {
        self.flush();
        if let Some(s) = self.items.last_mut() {
            *s = false;
        }
    }

    fn flush(&mut self) {
        if let Some(mut b) = self.cur.take() {
            if !b.kind.is_code() {
                mark_mentions(&mut b.inline);
            }
            self.blocks.push(b);
        }
    }
}

fn heading_level(l: HeadingLevel) -> u8 {
    match l {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        _ => 3,
    }
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '.')
}

/// `@nome` fora de código e de links vira menção (o nome acaba antes de `.`/`-` finais).
fn mark_mentions(inline: &mut Inline) {
    let text = inline.text().to_owned();
    let mut found = Vec::new();
    let mut prev: Option<char> = None;
    for (i, c) in text.char_indices() {
        let starts = c == '@' && !prev.is_some_and(|p| p.is_alphanumeric() || p == '_' || p == '@');
        prev = Some(c);
        if !starts {
            continue;
        }
        let rest = &text[i + 1..];
        let len = rest.find(|ch| !is_name_char(ch)).unwrap_or(rest.len());
        let name = rest[..len].trim_end_matches(['.', '-']);
        if name.is_empty() {
            continue;
        }
        let end = i + 1 + name.len();
        let inside = (i..end).any(|p| {
            inline
                .marks_on_byte(p)
                .iter()
                .any(|m| matches!(m, Mark::Code | Mark::Link(_)))
        });
        if !inside {
            found.push(i..end);
        }
    }
    for r in found {
        inline.add_mark(r, Mark::Mention);
    }
}
