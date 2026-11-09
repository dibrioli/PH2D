//! As outras duas aparências: **Cards** (um cartão por grupo e a Call stack, cada valor numa caixa
//! de campo, como o Inspector) e **Strip** (uma célula por variável de cima, nome pequeno em cima e
//! valor em baixo, a altura de antes).

use super::DebugCardView;
use crate::table_text::CodeCells;
use crate::view::CodeView;
use ph2d_code_debugcard::{DebugCard, VarGroup, VarRow, shown_value, value_ink};
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, paint_text, paint_text_title, resolve, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_ui_table::CellText;
use ph2d_vector::VectorScene;

/// A parte do cartão que a coluna dos nomes pode tomar.
const NAME_SHARE: f32 = 0.45; // LITERAL-PX-OK: fração da largura do cartão, não píxeis

/// O título e a contagem de um cartão; devolve o y da 1.ª linha.
fn card_head(
    card: Rect,
    title: &str,
    count: &str,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let rh = ph2d_tokens::row_pitch_px();
    let pad = Spacing::Sm.px();
    let font = TypeToken::Sm.px();
    let radius = frame_radius(theme, Radius::Sm.px());
    fill_rounded_rect(scene, card, radius, resolve(ColorToken::Bg2, theme));
    let ty = card.y + (rh - font) * 0.5;
    let tw = ts.layout(count, font, f32::INFINITY).width();
    let ink = resolve(ColorToken::Text1, theme);
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
    let nx = card.x + pad + f32::from(v.depth) * Spacing::Lg.px();
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
    fill_rounded_rect(scene, field, radius, resolve(ColorToken::Bg1, theme));
    if v.old.is_some() {
        let warn = resolve(ColorToken::Warn, theme);
        stroke_rounded_rect(scene, field, radius, StrokeToken::Thin.px(), warn);
    }
    let text = shown_value(v);
    let right = field.x + field.w - pad;
    let mut cells = CodeCells { view, theme };
    let vx = (right - cells.width(ts, &text, true)).max(field.x + pad);
    cells.paint(ts, scene, &text, value_ink(v), true, vx, (y, rh), right);
    st.painted.values.push((v.value.clone(), field));
}

/// ⭐ Cards de `c` em `body`.
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
    let groups = c.groups();
    // As linhas rolam por baixo dos títulos; a mais comprida decide quanto.
    let most = groups.iter().map(|g| g.1.len()).max().unwrap_or(0);
    let room = (ch - rh).max(0.0);
    st.painted.cards_max = (rh * most.max(c.stack.len()) as f32 - room).max(0.0);
    st.cards_scroll = st.cards_scroll.min(st.painted.cards_max);
    let shown = |card: Rect, y: f32| y >= card.y + rh && y + rh <= card.y + card.h;
    let mut cx = body.x + gap;
    for (group, vars) in groups {
        let title = match group {
            VarGroup::Locals => tr("code.debug.group.locals"),
            VarGroup::This => tr("code.debug.group.self"),
        };
        let card = Rect::new(cx, body.y + pad, cw, ch);
        let count = vars.iter().filter(|v| v.depth == 0).count().to_string();
        let mut y = card_head(card, title, &count, scene, ts, theme) - st.cards_scroll;
        let name_w = vars
            .iter()
            .map(|v| (v.name.chars().count() as f32 + f32::from(v.depth) * 2.0) * adv)
            .fold(0.0f32, f32::max)
            .min(card.w * NAME_SHARE);
        for v in vars {
            if shown(card, y) {
                field_row(st, v, (card, y, name_w), view, scene, ts, theme);
            }
            y += rh;
        }
        cx += cw + gap;
    }
    let card = Rect::new(cx, body.y + pad, cw, ch);
    let mut y = card_head(card, tr("code.debug.stack"), "", scene, ts, theme) - st.cards_scroll;
    let font = TypeToken::Sm.px();
    for s in &c.stack {
        if !shown(card, y) {
            y += rh;
            continue;
        }
        let name = if s.function.is_empty() {
            tr("code.debug.top_level").to_owned()
        } else {
            s.function.clone()
        };
        let t = tr_with(
            "code.debug.frame",
            &[("function", &name), ("line", &s.line)],
        );
        let w = (card.w - pad * 2.0).max(0.0);
        let ink = resolve(ColorToken::Text1, theme);
        paint_text(
            ts,
            scene,
            &t,
            card.x + pad,
            y + (rh - font) * 0.5,
            font,
            w,
            ink,
        );
        st.painted
            .stack
            .push((s.line, Rect::new(card.x, y, card.w, rh)));
        y += rh;
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
        let kw = ts.layout(&key, small, f32::INFINITY).width();
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
            ColorToken::Warn
        } else {
            ColorToken::Text3
        };
        let kx = x + pad * 2.0;
        let room = (cell.w - super::inset() - pad).max(0.0);
        paint_text(ts, scene, &key, kx, body.y + pad, small, room, ink(tone));
        let right = cell.x + cell.w - pad;
        let mut text = CodeCells { view, theme };
        let row = (body.y + pad + small, rh);
        text.paint(ts, scene, &value, value_ink(v), true, kx, row, right);
        st.painted.values.push((v.value.clone(), cell));
        x += cell.w;
    }
}
