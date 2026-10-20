//! A disposição de uma página de ajuda: blocos → trechos posicionados (coordenadas da PÁGINA, 0 no
//! topo do conteúdo). Pura: as larguras vêm de um [`Measure`] (a fonte de verdade na pintura, um
//! falso nos testes). A prosa parte nos espaços; o código dos exemplos não parte (corta-se).

use super::model::{Block, Span, SpanKind};
use crate::style::Ink;
use std::ops::Range;

/// A letra de um trecho.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Font {
    Prose,
    Strong,
    Mono,
    Title,
    Heading,
}

/// A cor de um trecho.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tone {
    Text,
    Muted,
    Heading,
    Code(Ink),
}

/// As larguras: a prosa (`bold`) mede-se na fonte, o mono é colunas × avanço.
pub(crate) trait Measure {
    fn prose(&mut self, s: &str, font: Font) -> f32;
    fn mono(&self, s: &str) -> f32;
}

/// As medidas de uma página (todas por token, resolvidas pela vista).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Dims {
    pub width: f32,
    pub prose_lh: f32,
    pub mono_lh: f32,
    pub title_lh: f32,
    pub head_lh: f32,
    /// Entre parágrafos.
    pub gap: f32,
    /// O recuo do corpo de um membro.
    pub indent: f32,
    /// O enchimento da caixa de um exemplo.
    pub pad: f32,
    /// O lado do botão de copiar.
    pub button: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Run {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub text: String,
    pub font: Font,
    pub tone: Tone,
    pub link: Option<u32>,
    /// Onde o texto começa em [`Laid::plain`].
    pub at: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Shape {
    /// O fundo de um exemplo.
    CodeBox,
    /// A linha entre dois membros.
    Rule,
    /// O ponto antes de uma assinatura.
    Bullet,
    /// O botão de copiar o exemplo `n`.
    Copy(u32),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Laid {
    pub height: f32,
    pub runs: Vec<Run>,
    pub shapes: Vec<(Shape, [f32; 4])>,
    /// `(membro, y)` de cada assinatura.
    pub anchors: Vec<(u32, f32)>,
    /// O `y` de cada título de secção, pela ordem.
    pub sections: Vec<f32>,
    /// O texto da página, linha a linha (a busca, a cópia, o leitor de ecrã).
    pub plain: String,
    /// O texto de cada exemplo em [`Laid::plain`] (o botão de copiar).
    pub examples: Vec<(u32, Range<usize>)>,
    /// Cada bloco com texto, pela ordem (o leitor de ecrã).
    pub paras: Vec<Para>,
}

/// O que um bloco é ao leitor de ecrã.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParaKind {
    /// Um título, com o nível (1 = a página, 2 = secção, 3 = assinatura).
    Heading(u8),
    Text,
    /// Um exemplo, com o número do botão de copiar.
    Code(u32),
}

/// Um bloco disposto: os trechos dele, o texto em [`Laid::plain`] e a faixa vertical.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Para {
    pub kind: ParaKind,
    pub runs: Range<usize>,
    pub text: Range<usize>,
    pub y: f32,
    pub h: f32,
}

fn para_kind(b: &Block) -> Option<ParaKind> {
    Some(match b {
        Block::Title(_) => ParaKind::Heading(1),
        Block::Heading(_) => ParaKind::Heading(2),
        Block::Sig { .. } => ParaKind::Heading(3),
        Block::Para(_) | Block::Strong(_) | Block::Body(_) | Block::Summary { .. } => {
            ParaKind::Text
        }
        Block::Code { copy, .. } => ParaKind::Code(*copy),
        Block::Gap => return None,
    })
}

/// Um pedaço que não se parte: uma palavra com o espaço que a segue.
struct Piece<'a> {
    text: &'a str,
    font: Font,
    tone: Tone,
    link: Option<u32>,
}

fn font_of(k: SpanKind, strong: bool) -> (Font, Tone) {
    match k {
        SpanKind::Prose if strong => (Font::Strong, Tone::Text),
        SpanKind::Prose => (Font::Prose, Tone::Text),
        SpanKind::Strong => (Font::Strong, Tone::Text),
        SpanKind::Muted => (Font::Prose, Tone::Muted),
        SpanKind::Code(ink) => (Font::Mono, Tone::Code(ink)),
    }
}

