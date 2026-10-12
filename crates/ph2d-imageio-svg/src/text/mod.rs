//! W18 do Quadro — **os `<text>` de um SVG desenham-se**. O usvg sem a feature `text` (recusa medida,
//! `ph2d-vec-svg/Cargo.toml`) apaga-os; esta passagem, ANTES dele, troca cada `<text>` por um `<g>`
//! com os contornos dos glifos em `<path>`, no MESMO espaço do utilizador — moldados pela stack da
//! casa (`parley` + fontes do sistema; contornos pelo `skrifa`). O `<g>` guarda os atributos do
//! elemento (`transform`, pintura, `opacity`, `class`, `style`) e as regras do `<style>` que casavam
//! com ele: a pintura, a herança e o recorte continuam a ser do usvg. O exportador de SVG (que grava
//! os bytes ORIGINAIS) não passa por aqui. Lei e oráculo: `docs/MiroClone/09_texto_dos_svg.md`.
//!
//! O que não se desenha como escrito fica em [`TextGaps`] (o aviso de quem larga o SVG).

mod cache;
mod fonts;
mod layout;
mod outline;
mod style;

#[cfg(test)]
mod oracle_tests;
#[cfg(test)]
mod tests;

use std::borrow::Cow;
use std::fmt::Write as _;
use std::sync::Arc;

use ph2d_imageio::MAX_ARCHIVE_TEXT_BYTES;
use usvg::roxmltree::{Document, Node, ParsingOptions};

use cache::Outlined;
use layout::Laid;
use style::{Style, StyleSheet};

const SVG_NS: &str = "http://www.w3.org/2000/svg";

/// O que os `<text>` de um SVG têm que o desenho NÃO mostra como o autor escreveu.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextGaps {
    /// As famílias pedidas que este sistema não tem (e que vinham antes da usada), sem repetição.
    pub missing_families: Vec<String>,
    /// As construções que não se desenham (`textPath`, `rotate`, `writing-mode`, …), sem repetição.
    pub unsupported: Vec<String>,
    /// Quantas ocorrências dessas construções (o número do aviso).
    pub undrawn: usize,
}

impl TextGaps {
    fn note_unsupported(&mut self, what: &str) {
        self.undrawn += 1;
        if !self.unsupported.iter().any(|u| u == what) {
            self.unsupported.push(what.to_owned());
        }
    }

    fn note_missing(&mut self, family: String) {
        if !self.missing_families.contains(&family) {
            self.missing_families.push(family);
        }
    }
}

/// O que falta aos `<text>` de `src` (um `.svgz` descomprime-se). Sem `<text>`, nada (e nada custa).
#[must_use]
pub fn text_gaps(src: &[u8]) -> TextGaps {
    crate::raster::plain(src)
        .and_then(|s| outline_texts(&s))
        .map(|o| o.gaps.clone())
        .unwrap_or_default()
}

/// A passagem de `src` (guardada por ficheiro): o SVG com os `<text>` em contornos. `None` = sem
/// `<text>` (o caminho rápido, que não paga nada) ou não é texto UTF-8.
pub(crate) fn outline_texts(src: &[u8]) -> Option<Arc<Outlined>> {
    if !has_text(src) {
        return None;
    }
    let txt = std::str::from_utf8(src).ok()?;
    Some(cache::get_or(txt, || match rewrite(txt) {
        Some((svg, gaps)) => Outlined {
            svg: Some(svg.into_bytes()),
            gaps,
        },
        None => Outlined {
            svg: None,
            gaps: TextGaps::default(),
        },
    }))
}

/// Há um `<text` (ou `<pfx:text`) — o caminho rápido de quem não tem texto nenhum.
fn has_text(src: &[u8]) -> bool {
    src.windows(5).enumerate().any(|(i, w)| {
        &w[1..] == b"text"
            && matches!(w[0], b'<' | b':')
            && src
                .get(i + 5)
                .is_some_and(|b| b.is_ascii_whitespace() || matches!(b, b'>' | b'/'))
    })
}

