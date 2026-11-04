//! A tinta do [`super::RichEditor`]: blocos (títulos, listas, checklist, citação, código,
//! régua), marcas inline, selecção, caret, pré-edição e a barra.

use super::layout::{BlockBox, Ink, Marker, Regua, indent_step};
use super::{RichEditor, toolbar};
use crate::icons::IconId;
use crate::paint::{
    fill_circle, fill_rounded_rect, paint_icon, paint_text, rect_to_vello, resolve,
    stroke_rounded_rect,
};
use crate::widget::TextInputState;
use crate::widget::card_ink::{card_body, tinta};
use crate::zones::Rect;
use parley::PositionedLayoutItem;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
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
    // Num cartão a citação não se confunde com o item feito (esse é o tom do rótulo).
    match card_body() {
        Some(ink) if t == ColorToken::Text2 => ink.color(t),
        _ => tinta(t, theme),
    }
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
            let coords = ph2d_text::run_coord_bits(r);
            let gs = GlyphStyle {
                font: r.font(),
                size: r.font_size(),
                coords: &coords,
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
            let m = r.font_metrics();
            let x0 = b.text_x + run.offset();
            let w = run.advance();
            let mut bar = |offset: f32, size: f32| {
                let y = b.text_y + run.baseline() - offset;
                scene.fill_rect(rect_to_vello(Rect::new(x0, y, w, size)), color);
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

fn paint_marker(scene: &mut VectorScene, ts: &mut TextSystem, b: &BlockBox, theme: Theme) {
    if let Some(ink) = card_body() {
        return paint_marker_no_cartao(scene, ts, b, ink);
    }
    let r = b.marker_rect;
    let cy = r.y + r.h * 0.5;
    match b.marker {
        Marker::None => {}
        Marker::Bullet => {
            let rad = (Spacing::Xs.px() + Spacing::Xxs.px()) * 0.5;
            fill_circle(
                scene,
                r.x + r.w * 0.5,
                cy,
                rad,
                resolve(ColorToken::Text2, theme),
            );
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
                paint_icon(
                    scene,
                    IconId::Check,
                    g,
                    resolve(ColorToken::AccentFg, theme),
                    StrokeToken::Default.px() * 2.0,
                );
            } else {
                // FRAME-RAW-OK: a caixa VAZIA de uma checklist do DOCUMENTO — o contorno é o glifo
                stroke_rounded_rect(
                    scene,
                    bx,
                    radius,
                    StrokeToken::Default.px() * 2.0,
                    resolve(ColorToken::Text3, theme),
                );
            }
        }
    }
}

/// Os marcadores na coluna `CHECK_BOX` do cartão: a caixa dele, o ponto e o número ao centro dela.
fn paint_marker_no_cartao(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    b: &BlockBox,
    ink: crate::widget::card_ink::CardInk,
) {
    let r = b.marker_rect;
    let tom = ink.color(ColorToken::CardLabel);
    match b.marker {
        Marker::None => {}
        Marker::Bullet => {
            let cx = r.x + m::CHECK_BOX * 0.5;
            fill_circle(scene, cx, r.y + r.h * 0.5, m::SEG_PAD, tom);
        }
        Marker::Number(n) => {
            let coluna = Rect::new(r.x, r.y, m::CHECK_BOX, r.h);
            let texto = format!("{n}.");
            crate::paint::paint_text_centered_com_orcamento(
                ts, scene, &texto, coluna, coluna.w, b.font, tom,
            );
        }
        Marker::Check(v) => {
            let caixa = crate::widget::checkbox::card_check_rect(r.x, r.y, r.h);
            crate::widget::checkbox::paint_card_check(scene, ink, caixa, v, false);
        }
    }
}

/// Código sobre a caixa do cartão, citação com o traço fino na coluna, régua fina.
fn paint_block_chrome_no_cartao(
    scene: &mut VectorScene,
    b: &BlockBox,
    ink: crate::widget::card_ink::CardInk,
) {
    match b.kind {
        BlockKind::Code(_) => {
            let fundo = ink.color(ColorToken::CardField);
            fill_rounded_rect(scene, b.rect, m::FIELD_RADIUS, fundo);
        }
        BlockKind::Quote => {
            let t = Spacing::Xxs.px();
            let bar = Rect::new(b.rect.x, b.rect.y, t, b.rect.h);
            fill_rounded_rect(scene, bar, t * 0.5, ink.color(ColorToken::CardLabel));
        }
        BlockKind::Rule => {
            let t = m::FIELD_BORDER;
            let line = Rect::new(b.rect.x, b.rect.y + (b.rect.h - t) * 0.5, b.rect.w, t);
            scene.fill_rect(rect_to_vello(line), ink.color(ColorToken::Border));
        }
        _ => {}
    }
}

/// O código inline sobre a caixa do cartão, com um respiro de cada lado sem sair da coluna.
fn paint_inline_code_no_cartao(
    scene: &mut VectorScene,
    b: &BlockBox,
    r: Rect,
    ink: crate::widget::card_ink::CardInk,
) {
    let x = (r.x - Spacing::Xxs.px()).max(b.rect.x);
    let w = r.x + r.w + Spacing::Xxs.px() - x;
    let raio = m::FIELD_RADIUS.min(r.h * 0.5);
    fill_rounded_rect(
        scene,
        Rect::new(x, r.y, w, r.h),
        raio,
        ink.color(ColorToken::CardField),
    );
}

/// Fundo do código, barra da citação, régua.
fn paint_block_chrome(scene: &mut VectorScene, b: &BlockBox, theme: Theme) {
    if let Some(ink) = card_body() {
        return paint_block_chrome_no_cartao(scene, b, ink);
    }
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    match b.kind {
        BlockKind::Code(_) => {
            fill_rounded_rect(scene, b.rect, radius, resolve(ColorToken::Bg2, theme))
        }
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
    ed.regua = Regua::daqui();
    let (bar, area) = ed.regions(rect);
    let ink_cartao = card_body();
    if let (true, Some(ink)) = (ed.editing, ink_cartao) {
        // A moldura de foco na folga do cartão (sem fundo: o código é que senta na caixa).
        let f = Rect::new(
            rect.x - m::SEG_INSET,
            rect.y - m::SEG_INSET,
            rect.w + m::SEG_INSET * 2.0,
            rect.h + m::SEG_INSET * 2.0,
        );
        let azul = ink.color(ColorToken::KanbanBlueBold);
        stroke_rounded_rect(scene, f, m::FIELD_RADIUS, m::FIELD_BORDER, azul);
    } else if ed.editing {
        let state = TextInputState::Focused;
        fill_rounded_rect(
            scene,
            rect,
            radius,
            crate::widget::text_input::field_fill(state, theme),
        );
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
    let ed_regua = ed.regua;
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
                if let Some(ink) = ink_cartao {
                    paint_inline_code_no_cartao(scene, b, r, ink);
                } else {
                    fill_rounded_rect(scene, r, radius, resolve(ColorToken::Bg3, theme));
                }
            }
        }
        paint_marker(scene, ts, b, theme);
        paint_glyphs(scene, b, inline.text(), theme);
    }
    if empty && !placeholder.is_empty() {
        let f = ed_regua.font_px(&BlockKind::Paragraph);
        paint_text(
            ts,
            scene,
            &placeholder,
            area.x,
            area.y,
            f,
            area.w,
            tinta(ColorToken::Text3, theme),
        );
    }
    if editing {
        let c = l.caret_rect(sel.head);
        if let Some(p) = preedit {
            let f = if ink_cartao.is_some() {
                ed_regua.font_px(&BlockKind::Paragraph)
            } else {
                TypeToken::Base.px()
            };
            paint_text(
                ts,
                scene,
                &p,
                c.x,
                c.y,
                f,
                area.x + area.w - c.x,
                tinta(ColorToken::Text2, theme),
            );
            let w = ts.prefix_width(&p, f);
            let t = StrokeToken::Default.px();
            scene.fill_rect(
                rect_to_vello(Rect::new(c.x, c.y + c.h - t, w, t)),
                tinta(ColorToken::Text2, theme),
            );
        } else {
            scene.fill_rect(rect_to_vello(c), tinta(ColorToken::Accent, theme));
        }
    }
    scene.pop_layer();
}

/// O menu «Aa» aberto, por CIMA de tudo o que o chamador pinta depois do editor (pinte-o no fim):
/// devolve a caixa dele, que o chamador regista como alvo do editor — senão o clique numa opção
/// que cai sobre um botão vizinho vai para o botão.
pub fn paint_rich_editor_menu(
    ed: &RichEditor,
    rect: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) -> Option<Rect> {
    let (bar, _) = ed.regions(rect);
    let bar = bar.filter(|_| ed.style_menu_open)?;
    Some(toolbar::paint_style_menu(ed, bar, scene, ts, theme))
}

/// O id a11y da área do documento (para o chamador registar o alvo do clique).
#[must_use]
pub fn area_node_id(editor: NodeId) -> NodeId {
    super::layout::doc_node_id(editor)
}
