//! `doc → md → doc`: o que o editor faz chega ao ficheiro e volta.

use ph2d_kanban_md::{parse, render};
use ph2d_richdoc::{Block, BlockKind as K, Inline, Mark, RichDoc};

fn kinds(d: &RichDoc) -> Vec<(K, usize, String)> {
    d.blocks()
        .iter()
        .map(|b| (b.kind.clone(), b.indent, b.inline.text().to_owned()))
        .collect()
}

#[test]
fn doc_md_parse_reads_every_block_kind_and_nesting() {
    let d = parse("# T\n\n- a\n  - [x] b\n\n  p\n1. n\n\n> q\n\n```rs\nx\n```\n\n---\n");
    assert_eq!(
        kinds(&d),
        vec![
            (K::Heading(1), 0, "T".into()),
            (K::Bullet, 0, "a".into()),
            (K::Check(true), 1, "b".into()),
            (K::Paragraph, 1, "p".into()),
            (K::Numbered, 0, "n".into()),
            (K::Quote, 0, "q".into()),
            (K::Code("rs".into()), 0, "x".into()),
            (K::Rule, 0, String::new()),
        ]
    );
}

#[test]
fn doc_md_parse_reads_marks_links_and_mentions_outside_code_and_links() {
    let d = parse("**b** *i* ~~s~~ `c` [l](u) @ana x@y `@no` [@no](v)");
    let inline = &d.blocks()[0].inline;
    let got: Vec<(String, Mark)> = inline
        .marks()
        .iter()
        .map(|s| (inline.text()[s.start..s.end].to_owned(), s.mark.clone()))
        .collect();
    let want = [
        ("b", Mark::Bold),
        ("i", Mark::Italic),
        ("s", Mark::Strike),
        ("c", Mark::Code),
        ("l", Mark::Link("u".into())),
        ("@ana", Mark::Mention),
        ("@no", Mark::Code),
        ("@no", Mark::Link("v".into())),
    ];
    let want: Vec<(String, Mark)> = want.into_iter().map(|(t, m)| (t.to_owned(), m)).collect();
    assert_eq!(got, want);
}

#[test]
fn doc_md_render_escapes_what_would_otherwise_become_markdown() {
    let d = RichDoc::from_blocks(vec![
        Block::paragraph("# not a title"),
        Block::paragraph("1. not a list *nor* [link](x) a_b _c_ <tag> &amp;"),
        Block::new(K::Heading(2), Inline::plain("issue #12 #")),
    ]);
    let md = render(&d);
    assert_eq!(
        md,
        "\\# not a title\n\n1\\. not a list \\*nor\\* \\[link\\](x) a_b \\_c\\_ \\<tag> \\&amp;\n\n\
         ## issue \\#12 \\#\n"
    );
    assert_eq!(kinds(&parse(&md)), kinds(&d));
}

#[test]
fn doc_md_render_writes_marks_nested_and_links_with_parentheses() {
    let mut i = Inline::plain("bold and italic code link");
    i.add_mark(0..15, Mark::Bold);
    i.add_mark(9..15, Mark::Italic);
    i.add_mark(16..20, Mark::Code);
    i.add_mark(21..25, Mark::Link("https://x.org/a_(b)".into()));
    let d = RichDoc::from_blocks(vec![Block::new(K::Paragraph, i.clone())]);
    let md = render(&d);
    assert_eq!(
        md,
        "**bold and *italic*** `code` [link](<https://x.org/a_(b)>)\n"
    );
    assert_eq!(parse(&md).blocks()[0].inline, i);
}

#[test]
fn doc_md_whitespace_inside_emphasis_edges_moves_outside_and_unreadable_emphasis_drops() {
    let mut i = Inline::plain("a b c x\"q\"y");
    i.add_mark(1..4, Mark::Bold); // " b " → `**b**` com os espaços fora
    i.add_mark(7..10, Mark::Italic); // entre letras e aspas o CommonMark não lê
    let md = render(&RichDoc::from_blocks(vec![Block::new(K::Paragraph, i)]));
    assert_eq!(md, "a **b** c x\"q\"y\n");
}

#[test]
fn doc_md_line_breaks_survive_inside_paragraphs_items_and_quotes() {
    let d = RichDoc::from_blocks(vec![
        Block::paragraph("a\nb"),
        Block::new(K::Bullet, Inline::plain("c\nd")),
        Block::new(K::Quote, Inline::plain("e\nf")),
    ]);
    let md = render(&d);
    assert_eq!(md, "a\\\nb\n\n- c\\\n  d\n\n> e\\\n> f\n");
    assert_eq!(kinds(&parse(&md)), kinds(&d));
}

#[test]
fn doc_md_empty_doc_and_empty_paragraphs() {
    assert_eq!(render(&RichDoc::new()), "");
    assert!(parse("").is_empty());
    let d = RichDoc::from_blocks(vec![
        Block::paragraph("a"),
        Block::paragraph(" "),
        Block::paragraph("b"),
    ]);
    assert_eq!(render(&d), "a\n\nb\n");
}

/// Gerador determinístico (LCG) de documentos com todos os blocos, marcas e caracteres maus.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as usize) % n
    }
}

fn random_doc(r: &mut Lcg) -> RichDoc {
    const ALPHABET: &[&str] = &[
        "a", "b", "z", " ", " ", "*", "_", "#", "-", "+", "1", ".", ")", "[", "]", "(", "`", "~",
        "\\", "<", ">", "&", "@", "é", "\n", "!", "\"", "|", "=",
    ];
    let marks = [
        Mark::Bold,
        Mark::Italic,
        Mark::Strike,
        Mark::Code,
        Mark::Link("u".into()),
        Mark::Link("v w".into()),
        Mark::Mention,
    ];
    let mut blocks = Vec::new();
    for _ in 0..1 + r.next(6) {
        let kind = match r.next(10) {
            0 => K::heading(1 + r.next(3) as u8),
            1 => K::Bullet,
            2 => K::Numbered,
            3 => K::Check(r.next(2) == 0),
            4 => K::Quote,
            5 => K::Code(["", "rs"][r.next(2)].into()),
            6 => K::Rule,
            _ => K::Paragraph,
        };
        let mut text = String::new();
        for _ in 0..r.next(12) {
            text.push_str(ALPHABET[r.next(ALPHABET.len())]);
        }
        let mut inline = Inline::plain(text.clone());
        let chars: Vec<usize> = text
            .char_indices()
            .map(|(i, _)| i)
            .chain([text.len()])
            .collect();
        for _ in 0..r.next(4) {
            let a = chars[r.next(chars.len())];
            let b = chars[r.next(chars.len())];
            inline.add_mark(a.min(b)..a.max(b), marks[r.next(marks.len())].clone());
        }
        blocks.push(Block::new(kind, inline).with_indent(r.next(3)));
    }
    RichDoc::from_blocks(blocks)
}

#[test]
fn doc_md_random_documents_render_to_a_fixed_point() {
    // `PH2D_MD_FUZZ=<n>` corre mais documentos (o portão corre 20 000 em ~0,5 s).
    let count = std::env::var("PH2D_MD_FUZZ")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20_000);
    let mut r = Lcg(0x5EED);
    for n in 0..count {
        let doc = random_doc(&mut r);
        let md1 = render(&doc);
        let md2 = render(&parse(&md1));
        assert_eq!(md1, md2, "documento {n}: {doc:#?}");
    }
}
