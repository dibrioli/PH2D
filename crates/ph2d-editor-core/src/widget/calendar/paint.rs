//! A disposição e a tinta do [`super::Calendar`] — a MESMA disposição responde ao pintor e ao
//! clique (uma régua só).

use super::{CALENDAR_CELLS, CALENDAR_COLS, CALENDAR_ROWS, Calendar};
use crate::icons::IconId;
use crate::paint::{
    fill_rounded_rect, paint_icon, paint_text_centered, paint_text_title, resolve,
    stroke_rounded_rect,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Density, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O que está sob um ponto do calendário.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CalendarHit {
    Cell(usize),
    Prev,
    Next,
    Today,
}

/// As caixas do calendário dentro de `rect`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CalendarLayout {
    pub prev: Rect,
    pub next: Rect,
    pub today: Rect,
    pub title: Rect,
    pub weekdays: Rect,
    pub grid: Rect,
    pub cell_w: f32,
    pub cell_h: f32,
}

impl CalendarLayout {
    fn header_h() -> f32 {
        ROW_H_PX
    }
    fn weekday_h() -> f32 {
        Density::Compact.row_h_px()
    }
    fn cell_h() -> f32 {
        ROW_H_PX
    }

    /// A altura que o calendário pede (a largura é a do chamador).
    #[must_use]
    pub fn preferred_height() -> f32 {
        Self::header_h() + Self::weekday_h() + Self::cell_h() * CALENDAR_ROWS as f32
    }

    #[must_use]
    pub fn of(rect: Rect) -> Self {
        let hh = Self::header_h();
        let side = hh;
        let prev = Rect::new(rect.x, rect.y, side, hh);
        let next = Rect::new(rect.x + rect.w - side, rect.y, side, hh);
        let today_w = side * 2.0 + Spacing::Xl2.px();
        let today = Rect::new(next.x - today_w, rect.y, today_w, hh);
        let title = Rect::new(
            prev.x + side,
            rect.y,
            (today.x - prev.x - side).max(0.0),
            hh,
        );
        let weekdays = Rect::new(rect.x, rect.y + hh, rect.w, Self::weekday_h());
        let cell_h = Self::cell_h();
        let grid = Rect::new(
            rect.x,
            weekdays.y + weekdays.h,
            rect.w,
            cell_h * CALENDAR_ROWS as f32,
        );
        Self {
            prev,
            next,
            today,
            title,
            weekdays,
            grid,
            cell_w: rect.w / CALENDAR_COLS as f32,
            cell_h,
        }
    }

    /// A caixa da célula `i` (0..42, por linhas).
    #[must_use]
    pub fn cell(&self, i: usize) -> Rect {
        let (r, c) = (i / CALENDAR_COLS, i % CALENDAR_COLS);
        Rect::new(
            self.grid.x + c as f32 * self.cell_w,
            self.grid.y + r as f32 * self.cell_h,
            self.cell_w,
            self.cell_h,
        )
    }

    #[must_use]
    pub fn hit(&self, x: f32, y: f32) -> Option<CalendarHit> {
        let inside = |r: Rect| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h;
        if inside(self.prev) {
            return Some(CalendarHit::Prev);
        }
        if inside(self.next) {
            return Some(CalendarHit::Next);
        }
        if inside(self.today) {
            return Some(CalendarHit::Today);
        }
        if !inside(self.grid) || self.cell_w <= 0.0 {
            return None;
        }
        let c = ((x - self.grid.x) / self.cell_w) as usize;
        let r = ((y - self.grid.y) / self.cell_h) as usize;
        let i = r * CALENDAR_COLS + c.min(CALENDAR_COLS - 1);
        (i < CALENDAR_CELLS).then_some(CalendarHit::Cell(i))
    }
}

/// O id a11y de cada célula, derivado do id do calendário.
#[must_use]
pub fn cell_node_id(cal: NodeId, i: usize) -> NodeId {
    NodeId(cal.0.wrapping_add(1 + i as u64))
}