fn pieces(spans: &[Span], strong: bool) -> Vec<Piece<'_>> {
    let mut out = Vec::new();
    for s in spans {
        let (font, tone) = font_of(s.kind, strong);
        let mut start = 0;
        for (i, c) in s.text.char_indices() {
            if c == ' ' {
                out.push(Piece {
                    text: &s.text[start..=i],
                    font,
                    tone,
                    link: s.link,
                });
                start = i + 1;
            }
        }
        if start < s.text.len() {
            out.push(Piece {
                text: &s.text[start..],
                font,
                tone,
                link: s.link,
            });
        }
    }
    out
}

struct Lay<'m, M: Measure> {
    d: Dims,
    m: &'m mut M,
    out: Laid,
    y: f32,
}

impl<M: Measure> Lay<'_, M> {
    fn width(&mut self, s: &str, f: Font) -> f32 {
        if f == Font::Mono {
            self.m.mono(s)
        } else {
            self.m.prose(s, f)
        }
    }

    fn line_h(&self, f: Font) -> f32 {
        match f {
            Font::Mono => self.d.mono_lh,
            Font::Title => self.d.title_lh,
            Font::Heading => self.d.head_lh,
            Font::Prose | Font::Strong => self.d.prose_lh,
        }
    }

    fn newline(&mut self) {
        self.out.plain.push('\n');
    }

    /// Os pedaços partidos em linhas entre `x0` e `x0 + w`; os vizinhos iguais juntam-se num trecho.
    fn flow(&mut self, ps: &[Piece<'_>], x0: f32, w: f32) {
        let lh = ps
            .iter()
            .map(|p| self.line_h(p.font))
            .fold(self.d.prose_lh, f32::max);
        let mut x = x0;
        let mut line: Vec<Run> = Vec::new();
        for p in ps {
            let pw = self.width(p.text.trim_end(), p.font);
            if x > x0 && x + pw > x0 + w {
                self.commit(&mut line, lh);
                x = x0;
            }
            match line.last_mut() {
                Some(r) if r.font == p.font && r.tone == p.tone && r.link == p.link => {
                    r.text.push_str(p.text)
                }
                _ => line.push(Run {
                    x,
                    y: self.y,
                    w: 0.0,
                    h: lh,
                    text: p.text.to_owned(),
                    font: p.font,
                    tone: p.tone,
                    link: p.link,
                    at: 0,
                }),
            }
            x += self.width(p.text, p.font);
        }
        self.commit(&mut line, lh);
    }

    fn commit(&mut self, line: &mut Vec<Run>, lh: f32) {
        if line.is_empty() {
            return;
        }
        if let Some(last) = line.last_mut() {
            let t = last.text.trim_end().len();
            last.text.truncate(t);
        }
        for mut r in line.drain(..) {
            r.w = self.width(&r.text, r.font);
            r.y = self.y;
            r.at = self.out.plain.len();
            self.out.plain.push_str(&r.text);
            self.out.runs.push(r);
        }
        self.newline();
        self.y += lh;
    }

    fn block(&mut self, b: &Block, ret_col: f32) {
        let d = self.d;
        match b {
            Block::Title(t) => {
                self.flow(
                    &[Piece {
                        text: t,
                        font: Font::Title,
                        tone: Tone::Text,
                        link: None,
                    }],
                    0.0,
                    d.width,
                );
                self.y += d.gap;
            }
            Block::Heading(t) => {
                self.y += d.gap;
                self.out.sections.push(self.y);
                self.flow(
                    &[Piece {
                        text: t,
                        font: Font::Heading,
                        tone: Tone::Heading,
                        link: None,
                    }],
                    0.0,
                    d.width,
                );
                self.y += d.gap * 0.5;
            }
            Block::Para(s) | Block::Strong(s) => {
                self.flow(&pieces(s, matches!(b, Block::Strong(_))), 0.0, d.width);
                self.y += d.gap;
            }
            Block::Body(s) => {
                self.flow(&pieces(s, false), d.indent, d.width - d.indent);
                self.y += d.gap;
            }
            Block::Sig {
                spans,
                member,
                first,
            } => {
                if !first {
                    self.out
                        .shapes
                        .push((Shape::Rule, [0.0, self.y, d.width, 0.0]));
                    self.y += d.gap;
                }
                self.out.anchors.push((*member, self.y));
                let dot = d.mono_lh * 0.5;
                self.out
                    .shapes
                    .push((Shape::Bullet, [dot * 0.5, self.y + dot * 0.5, dot, dot]));
                self.flow(&pieces(spans, false), dot * 2.0, d.width - dot * 2.0);
                self.y += d.gap * 0.5;
            }
            Block::Summary { ret, call } => {
                let rw: f32 = ret.iter().map(|s| self.width(&s.text, Font::Mono)).sum();
                let y = self.y;
                let mut x = ret_col - rw;
                for s in ret {
                    let (font, tone) = font_of(s.kind, false);
                    let w = self.width(&s.text, font);
                    self.out.runs.push(Run {
                        x,
                        y,
                        w,
                        h: d.mono_lh,
                        text: s.text.clone(),
                        font,
                        tone,
                        link: s.link,
                        at: self.out.plain.len(),
                    });
                    self.out.plain.push_str(&s.text);
                    x += w;
                }
                self.out.plain.push(' ');
                let x0 = ret_col + d.pad;
                self.flow(&pieces(call, false), x0, d.width - x0);
            }
            Block::Code { lines, copy } => self.code(lines, *copy),
            Block::Gap => self.y += d.gap,
        }
    }

    fn code(&mut self, lines: &[Vec<Span>], copy: u32) {
        let d = self.d;
        let (x0, top) = (d.indent, self.y);
        let h = lines.len() as f32 * d.mono_lh + 2.0 * d.pad;
        self.out
            .shapes
            .push((Shape::CodeBox, [x0, top, d.width - x0, h]));
        self.out.shapes.push((
            Shape::Copy(copy),
            [d.width - d.pad - d.button, top + d.pad, d.button, d.button],
        ));
        self.y += d.pad;
        let start = self.out.plain.len();
        for l in lines {
            let mut x = x0 + d.pad;
            for s in l {
                let (font, tone) = font_of(s.kind, false);
                let w = self.width(&s.text, font);
                self.out.runs.push(Run {
                    x,
                    y: self.y,
                    w,
                    h: d.mono_lh,
                    text: s.text.clone(),
                    font,
                    tone,
                    link: s.link,
                    at: self.out.plain.len(),
                });
                self.out.plain.push_str(&s.text);
                x += w;
            }
            self.newline();
            self.y += d.mono_lh;
        }
        let end = self.out.plain.len().saturating_sub(1).max(start);
        self.out.examples.push((copy, start..end));
        self.y = top + h + d.gap;
    }
}

