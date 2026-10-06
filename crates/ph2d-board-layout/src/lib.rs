//! **O texto do Quadro** (MiroClone, W1): moldado em unidades do MUNDO (quebra à largura da caixa da
//! forma, centrado), guardado moldado, e desenhado a qualquer zoom e rotação pela transformação.
//!
//! ⚠️ **Não passa pelo `TextSystem::layout`**: aquele aplica o estilo da INTERFACE (escala e peso que
//! o artista escolhe para os menus). O texto de um quadro é DOCUMENTO — o tamanho é o que está
//! gravado na forma. Os contextos do parley e a fonte da casa vêm de [`TextSystem::contexts`].
//!
//! ⚠️ **Moldar custa** (dezenas de µs por texto curto): 10 mil formas com texto remoldadas a cada
//! quadro seriam centenas de ms. A [`TextCache`] guarda o moldado por dono e só remolda quando o
//! texto, o tamanho ou a largura mudam.

use std::borrow::Cow;
use std::collections::HashMap;

use parley::{
    Alignment, AlignmentOptions, FontFamily, FontWeight, Layout, OverflowWrap, PlainEditor,
    PositionedLayoutItem, StyleProperty,
};
use ph2d_text::TextSystem;
use ph2d_vector::{Affine, Color, Fill, Glyph, Shape as _, VectorScene};

/// Moldar `text` a `font_size` (mundo), com quebra em `max_width` (mundo), centrado.
pub fn shape(ts: &mut TextSystem, text: &str, font_size: f32, max_width: f32) -> Layout<()> {
    let (fcx, lcx, stack) = ts.contexts();
    let mut b = lcx.ranged_builder(fcx, text, 1.0, false);
    b.push_default(StyleProperty::FontFamily(FontFamily::Source(
        Cow::Borrowed(stack),
    )));
    b.push_default(StyleProperty::FontSize(font_size));
    b.push_default(StyleProperty::FontWeight(FontWeight::NORMAL));
    // Uma palavra maior que a forma quebra a meio em vez de sair por ela.
    b.push_default(StyleProperty::OverflowWrap(OverflowWrap::Anywhere));
    let mut layout: Layout<()> = b.build(text);
    layout.break_all_lines(Some(max_width.max(1.0)));
    layout.align(Alignment::Center, AlignmentOptions::default());
    layout
}

/// Desenha `layout` com `transform` (do espaço do texto — origem no canto superior esquerdo do
/// bloco, em unidades do mundo — para o ecrã).
pub fn paint(scene: &mut VectorScene, layout: &Layout<()>, transform: Affine, color: Color) {
    let inner = scene.inner_mut();
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            let r = run.run();
            inner
                .draw_glyphs(r.font())
                .font_size(r.font_size())
                .hint(false)
                .normalized_coords(r.normalized_coords())
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
        }
    }
}

/// Uma barra por linha (a largura e a altura do x da linha), num só caminho — o que se desenha no
/// lugar do texto quando a letra é pequena demais para ler.
#[must_use]
pub fn line_bars(layout: &Layout<()>) -> ph2d_vector::BezPath {
    let mut p = ph2d_vector::BezPath::new();
    for line in layout.lines() {
        let m = line.metrics();
        let x0 = f64::from(m.offset);
        let w = f64::from(m.advance - m.trailing_whitespace);
        let y1 = f64::from(m.baseline);
        let y0 = y1 - f64::from(m.ascent) * 0.6;
        if w > 0.0 {
            p.extend(ph2d_vector::Rect::new(x0, y0, x0 + w, y1).path_elements(0.1));
        }
    }
    p
}

/// Quantos quadros um moldado fica guardado sem ser usado.
const KEEP_FRAMES: u64 = 120;

struct Entry {
    text: String,
    size_bits: u32,
    width_bits: u32,
    layout: Layout<()>,
    used: u64,
}

/// O moldado de cada dono (uma forma de um quadro), refeito só quando muda.
#[derive(Default)]
pub struct TextCache {
    map: HashMap<(u64, u64), Entry>,
    frame: u64,
    shaped: u64,
}

