//! A faixa de cima do quadro: ■ Paused · line 14 · update() · ● Circle · [chip] … [Continue] F8
//! [Next line] F10. E o ● nome de um objecto (a pílula e a lista Waiting here).

use super::{ButtonIds, DebugCardView};
use crate::metrics::top_on_baseline;
use ph2d_code_debugcard::{Ask, CardState, CardStyle, DebugCard, WhoTag};
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, folga_do_texto, frame_radius, paint_text, paint_text_title,
    resolve, token_to_vello,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

fn text_w(ts: &mut TextSystem, s: &str) -> f32 {
    ts.layout(s, TypeToken::Sm.px(), f32::INFINITY).width()
}

/// Os botões à direita, a tecla DENTRO de cada um (maquete 14): Continue cheio no acento, Next
/// line neutro com moldura. Devolve o x onde começam.
fn buttons(
    st: &mut DebugCardView,
    c: &DebugCard,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let pad = Spacing::Md.px();
    let gap = Spacing::Sm.px();
    let font = TypeToken::Sm.px();
    let small = TypeToken::Xs.px();
    let bh = r.h - ph2d_tokens::control_gap_px();
    let ink = |t: ColorToken| resolve(t, theme);
    let mut list = Vec::new();
    if c.stopped() {
        list.push((
            Ask::Continue,
            tr("code.debug.continue"),
            tr("code.debug.continue_key"),
        ));
        if c.can_step {
            list.push((
                Ask::NextLine,
                tr("code.debug.next_line"),
                tr("code.debug.next_line_key"),
            ));
        }
    }
    let sized: Vec<_> = list
        .into_iter()
        .map(|(ask, label, key)| {
            let lw = text_w(ts, label);
            let kw = ts.layout(key, small, f32::INFINITY).width();
            (ask, label, key, lw, kw, lw + kw + gap + pad * 2.0)
        })
        .collect();
    let total: f32 = sized.iter().map(|b| b.5 + gap).sum();
    let start = (r.x + r.w - total + gap).max(r.x);
    let mut bx = start;
    let radius = frame_radius(theme, Radius::Sm.px());
    for (i, (ask, label, key, lw, kw, bw)) in sized.into_iter().enumerate() {
        let br = Rect::new(bx, r.y + (r.h - bh) * 0.5, bw, bh);
        let (fill, fg, key_ink) = if i == 0 {
            (
                ColorToken::Accent,
                ColorToken::AccentFg,
                ColorToken::AccentFg,
            )
        } else {
            (ColorToken::Bg3, ColorToken::Text1, ColorToken::Text3)
        };
        fill_rounded_rect(scene, br, radius, ink(fill));
        if i > 0 {
            let line = StrokeToken::Thin.px();
            ph2d_editor_core::paint::stroke_frame(
                scene,
                br,
                radius,
                theme,
                ph2d_tokens::visuals::Feel::Rest,
                line,
                ink(ColorToken::Border),
            );
        }
        // O rótulo centrado pelas maiúsculas; a tecla na linha de base DELE (dono 10/11).
        let ty = br.y + folga_do_texto(br.h, font);
        paint_text(ts, scene, label, br.x + pad, ty, font, lw + gap, ink(fg));
        let kx = br.x + pad + lw + gap;
        let ky = top_on_baseline(ts, ty, (label, font), (key, small));
        paint_text(ts, scene, key, kx, ky, small, kw + gap, ink(key_ink));
        st.painted.buttons.push((ask, br));
        bx += bw + gap;
    }
    start
}

/// `line 14 · update()`: a palavra e o ponto apagados, o número e a função em destaque.
fn place(
    c: &DebugCard,
    x: f32,
    ty: f32,
    limit: f32,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) -> f32 {
    let font = TypeToken::Sm.px();
    let space = Spacing::Xs.px();
    let f = if c.function.is_empty() {
        tr("code.debug.top_level").to_owned()
    } else {
        tr_with("code.debug.call", &[("function", &c.function)])
    };
    let parts = [
        (tr("code.debug.line_word").to_owned(), false),
        (c.line.to_string(), true),
        (tr("code.debug.sep").to_owned(), false),
        (f, true),
    ];
    let mut x = x;
    for (s, strong) in parts {
        let room = (limit - x).max(0.0);
        if strong {
            let tone = resolve(ColorToken::Text1, theme);
            paint_text_title(ts, scene, &s, x, ty, font, room, tone);
            x += ts.layout(&s, font, f32::INFINITY).width();
        } else {
            paint_text(
                ts,
                scene,
                &s,
                x,
                ty,
                font,
                room,
                resolve(ColorToken::Text2, theme),
            );
            x += text_w(ts, &s);
        }
        x += space;
    }
    x
}

