//! A disposição de um [`RichDoc`]: um layout do parley por bloco, com os estilos das marcas por
//! intervalo. O MESMO layout pinta, mede e acha o caret (clique, setas, selecção).

use crate::zones::Rect;
use parley::{
    Affinity, Alignment, AlignmentOptions, Cursor, FontFamily, FontStyle, FontWeight, Layout,
    LayoutContext, LineHeight, OverflowWrap, Selection as PSelection, StyleProperty,
};
use ph2d_a11y::NodeId;
use ph2d_richdoc::{BlockKind, Caret, Mark, RichDoc, Selection};
use ph2d_text::TextSystem;
use ph2d_tokens::{FONT_MONO, LineHeight as LineToken, Spacing, StrokeToken, TypeToken};
use std::borrow::Cow;
use std::cell::RefCell;

/// A tinta de um troço (o pintor resolve-a no tema).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Ink {
    #[default]
    Body,
    /// Links e menções.
    Accent,
    /// Item de checklist marcado.
    Muted,
    /// Código inline (fundo próprio).
    Code,
}

/// O que se desenha à esquerda do texto de um bloco.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Marker {
    None,
    Bullet,
    Number(usize),
    Check(bool),
}

pub struct BlockBox {
    pub index: usize,
    pub kind: BlockKind,
    /// A caixa inteira do bloco (fundo do código, barra da citação, régua).
    pub rect: Rect,
    /// A origem do texto.
    pub text_x: f32,
    pub text_y: f32,
    pub layout: Layout<Ink>,
    pub marker: Marker,
    /// A caixa do marcador (o visto é clicável).
    pub marker_rect: Rect,
    /// O comprimento do texto, em bytes.
    pub len: usize,
}

pub struct DocLayout {
    pub blocks: Vec<BlockBox>,
    pub height: f32,
}

thread_local! {
    static LCX: RefCell<LayoutContext<Ink>> = RefCell::new(LayoutContext::new());
}

/// O peso da letra de um tipo de bloco.
#[must_use]
pub fn font_weight(kind: &BlockKind) -> FontWeight {
    if matches!(kind, BlockKind::Heading(_)) {
        FontWeight::BOLD
    } else {
        FontWeight::MEDIUM
    }
}

/// O tamanho da letra de um tipo de bloco.
#[must_use]
pub fn font_px(kind: &BlockKind) -> f32 {
    let t = match kind {
        BlockKind::Heading(1) => TypeToken::Xl,
        BlockKind::Heading(2) => TypeToken::Lg,
        BlockKind::Heading(_) => TypeToken::Md,
        BlockKind::Code(_) => TypeToken::Sm,
        _ => TypeToken::Base,
    };
    ph2d_text::displayed_font_px(t.px())
}

pub(super) fn line_ratio() -> f32 {
    LineToken::Normal.ratio()
}

fn shape(ts: &mut TextSystem, doc: &RichDoc, i: usize, max_w: f32) -> Layout<Ink> {
    let b = &doc.blocks()[i];
    let text = b.inline.text();
    let size = font_px(&b.kind);
    let (fcx, _, stack) = ts.contexts();
    let family = if b.kind.is_code() {
        Cow::Borrowed(FONT_MONO)
    } else {
        Cow::Owned(stack.to_owned())
    };
    LCX.with(|lcx| {
        let mut lcx = lcx.borrow_mut();
        let mut bld = lcx.ranged_builder(fcx, text, 1.0, true);
        bld.push_default(StyleProperty::FontFamily(FontFamily::Source(family)));
        bld.push_default(StyleProperty::FontSize(size));
        bld.push_default(StyleProperty::FontWeight(font_weight(&b.kind)));
        bld.push_default(StyleProperty::LineHeight(LineHeight::FontSizeRelative(
            line_ratio(),
        )));
        bld.push_default(StyleProperty::OverflowWrap(OverflowWrap::Anywhere));
        let ink = if b.kind == BlockKind::Check(true) {
            Ink::Muted
        } else {
            Ink::Body
        };
        bld.push_default(StyleProperty::Brush(ink));
        for s in b.inline.marks() {
            let r = s.start..s.end;
            match &s.mark {
                Mark::Bold => bld.push(StyleProperty::FontWeight(FontWeight::BOLD), r),
                Mark::Italic => bld.push(StyleProperty::FontStyle(FontStyle::Italic), r),
                Mark::Strike => bld.push(StyleProperty::Strikethrough(true), r),
                Mark::Code => {
                    bld.push(
                        StyleProperty::FontFamily(FontFamily::Source(Cow::Borrowed(FONT_MONO))),
                        r.clone(),
                    );
                    bld.push(StyleProperty::Brush(Ink::Code), r);
                }
                Mark::Link(_) => {
                    bld.push(StyleProperty::Underline(true), r.clone());
                    bld.push(StyleProperty::Brush(Ink::Accent), r);
                }
                Mark::Mention => {
                    bld.push(StyleProperty::FontWeight(FontWeight::SEMI_BOLD), r.clone());
                    bld.push(StyleProperty::Brush(Ink::Accent), r);
                }
            }
        }
        let mut layout = bld.build(text);
        layout.break_all_lines(Some(max_w.max(1.0)));
        layout.align(Alignment::Start, AlignmentOptions::default());
        layout
    })
}