impl TextCache {
    /// O moldado de `owner`, moldando outra vez só se o texto, o tamanho ou a largura mudaram.
    pub fn get(
        &mut self,
        ts: &mut TextSystem,
        owner: (u64, u64),
        text: &str,
        font_size: f32,
        max_width: f32,
    ) -> &Layout<()> {
        let frame = self.frame;
        let (sb, wb) = (font_size.to_bits(), max_width.to_bits());
        let stale = self
            .map
            .get(&owner)
            .is_none_or(|e| e.text != text || e.size_bits != sb || e.width_bits != wb);
        if stale {
            self.shaped += 1;
            self.map.insert(
                owner,
                Entry {
                    text: text.to_owned(),
                    size_bits: sb,
                    width_bits: wb,
                    layout: shape(ts, text, font_size, max_width),
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

/// ⭐ **Editar o texto de uma forma** — o `PlainEditor` do parley (cursor, selecção, palavras,
/// linhas, apagar) moldado como o texto desenhado ([`shape`]): mesma fonte, tamanho, largura e
/// centrado, para o que se edita ser o que se vê.
pub struct TextEdit {
    editor: PlainEditor<()>,
}

/// Um movimento do cursor (com `extend` = Shift, a selecção cresce em vez de colapsar).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    Left,
    Right,
    WordLeft,
    WordRight,
    Up,
    Down,
    LineStart,
    LineEnd,
    TextStart,
    TextEnd,
}

impl TextEdit {
    /// Abre `text` para edição com tudo seleccionado (escrever substitui; uma seta colapsa).
    #[must_use]
    pub fn new(ts: &mut TextSystem, text: &str, font_size: f32, max_width: f32) -> Self {
        let mut editor = PlainEditor::new(font_size);
        let (_, _, stack) = ts.contexts();
        let styles = editor.edit_styles();
        styles.insert(StyleProperty::FontFamily(FontFamily::Source(Cow::Owned(
            stack.to_owned(),
        ))));
        styles.insert(StyleProperty::FontWeight(FontWeight::NORMAL));
        styles.insert(StyleProperty::OverflowWrap(OverflowWrap::Anywhere));
        editor.set_quantize(false);
        editor.set_alignment(Alignment::Center);
        editor.set_width(Some(max_width.max(1.0)));
        editor.set_text(text);
        let mut e = Self { editor };
        e.with(ts, |d| d.select_all());
        e
    }

    fn with(
        &mut self,
        ts: &mut TextSystem,
        f: impl FnOnce(&mut parley::PlainEditorDriver<'_, ()>),
    ) {
        let (fcx, lcx, _) = ts.contexts();
        f(&mut self.editor.driver(fcx, lcx));
    }

    #[must_use]
    pub fn text(&self) -> String {
        self.editor.raw_text().to_owned()
    }

    /// A largura de quebra mudou (a forma foi redimensionada).
    pub fn set_width(&mut self, max_width: f32) {
        self.editor.set_width(Some(max_width.max(1.0)));
    }

    pub fn insert(&mut self, ts: &mut TextSystem, s: &str) {
        self.with(ts, |d| d.insert_or_replace_selection(s));
    }

    pub fn backspace(&mut self, ts: &mut TextSystem, word: bool) {
        self.with(ts, |d| {
            if word {
                d.backdelete_word()
            } else {
                d.backdelete()
            }
        });
    }

    pub fn delete(&mut self, ts: &mut TextSystem, word: bool) {
        self.with(ts, |d| if word { d.delete_word() } else { d.delete() });
    }

    pub fn select_all(&mut self, ts: &mut TextSystem) {
        self.with(ts, |d| d.select_all());
    }

    /// O texto seleccionado (para copiar), se há selecção.
    #[must_use]
    pub fn selected(&self) -> Option<String> {
        self.editor.selected_text().map(str::to_owned)
    }

    pub fn motion(&mut self, ts: &mut TextSystem, m: Move, extend: bool) {
        self.with(ts, |d| match (m, extend) {
            (Move::Left, false) => d.move_left(),
            (Move::Left, true) => d.select_left(),
            (Move::Right, false) => d.move_right(),
            (Move::Right, true) => d.select_right(),
            (Move::WordLeft, false) => d.move_word_left(),
            (Move::WordLeft, true) => d.select_word_left(),
            (Move::WordRight, false) => d.move_word_right(),
            (Move::WordRight, true) => d.select_word_right(),
            (Move::Up, false) => d.move_up(),
            (Move::Up, true) => d.select_up(),
            (Move::Down, false) => d.move_down(),
            (Move::Down, true) => d.select_down(),
            (Move::LineStart, false) => d.move_to_line_start(),
            (Move::LineStart, true) => d.select_to_line_start(),
            (Move::LineEnd, false) => d.move_to_line_end(),
            (Move::LineEnd, true) => d.select_to_line_end(),
            (Move::TextStart, false) => d.move_to_text_start(),
            (Move::TextStart, true) => d.select_to_text_start(),
            (Move::TextEnd, false) => d.move_to_text_end(),
            (Move::TextEnd, true) => d.select_to_text_end(),
        });
    }

    /// Um clique em `(x, y)` (espaço do texto). `extend` = Shift.
    pub fn click(&mut self, ts: &mut TextSystem, x: f32, y: f32, extend: bool) {
        self.with(ts, |d| {
            if extend {
                d.shift_click_extension(x, y);
            } else {
                d.move_to_point(x, y);
            }
        });
    }

    /// Arrastar com o botão em baixo: a selecção vai até `(x, y)`.
    pub fn drag_to(&mut self, ts: &mut TextSystem, x: f32, y: f32) {
        self.with(ts, |d| d.extend_selection_to_point(x, y));
    }

    /// Duplo-clique: a palavra em `(x, y)`.
    pub fn select_word_at(&mut self, ts: &mut TextSystem, x: f32, y: f32) {
        self.with(ts, |d| d.select_word_at_point(x, y));
    }

    /// O moldado actual (refeito se o texto mudou).
    pub fn layout(&mut self, ts: &mut TextSystem) -> &Layout<()> {
        let (fcx, lcx, _) = ts.contexts();
        self.editor.layout(fcx, lcx)
    }

    /// Os rectângulos da selecção e o do cursor, `[x0, y0, x1, y1]` no espaço do texto. Pede
    /// [`Self::layout`] antes (é ele que põe o moldado em dia).
    #[must_use]
    pub fn decorations(&self, caret_w: f32) -> (Vec<[f64; 4]>, Option<[f64; 4]>) {
        let sel = self
            .editor
            .selection_geometry()
            .into_iter()
            .map(|(b, _)| [b.x0, b.y0, b.x1, b.y1])
            .collect();
        let caret = self
            .editor
            .cursor_geometry(caret_w)
            .map(|b| [b.x0, b.y0, b.x1, b.y1]);
        (sel, caret)
    }
}

#[cfg(test)]
mod tests;