/// ⭐ A faixa de cima em `r`.
pub(super) fn paint(
    st: &mut DebugCardView,
    c: &DebugCard,
    _ids: ButtonIds,
    r: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let pad = Spacing::Sm.px();
    let gap = Spacing::Md.px();
    let font = TypeToken::Sm.px();
    let ink = |t: ColorToken| resolve(t, theme);
    let ty = r.y + folga_do_texto(r.h, font);
    let bh = r.h - ph2d_tokens::control_gap_px();
    let mut limit = (buttons(st, c, r, scene, ts, theme) - gap).max(r.x);
    if c.state == CardState::Live {
        // O ritmo da amostra à direita (a maquete: «2.5×/s»).
        let rate = crate::live_sampler::rate_text();
        let small = TypeToken::Xs.px();
        let rw = ts.layout(&rate, small, f32::INFINITY).width();
        let rx = (r.x + r.w - rw).max(r.x);
        let tone = ink(ColorToken::Text3);
        paint_text(
            ts,
            scene,
            &rate,
            rx,
            r.y + folga_do_texto(r.h, small),
            small,
            rw + pad,
            tone,
        );
        limit = (rx - gap).max(r.x);
    }
    let (word, tone) = match c.state {
        CardState::Paused => (
            tr("code.debug.paused"),
            token_to_vello(ph2d_tokens::changed_ink(theme)),
        ),
        CardState::Error => (tr("code.debug.error"), ink(ColorToken::Danger)),
        CardState::Live => (tr("code.debug.live_state"), ink(ColorToken::Success)),
    };
    let sq = font * super::MARK_SHARE;
    let mut x = r.x;
    // ■ parado, ● ao vivo (a maquete 14).
    let corner = if c.stopped() {
        StrokeToken::Thick.px()
    } else {
        sq * 0.5
    };
    let mark = Rect::new(x, r.y + (r.h - sq) * 0.5, sq, sq);
    fill_rounded_rect(scene, mark, corner, tone);
    x += sq + Spacing::Xs.px() * 2.0;
    let ww = text_w(ts, word);
    paint_text_title(ts, scene, word, x, ty, font, (limit - x).max(0.0), tone);
    x += ww + gap;
    if c.stopped() {
        x = place(c, x, ty, limit, scene, ts, theme) + pad;
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
        let small = TypeToken::Xs.px();
        let chip_w = ts.layout(&chip, small, f32::INFINITY).width();
        let cw = (chip_w + pad * 2.0).min(limit - x);
        let cr = Rect::new(x, r.y + (r.h - bh) * 0.5, cw, bh);
        fill_rounded_rect(
            scene,
            cr,
            frame_radius(theme, Radius::Full.px().min(bh * 0.5)),
            ink(ColorToken::Bg3),
        );
        let room = (cr.w - pad).max(0.0);
        let cy = cr.y + folga_do_texto(cr.h, small);
        let tone = ink(ColorToken::Text2);
        paint_text(ts, scene, &chip, cr.x + pad, cy, small, room, tone);
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
    let bh = r.h - ph2d_tokens::control_gap_px() * 2.0;
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
    who_plain(who, inner, ColorToken::Text1, scene, ts, theme);
    x + pw
}

/// ● Nome, sem pílula (a lista Waiting here).
pub(super) fn who_plain(
    who: &WhoTag,
    r: Rect,
    ink: ColorToken,
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
        r.y + folga_do_texto(r.h, font),
        font,
        (r.x + r.w - tx).max(0.0),
        resolve(ink, theme),
    );
}
