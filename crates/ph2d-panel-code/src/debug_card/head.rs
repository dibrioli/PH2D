//! A faixa de cima do quadro: ■ Paused · line 14 · update() · ● Circle · [chip] … [Continue] F8
//! [Next line] F10. E o ● nome de um objecto (a pílula e a lista Waiting here).

use super::{ButtonIds, DebugCardView};
use ph2d_code_debugcard::{Ask, CardState, CardStyle, DebugCard, WhoTag};
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, frame_radius, paint_text, paint_text_title, resolve,
};
use ph2d_editor_core::widget::{Button, paint_button};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

fn text_w(ts: &mut TextSystem, s: &str) -> f32 {
    ts.layout(s, TypeToken::Sm.px(), f32::INFINITY).width()
}

/// Os botões à direita; devolve o x onde começam.
fn buttons(
    st: &mut DebugCardView,
    c: &DebugCard,
    ids: ButtonIds,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let pad = Spacing::Sm.px();
    let gap = Spacing::Md.px();
    let font = TypeToken::Sm.px();
    let bh = r.h - ph2d_tokens::control_gap_px();
    let mut list = Vec::new();
    if c.stopped() {
        list.push((
            Ask::Continue,
            ids[0],
            tr("code.debug.continue"),
            tr("code.debug.continue_key"),
        ));
        if c.can_step {
            list.push((
                Ask::NextLine,
                ids[1],
                tr("code.debug.next_line"),
                tr("code.debug.next_line_key"),
            ));
        }
    }
    let sized: Vec<_> = list
        .into_iter()
        .map(|(ask, id, label, key)| {
            let bw = text_w(ts, label) + super::inset() * 2.0;
            let kw = text_w(ts, key) + Spacing::Xs.px();
            (ask, id, label, key, bw, kw)
        })
        .collect();
    let total: f32 = sized.iter().map(|b| b.4 + pad + b.5 + gap).sum();
    let start = (r.x + r.w - total + gap).max(r.x);
    let mut bx = start;
    for (i, (ask, id, label, key, bw, kw)) in sized.into_iter().enumerate() {
        let br = Rect::new(bx, r.y + (r.h - bh) * 0.5, bw, bh);
        let b = Button::new(id, label);
        let b = if i == 0 { b.accent() } else { b };
        paint_button(&b, br, scene, ts, theme);
        st.painted.buttons.push((ask, br));
        bx += bw + pad;
        paint_text(
            ts,
            scene,
            key,
            bx,
            r.y + (r.h - font) * 0.5,
            font,
            kw,
            resolve(ColorToken::Text3, theme),
        );
        bx += kw + gap;
    }
    start
}

/// ⭐ A faixa de cima em `r`.
pub(super) fn paint(
    st: &mut DebugCardView,
    c: &DebugCard,
    ids: ButtonIds,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let pad = Spacing::Sm.px();
    let gap = Spacing::Md.px();
    let font = TypeToken::Sm.px();
    let ink = |t: ColorToken| resolve(t, theme);
    let ty = r.y + (r.h - font) * 0.5;
    let bh = r.h - ph2d_tokens::control_gap_px();
    let limit = (buttons(st, c, ids, r, scene, ts, theme) - gap).max(r.x);
    let (word, tone) = match c.state {
        CardState::Paused => (tr("code.debug.paused"), ColorToken::Warn),
        CardState::Error => (tr("code.debug.error"), ColorToken::Danger),
        CardState::Live => (tr("code.debug.live_state"), ColorToken::Success),
    };
    let sq = font * super::MARK_SHARE;
    let mut x = r.x;
    fill_rounded_rect(
        scene,
        Rect::new(x, r.y + (r.h - sq) * 0.5, sq, sq),
        frame_radius(theme, Radius::Xs.px()),
        ink(tone),
    );
    x += sq + Spacing::Xs.px() * 2.0;
    let ww = text_w(ts, word);
    paint_text_title(
        ts,
        scene,
        word,
        x,
        ty,
        font,
        (limit - x).max(0.0),
        ink(tone),
    );
    x += ww + gap;
    if c.stopped() {
        let place = c.place();
        let pw = text_w(ts, &place);
        let room = (limit - x).max(0.0);
        paint_text(ts, scene, &place, x, ty, font, room, ink(ColorToken::Text2));
        x += pw + gap;
    }
    x = who_pill(&c.who, x, r, limit, scene, ts, theme) + gap;
    // Quem espera: um chip nos Cards e na Strip (a Sheet lista-os ao lado).
    let chip = match st.style {
        CardStyle::Sheet => None,
        CardStyle::Cards => c.waiting_text(),
        CardStyle::Strip => (!c.waiting.is_empty()).then(|| {
            let names: Vec<&str> = c.waiting.iter().map(|w| w.name.as_str()).collect();
            tr_with("code.debug.waiting_names", &[("names", &names.join(", "))])
        }),
    };
    if let Some(chip) = chip
        && x < limit
    {
        let cw = (text_w(ts, &chip) + pad * 2.0).min(limit - x);
        let cr = Rect::new(x, r.y + (r.h - bh) * 0.5, cw, bh);
        fill_rounded_rect(
            scene,
            cr,
            frame_radius(theme, Radius::Full.px().min(bh * 0.5)),
            ink(ColorToken::Bg3),
        );
        let room = (cr.w - pad).max(0.0);
        paint_text(
            ts,
            scene,
            &chip,
            cr.x + pad,
            ty,
            font,
            room,
            ink(ColorToken::Text2),
        );
    }
}

/// ● Nome numa pílula, na cor do objecto. Devolve o x depois dela.
fn who_pill(
    who: &WhoTag,
    x: f32,
    r: Rect,
    limit: f32,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) -> f32 {
    if who.name.is_empty() || x >= limit {
        return x;
    }
    let pad = Spacing::Sm.px();
    let bh = r.h - ph2d_tokens::control_gap_px();
    let dot = TypeToken::Sm.px() * 0.5;
    let pw = (dot + super::inset() + pad + text_w(ts, &who.name)).min(limit - x);
    let pill = Rect::new(x, r.y + (r.h - bh) * 0.5, pw, bh);
    fill_rounded_rect(
        scene,
        pill,
        frame_radius(theme, Radius::Full.px().min(bh * 0.5)),
        resolve(ColorToken::Bg3, theme),
    );
    let inner = Rect::new(x + pad, r.y, (pw - pad).max(0.0), r.h);
    who_plain(who, inner, scene, ts, theme);
    x + pw
}

/// ● Nome, sem pílula (a lista Waiting here).
pub(super) fn who_plain(
    who: &WhoTag,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let font = TypeToken::Sm.px();
    let rad = font * super::MARK_SHARE * 0.5;
    let tone = who.color.map_or(ColorToken::Text3, ColorToken::layer);
    fill_circle(scene, r.x + rad, r.y + r.h * 0.5, rad, resolve(tone, theme));
    let tx = r.x + rad * 2.0 + Spacing::Sm.px();
    paint_text(
        ts,
        scene,
        &who.name,
        tx,
        r.y + (r.h - font) * 0.5,
        font,
        (r.x + r.w - tx).max(0.0),
        resolve(ColorToken::Text1, theme),
    );
}
