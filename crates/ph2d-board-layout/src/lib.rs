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
use ph2d_board_model::{Rgba, RichText, Span, TextAlign};
use ph2d_text::TextSystem;
use ph2d_vector::{
    Affine, BezPath, Brush, Color, Glyph, GlyphStyle, Rect, Shape as _, VectorScene,
};

pub use edit::{Move, TextEdit};

/// O pincel de um trecho: a sua cor, ou `None` = a tinta da forma.
pub type Ink = Option<Rgba>;

/// A cara de um texto do quadro: na letra À MÃO ([`ph2d_text::HAND_FAMILY`]) ou na da interface, e o
/// alinhamento das linhas (W9: o `textAlign` que vem do Excalidraw; centrado de nascença). Um `bool`
/// é a letra, centrada.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Face {
    pub hand: bool,
    pub align: TextAlign,
}

impl From<bool> for Face {
    fn from(hand: bool) -> Self {
        Self {
            hand,
            align: TextAlign::Center,
        }
    }
}

/// Moldar `text` com `spans` a `font_size` (mundo), com quebra em `max_width` (mundo), alinhado pela
/// `face` (de nascença, centrado). `face.hand` = na letra À MÃO ([`ph2d_text::HAND_FAMILY`], o
/// rascunho), com a da interface por trás para o que ela não tiver.
pub fn shape(
    ts: &mut TextSystem,
    lcx: &mut LayoutContext<Ink>,
    text: &str,
    spans: &[Span],
    font_size: f32,
    max_width: f32,
    face: Face,
) -> Layout<Ink> {
    let (fcx, _, stack) = ts.contexts();
    let stack = if face.hand {
        Cow::Owned(format!("{}, {stack}", ph2d_text::HAND_FAMILY))
    } else {
        Cow::Borrowed(stack)
    };
    let mut b = lcx.ranged_builder(fcx, text, 1.0, false);
    b.push_default(StyleProperty::FontFamily(FontFamily::Source(stack)));
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
    let align = match face.align {
        TextAlign::Center => Alignment::Center,
        TextAlign::Left => Alignment::Left,
        TextAlign::Right => Alignment::Right,
    };
    layout.align(align, AlignmentOptions::default());
    layout
}

/// A altura (mundo) de `text` moldado a `font_size` com quebra em `max_width` na cara `face` — a conta
/// do «a forma cresce» para quem não guarda o moldado: as NOTAS (à mão só a que veio de um
/// `.excalidraw` com a letra dele, W9 — a altura mede-se na letra em que se DESENHA).
pub fn text_height(
    ts: &mut TextSystem,
    text: &RichText,
    font_size: f32,
    max_width: f32,
    face: Face,
) -> f32 {
    let mut lcx = LayoutContext::new();
    shape(
        ts,
        &mut lcx,
        text.as_str(),
        text.spans(),
        font_size,
        max_width,
        face,
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
/// `text` é o texto que o `layout` moldou: cada corrida grava o seu pedaço (o PDF devolve-o à busca).
pub fn paint(
    scene: &mut VectorScene,
    layout: &Layout<Ink>,
    text: &str,
    transform: Affine,
    ink: Color,
) {
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
            let gs = GlyphStyle {
                font: r.font(),
                size: r.font_size(),
                coords: r.normalized_coords(),
                skew,
                brush: color,
                transform,
                text: text.get(r.text_range()).unwrap_or_default(),
            };
            scene.glyph_run(
                &gs,
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

/// ⭐ W23 B5(2) — [`paint`] LETRA A LETRA, cada uma com o seu sítio: `place(x, y)` dá a transformação
/// do espaço de uma letra (origem no ponto `(x, y)` do bloco — o meio dela na linha de base) para o
/// ecrã. É o texto que dobra com uma linha curva. Só o texto simples (sem sublinhado nem riscado:
/// os textos das setas não têm marcas).
pub fn paint_along(
    scene: &mut VectorScene,
    layout: &Layout<Ink>,
    text: &str,
    place: impl Fn(f64, f64) -> Affine,
    ink: Color,
) {
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            let color = run.style().brush.map_or(ink, doc_color);
            let r = run.run();
            let skew = r
                .synthesis()
                .skew()
                .map(|deg| Affine::skew(f64::from(deg).to_radians().tan(), 0.0));
            for g in run.positioned_glyphs() {
                let half = g.advance / 2.0;
                let gs = GlyphStyle {
                    font: r.font(),
                    size: r.font_size(),
                    coords: r.normalized_coords(),
                    skew,
                    brush: color,
                    transform: place(f64::from(g.x + half), f64::from(g.y)),
                    text: text.get(r.text_range()).unwrap_or_default(),
                };
                let one = Glyph {
                    id: g.id,
                    x: -half,
                    y: 0.0,
                };
                scene.glyph_run(&gs, std::iter::once(one));
            }
        }
    }
}

/// As barras de [`line_bars`], uma `[x0, y0, x1, y1]` por linha.
#[must_use]
pub fn line_bar_rects(layout: &Layout<Ink>) -> Vec<[f64; 4]> {
    layout
        .lines()
        .filter_map(|line| {
            let m = line.metrics();
            let x0 = f64::from(m.offset);
            let w = f64::from(m.advance - m.trailing_whitespace);
            let y1 = f64::from(m.baseline);
            (w > 0.0).then(|| [x0, y1 - f64::from(m.ascent) * 0.6, x0 + w, y1])
        })
        .collect()
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

/// W23 B5(2) — o recorte de um texto de seta: as barras das linhas com `pad` de folga (no espaço do
/// bloco). É o que se vê por baixo das letras e o que o clique agarra.
#[must_use]
pub fn knock_box(layout: &Layout<Ink>, pad: f64) -> [f64; 4] {
    use ph2d_vector::Shape as _;
    let b = line_bars(layout).bounding_box().inflate(pad, pad);
    [b.x0, b.y0, b.x1, b.y1]
}

/// Quantos quadros um moldado fica guardado sem ser usado.
const KEEP_FRAMES: u64 = 120;

struct Entry {
    text: String,
    spans: Vec<Span>,
    size_bits: u32,
    width_bits: u32,
    face: Face,
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
        face: impl Into<Face>,
    ) -> &Layout<Ink> {
        self.get(
            ts,
            owner,
            text.as_str(),
            text.spans(),
            font_size,
            max_width,
            face,
        )
    }

    /// O moldado de `owner`, moldando outra vez só se o texto, os trechos, o tamanho, a largura ou a
    /// letra (à mão ou não) mudaram.
    #[allow(clippy::too_many_arguments)]
    pub fn get(
        &mut self,
        ts: &mut TextSystem,
        owner: (u64, u64),
        text: &str,
        spans: &[Span],
        font_size: f32,
        max_width: f32,
        face: impl Into<Face>,
    ) -> &Layout<Ink> {
        let face = face.into();
        let frame = self.frame;
        let (sb, wb) = (font_size.to_bits(), max_width.to_bits());
        let stale = self.map.get(&owner).is_none_or(|e| {
            e.text != text
                || e.spans != spans
                || e.size_bits != sb
                || e.width_bits != wb
                || e.face != face
        });
        if stale {
            self.shaped += 1;
            let layout = shape(ts, &mut self.lcx, text, spans, font_size, max_width, face);
            self.map.insert(
                owner,
                Entry {
                    text: text.to_owned(),
                    spans: spans.to_vec(),
                    size_bits: sb,
                    width_bits: wb,
                    face,
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