fn is_svg(n: &Node, name: &str) -> bool {
    n.is_element() && n.tag_name().namespace() == Some(SVG_NS) && n.tag_name().name() == name
}

/// Os `<text>` de topo do documento.
fn texts<'a, 'i>(doc: &'a Document<'i>) -> Vec<Node<'a, 'i>> {
    doc.descendants()
        .filter(|n| is_svg(n, "text") && !n.ancestors().skip(1).any(|a| is_svg(&a, "text")))
        .collect()
}

/// O estilo herdado até ao `<text>` (ele incluído).
fn chain_style(t: Node, sheet: &StyleSheet) -> Style {
    let chain: Vec<Node> = t.ancestors().filter(Node::is_element).collect();
    chain
        .iter()
        .rev()
        .fold(Style::initial(), |s, n| s.child(*n, sheet))
}

fn parse(txt: &str) -> Option<Document<'_>> {
    let opts = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    Document::parse_with_options(txt, opts).ok()
}

fn rewrite(txt: &str) -> Option<(String, TextGaps)> {
    let doc = parse(txt)?;
    let sheet = style::sheet(&doc);
    let texts = texts(&doc);
    if texts.is_empty() {
        return None;
    }
    let mut gaps = TextGaps::default();
    let mut out = String::with_capacity(txt.len());
    let mut cursor = 0;
    fonts::with_shaper(|sh| {
        for t in texts {
            let r = t.range();
            // O prefixo do nome como está no ficheiro (`<svg:text` → `svg:`); um `<text>` vindo de
            // uma entidade não tem o seu lugar nos bytes e não se troca.
            let qname = txt.get(r.start + 1..r.end).and_then(|s| {
                s.split(|c: char| c.is_whitespace() || c == '>' || c == '/')
                    .next()
            });
            let Some(prefix) = qname
                .filter(|_| r.start >= cursor && txt[r.start..].starts_with('<'))
                .and_then(|q| q.strip_suffix("text"))
                .filter(|p| p.is_empty() || p.ends_with(':'))
            else {
                gaps.note_unsupported("text");
                continue;
            };
            let style = chain_style(t, &sheet);
            if style.hidden {
                continue;
            }
            let laid = layout::lay_out(t, style, &sheet, sh, &mut gaps);
            let mut d = vec![String::new(); laid.elems.len()];
            for run in &laid.runs {
                outline::append(run, &mut d);
            }
            let clip = t.ancestors().any(|a| is_svg(&a, "clipPath"));
            let rep = emit(&laid, &d, prefix, &sheet, clip);
            let projected = out.len() + (r.start - cursor) + rep.len() + (txt.len() - r.end);
            if projected as u64 > MAX_ARCHIVE_TEXT_BYTES {
                gaps.note_unsupported("MAX_ARCHIVE_TEXT_BYTES");
                continue;
            }
            out.push_str(&txt[cursor..r.start]);
            out.push_str(&rep);
            cursor = r.end;
        }
    });
    out.push_str(&txt[cursor..]);
    Some((out, gaps))
}

/// O que substitui o `<text>`: `<g>` por elemento com o `<path>` dos glifos dele; dentro de um
/// `<clipPath>` (que não aceita `<g>`), um só `<path>`.
fn emit(laid: &Laid, d: &[String], p: &str, sheet: &StyleSheet, clip: bool) -> String {
    let mut s = String::new();
    if clip {
        let _ = write!(s, "<{p}path");
        attrs(&mut s, laid.elems[0].node, sheet, true, true);
        let _ = write!(s, " d=\"{}\"/>", d.concat());
        return s;
    }
    emit_g(0, laid, d, p, sheet, &mut s);
    s
}