/// A página `blocks` na largura `d.width`.
pub(crate) fn lay_out<M: Measure>(blocks: &[Block], d: Dims, m: &mut M) -> Laid {
    // a coluna dos tipos de retorno da tabela: a do mais largo (alinhados à direita, como no Godot)
    let ret_col = blocks
        .iter()
        .filter_map(|b| match b {
            Block::Summary { ret, .. } => Some(ret.iter().map(|s| m.mono(&s.text)).sum::<f32>()),
            _ => None,
        })
        .fold(0.0_f32, f32::max)
        .min(d.width * 0.4); // LITERAL-PX-OK: a coluna dos retornos fica em 40 % da página
    let mut lay = Lay {
        d,
        m,
        out: Laid::default(),
        y: 0.0,
    };
    for b in blocks {
        let (r0, t0) = (lay.out.runs.len(), lay.out.plain.len());
        lay.block(b, d.indent + ret_col);
        let r1 = lay.out.runs.len();
        if let Some(kind) = para_kind(b)
            && r1 > r0
        {
            let (first, last) = (&lay.out.runs[r0], &lay.out.runs[r1 - 1]);
            let (y, h) = (first.y, last.y + last.h - first.y);
            let t1 = lay.out.plain.trim_end_matches('\n').len().max(t0);
            lay.out.paras.push(Para {
                kind,
                runs: r0..r1,
                text: t0..t1,
                y,
                h,
            });
        }
    }
    lay.out.height = lay.y;
    lay.out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cada carácter mede 10 (prosa) ou 8 (mono).
    struct Fake;
    impl Measure for Fake {
        fn prose(&mut self, s: &str, _: Font) -> f32 {
            s.chars().count() as f32 * 10.0
        }
        fn mono(&self, s: &str) -> f32 {
            s.chars().count() as f32 * 8.0
        }
    }

    fn dims(width: f32) -> Dims {
        Dims {
            width,
            prose_lh: 20.0,
            mono_lh: 18.0,
            title_lh: 30.0,
            head_lh: 24.0,
            gap: 10.0,
            indent: 16.0,
            pad: 6.0,
            button: 12.0,
        }
    }

    fn prose(t: &str) -> Span {
        Span {
            text: t.to_owned(),
            kind: SpanKind::Prose,
            link: None,
        }
    }

    #[test]
    fn a_prosa_parte_nos_espacos_e_nada_se_perde() {
        let blocks = [Block::Para(vec![prose("one two three four five")])];
        let l = lay_out(&blocks, dims(100.0), &mut Fake);
        let lines: Vec<&str> = l.runs.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(lines, ["one two", "three four", "five"]);
        assert!(l.runs.iter().all(|r| r.x + r.w <= 100.0));
        assert_eq!(l.plain, "one two\nthree four\nfive\n");
        assert_eq!(l.runs[1].y - l.runs[0].y, 20.0);
    }

    #[test]
    fn o_link_e_um_trecho_proprio_e_o_codigo_mede_em_mono() {
        let s = vec![
            prose("see "),
            Span {
                text: "math.floor".into(),
                kind: SpanKind::Code(Ink::Function),
                link: Some(7),
            },
            prose(" too"),
        ];
        let l = lay_out(&[Block::Para(s)], dims(1000.0), &mut Fake);
        let link = l.runs.iter().find(|r| r.link == Some(7)).expect("o link");
        assert_eq!(
            (link.text.as_str(), link.w, link.x),
            ("math.floor", 80.0, 40.0)
        );
        assert_eq!(&l.plain[link.at..link.at + 10], "math.floor");
    }

    #[test]
    fn as_ancoras_e_as_seccoes_ficam_onde_se_pintam() {
        let sig = |m, first| Block::Sig {
            spans: vec![prose("f()")],
            member: m,
            first,
        };
        let blocks = [
            Block::Heading("Functions".into()),
            sig(0, true),
            Block::Body(vec![prose("x")]),
            sig(1, false),
        ];
        let l = lay_out(&blocks, dims(300.0), &mut Fake);
        assert_eq!(l.sections.len(), 1);
        assert_eq!(l.anchors.len(), 2);
        let y1 = l.anchors[1].1;
        assert!(
            l.runs.iter().any(|r| r.text == "f()" && r.y == y1),
            "a assinatura pinta-se na âncora"
        );
        assert!(
            l.shapes.iter().any(|(s, r)| *s == Shape::Rule && r[1] < y1),
            "a linha antes do 2.º membro"
        );
    }

    #[test]
    fn o_exemplo_nao_parte_e_o_copiar_leva_o_texto_dele() {
        let code = |t: &str| {
            vec![Span {
                text: t.into(),
                kind: SpanKind::Code(Ink::Plain),
                link: None,
            }]
        };
        let blocks = [Block::Code {
            lines: vec![code("local a = 1"), code("print(a) --> 1")],
            copy: 3,
        }];
        let l = lay_out(&blocks, dims(40.0), &mut Fake);
        assert_eq!(l.runs.len(), 2, "sem partir, mesmo mais largo que a página");
        let (n, r) = &l.examples[0];
        assert_eq!(
            (*n, &l.plain[r.clone()]),
            (3, "local a = 1\nprint(a) --> 1")
        );
        assert!(l.shapes.iter().any(|(s, _)| *s == Shape::Copy(3)));
    }

    #[test]
    fn a_tabela_alinha_os_retornos_a_direita() {
        let row = |ret: &str, call: &str| Block::Summary {
            ret: vec![Span {
                text: ret.into(),
                kind: SpanKind::Code(Ink::Type),
                link: None,
            }],
            call: vec![prose(call)],
        };
        let l = lay_out(
            &[row("number", "abs(n)"), row("boolean", "isnan(n)")],
            dims(800.0),
            &mut Fake,
        );
        let ends: Vec<f32> = l
            .runs
            .iter()
            .filter(|r| r.tone == Tone::Code(Ink::Type))
            .map(|r| r.x + r.w)
            .collect();
        assert_eq!(ends[0], ends[1]);
    }

    #[test]
    fn cada_bloco_com_texto_e_um_paragrafo_do_leitor() {
        let code = vec![vec![Span::new("x = 1", SpanKind::Code(Ink::Plain))]];
        let blocks = [
            Block::Title("math".into()),
            Block::Gap,
            Block::Para(vec![prose("one two three four five")]),
            Block::Code {
                lines: code,
                copy: 4,
            },
        ];
        let l = lay_out(&blocks, dims(100.0), &mut Fake);
        let kinds: Vec<ParaKind> = l.paras.iter().map(|p| p.kind).collect();
        assert_eq!(
            kinds,
            [ParaKind::Heading(1), ParaKind::Text, ParaKind::Code(4)]
        );
        let texts: Vec<&str> = l.paras.iter().map(|p| &l.plain[p.text.clone()]).collect();
        assert_eq!(texts, ["math", "one two\nthree four\nfive", "x = 1"]);
        assert_eq!(l.paras[1].runs, 1..4);
        assert_eq!(l.paras[1].h, 60.0, "três linhas de prosa");
    }
}
