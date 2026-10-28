//! A gaveta pintada: cada miniatura é o fundo verdadeiro pela MESMA lei que pinta o quadro.

use super::{Look, Placed, Row, Target, pattern_of};
use crate::metrics as m;
use crate::paint::{c, icon, text_top};
use crate::tab::Tab;
use ph2d_board_render::kanban_tint;
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_text_elided, paint_text_title_elided, rect_to_vello,
    stroke_rounded_rect,
};
use ph2d_editor_core::widget::hsv_to_rgba8;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_kanban_model::{Background, KanbanSet, Pattern, Tint};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{Brush, Circle, Color, Gradient, Point, RoundedRect, VectorScene};

fn rgba(c: [u8; 4]) -> Color {
    Color::from_rgba8(c[0], c[1], c[2], c[3])
}

fn rounded(r: Rect) -> RoundedRect {
    RoundedRect::new(
        f64::from(r.x),
        f64::from(r.y),
        f64::from(r.x + r.w),
        f64::from(r.y + r.h),
        f64::from(m::BUTTON_RADIUS),
    )
}

/// O fundo que a miniatura de `row` mostra (`None` = não é uma miniatura de fundo).
fn thumb(bg: Background, row: Row) -> Option<Background> {
    match row {
        Row::Color(_) | Row::Gradient(_) | Row::Pattern(_) => super::chosen(bg, row),
        Row::Photo => Some(bg),
        Row::Custom(Target::Solid) => matches!(bg, Background::Custom(_)).then_some(bg),
        _ => None,
    }
}

/// A linha é a escolha actual (o anel).
fn current(bg: Background, row: Row) -> bool {
    match (row, bg) {
        (Row::Color(h), Background::Color(x)) => h == x,
        (Row::Gradient(i), Background::Gradient(x)) => i == x,
        (Row::Pattern(k), Background::Pattern(p)) => p.kind == k,
        (Row::Photo, Background::Image(_)) | (Row::Custom(Target::Solid), Background::Custom(_)) => {
            true
        }
        (Row::Tint(t, h), Background::Pattern(p)) => tint_of(p, t) == Tint::Hue(h),
        (Row::Custom(t), Background::Pattern(p)) if t != Target::Solid => {
            matches!(tint_of(p, t), Tint::Rgb(_))
        }
        _ => false,
    }
}

fn tint_of(p: Pattern, t: Target) -> Tint {
    if t == Target::Ink { p.ink } else { p.base }
}

/// A faixa do arco-íris (a cor livre por escolher e a faixa do matiz do seletor).
fn rainbow(scene: &mut VectorScene, r: Rect) {
    #[allow(clippy::cast_precision_loss)]
    let stops: Vec<Color> = (0..=6)
        .map(|i| rgba(hsv_to_rgba8(i as f32 / 6.0, 1.0, 1.0, 1.0)))
        .collect();
    let g = Gradient::new_linear(
        Point::new(f64::from(r.x), 0.0),
        Point::new(f64::from(r.x + r.w), 0.0),
    )
    .with_stops(stops.as_slice());
    scene.fill_path(
        &ph2d_vector::Shape::to_path(&rounded(r), 0.1),
        &Brush::Gradient(g),
        ph2d_vector::Affine::IDENTITY,
    );
}

/// O seletor da cor livre: o quadrado saturação × valor sobre o matiz, e a faixa do matiz.
fn picker(scene: &mut VectorScene, theme: Theme, p: &Placed, hsv: [f32; 3]) {
    let (Some(sv), Some(hue)) = (p.sv, p.hue) else {
        return;
    };
    let [h, s, v] = hsv;
    let white = rgba(hsv_to_rgba8(0.0, 0.0, 1.0, 1.0));
    let black = rgba(hsv_to_rgba8(0.0, 0.0, 0.0, 1.0));
    fill_rounded_rect(scene, sv, m::BUTTON_RADIUS, rgba(hsv_to_rgba8(h, 1.0, 1.0, 1.0)));
    let shape = ph2d_vector::Shape::to_path(&rounded(sv), 0.1);
    let (x0, y0) = (f64::from(sv.x), f64::from(sv.y));
    let across = Gradient::new_linear(Point::new(x0, y0), Point::new(x0 + f64::from(sv.w), y0))
        .with_stops([white, white.with_alpha(0.0)]);
    let down = Gradient::new_linear(Point::new(x0, y0), Point::new(x0, y0 + f64::from(sv.h)))
        .with_stops([black.with_alpha(0.0), black]);
    let id = ph2d_vector::Affine::IDENTITY;
    scene.fill_path(&shape, &Brush::Gradient(across), id);
    scene.fill_path(&shape, &Brush::Gradient(down), id);
    rainbow(scene, hue);
    let ring = c(theme, T::KanbanCard);
    let edge = c(theme, T::KanbanCardFg);
    for (cx, cy) in [
        (sv.x + s * sv.w, sv.y + (1.0 - v) * sv.h),
        (hue.x + h * hue.w, hue.y + hue.h / 2.0),
    ] {
        let k = Circle::new((f64::from(cx), f64::from(cy)), f64::from(m::BG_KNOB_R));
        let path = ph2d_vector::Shape::to_path(&k, 0.1);
        scene.fill_path(&path, &Brush::Solid(edge), id);
        let inner = Circle::new(
            (f64::from(cx), f64::from(cy)),
            f64::from(m::BG_KNOB_R - m::FOCUS_RING),
        );
        let path = ph2d_vector::Shape::to_path(&inner, 0.1);
        scene.fill_path(&path, &Brush::Solid(ring), id);
    }
}

