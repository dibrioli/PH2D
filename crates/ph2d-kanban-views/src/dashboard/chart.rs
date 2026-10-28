//! Um gráfico do painel: barras horizontais (o nome à esquerda, o valor na ponta) ou um anel com a
//! legenda ao lado. As marcas são as da skill de dataviz: barra ≤ 24 px com a ponta de dados
//! arredondada a 4 e quadrada na base, um intervalo de 2 px na cor da superfície entre fatias, a
//! base num filete; o texto veste os tokens de TEXTO, nunca a cor da série.

use std::f32::consts::{FRAC_PI_2, TAU};

use ph2d_editor_core::paint::{
    fill_rounded_rect, fill_rounded_rect_radii, paint_text_elided, paint_text_title,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::{FontWeight, TextSystem};
use ph2d_tokens::{ColorToken as T, Theme};
use ph2d_vector::{Affine, BezPath, Brush, VectorScene};

use super::data::{Bar, Who};
use super::metrics as m;
use crate::common::{avatar_hue, c, initials, label_colors, text_top};

/// O anel pintado: o centro, os raios e o arco de cada fatia (início, fim, índice da barra).
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Ring {
    pub center: (f32, f32),
    pub r_in: f32,
    pub r_out: f32,
    pub spans: Vec<(f32, f32, usize)>,
}

impl Ring {
    /// A fatia sob `(x, y)` (o alvo é o anel inteiro com a folga do levantar, não só a tinta).
    pub(super) fn hit(&self, x: f32, y: f32) -> Option<usize> {
        let (dx, dy) = (x - self.center.0, y - self.center.1);
        let r = dx.hypot(dy);
        if r < self.r_in - m::RING_LIFT || r > self.r_out + m::RING_LIFT {
            return None;
        }
        let a = (dy.atan2(dx) + FRAC_PI_2).rem_euclid(TAU);
        self.spans
            .iter()
            .find(|(a0, a1, _)| (*a0..*a1).contains(&a))
            .map(|s| s.2)
    }
}

/// O que um gráfico pintou: as barras dobradas, o alvo de cada linha e o anel.
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct Painted {
    pub bars: Vec<Bar>,
    /// O alvo de cada barra (a linha inteira: nome, barra e valor).
    pub rows: Vec<(Rect, usize)>,
    pub ring: Option<Ring>,
}

impl Painted {
    pub(super) fn hit(&self, x: f32, y: f32) -> Option<usize> {
        if let Some(r) = &self.ring {
            return r.hit(x, y).or_else(|| self.row_at(x, y));
        }
        self.row_at(x, y)
    }

    fn row_at(&self, x: f32, y: f32) -> Option<usize> {
        self.rows
            .iter()
            .find(|(r, _)| r.x <= x && x < r.x + r.w && r.y <= y && y < r.y + r.h)
            .map(|(_, i)| *i)
    }
}

fn mark_color(theme: Theme, who: &Who) -> ph2d_vector::Color {
    match who {
        Who::Label(_, Some(l)) => label_colors(theme, *l).0,
        other => c(theme, other.token(theme)),
    }
}

fn is_rest(who: &Who) -> bool {
    matches!(who, Who::None | Who::Other)
}

/// O avatar de um membro (a mesma cor que na frente do cartão).
fn avatar(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    who: &ph2d_kanban_model::Member,
    r: Rect,
) {
    let (bg, _) = label_colors(
        theme,
        ph2d_kanban_model::LabelColor {
            hue: avatar_hue(who),
            shade: ph2d_kanban_model::Shade::Normal,
        },
    );
    fill_rounded_rect(out, r, r.w / 2.0, bg);
    let s = initials(who);
    let w = ts.prefix_width(&s, m::AVATAR_PX);
    let y = text_top(ts, r.y, r.h, m::AVATAR_PX);
    paint_text_elided(
        ts,
        out,
        &s,
        r.x + (r.w - w) / 2.0,
        y,
        m::AVATAR_PX,
        r.w,
        c(theme, T::KanbanAvatarInk),
    );
}

/// ⭐ As barras de `bars` em `area`; `hover` é o índice sob o rato.
pub(super) fn bars(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    bars: &[Bar],
    hover: Option<usize>,
) -> Painted {
    let fits = ((area.h + m::BAR_AIR) / (m::BAR_MIN + m::BAR_AIR))
        .floor()
        .max(0.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let bars = super::data::fold(bars, fits as usize);
    let mut painted = Painted {
        bars: bars.clone(),
        ..Painted::default()
    };
    if bars.is_empty() {
        return painted;
    }
    #[allow(clippy::cast_precision_loss)]
    let n = bars.len() as f32;
    let pitch = (area.h / n).min(m::BAR_MAX + m::BAR_AIR);
    let thick = (pitch - m::BAR_AIR).clamp(m::BAR_MIN.min(pitch), m::BAR_MAX);
    let has_avatar = bars.iter().any(|b| matches!(b.who, Who::Member(_)));
    let lead = if has_avatar {
        m::AVATAR + m::SWATCH_GAP
    } else {
        0.0
    };
    let name_w = bars
        .iter()
        .map(|b| ts.prefix_width(&b.name, m::BODY_PX))
        .fold(0.0_f32, f32::max)
        .clamp(m::NAME_MIN_W, m::NAME_MAX_W)
        + lead;
    let max = bars.iter().map(|b| b.count).max().unwrap_or(0).max(1);
    let value_w = ts.prefix_width_weighted(&max.to_string(), m::BODY_PX, FontWeight::SEMI_BOLD)
        + m::VALUE_GAP;
    let x0 = area.x + name_w + m::NAME_GAP;
    let plot_w = (area.x + area.w - value_w - x0).max(0.0);
    let ink = c(theme, T::KanbanCardFg);
    let meta = c(theme, T::KanbanCardMeta);
    for (i, b) in bars.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let y = area.y + pitch * i as f32;
        let row = Rect::new(area.x, y, area.w, pitch);
        painted.rows.push((row, i));
        if hover == Some(i) {
            fill_rounded_rect(out, row, m::BAR_END_RADIUS, c(theme, T::KanbanSlot));
        }
        let mut nx = area.x;
        if let Who::Member(who) = &b.who {
            let ay = y + (pitch - m::AVATAR) / 2.0;
            avatar(out, ts, theme, who, Rect::new(nx, ay, m::AVATAR, m::AVATAR));
            nx += lead;
        }
        let ty = text_top(ts, y, pitch, m::BODY_PX);
        let name_ink = if is_rest(&b.who) { meta } else { ink };
        paint_text_elided(
            ts,
            out,
            &b.name,
            nx,
            ty,
            m::BODY_PX,
            name_w - lead,
            name_ink,
        );
        #[allow(clippy::cast_precision_loss)]
        let len = plot_w * b.count as f32 / max as f32;
        let by = y + (pitch - thick) / 2.0;
        if len > 0.0 {
            let r = m::BAR_END_RADIUS.min(len / 2.0).min(thick / 2.0);
            let bar = Rect::new(x0, by, len, thick);
            fill_rounded_rect_radii(out, bar, (0.0, r, r, 0.0), mark_color(theme, &b.who));
        }
        let vy = text_top(ts, y, pitch, m::BODY_PX);
        paint_text_title(
            ts,
            out,
            &b.count.to_string(),
            x0 + len + m::VALUE_GAP,
            vy,
            m::BODY_PX,
            value_w,
            ink,
        );
    }
    let axis = Rect::new(x0 - m::AXIS_W, area.y, m::AXIS_W, pitch * n);
    out.fill_rect(crate::dashboard::vrect(axis), c(theme, T::Border));
    painted
}

/// Aplana o arco `[a0, a1]` (radianos, 0 = cima, sentido do relógio) de raio `r` em `path`.
fn arc(path: &mut BezPath, center: (f32, f32), r: f32, a0: f32, a1: f32, first: bool) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let steps = (((a1 - a0).abs() * r / m::ARC_STEP).ceil() as usize).max(2);
    for s in 0..=steps {
        #[allow(clippy::cast_precision_loss)]
        let a = a0 + (a1 - a0) * s as f32 / steps as f32 - FRAC_PI_2;
        let p = (
            f64::from(center.0 + r * a.cos()),
            f64::from(center.1 + r * a.sin()),
        );
        if first && s == 0 {
            path.move_to(p);
        } else {
            path.line_to(p);
        }
    }
}

