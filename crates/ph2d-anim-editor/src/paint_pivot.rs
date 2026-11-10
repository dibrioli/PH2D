//! **A pintura do Pivot no Align** ([`super::align_pivot`]): a coluna da moldura (rótulo, grade 5 × 5, «By hand», o
//! nome do ponto) e, na prévia, a caixa tracejada de todos os quadros com os 25 pontos. Lê; não decide nada.

use super::align::View;
use super::align_pivot::{SIDE, grid, grid_point};
use super::doc::AnimDoc;
use super::paint::tom;
use super::paint_align::{Look, chip};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_circle, fill_rounded_rect, frame_radius, paint_text, stroke_polyline, stroke_rounded_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O raio de um ponto da grade, em frações do passo entre pontos; o escolhido é maior.
const DOT: f32 = 0.14; // LITERAL-PX-OK: fração do passo
const DOT_ON: f32 = 0.3; // LITERAL-PX-OK: fração do passo

/// A coluna do Pivot na moldura do Align. Sem o Align ligado, apagada.
pub(super) fn paint_pivot_column(
    doc: &mut AnimDoc,
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
) {
    let l = doc.layout.align;
    if l.pivot_grid.w <= 0.0 {
        return;
    }
    let on = doc.align.on;
    // A submoldura: o Pivot é um assunto à parte dentro do Align.
    let sub = l.pivot_frame;
    let raio = frame_radius(theme, m::FIELD_RADIUS);
    fill_rounded_rect(scene, sub, raio, tom(ColorToken::Bg1, theme));
    let fio = StrokeToken::Thin.px();
    stroke_rounded_rect(scene, sub, raio, fio, tom(ColorToken::Bg3, theme));
    let font = TypeToken::Base.px();
    let t = l.pivot_title;
    let title_c = if on {
        ColorToken::Text2
    } else {
        ColorToken::TextDisabled
    };
    paint_text(
        text,
        scene,
        tr("anim_editor.pivot"),
        t.x,
        t.y,
        font,
        t.w,
        tom(title_c, theme),
    );
    let g = l.pivot_grid;
    fill_rounded_rect(
        scene,
        g,
        frame_radius(theme, m::FIELD_RADIUS),
        tom(ColorToken::Bg3, theme),
    );
    let here = if on { doc.pivot_on_grid() } else { None };
    let step = g.w / f32::from(SIDE);
    let dot = if on {
        ColorToken::Text3
    } else {
        ColorToken::TextDisabled
    };
    for q in grid() {
        let cx = g.x + (f32::from(q[0]) + 0.5) * step; // LITERAL-PX-OK: o meio da casa
        let cy = g.y + (f32::from(q[1]) + 0.5) * step; // LITERAL-PX-OK: o meio da casa
        if here == Some(q) {
            fill_circle(scene, cx, cy, step * DOT_ON, tom(ColorToken::Accent, theme));
        } else {
            fill_circle(scene, cx, cy, step * DOT, tom(dot, theme));
        }
    }
    let hand = (IconId::Move, tr("anim_editor.pivot_by_hand"));
    chip(
        scene,
        text,
        theme,
        l.by_hand,
        hand,
        Look::of(on, doc.align.pivot.by_hand),
        fio,
    );
    let follow = (IconId::Link, tr("anim_editor.pivot_follow"));
    chip(
        scene,
        text,
        theme,
        l.pivot_follow,
        follow,
        Look::of(on, doc.align.pivot.follow),
        fio,
    );
}

/// ⭐ Na prévia: a caixa tracejada do desenho de TODOS os quadros e os 25 pontos sobre ela (o escolhido em acento).
/// Devolve a caixa no ecrã.
pub(super) fn paint_pivot_preview(
    doc: &mut AnimDoc,
    scene: &mut VectorScene,
    v: View,
    theme: Theme,
) -> Option<Rect> {
    let caixa = doc.all_frames_box()?;
    let (lo, hi) = caixa;
    let r = v.quad((
        [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0],
        [hi[0] - lo[0], hi[1] - lo[1]],
    ));
    let warn = tom(ColorToken::Warn, theme);
    let thin = StrokeToken::Thin.px();
    let dash = Spacing::Sm.px();
    let lados = [
        ((r.x, r.y), (r.x + r.w, r.y)),
        ((r.x, r.y + r.h), (r.x + r.w, r.y + r.h)),
        ((r.x, r.y), (r.x, r.y + r.h)),
        ((r.x + r.w, r.y), (r.x + r.w, r.y + r.h)),
    ];
    for ((x0, y0), (x1, y1)) in lados {
        let len = (x1 - x0).hypot(y1 - y0);
        let n = (len / dash).ceil() as usize;
        for i in (0..n).step_by(2) {
            let (a, b) = (
                i as f32 * dash / len,
                ((i + 1) as f32 * dash / len).min(1.0),
            );
            let at = |t: f32| (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
            stroke_polyline(scene, &[at(a), at(b)], thin, warn);
        }
    }
    let here = doc.pivot_on_grid();
    let size = StrokeToken::Thick.px();
    for q in grid() {
        let (x, y) = v.to_screen(grid_point(caixa, q));
        if here == Some(q) {
            continue; // a cruz do Pivot já está lá
        }
        fill_circle(scene, x, y, size + thin, tom(ColorToken::Bg0, theme));
        fill_circle(scene, x, y, size, tom(ColorToken::Text1, theme));
    }
    Some(r)
}