/// ⭐ A gaveta aberta da aba.
pub(crate) fn paint(
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    set: &KanbanSet,
    tab: &Tab,
    bg: Background,
) {
    let Some(d) = tab.bg.as_deref() else {
        return;
    };
    let p = super::place(d, bg, tab.layout.board);
    crate::paint::shadow(
        scene,
        p.rect,
        0.0,
        c(theme, T::KanbanShadowFlight),
        m::SHADOW_FLIGHT_NEAR,
    );
    fill_rounded_rect(scene, p.rect, 0.0, c(theme, T::KanbanList));
    let fg = c(theme, T::KanbanListFg);
    let meta = c(theme, T::KanbanListMeta);
    let ty = text_top(ts, p.title.y, p.title.h, m::BUTTON_TEXT_PX);
    paint_text_title_elided(
        ts,
        scene,
        tr("kanban.bg.title"),
        p.title.x,
        ty,
        m::BUTTON_TEXT_PX,
        p.title.w,
        fg,
    );
    if d.hover == Some(Row::Close) {
        fill_rounded_rect(scene, p.close, m::BUTTON_RADIUS, c(theme, T::KanbanSlot));
    }
    icon(scene, IconId::Close, p.close, meta);
    scene.push_clip(&rect_to_vello(p.body));
    for (key, r) in &p.sections {
        let ty = text_top(ts, r.y, r.h, m::SMALL_PX);
        paint_text_title_elided(ts, scene, tr(key), r.x, ty, m::SMALL_PX, r.w, meta);
    }
    let ring = c(theme, T::Accent);
    for (row, r) in &p.rows {
        scene.push_clip(&rounded(*r));
        match (*row, thumb(bg, *row)) {
            (Row::Photo, _) => {
                let look = Look::of(set, bg);
                look.paint(scene, theme, *r, 1.0);
            }
            (_, Some(b)) => Look::flat(b).paint(scene, theme, *r, 1.0),
            (Row::Upload, _) => {
                fill_rounded_rect(scene, *r, m::BUTTON_RADIUS, c(theme, T::KanbanSlot));
                let label = tr("kanban.bg.upload");
                let w = ts.prefix_width(label, m::BUTTON_TEXT_PX) + m::ICON_PX + m::HEADER_GAP;
                let x = r.x + ((r.w - w) / 2.0).max(m::HEADER_GAP);
                icon(scene, IconId::Plus, Rect::new(x, r.y, m::ICON_PX, r.h), fg);
                let tx = x + m::ICON_PX + m::HEADER_GAP;
                let ty = text_top(ts, r.y, r.h, m::BUTTON_TEXT_PX);
                paint_text_elided(ts, scene, label, tx, ty, m::BUTTON_TEXT_PX, r.x + r.w - tx, fg);
            }
            (Row::Tint(t, h), _) => {
                let col = kanban_tint(theme, Tint::Hue(h), t == Target::Ink);
                fill_rounded_rect(scene, *r, m::BUTTON_RADIUS, col);
            }
            (Row::Custom(t), _) => {
                match pattern_of(bg).map(|p| tint_of(p, t)) {
                    Some(mine @ Tint::Rgb(_)) => {
                        let col = kanban_tint(theme, mine, t == Target::Ink);
                        fill_rounded_rect(scene, *r, m::BUTTON_RADIUS, col);
                    }
                    _ => rainbow(scene, *r),
                }
                icon(scene, IconId::Plus, *r, c(theme, T::KanbanLabelInkBold));
            }
            _ => {}
        }
        scene.pop_layer();
        if current(bg, *row) || d.custom.is_some_and(|t| *row == Row::Custom(t)) {
            stroke_rounded_rect(scene, *r, m::BUTTON_RADIUS, m::FOCUS_RING, ring);
        } else if d.hover == Some(*row) {
            stroke_rounded_rect(scene, *r, m::BUTTON_RADIUS, m::FOCUS_RING, meta);
        }
    }
    picker(scene, theme, &p, d.hsv);
    scene.pop_layer();
}