/// Uma fatia do anel.
fn slice(
    out: &mut VectorScene,
    center: (f32, f32),
    r_in: f32,
    r_out: f32,
    a0: f32,
    a1: f32,
    color: ph2d_vector::Color,
) {
    let mut p = BezPath::new();
    arc(&mut p, center, r_out, a0, a1, true);
    arc(&mut p, center, r_in, a1, a0, false);
    p.close_path();
    out.fill_path(&p, &Brush::Solid(color), Affine::IDENTITY);
}

/// ⭐ O anel de `bars` (que PARTEM os cartões) em `area`, com a legenda à direita.
pub(super) fn ring(
    out: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
    area: Rect,
    bars: &[Bar],
    hover: Option<usize>,
) -> Painted {
    let total: usize = bars.iter().map(|b| b.count).sum();
    let legend_w = (area.w * m::LEGEND_SHARE)
        .max(m::LEGEND_MIN_W)
        .min(area.w / 2.0);
    let d = (area.w - legend_w - m::LEGEND_GAP).min(area.h).max(0.0) - 2.0 * m::RING_LIFT;
    let r_out = (d / 2.0).max(0.0);
    let r_in = r_out * (1.0 - m::RING_RATIO);
    let center = (area.x + m::RING_LIFT + r_out, area.y + area.h / 2.0);
    let mut ring = Ring {
        center,
        r_in,
        r_out,
        spans: Vec::new(),
    };
    let ink = c(theme, T::KanbanCardFg);
    let meta = c(theme, T::KanbanCardMeta);
    if total == 0 {
        slice(out, center, r_in, r_out, 0.0, TAU, c(theme, T::KanbanSlot));
    } else {
        let gap = if bars.iter().filter(|b| b.count > 0).count() > 1 {
            m::RING_GAP / r_out.max(1.0)
        } else {
            0.0
        };
        let mut a = 0.0_f32;
        for (i, b) in bars.iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let span = TAU * b.count as f32 / total as f32;
            if span > 0.0 {
                ring.spans.push((a, a + span, i));
                let (s0, s1) = (a + gap / 2.0, a + span - gap / 2.0);
                if s1 > s0 {
                    let lift = if hover == Some(i) { m::RING_LIFT } else { 0.0 };
                    slice(
                        out,
                        center,
                        r_in,
                        r_out + lift,
                        s0,
                        s1,
                        mark_color(theme, &b.who),
                    );
                }
            }
            a += span;
        }
    }
    let num = total.to_string();
    let w = ts.prefix_width_weighted(&num, m::RING_TOTAL_PX, FontWeight::SEMI_BOLD);
    let sub = tr("kanban.views.dashboard.cards");
    let sw = ts.prefix_width(sub, m::SMALL_PX);
    let ny = center.1 - m::RING_TOTAL_PX * m::RING_TOTAL_RISE;
    paint_text_title(
        ts,
        out,
        &num,
        center.0 - w / 2.0,
        ny,
        m::RING_TOTAL_PX,
        r_in * 2.0,
        ink,
    );
    paint_text_elided(
        ts,
        out,
        sub,
        center.0 - sw / 2.0,
        center.1 + m::RING_TOTAL_PX * m::RING_SUB_DROP,
        m::SMALL_PX,
        r_in * 2.0,
        meta,
    );
    let lx = area.x + area.w - legend_w;
    #[allow(clippy::cast_precision_loss)]
    let ly = center.1 - m::LEGEND_ROW_H * bars.len() as f32 / 2.0;
    let mut painted = Painted {
        bars: bars.to_vec(),
        ring: Some(ring),
        ..Painted::default()
    };
    for (i, b) in bars.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let row = Rect::new(
            lx,
            ly + m::LEGEND_ROW_H * i as f32,
            legend_w,
            m::LEGEND_ROW_H,
        );
        painted.rows.push((row, i));
        if hover == Some(i) {
            fill_rounded_rect(out, row, m::BAR_END_RADIUS, c(theme, T::KanbanSlot));
        }
        let sy = row.y + (row.h - m::SWATCH) / 2.0;
        fill_rounded_rect(
            out,
            Rect::new(row.x + m::SWATCH_GAP, sy, m::SWATCH, m::SWATCH),
            m::BAR_END_RADIUS / 2.0,
            mark_color(theme, &b.who),
        );
        let count = b.count.to_string();
        let cw = ts.prefix_width_weighted(&count, m::BODY_PX, FontWeight::SEMI_BOLD);
        let tx = row.x + 2.0 * m::SWATCH_GAP + m::SWATCH;
        let ty = text_top(ts, row.y, row.h, m::BODY_PX);
        let name_ink = if b.count == 0 { meta } else { ink };
        paint_text_elided(
            ts,
            out,
            &b.name,
            tx,
            ty,
            m::BODY_PX,
            row.x + row.w - tx - cw - m::VALUE_GAP,
            name_ink,
        );
        paint_text_title(
            ts,
            out,
            &count,
            row.x + row.w - cw - m::SWATCH_GAP,
            ty,
            m::BODY_PX,
            cw + m::VALUE_GAP,
            name_ink,
        );
    }
    painted
}
