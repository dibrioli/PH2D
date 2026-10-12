//! A disposição de um `<text>` à maneira do SVG (a do Chromium): os caracteres pelo espaço em branco
//! (colapsado, ou `xml:space="preserve"`), as listas `x`/`y`/`dx`/`dy` por caracter (o descendente
//! manda), um BLOCO novo em cada `x`/`y` absoluto, cada bloco moldado pelo `parley` e ancorado pelo
//! `text-anchor` do seu 1.º caracter.

use std::borrow::Cow;
use std::ops::Range;

use parley::{FontFamily, FontFamilyName, Layout, PositionedLayoutItem, StyleProperty};
use usvg::roxmltree::Node;

use super::TextGaps as Gaps;
use super::fonts::Shaper;
use super::style::{Anchor, Style, StyleSheet, length};

const SVG_NS: &str = "http://www.w3.org/2000/svg";

/// Um elemento de conteúdo de texto (`<text>`, `<tspan>`, `<a>`), em pré-ordem.
pub(crate) struct Elem<'a, 'i> {
    pub node: Node<'a, 'i>,
    pub parent: Option<usize>,
    pub style: Style,
    /// Os caracteres (depois do espaço em branco) da sua sub-árvore.
    pub chars: Range<usize>,
}

/// Uma corrida de glifos de uma fonte, já posta no espaço do `<text>`.
pub(crate) struct Run {
    pub font: parley::FontData,
    pub size: f32,
    pub coords: Vec<i16>,
    /// A inclinação do itálico SINTÉTICO (tangente), 0 sem ela.
    pub skew: f32,
    /// (glifo, origem na linha de base, elemento dono).
    pub glyphs: Vec<(u32, [f32; 2], usize)>,
}

/// O `<text>` disposto: os elementos, a origem de cada caracter, e os glifos.
pub(crate) struct Laid<'a, 'i> {
    pub elems: Vec<Elem<'a, 'i>>,
    pub at: Vec<[f32; 2]>,
    pub runs: Vec<Run>,
}

struct Ch {
    c: char,
    owner: usize,
    collapsible: bool,
}