fn emit_g(i: usize, laid: &Laid, d: &[String], p: &str, sheet: &StyleSheet, s: &mut String) {
    let _ = write!(s, "<{p}g");
    attrs(s, laid.elems[i].node, sheet, i == 0, false);
    s.push('>');
    if !d[i].is_empty() {
        // O texto pinta-se sempre não-zero: o `fill-rule` herdado não vale para os glifos.
        let _ = write!(
            s,
            "<{p}path style=\"fill-rule:nonzero;clip-rule:nonzero\" d=\"{}\"/>",
            d[i]
        );
    }
    for (j, _) in laid
        .elems
        .iter()
        .enumerate()
        .filter(|(_, e)| e.parent == Some(i))
    {
        emit_g(j, laid, d, p, sheet, s);
    }
    let _ = write!(s, "</{p}g>");
}

/// Os atributos do elemento sem os de disposição do texto, e o `style` com as regras que casavam.
fn attrs(s: &mut String, node: Node, sheet: &StyleSheet, transform: bool, nonzero: bool) {
    for a in node.attributes().filter(|a| a.namespace().is_none()) {
        let skip = matches!(
            a.name(),
            "x" | "y" | "dx" | "dy" | "rotate" | "textLength" | "lengthAdjust" | "style"
        ) || (a.name() == "transform" && !transform)
            || (nonzero && matches!(a.name(), "fill-rule" | "clip-rule"));
        if !skip {
            let _ = write!(s, " {}=\"{}\"", a.name(), escape(a.value()));
        }
    }
    let mut css = style::rule_css(node, sheet);
    if let Some(v) = node.attribute("style") {
        css.push_str(v);
        css.push(';');
    }
    if nonzero {
        css.push_str("fill-rule:nonzero;clip-rule:nonzero");
    }
    if !css.is_empty() {
        let _ = write!(s, " style=\"{}\"", escape(&css));
    }
}

fn escape(v: &str) -> Cow<'_, str> {
    if !v.contains(['&', '<', '>', '"']) {
        return Cow::Borrowed(v);
    }
    let mut o = String::with_capacity(v.len() + 8);
    for c in v.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            c => o.push(c),
        }
    }
    Cow::Owned(o)
}

/// Para os gates: cada `<text>` disposto — os `id` dos elementos, o pai de cada um, e a origem e o
/// dono de cada caracter.
#[cfg(test)]
pub(crate) struct LaidText {
    pub ids: Vec<Option<String>>,
    pub parents: Vec<Option<usize>>,
    pub at: Vec<[f32; 2]>,
    pub owner: Vec<usize>,
    /// Os glifos `.notdef` (o caracter que nenhuma fonte tem).
    pub notdef: usize,
}

#[cfg(test)]
pub(crate) fn lay_out_texts(txt: &str) -> Vec<LaidText> {
    let doc = parse(txt).expect("XML");
    let sheet = style::sheet(&doc);
    let mut gaps = TextGaps::default();
    fonts::with_shaper(|sh| {
        texts(&doc)
            .into_iter()
            .map(|t| {
                let laid = layout::lay_out(t, chain_style(t, &sheet), &sheet, sh, &mut gaps);
                LaidText {
                    ids: laid
                        .elems
                        .iter()
                        .map(|e| e.node.attribute("id").map(str::to_owned))
                        .collect(),
                    parents: laid.elems.iter().map(|e| e.parent).collect(),
                    // O dono de um caracter: o elemento mais fundo (o último em pré-ordem) que o tem.
                    owner: (0..laid.at.len())
                        .map(|k| {
                            laid.elems
                                .iter()
                                .rposition(|e| e.chars.contains(&k))
                                .unwrap_or(0)
                        })
                        .collect(),
                    notdef: laid
                        .runs
                        .iter()
                        .flat_map(|r| &r.glyphs)
                        .filter(|g| g.0 == 0)
                        .count(),
                    at: laid.at,
                }
            })
            .collect()
    })
}

#[cfg(test)]
pub(crate) fn installed(family: &str) -> Option<String> {
    fonts::with_shaper(|sh| sh.installed(family))
}
