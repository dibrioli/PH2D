//! As outras duas aparências: **Cards** (um cartão por grupo e a Call stack, cada valor numa caixa
//! de campo, como o Inspector) e **Strip** (uma célula por variável de cima, nome pequeno em cima e
//! valor em baixo, a altura de antes).

use super::{CardBox, DebugCardView};
use crate::peso::paint_text;
use crate::table_text::CodeCells;
use crate::view::CodeView;
use ph2d_code_debugcard::{DebugCard, VarGroup, VarRow, shown_value, value_ink};
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, paint_text_title, resolve, token_to_vello,
};
use ph2d_editor_core::widget::scroll_area::OwnBar;
use ph2d_editor_core::widget::{ScrollbarState, paint_scrollbar, scrollbar_track_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_ui_table::CellText;
use ph2d_vector::VectorScene;

/// A parte do cartão que a coluna dos nomes pode tomar.
const NAME_SHARE: f32 = 0.45; // LITERAL-PX-OK: fração da largura do cartão, não píxeis

/// O título e a contagem de um cartão; devolve o y da 1.ª linha.
pub(super) fn card_head(
    card: Rect,
    title: &str,
    count: &str,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let font = TypeToken::Xs.px();
    let radius = frame_radius(theme, Radius::Md.px());
    let ground = resolve(crate::cards::ground_tone(), theme);
    fill_rounded_rect(scene, card, radius, ground);
    let ty = card.y + (rh - font) * 0.5;
    let tw = crate::peso::largura(ts, count, font);
    let ink = resolve(ColorToken::Text2, theme);
    paint_text_title(ts, scene, title, card.x + pad, ty, font, card.w, ink);
    let cx = card.x + card.w - pad - tw;
    paint_text(
        ts,
        scene,
        count,
        cx,
        ty,
        font,
        tw + pad,
        resolve(ColorToken::Text3, theme),
    );
    card.y + rh
}

/// Uma variável num cartão: o nome recuado, o valor numa caixa de campo (âmbar se mudou).
#[allow(clippy::too_many_arguments)]
fn field_row(
    st: &mut DebugCardView,
    v: &VarRow,
    (card, y, name_w): (Rect, f32, f32),
    view: &mut CodeView,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let gap = ph2d_tokens::control_gap_px();
    let radius = frame_radius(theme, Radius::Sm.px());
    let nx = card.x + pad + f32::from(v.depth) * ph2d_tokens::list_indent_px();
    let hint = crate::style::Ink::Hint;
    view.paint_code_value(
        ts,
        scene,
        theme,
        &v.name,
        hint,
        (nx, y, rh),
        card.x + pad + name_w,
    );
    let fx = card.x + pad * 2.0 + name_w;
    let field = Rect::new(
        fx,
        y + gap * 0.5,
        (card.w - name_w - super::inset() - pad).max(0.0),
        rh - gap,
    );
    let fill = resolve(crate::cards::card_tone(), theme);
    fill_rounded_rect(scene, field, radius, fill);
    let edge = if v.old.is_some() {
        token_to_vello(ph2d_tokens::changed_ink(theme))
    } else {
        resolve(ColorToken::Border, theme)
    };
    crate::cards::value_outline(scene, field, radius, edge);
    let text = shown_value(v);
    let right = field.x + field.w - pad;
    let mut cells = CodeCells { view, theme };
    let vx = (right - cells.width(ts, &text, true)).max(field.x + pad);
    cells.paint(ts, scene, &text, value_ink(v), true, vx, (y, rh), right);
    st.painted.values.push((v.value.clone(), field));
}

/// A rolagem do cartão `i` com `n` linhas (presa às dele e em linhas inteiras: nenhuma pinta a
/// meio): regista o cartão, pinta o indicador da casa se transborda e devolve `(primeira, quantas,
/// o cartão sem a faixa do indicador)`.
fn scroll_card(
    st: &mut DebugCardView,
    i: usize,
    card: Rect,
    n: usize,
    scene: &mut VectorScene,
    theme: Theme,
) -> (usize, usize, Rect) {
    let rh = ph2d_tokens::row_pitch_px();
    let fits = ((card.h - rh).max(0.0) / rh).floor() as usize;
    let over = n.saturating_sub(fits);
    let max = over as f32 * rh;
    st.cards_scroll[i] = st.cards_scroll[i].min(max);
    let first = ((st.cards_scroll[i] / rh).round() as usize).min(over);
    let rows = Rect::new(card.x, card.y + rh, card.w, fits as f32 * rh);
    let state = if st.cards_drag.is_some_and(|(k, _)| k == i) {
        ScrollbarState::Dragging
    } else {
        ScrollbarState::Normal
    };
    let content = n as f32 * rh;
    let (y, visual) = (first as f32 * rh, (state, 0.0));
    let bar = paint_scrollbar(rows, y, content, rows.h, visual, scene, theme);
    let track = OwnBar {
        body: rows,
        content_h: content,
        visible_h: rows.h,
    };
    st.painted.cards.push(CardBox {
        rect: card,
        max,
        bar,
        track,
    });
    let keep = bar.map_or(card.w, |_| scrollbar_track_rect(rows).x - card.x);
    (first, fits.min(n), Rect::new(card.x, card.y, keep, card.h))
}

/// ⭐ Cards de `c` em `body`: Locals, self e a Call stack, cada um com a sua rolagem (a roda sobre
/// um cartão rola só esse; o título fica).
pub(super) fn paint_cards(
    st: &mut DebugCardView,
    c: &DebugCard,
    body: Rect,
    view: &mut CodeView,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let gap = Spacing::Md.px();
    let n = 3.0; // LITERAL-PX-OK: três cartões (Locals, self, Call stack), não píxeis
    let cw = ((body.w - gap * (n + 1.0)) / n).max(0.0);
    let ch = (body.h - pad * 2.0).max(0.0);
    let adv = view.metrics().advance;
    let mut cx = body.x + gap;
    for (i, (group, vars)) in c.groups().into_iter().enumerate() {
        let title = match group {
            VarGroup::Locals => tr("code.debug.group.locals"),
            VarGroup::This => tr("code.debug.group.self"),
        };
        let card = Rect::new(cx, body.y + pad, cw, ch);
        let count = vars.iter().filter(|v| v.depth == 0).count().to_string();
        let y0 = card_head(card, title, &count, scene, ts, theme);
        let (first, fits, inner) = scroll_card(st, i, card, vars.len(), scene, theme);
        let name_w = vars
            .iter()
            .map(|v| (v.name.chars().count() as f32 + f32::from(v.depth) * 2.0) * adv)
            .fold(0.0f32, f32::max)
            .min(inner.w * NAME_SHARE);
        for (k, v) in vars.iter().skip(first).take(fits).enumerate() {
            let y = y0 + rh * k as f32;
            field_row(st, v, (inner, y, name_w), view, scene, ts, theme);
        }
        cx += cw + gap;
    }
    let card = Rect::new(cx, body.y + pad, cw, ch);
    let y0 = card_head(card, tr("code.debug.stack"), "", scene, ts, theme);
    let (first, fits, inner) = scroll_card(st, 2, card, c.stack.len(), scene, theme);
    let font = TypeToken::Sm.px();
    for (k, s) in c.stack.iter().skip(first).take(fits).enumerate() {
        let y = y0 + rh * k as f32;
        let name = if s.function.is_empty() {
            tr("code.debug.top_level").to_owned()
        } else {
            s.function.clone()
        };
        let t = tr_with(
            "code.debug.frame",
            &[("function", &name), ("line", &s.line)],
        );
        let w = (inner.w - pad * 2.0).max(0.0);
        let ink = resolve(ColorToken::Text1, theme);
        paint_text(
            ts,
            scene,
            &t,
            inner.x + pad,
            y + (rh - font) * 0.5,
            font,
            w,
            ink,
        );
        st.painted
            .stack
            .push((s.line, Rect::new(inner.x, y, inner.w, rh)));
    }
}

/// ⭐ Strip de `c` em `body`: as variáveis de cima (locais, depois o self), lado a lado.
pub(super) fn paint_strip(
    st: &mut DebugCardView,
    c: &DebugCard,
    body: Rect,
    view: &mut CodeView,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let small = TypeToken::Xs.px();
    let ink = |t: ColorToken| resolve(t, theme);
    let adv = view.metrics().advance;
    let locals = c.locals.iter().filter(|l| l.depth == 0).count();
    let vars = c.locals.iter().chain(&c.this).filter(|v| v.depth == 0);
    // As células que cabem, na largura natural; a sobra reparte-se por elas (a faixa enche-se).
    let mut cells: Vec<(&VarRow, String, String, f32)> = Vec::new();
    let mut used = 0.0;
    for (i, v) in vars.enumerate() {
        let key = if v.old.is_some() {
            tr_with("code.debug.type_changed", &[("type", &v.key)])
        } else if i < locals {
            tr_with("code.debug.local_name", &[("name", &v.key)])
        } else {
            v.key.clone()
        };
        let value = shown_value(v);
        let kw = crate::peso::largura(ts, &key, small);
        let cw = (value.chars().count() as f32 * adv).max(kw) + super::inset() * 2.0;
        if used + cw > body.w && !cells.is_empty() {
            break;
        }
        used += cw;
        cells.push((v, key, value, cw));
    }
    let extra = (body.w - used).max(0.0) / cells.len().max(1) as f32;
    let mut x = body.x;
    for (v, key, value, cw) in cells {
        let cell = Rect::new(x, body.y, (cw + extra).min(body.x + body.w - x), body.h);
        if v.old.is_some() {
            fill_rounded_rect(scene, cell, 0.0, ph2d_ui_table::changed_tint(theme));
        }
        if x > body.x {
            let hair = Rect::new(x, body.y, StrokeToken::Thin.px(), body.h);
            fill_rounded_rect(scene, hair, 0.0, ink(ColorToken::Border));
        }
        let tone = if v.old.is_some() {
            token_to_vello(ph2d_tokens::changed_ink(theme))
        } else {
            ink(ColorToken::Text3)
        };
        let kx = x + pad * 2.0;
        let room = (cell.w - super::inset() - pad).max(0.0);
        paint_text(ts, scene, &key, kx, body.y + pad, small, room, tone);
        let right = cell.x + cell.w - pad;
        let mut text = CodeCells { view, theme };
        let row = (body.y + pad + small, rh);
        text.paint(ts, scene, &value, value_ink(v), true, kx, row, right);
        st.painted.values.push((v.value.clone(), cell));
        x += cell.w;
    }
}