pub(crate) fn lay_out<'a, 'i>(
    text: Node<'a, 'i>,
    style: Style,
    sheet: &StyleSheet,
    sh: &mut Shaper,
    gaps: &mut Gaps,
) -> Laid<'a, 'i> {
    let mut elems = vec![Elem {
        node: text,
        parent: None,
        style,
        chars: 0..0,
    }];
    let mut chars = Vec::new();
    walk(0, &mut elems, &mut chars, sheet, gaps);
    if chars.last().is_some_and(|c: &Ch| c.collapsible) {
        chars.pop();
        for e in &mut elems {
            e.chars.start = e.chars.start.min(chars.len());
            e.chars.end = e.chars.end.min(chars.len());
        }
    }
    for e in &elems {
        for p in &e.style.unsupported {
            gaps.note_unsupported(p);
        }
    }
    let n = chars.len();
    let (mut ax, mut ay) = (vec![None; n], vec![None; n]);
    let (mut dx, mut dy) = (vec![0.0f32; n], vec![0.0f32; n]);
    let vp = viewport(text);
    for e in &elems {
        let em = e.style.size;
        let put = |name: &str, pct: f32, set: &mut dyn FnMut(usize, f32)| {
            let Some(v) = e.node.attribute(name) else {
                return;
            };
            let list = svgtypes::LengthListParser::from(v).filter_map(Result::ok);
            for (i, l) in list.enumerate() {
                let k = e.chars.start + i;
                if k < e.chars.end {
                    set(k, length(l, em, pct));
                }
            }
        };
        put("x", vp[0], &mut |k, v| ax[k] = Some(v));
        put("y", vp[1], &mut |k, v| ay[k] = Some(v));
        put("dx", vp[0], &mut |k, v| dx[k] = v);
        put("dy", vp[1], &mut |k, v| dy[k] = v);
        let rotated = e.node.attribute("rotate").is_some_and(|v| {
            svgtypes::NumberListParser::from(v).any(|r| r.is_ok_and(|r| r != 0.0))
        });
        if rotated {
            gaps.note_unsupported("rotate");
        }
        if e.node.has_attribute("textLength") {
            gaps.note_unsupported("textLength");
        }
    }
    let mut laid = Laid {
        at: vec![[0.0; 2]; n],
        runs: Vec::new(),
        elems,
    };
    let (mut px, mut py) = (0.0f32, 0.0f32);
    let mut s = 0;
    while s < n {
        let e = (s + 1..n)
            .find(|&k| ax[k].is_some() || ay[k].is_some())
            .unwrap_or(n);
        px = ax[s].unwrap_or(px);
        py = ay[s].unwrap_or(py);
        let (cx, adv, mut runs) = shape(&chars[s..e], &laid.elems, sh, gaps);
        let (mut sx, mut sy) = (vec![0.0; e - s], vec![0.0; e - s]);
        let (mut cdx, mut cdy) = (0.0, 0.0);
        for k in s..e {
            cdx += dx[k];
            cdy += dy[k];
            (sx[k - s], sy[k - s]) = (cdx, cdy);
        }
        let width = adv + cdx;
        let off = match laid.elems[chars[s].owner].style.anchor {
            Anchor::Start => 0.0,
            Anchor::Middle => width / 2.0,
            Anchor::End => width,
        };
        for k in s..e {
            laid.at[k] = [px + cx[k - s] + sx[k - s] - off, py + sy[k - s]];
        }
        for r in &mut runs {
            for (_, at, owner) in &mut r.glyphs {
                // `owner` traz, até aqui, o caracter (do bloco) do glifo.
                let k = *owner;
                *at = [px + at[0] + sx[k] - off, py + sy[k] + at[1]];
                *owner = chars[s + k].owner;
            }
        }
        laid.runs.extend(runs);
        px += width - off;
        py += cdy;
        s = e;
    }
    laid
}

fn walk(
    idx: usize,
    elems: &mut Vec<Elem>,
    chars: &mut Vec<Ch>,
    sheet: &StyleSheet,
    gaps: &mut Gaps,
) {
    let node = elems[idx].node;
    elems[idx].chars.start = chars.len();
    for child in node.children() {
        if child.is_text() {
            let preserve = elems[idx].style.preserve;
            for c in child.text().unwrap_or_default().chars() {
                push(chars, c, idx, preserve);
            }
            continue;
        }
        if !child.is_element() || child.tag_name().namespace() != Some(SVG_NS) {
            continue;
        }
        match child.tag_name().name() {
            "tspan" | "a" => {
                let style = elems[idx].style.child(child, sheet);
                if style.hidden {
                    continue;
                }
                elems.push(Elem {
                    node: child,
                    parent: Some(idx),
                    style,
                    chars: 0..0,
                });
                walk(elems.len() - 1, elems, chars, sheet, gaps);
            }
            "textPath" => gaps.note_unsupported("textPath"),
            "tref" => gaps.note_unsupported("tref"),
            _ => {}
        }
    }
    elems[idx].chars.end = chars.len();
}

/// Um caracter pelo espaço em branco: quebras e tabs são espaço; sem `preserve`, o espaço do início
/// e o que segue outro espaço colapsável não entram.
fn push(chars: &mut Vec<Ch>, c: char, owner: usize, preserve: bool) {
    let c = if matches!(c, '\n' | '\r' | '\t') {
        ' '
    } else {
        c
    };
    let collapsible = !preserve && c == ' ';
    if collapsible && chars.last().is_none_or(|l| l.collapsible) {
        return;
    }
    chars.push(Ch {
        c,
        owner,
        collapsible,
    });
}