/// As medidas de bloco, todas dos tokens.
pub(super) fn indent_step() -> f32 {
    Spacing::Xl.px()
}
pub(super) fn code_pad() -> f32 {
    Spacing::Md.px()
}
fn block_gap() -> f32 {
    Spacing::Xs.px()
}

impl DocLayout {
    /// Dispõe `doc` na largura `w`, com o topo em `(x, y)`.
    pub fn build(ts: &mut TextSystem, doc: &RichDoc, x: f32, y: f32, w: f32) -> Self {
        let mut blocks = Vec::with_capacity(doc.blocks().len());
        let mut cy = y;
        // A numeração de cada nível: recomeça quando outro tipo interrompe a lista.
        let mut numbers: Vec<usize> = Vec::new();
        for (i, b) in doc.blocks().iter().enumerate() {
            if i > 0 {
                cy += block_gap();
                if matches!(b.kind, BlockKind::Heading(_)) {
                    cy += Spacing::Sm.px();
                }
            }
            let indent = b.indent as f32 * indent_step();
            numbers.truncate(b.indent + 1);
            let marker = match b.kind {
                BlockKind::Bullet => Marker::Bullet,
                BlockKind::Check(v) => Marker::Check(v),
                BlockKind::Numbered => {
                    numbers.resize(b.indent + 1, 0);
                    numbers[b.indent] += 1;
                    Marker::Number(numbers[b.indent])
                }
                _ => Marker::None,
            };
            if b.kind != BlockKind::Numbered && numbers.len() > b.indent {
                numbers[b.indent] = 0;
            }
            let gutter = if marker == Marker::None {
                0.0
            } else {
                indent_step()
            };
            let (pad_x, pad_y) = match b.kind {
                BlockKind::Code(_) => (code_pad(), code_pad()),
                BlockKind::Quote => (Spacing::Md.px(), 0.0),
                _ => (0.0, 0.0),
            };
            let text_x = x + indent + gutter + pad_x;
            let text_w = (x + w - text_x - pad_x).max(1.0);
            let layout = shape(ts, doc, i, text_w);
            let size = font_px(&b.kind);
            let text_h = if b.kind == BlockKind::Rule {
                Spacing::Lg.px()
            } else {
                layout.height().max(size * line_ratio())
            };
            let h = text_h + pad_y * 2.0;
            let rect = Rect::new(x + indent, cy, (w - indent).max(0.0), h);
            let first_line_h = size * line_ratio();
            let marker_rect = Rect::new(x + indent, cy + pad_y, gutter, first_line_h);
            blocks.push(BlockBox {
                index: i,
                kind: b.kind.clone(),
                rect,
                text_x,
                text_y: cy + pad_y,
                layout,
                marker,
                marker_rect,
                len: b.inline.len(),
            });
            cy += h;
        }
        Self {
            blocks,
            height: cy - y,
        }
    }

