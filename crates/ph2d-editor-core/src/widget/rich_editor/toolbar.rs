//! A barra de formatação compacta do [`super::RichEditor`] (a do Trello): «Aa», negrito, itálico,
//! riscado, código, listas, checklist, citação, bloco de código, link, régua.

use crate::paint::{fill_rounded_rect, resolve};
use crate::widget::{Popover, paint_popover};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_richdoc::{BlockKind, Mark, RichDoc, Selection};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RichTool {
    TextStyle,
    Bold,
    Italic,
    Strike,
    Code,
    Bullet,
    Numbered,
    Check,
    Quote,
    CodeBlock,
    Link,
    Rule,
}

/// Os botões, pela ordem da barra.
pub const RICH_TOOLS: [RichTool; 12] = [
    RichTool::TextStyle,
    RichTool::Bold,
    RichTool::Italic,
    RichTool::Strike,
    RichTool::Code,
    RichTool::Bullet,
    RichTool::Numbered,
    RichTool::Check,
    RichTool::Quote,
    RichTool::CodeBlock,
    RichTool::Link,
    RichTool::Rule,
];

/// As escolhas do menu «Aa».
pub const TEXT_STYLES: [BlockKind; 4] = [
    BlockKind::Paragraph,
    BlockKind::Heading(1),
    BlockKind::Heading(2),
    BlockKind::Heading(3),
];

/// O botão `.0` (índice em [`RICH_TOOLS`]) sob o rato.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ToolbarHit(pub usize);

pub(super) enum Action {
    StyleMenu,
    Link,
    Mark(Mark),
    Block(BlockKind),
}

fn current_kind(doc: &RichDoc, sel: Selection) -> &BlockKind {
    &doc.blocks()[doc.clamp(sel.head).block].kind
}

/// Um botão de bloco já activo devolve o bloco a parágrafo (como no Trello).
fn block(doc: &RichDoc, sel: Selection, kind: BlockKind) -> Action {
    let cur = current_kind(doc, sel);
    let same = std::mem::discriminant(cur) == std::mem::discriminant(&kind);
    Action::Block(if same { BlockKind::Paragraph } else { kind })
}

pub(super) fn action(tool: RichTool, doc: &RichDoc, sel: Selection) -> Action {
    match tool {
        RichTool::TextStyle => Action::StyleMenu,
        RichTool::Bold => Action::Mark(Mark::Bold),
        RichTool::Italic => Action::Mark(Mark::Italic),
        RichTool::Strike => Action::Mark(Mark::Strike),
        RichTool::Code => Action::Mark(Mark::Code),
        RichTool::Bullet => block(doc, sel, BlockKind::Bullet),
        RichTool::Numbered => block(doc, sel, BlockKind::Numbered),
        RichTool::Check => block(doc, sel, BlockKind::Check(false)),
        RichTool::Quote => block(doc, sel, BlockKind::Quote),
        RichTool::CodeBlock => block(doc, sel, BlockKind::Code(String::new())),
        RichTool::Link => Action::Link,
        RichTool::Rule => Action::Block(BlockKind::Rule),
    }
}

/// O botão está «ligado» (a marca cobre a selecção, o bloco é desse tipo).
fn is_on(tool: RichTool, doc: &RichDoc, sel: Selection, active: &[Mark]) -> bool {
    let has = |m: Mark| active.iter().any(|a| a.same_kind(&m));
    let kind = current_kind(doc, sel);
    match tool {
        RichTool::Bold => has(Mark::Bold),
        RichTool::Italic => has(Mark::Italic),
        RichTool::Strike => has(Mark::Strike),
        RichTool::Code => has(Mark::Code),
        RichTool::Link => has(Mark::Link(String::new())),
        RichTool::Bullet => *kind == BlockKind::Bullet,
        RichTool::Numbered => *kind == BlockKind::Numbered,
        RichTool::Check => matches!(kind, BlockKind::Check(_)),
        RichTool::Quote => *kind == BlockKind::Quote,
        RichTool::CodeBlock => kind.is_code(),
        RichTool::TextStyle => matches!(kind, BlockKind::Heading(_)),
        RichTool::Rule => false,
    }
}

fn side() -> f32 {
    ROW_H_PX
}

/// A caixa do botão `i` na barra `bar`.
#[must_use]
pub fn button(bar: Rect, i: usize) -> Rect {
    let s = side();
    // Um respiro depois do «Aa», das marcas e dos blocos (os três grupos do Trello).
    let groups = [0usize, 1, 5, 10]
        .iter()
        .filter(|g| i >= **g && **g > 0)
        .count();
    let x = bar.x + i as f32 * s + groups as f32 * Spacing::Sm.px();
    Rect::new(x, bar.y, s, bar.h)
}

pub(super) fn hit(bar: Rect, x: f32, y: f32) -> Option<ToolbarHit> {
    (0..RICH_TOOLS.len())
        .find(|i| {
            let r = button(bar, *i);
            x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h && r.x + r.w <= bar.x + bar.w
        })
        .map(ToolbarHit)
}

/// A caixa do menu «Aa» aberto e a de cada opção.
#[must_use]
pub fn style_menu(bar: Rect) -> Rect {
    let b = button(bar, 0);
    let w = side() * TEXT_STYLES.len() as f32;
    Rect::new(b.x, b.y + b.h, w, side() * TEXT_STYLES.len() as f32)
}

#[must_use]
pub fn style_option(bar: Rect, i: usize) -> Rect {
    let m = style_menu(bar);
    Rect::new(m.x, m.y + i as f32 * side(), m.w, side())
}

pub(super) fn style_option_at(bar: Rect, x: f32, y: f32) -> Option<usize> {
    (0..TEXT_STYLES.len()).find(|i| {
        let r = style_option(bar, *i);
        x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_toolbar(
    ed: &super::RichEditor,
    bar: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    let doc = ed.doc();
    let sel = ed.selection();
    let active = ed.editor.active_marks();
    for (i, tool) in RICH_TOOLS.iter().enumerate() {
        let r = button(bar, i);
        if r.x + r.w > bar.x + bar.w {
            break;
        }
        let on = is_on(*tool, doc, sel, &active);
        if on {
            fill_rounded_rect(scene, r, radius, resolve(ColorToken::AccentSoft, theme));
        } else if ed.hover == Some(ToolbarHit(i)) {
            fill_rounded_rect(scene, r, radius, resolve(ColorToken::Bg2, theme));
        }
        let ink = if on {
            ColorToken::Accent
        } else {
            ColorToken::Text2
        };
        crate::paint::paint_text_centered_com_orcamento(
            ts,
            scene,
            &ed.labels.tools[i],
            r,
            r.w,
            TypeToken::Sm.px(),
            resolve(ink, theme),
        );
    }
}

pub(super) fn paint_style_menu(
    ed: &super::RichEditor,
    bar: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let m = style_menu(bar);
    paint_popover(
        &Popover::new(NodeId(ed.id.0.wrapping_add(2))),
        m,
        scene,
        theme,
    );
    let cur = current_kind(ed.doc(), ed.selection());
    for (i, kind) in TEXT_STYLES.iter().enumerate() {
        let r = style_option(bar, i);
        if kind == cur {
            crate::widget::paint_popover_row_highlight(scene, r, theme);
        }
        let size = super::layout::font_px(kind);
        let pad = Spacing::Md.px();
        crate::paint::paint_text_title(
            ts,
            scene,
            &ed.labels.styles[i],
            r.x + pad,
            r.y + (r.h - size) * 0.5,
            size,
            r.w - pad * 2.0,
            resolve(ColorToken::Text1, theme),
        );
    }
}