/// Molda um bloco: a origem de cada caracter na linha, o avanço, e as corridas (com o índice do
/// caracter do bloco no lugar do dono).
fn shape(
    chars: &[Ch],
    elems: &[Elem],
    sh: &mut Shaper,
    gaps: &mut Gaps,
) -> (Vec<f32>, f32, Vec<Run>) {
    let text: String = chars.iter().map(|c| c.c).collect();
    let mut char_at = vec![0usize; text.len() + 1];
    let mut spans: Vec<(Range<usize>, usize, Vec<String>)> = Vec::new();
    for (i, (b, c)) in text.char_indices().enumerate() {
        char_at[b..b + c.len_utf8()].fill(i);
        let owner = chars[i].owner;
        match spans.last_mut() {
            Some((r, o, _)) if *o == owner => r.end = b + c.len_utf8(),
            _ => {
                let fam = sh.resolve(&elems[owner].style.family);
                for m in fam.missing {
                    gaps.note_missing(m);
                }
                spans.push((b..b + c.len_utf8(), owner, fam.names));
            }
        }
    }
    let (fcx, lcx) = (&mut sh.fcx, &mut sh.lcx);
    let mut b = lcx.ranged_builder(fcx, &text, 1.0, false);
    for (r, owner, fam) in &spans {
        let st = &elems[*owner].style;
        let list: Vec<FontFamilyName> = fam
            .iter()
            .map(|n| FontFamilyName::Named(Cow::Borrowed(n.as_str())))
            .collect();
        b.push(
            StyleProperty::FontFamily(FontFamily::List(Cow::Owned(list))),
            r.clone(),
        );
        b.push(StyleProperty::FontSize(st.size), r.clone());
        b.push(StyleProperty::FontWeight(st.weight), r.clone());
        b.push(StyleProperty::FontStyle(st.style), r.clone());
        b.push(StyleProperty::FontWidth(st.width), r.clone());
        b.push(StyleProperty::LetterSpacing(st.letter), r.clone());
        b.push(StyleProperty::WordSpacing(st.word), r.clone());
    }
    let mut layout: Layout<()> = b.build(&text);
    layout.break_all_lines(None);
    let mut cx = vec![f32::NAN; chars.len()];
    let mut runs = Vec::new();
    let mut adv = 0.0f32;
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(gr) = item else {
                continue;
            };
            let run = gr.run();
            let syn = run.synthesis();
            if syn.embolden() {
                gaps.note_unsupported("synthetic bold");
            }
            let mut out = Run {
                font: run.font().clone(),
                size: run.font_size(),
                coords: run.normalized_coords().to_vec(),
                skew: syn.skew().map_or(0.0, |d| d.to_radians().tan()),
                glyphs: Vec::new(),
            };
            let (mut gpen, mut cpen) = (gr.offset(), gr.offset());
            for cl in run.visual_clusters() {
                let tr = cl.text_range();
                let k = char_at[tr.start];
                for c in &mut cx[k..=char_at[tr.end.saturating_sub(1).max(tr.start)]] {
                    *c = cpen;
                }
                cpen += cl.advance();
                for g in cl.glyphs() {
                    out.glyphs.push((g.id, [gpen + g.x, g.y], k));
                    gpen += g.advance;
                }
            }
            adv = adv.max(cpen);
            runs.push(out);
        }
    }
    let mut last = 0.0;
    for c in &mut cx {
        if c.is_nan() {
            *c = last;
        }
        last = *c;
    }
    (cx, adv, runs)
}

/// O tamanho da janela de vista mais próxima (para os `%` de `x`/`y`).
fn viewport(node: Node) -> [f32; 2] {
    let Some(svg) = node.ancestors().find(|n| n.has_tag_name((SVG_NS, "svg"))) else {
        return [0.0; 2];
    };
    if let Some(vb) = svg
        .attribute("viewBox")
        .and_then(|v| v.parse::<svgtypes::ViewBox>().ok())
    {
        return [vb.w as f32, vb.h as f32];
    }
    let dim = |a: &str| {
        svg.attribute(a)
            .and_then(|v| v.parse::<svgtypes::Length>().ok())
            .map_or(0.0, |l| length(l, 0.0, 0.0))
    };
    [dim("width"), dim("height")]
}