    /// O bloco mais perto da altura `y`.
    fn block_at_y(&self, y: f32) -> &BlockBox {
        self.blocks
            .iter()
            .find(|b| y < b.rect.y + b.rect.h)
            .unwrap_or_else(|| &self.blocks[self.blocks.len() - 1])
    }

    /// O caret sob `(x, y)`.
    #[must_use]
    pub fn caret_at(&self, x: f32, y: f32) -> Caret {
        let b = self.block_at_y(y);
        if b.kind == BlockKind::Rule {
            return Caret::new(b.index, 0);
        }
        let c = Cursor::from_point(&b.layout, x - b.text_x, y - b.text_y);
        Caret::new(b.index, c.index())
    }

    /// A palavra sob `(x, y)` (o duplo clique).
    #[must_use]
    pub fn word_at(&self, x: f32, y: f32) -> Selection {
        let b = self.block_at_y(y);
        let s = PSelection::word_from_point(&b.layout, x - b.text_x, y - b.text_y);
        let r = s.text_range();
        Selection::new(Caret::new(b.index, r.start), Caret::new(b.index, r.end))
    }

    /// A caixa do caret `c`.
    #[must_use]
    pub fn caret_rect(&self, c: Caret) -> Rect {
        let Some(b) = self.blocks.get(c.block) else {
            return Rect::new(0.0, 0.0, 0.0, 0.0);
        };
        let w = StrokeToken::Default.px();
        let size = font_px(&b.kind);
        if b.kind == BlockKind::Rule {
            return Rect::new(b.text_x, b.rect.y, w, b.rect.h);
        }
        let g =
            Cursor::from_byte_index(&b.layout, c.byte, Affinity::Downstream).geometry(&b.layout, w);
        let h = (g.y1 - g.y0) as f32;
        Rect::new(
            b.text_x + g.x0 as f32,
            b.text_y + g.y0 as f32,
            w,
            if h > 0.0 { h } else { size * line_ratio() },
        )
    }

    /// Os rectângulos da selecção.
    #[must_use]
    pub fn selection_rects(&self, sel: Selection) -> Vec<Rect> {
        let (a, z) = sel.ordered();
        let mut out = Vec::new();
        for b in &self.blocks {
            if b.index < a.block || b.index > z.block {
                continue;
            }
            if b.kind == BlockKind::Rule {
                out.push(b.rect);
                continue;
            }
            let s = if b.index == a.block { a.byte } else { 0 };
            let e = if b.index == z.block { z.byte } else { b.len };
            out.extend(self.range_rects(b, s, e));
            if b.index < z.block {
                // O fim do bloco também fica marcado (a quebra entra na selecção).
                let end = self.caret_rect(Caret::new(b.index, b.len));
                out.push(Rect::new(end.x, end.y, Spacing::Xs.px(), end.h));
            }
        }
        out
    }

    /// Os rectângulos do troço `s..e` do bloco `b` (selecção e fundo do código inline).
    #[must_use]
    pub fn range_rects(&self, b: &BlockBox, s: usize, e: usize) -> Vec<Rect> {
        if s >= e {
            return Vec::new();
        }
        let ps = PSelection::new(
            Cursor::from_byte_index(&b.layout, s, Affinity::Downstream),
            Cursor::from_byte_index(&b.layout, e, Affinity::Upstream),
        );
        ps.geometry(&b.layout)
            .into_iter()
            .map(|(g, _)| {
                Rect::new(
                    b.text_x + g.x0 as f32,
                    b.text_y + g.y0 as f32,
                    (g.x1 - g.x0) as f32,
                    (g.y1 - g.y0) as f32,
                )
            })
            .collect()
    }

    /// O item de checklist cujo visto está sob `(x, y)`.
    #[must_use]
    pub fn check_at(&self, x: f32, y: f32) -> Option<usize> {
        self.blocks.iter().find_map(|b| {
            let r = b.marker_rect;
            (matches!(b.marker, Marker::Check(_))
                && x >= r.x
                && x < r.x + r.w
                && y >= r.y
                && y < r.y + r.h)
                .then_some(b.index)
        })
    }
}

/// O id a11y do documento, derivado do do editor.
#[must_use]
pub fn doc_node_id(editor: NodeId) -> NodeId {
    NodeId(editor.0.wrapping_add(1))
}
