//! A tinta do [`super::RichEditor`]: blocos (títulos, listas, checklist, citação, código,
//! régua), marcas inline, selecção, caret, pré-edição e a barra.

use super::layout::{BlockBox, Ink, Marker, indent_step};
use super::{RichEditor, toolbar};
use crate::icons::IconId;
use crate::paint::{
    fill_circle, fill_rounded_rect, paint_icon, paint_text, rect_to_vello, resolve, stroke_rounded_rect,
};
use crate::widget::TextInputState;
use crate::zones::Rect;
use parley::PositionedLayoutItem;
use ph2d_a11y::NodeId;
use ph2d_richdoc::{BlockKind, Mark};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::{Affine, Glyph, GlyphStyle, VectorScene};

fn ink(i: Ink, quote: bool, theme: Theme) -> ph2d_vector::Color {
    let t = match i {
        Ink::Accent => ColorToken::Accent,
        Ink::Muted => ColorToken::Text3,
        Ink::Body | Ink::Code if quote => ColorToken::Text2,
        Ink::Body | Ink::Code => ColorToken::Text1,
    };
    resolve(t, theme)
}

/// Os glifos de um bloco, com sublinhado e risco.
fn paint_glyphs(scene: &mut VectorScene, b: &BlockBox, text: &str, theme: Theme) {
    let quote = b.kind == BlockKind::Quote;
    let transform = Affine::translate((f64::from(b.text_x), f64::from(b.text_y)));
    for line in b.layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(run) = item else {
                continue;
            };
            let style = run.style();
            let color = ink(style.brush, quote, theme);
            let r = run.run();
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
            let x0 = b.text_x + run.offset();
            let w = run.advance();
            let mut bar = |offset: f32, size: f32| {
                let y = b.text_y + run.baseline() - offset;
                scene.fill_rect(rect_to_vello(Rect::new(x0, y, w, size)), color);
            };
            if let Some(u) = &style.underline {
                bar(u.offset.unwrap_or(m.underline_offset), u.size.unwrap_or(m.underline_size));
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

fn paint_marker(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    b: &BlockBox,
    theme: Theme,
) {
    let r = b.marker_rect;
    let cy = r.y + r.h * 0.5;
    match b.marker {
        Marker::None => {}
        Marker::Bullet => {
            let rad = (Spacing::Xs.px() + Spacing::Xxs.px()) * 0.5;
            fill_circle(scene, r.x + r.w * 0.5, cy, rad, resolve(ColorToken::Text2, theme));
        }
        Marker::Number(n) => {
            let f = super::layout::font_px(&b.kind);
            paint_text(
                ts,
                scene,
                &format!("{n}."),
                r.x,
                r.y + (r.h - f) * 0.5,
                f,
                r.w,
                resolve(ColorToken::Text2, theme),
            );
        }
        Marker::Check(v) => {
            // A caixa do Trello: contorno vazia, cheia de acento com o visto quando marcada.
            let side = (r.h.min(indent_step()) - Spacing::Xs.px()).max(1.0);
            let bx = Rect::new(r.x, r.y + (r.h - side) * 0.5, side, side);
            let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
            if v {
                fill_rounded_rect(scene, bx, radius, resolve(ColorToken::Accent, theme));
                let pad = Spacing::Xxs.px();
                let g = Rect::new(bx.x + pad, bx.y + pad, bx.w - pad * 2.0, bx.h - pad * 2.0);
                paint_icon(scene, IconId::Check, g, resolve(ColorToken::AccentFg, theme), StrokeToken::Default.px() * 2.0);
            } else {
                stroke_rounded_rect(scene, bx, radius, StrokeToken::Default.px() * 2.0, resolve(ColorToken::Text3, theme));
            }
        }
    }
}

/// Fundo do código, barra da citação, régua.
fn paint_block_chrome(scene: &mut VectorScene, b: &BlockBox, theme: Theme) {
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    match b.kind {
        BlockKind::Code(_) => fill_rounded_rect(scene, b.rect, radius, resolve(ColorToken::Bg2, theme)),
        BlockKind::Quote => {
            let bar = Rect::new(b.rect.x, b.rect.y, Spacing::Xs.px(), b.rect.h);
            fill_rounded_rect(scene, bar, 0.0, resolve(ColorToken::BorderStrong, theme));
        }
        BlockKind::Rule => {
            let t = StrokeToken::Default.px();
            let line = Rect::new(b.rect.x, b.rect.y + (b.rect.h - t) * 0.5, b.rect.w, t);
            scene.fill_rect(rect_to_vello(line), resolve(ColorToken::Border, theme));
        }
        _ => {}
    }
}

pub fn paint_rich_editor(
    ed: &mut RichEditor,
    rect: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    let (bar, area) = ed.regions(rect);
    if ed.editing {
        let state = TextInputState::Focused;
        fill_rounded_rect(scene, rect, radius, crate::widget::text_input::field_fill(state, theme));
        crate::paint::stroke_frame(
            scene,
            rect,
            radius,
            theme,
            crate::widget::text_input::feel_of(state),
            StrokeToken::Default.px(),
            crate::widget::text_input::border_color(state, crate::motion::SETTLED, theme),
        );
    }
    if let Some(bar) = bar {
        toolbar::paint_toolbar(ed, bar, scene, ts, theme);
    }
    scene.push_clip(&rect_to_vello(rect));
    let editing = ed.editing;
    let sel = ed.selection();
    let empty = ed.doc().is_empty();
    let preedit = ed.preedit.clone();
    let doc = ed.doc().clone();
    let placeholder = ed.labels.placeholder.clone();
    let l = ed.layout(ts, rect);

    for b in &l.blocks {
        paint_block_chrome(scene, b, theme);
    }
    if editing && !sel.is_collapsed() {
        for r in l.selection_rects(sel) {
            fill_rounded_rect(scene, r, 0.0, resolve(ColorToken::Selection, theme));
        }
    }
    for b in &l.blocks {
        let inline = &doc.blocks()[b.index].inline;
        for s in inline.marks().iter().filter(|s| s.mark == Mark::Code) {
            for r in l.range_rects(b, s.start, s.end) {
                fill_rounded_rect(scene, r, radius, resolve(ColorToken::Bg3, theme));
            }
        }
        paint_marker(scene, ts, b, theme);
        paint_glyphs(scene, b, inline.text(), theme);
    }
    if empty && !placeholder.is_empty() {
        let f = super::layout::font_px(&BlockKind::Paragraph);
        paint_text(
            ts,
            scene,
            &placeholder,
            area.x,
            area.y,
            f,
            area.w,
            resolve(ColorToken::Text3, theme),
        );
    }
    if editing {
        let c = l.caret_rect(sel.head);
        if let Some(p) = preedit {
            let f = TypeToken::Base.px();
            paint_text(ts, scene, &p, c.x, c.y, f, area.x + area.w - c.x, resolve(ColorToken::Text2, theme));
            let w = ts.prefix_width(&p, f);
            let t = StrokeToken::Default.px();
            scene.fill_rect(rect_to_vello(Rect::new(c.x, c.y + c.h - t, w, t)), resolve(ColorToken::Text2, theme));
        } else {
            scene.fill_rect(rect_to_vello(c), resolve(ColorToken::Accent, theme));
        }
    }
    scene.pop_layer();
    if let Some(bar) = bar
        && ed.style_menu_open
    {
        toolbar::paint_style_menu(ed, bar, scene, ts, theme);
    }
}

/// O id a11y da área do documento (para o chamador registar o alvo do clique).
#[must_use]
pub fn area_node_id(editor: NodeId) -> NodeId {
    super::layout::doc_node_id(editor)
}
