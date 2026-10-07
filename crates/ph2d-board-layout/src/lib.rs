//! **O texto do Quadro** (MiroClone, W1): moldado em unidades do MUNDO (quebra à largura da caixa da
//! forma, centrado), guardado moldado, e desenhado a qualquer zoom e rotação pela transformação. Na
//! W3, com estilo por TRECHO ([`RichText`]): negrito, itálico, sublinhado, riscado e cor.
//!
//! ⚠️ **Não passa pelo `TextSystem::layout`**: aquele aplica o estilo da INTERFACE (escala e peso que
//! o artista escolhe para os menus). O texto de um quadro é DOCUMENTO — o tamanho é o que está
//! gravado na forma. A fonte da casa vem de [`TextSystem::contexts`]; o contexto de moldar é deste
//! crate, porque o pincel é a cor do trecho ([`Ink`]).
//!
//! ⚠️ Medido 06/10 (`tests::bold_is_a_real_weight_and_italic_a_skew`): a Inter da casa é VARIÁVEL —
//! o negrito é o eixo `wght` 700 (as coordenadas vão no desenho); itálico ela não tem, e o parley
//! devolve uma inclinação SINTÉTICA de 14° que o desenho aplica a cada glifo.
//!
//! ⚠️ **Moldar custa** (dezenas de µs por texto curto): a [`TextCache`] guarda o moldado por dono e
//! só remolda quando o texto, os trechos, o tamanho ou a largura mudam.

mod edit;

use std::borrow::Cow;
use std::collections::BTreeMap;

use parley::{
    Alignment, AlignmentOptions, FontFamily, FontStyle, FontWeight, Layout, LayoutContext,
    OverflowWrap, PositionedLayoutItem, StyleProperty,
};
use ph2d_board_model::{Rgba, RichText, Span};
use ph2d_text::TextSystem;
use ph2d_vector::{Affine, BezPath, Brush, Color, Fill, Glyph, Rect, Shape as _, VectorScene};

pub use edit::{Move, TextEdit};

/// O pincel de um trecho: a sua cor, ou `None` = a tinta da forma.
pub type Ink = Option<Rgba>;

/// Moldar `text` com `spans` a `font_size` (mundo), com quebra em `max_width` (mundo), centrado.
pub fn shape(
    ts: &mut TextSystem,
    lcx: &mut LayoutContext<Ink>,
    text: &str,
    spans: &[Span],
    font_size: f32,
    max_width: f32,
) -> Layout<Ink> {
    let (fcx, _, stack) = ts.contexts();
    let mut b = lcx.ranged_builder(fcx, text, 1.0, false);
    b.push_default(StyleProperty::FontFamily(FontFamily::Source(
        Cow::Borrowed(stack),
    )));
    b.push_default(StyleProperty::FontSize(font_size));
    b.push_default(StyleProperty::FontWeight(FontWeight::NORMAL));
    // Uma palavra maior que a forma quebra a meio em vez de sair por ela.
    b.push_default(StyleProperty::OverflowWrap(OverflowWrap::Anywhere));
    for s in spans {
        let (r, m) = (s.range(), s.marks);
        if m.bold {
            b.push(StyleProperty::FontWeight(FontWeight::BOLD), r.clone());
        }
        if m.italic {
            b.push(StyleProperty::FontStyle(FontStyle::Italic), r.clone());
        }
        if m.underline {
            b.push(StyleProperty::Underline(true), r.clone());
        }
        if m.strike {
            b.push(StyleProperty::Strikethrough(true), r.clone());
        }
        if m.color.is_some() {
            b.push(StyleProperty::Brush(m.color), r);
        }
    }
    let mut layout: Layout<Ink> = b.build(text);
    layout.break_all_lines(Some(max_width.max(1.0)));
    layout.align(Alignment::Center, AlignmentOptions::default());
    layout
}

/// A altura (mundo) de `text` moldado a `font_size` com quebra em `max_width` — a conta do «a
/// forma cresce» para quem não guarda o moldado.
pub fn text_height(ts: &mut TextSystem, text: &RichText, font_size: f32, max_width: f32) -> f32 {
    let mut lcx = LayoutContext::new();
    shape(
        ts,
        &mut lcx,
        text.as_str(),
        text.spans(),
        font_size,
        max_width,
    )
    .height()
}

/// A cor de documento → a do desenho.
#[must_use]
pub fn doc_color(Rgba([r, g, b, a]): Rgba) -> Color {
    Color::from_rgba8(r, g, b, a) // LITERAL-COLOR-OK: cor do DOCUMENTO (dado do artista), não da UI
}