/// O quadrado de cor por trás do número (o dia, não a célula inteira).
fn chip(cell: Rect) -> Rect {
    let pad = Spacing::Xxs.px();
    Rect::new(
        cell.x + pad,
        cell.y + pad,
        cell.w - pad * 2.0,
        cell.h - pad * 2.0,
    )
}

pub fn paint_calendar(
    cal: &Calendar,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let l = CalendarLayout::of(rect);
    let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
    let hover_fill = resolve(ColorToken::Bg2, theme);
    let stroke = StrokeToken::Default.px();
    let icon_pad = Spacing::Xs.px();

    // Cabeçalho: ‹ título [hoje] ›
    for (hit, r, icon) in [
        (CalendarHit::Prev, l.prev, IconId::ChevronLeft),
        (CalendarHit::Next, l.next, IconId::ChevronRight),
    ] {
        if cal.hover == Some(hit) {
            fill_rounded_rect(scene, chip(r), radius, hover_fill);
        }
        let g = Rect::new(
            r.x + icon_pad,
            r.y + icon_pad,
            r.w - icon_pad * 2.0,
            r.h - icon_pad * 2.0,
        );
        paint_icon(scene, icon, g, resolve(ColorToken::Text2, theme), stroke);
    }
    if cal.hover == Some(CalendarHit::Today) {
        fill_rounded_rect(scene, chip(l.today), radius, hover_fill);
    }
    paint_text_centered(
        text_system,
        scene,
        &cal.today_label,
        l.today,
        TypeToken::Sm.px(),
        resolve(ColorToken::Accent, theme),
    );
    let tf = TypeToken::Base.px();
    paint_text_title(
        text_system,
        scene,
        &cal.title,
        l.title.x + Spacing::Xs.px(),
        l.title.y + (l.title.h - tf) * 0.5,
        tf,
        l.title.w,
        resolve(ColorToken::Text1, theme),
    );

    // Os dias da semana.
    for (c, name) in cal.weekdays.iter().enumerate() {
        let r = Rect::new(
            l.weekdays.x + c as f32 * l.cell_w,
            l.weekdays.y,
            l.cell_w,
            l.weekdays.h,
        );
        paint_text_centered(
            text_system,
            scene,
            name,
            r,
            TypeToken::Xs.px(),
            resolve(ColorToken::Text3, theme),
        );
    }

    // A grelha.
    for (i, cell) in cal.cells.iter().enumerate() {
        let r = l.cell(i);
        let d = cell.date();
        let selected = cal.selected == Some(d);
        let in_range = cal.in_range(d);
        let c = chip(r);
        if in_range && !selected {
            // A faixa do intervalo ocupa a célula inteira na horizontal: as pontas tocam-se.
            let band = Rect::new(r.x, c.y, r.w, c.h);
            fill_rounded_rect(scene, band, 0.0, resolve(ColorToken::AccentSoft, theme));
        }
        if selected {
            fill_rounded_rect(scene, c, radius, resolve(ColorToken::Accent, theme));
        } else if cal.hover == Some(CalendarHit::Cell(i)) {
            fill_rounded_rect(scene, c, radius, hover_fill);
        }
        if cell.is_today && !selected {
            // FRAME-RAW-OK: o anel de HOJE — o contorno de acento é a marca do dia, não moldura
            stroke_rounded_rect(scene, c, radius, stroke, resolve(ColorToken::Accent, theme));
        }
        if cal.focused && cal.focus == Some(d) {
            // FRAME-RAW-OK: o ANEL DE FOCO do teclado na célula — vê-se em qualquer pele
            stroke_rounded_rect(
                scene,
                r,
                radius,
                stroke * 2.0,
                resolve(ColorToken::FocusRing, theme),
            );
        }
        let ink = if selected {
            ColorToken::AccentFg
        } else if cell.is_today {
            ColorToken::Accent
        } else if cell.in_month {
            ColorToken::Text1
        } else {
            ColorToken::Text3
        };
        paint_text_centered(
            text_system,
            scene,
            &cell.day.to_string(),
            r,
            TypeToken::Sm.px(),
            resolve(ink, theme),
        );
    }
}