/// Desenha `layout` com `transform` (do espaço do texto — origem no canto superior esquerdo do
/// bloco, em unidades do mundo — para o ecrã); `ink` é a tinta da forma (a dos trechos sem cor).
pub fn paint(scene: &mut VectorScene, layout: &Layout<Ink>, transform: Affine, ink: Color) {
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            let style = run.style();
            let color = style.brush.map_or(ink, doc_color);
            let r = run.run();
            // O itálico SINTÉTICO (a Inter não o tem): a inclinação que o parley escolheu.
            let skew = r
                .synthesis()
                .skew()
                .map(|deg| Affine::skew(f64::from(deg).to_radians().tan(), 0.0));
            scene
                .inner_mut()
                .draw_glyphs(r.font())
                .font_size(r.font_size())
                .hint(false)
                .normalized_coords(r.normalized_coords())
                .glyph_transform(skew)
                .brush(color)
                .transform(transform)
                .draw(
                    Fill::NonZero,
                    run.positioned_glyphs().map(|g| Glyph {
                        id: g.id,
                        x: g.x,
                        y: g.y,
                    }),
                );
            let m = r.metrics();
            let (x0, x1) = (run.offset(), run.offset() + run.advance());
            let mut bar = |offset: f32, size: f32| {
                let y = run.baseline() - offset;
                let rect = Rect::new(
                    f64::from(x0),
                    f64::from(y),
                    f64::from(x1),
                    f64::from(y + size),
                );
                scene.fill_path(&rect.to_path(0.1), &Brush::Solid(color), transform);
            };
            if let Some(u) = &style.underline {
                bar(
                    u.offset.unwrap_or(m.underline_offset),
                    u.size.unwrap_or(m.underline_size),
                );
            }
            if let Some(s) = &style.strikethrough {
                bar(
                    s.offset.unwrap_or(m.strikethrough_offset),
                    s.size.unwrap_or(m.strikethrough_size),
                );
            }
        }
    }
}

/// Uma barra por linha (a largura e a altura do x da linha), num só caminho — o que se desenha no
/// lugar do texto quando a letra é pequena demais para ler.
#[must_use]
pub fn line_bars(layout: &Layout<Ink>) -> BezPath {
    let mut p = BezPath::new();
    for line in layout.lines() {
        let m = line.metrics();
        let x0 = f64::from(m.offset);
        let w = f64::from(m.advance - m.trailing_whitespace);
        let y1 = f64::from(m.baseline);
        let y0 = y1 - f64::from(m.ascent) * 0.6;
        if w > 0.0 {
            p.extend(Rect::new(x0, y0, x0 + w, y1).path_elements(0.1));
        }
    }
    p
}

/// Quantos quadros um moldado fica guardado sem ser usado.
const KEEP_FRAMES: u64 = 120;

struct Entry {
    text: String,
    spans: Vec<Span>,
    size_bits: u32,
    width_bits: u32,
    layout: Layout<Ink>,
    used: u64,
}

/// O moldado de cada dono (uma forma de um quadro), refeito só quando muda.
#[derive(Default)]
pub struct TextCache {
    lcx: LayoutContext<Ink>,
    map: BTreeMap<(u64, u64), Entry>,
    frame: u64,
    shaped: u64,
}

impl TextCache {
    /// O moldado de `owner` para um texto com trechos ([`RichText`]).
    pub fn get_rich(
        &mut self,
        ts: &mut TextSystem,
        owner: (u64, u64),
        text: &RichText,
        font_size: f32,
        max_width: f32,
    ) -> &Layout<Ink> {
        self.get(ts, owner, text.as_str(), text.spans(), font_size, max_width)
    }

    /// O moldado de `owner`, moldando outra vez só se o texto, os trechos, o tamanho ou a largura
    /// mudaram.
    pub fn get(
        &mut self,
        ts: &mut TextSystem,
        owner: (u64, u64),
        text: &str,
        spans: &[Span],
        font_size: f32,
        max_width: f32,
    ) -> &Layout<Ink> {
        let frame = self.frame;
        let (sb, wb) = (font_size.to_bits(), max_width.to_bits());
        let stale = self.map.get(&owner).is_none_or(|e| {
            e.text != text || e.spans != spans || e.size_bits != sb || e.width_bits != wb
        });
        if stale {
            self.shaped += 1;
            let layout = shape(ts, &mut self.lcx, text, spans, font_size, max_width);
            self.map.insert(
                owner,
                Entry {
                    text: text.to_owned(),
                    spans: spans.to_vec(),
                    size_bits: sb,
                    width_bits: wb,
                    layout,
                    used: frame,
                },
            );
        }
        let e = self.map.get_mut(&owner).expect("acabou de entrar");
        e.used = frame;
        &e.layout
    }

    /// Fecha um quadro: esquece o que não se usou há [`KEEP_FRAMES`] quadros.
    pub fn end_frame(&mut self) {
        self.frame += 1;
        if self.frame.is_multiple_of(KEEP_FRAMES) {
            let cut = self.frame.saturating_sub(KEEP_FRAMES);
            self.map.retain(|_, e| e.used >= cut);
        }
    }

    /// Quantos textos já foram moldados (o instrumento da cache: num ecrã parado não sobe).
    #[must_use]
    pub fn shaped(&self) -> u64 {
        self.shaped
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests;
