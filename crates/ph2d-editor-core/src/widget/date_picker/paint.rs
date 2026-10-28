//! A disposição e a tinta do [`super::DatePicker`] — uma régua para o pintor e para o clique.

use super::{DateField, DatePicker, DateSlot, Reminder};
use crate::icons::IconId;
use crate::paint::{fill_rounded_rect, paint_icon, paint_text_centered, paint_text_title, resolve};
use crate::widget::calendar::{CalendarLayout, paint_calendar};
use crate::widget::checkbox::{BooleanMark, paint_boolean_mark};
use crate::widget::{
    Button, ButtonKind, CheckboxState, CheckboxValue, Dropdown, DropdownOption, Popover, TextInput,
    TextInputState, paint_button, paint_dropdown_chip, paint_popover, paint_text_input_with_buffer,
};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Density, ROW_H_PX, Radius, Spacing, StrokeToken, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DatePickerHit {
    Close,
    Calendar,
    Check(DateSlot),
    Field(DateField),
    Reminder,
    Save,
    Remove,
}

/// As caixas do painel dentro de `rect`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DatePickerLayout {
    pub title: Rect,
    pub close: Rect,
    pub calendar: Rect,
    pub start_label: Rect,
    pub start_check: Rect,
    pub start_field: Rect,
    pub due_label: Rect,
    pub due_check: Rect,
    pub due_field: Rect,
    pub time_field: Rect,
    pub reminder_label: Rect,
    pub reminder: Rect,
    pub save: Rect,
    pub remove: Rect,
}

fn pad() -> f32 {
    Spacing::Md.px()
}
fn gap() -> f32 {
    Spacing::Sm.px()
}
fn label_h() -> f32 {
    Density::Compact.row_h_px()
}

/// O menu do lembrete — a mesma lista para pintar e para escolher.
fn reminder_menu(p: Option<&DatePicker>) -> Dropdown<Reminder> {
    let options = Reminder::ALL
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let label = p.map_or(String::new(), |p| p.labels.reminder_options[i].clone());
            let id = p.map_or(NodeId(0), |p| NodeId(p.id.0.wrapping_add(16 + i as u64)));
            DropdownOption::new(id, *r, label)
        })
        .collect();
    let id = p.map_or(NodeId(0), |p| NodeId(p.id.0.wrapping_add(15)));
    let label = p.map_or(String::new(), |p| p.labels.reminder.clone());
    let mut d = Dropdown::new(id, label, options);
    if let Some(p) = p {
        d.select(p.reminder);
        d.open = p.reminder_open;
    }
    d
}

impl DatePickerLayout {
    /// A altura que o painel pede.
    #[must_use]
    pub fn preferred_height() -> f32 {
        let r = Self::of(Rect::new(0.0, 0.0, CalendarLayout::preferred_height(), 0.0));
        r.remove.y + r.remove.h + pad()
    }

    #[must_use]
    pub fn of(rect: Rect) -> Self {
        let (p, g, lh, row) = (pad(), gap(), label_h(), ROW_H_PX);
        let x = rect.x + p;
        let w = (rect.w - p * 2.0).max(0.0);
        let mut y = rect.y + p;
        let close = Rect::new(x + w - row, y, row, row);
        let title = Rect::new(x + row, y, (w - row * 2.0).max(0.0), row);
        y += row + g;
        let calendar = Rect::new(x, y, w, CalendarLayout::preferred_height());
        y += calendar.h + g;
        let field_x = x + row + g;
        let rest = (w - row - g).max(0.0);
        let half = ((rest - g) * 0.5).max(0.0);
        let start_label = Rect::new(x, y, w, lh);
        y += lh;
        let start_check = Rect::new(x, y, row, row);
        let start_field = Rect::new(field_x, y, half, row);
        y += row + g;
        let due_label = Rect::new(x, y, w, lh);
        y += lh;
        let due_check = Rect::new(x, y, row, row);
        let due_field = Rect::new(field_x, y, half, row);
        let time_field = Rect::new(field_x + half + g, y, half, row);
        y += row + g;
        let reminder_label = Rect::new(x, y, w, lh);
        y += lh;
        let reminder = Rect::new(x, y, w, row);
        y += row + p;
        let save = Rect::new(x, y, w, row);
        y += row + Spacing::Xs.px();
        let remove = Rect::new(x, y, w, row);
        Self {
            title,
            close,
            calendar,
            start_label,
            start_check,
            start_field,
            due_label,
            due_check,
            due_field,
            time_field,
            reminder_label,
            reminder,
            save,
            remove,
        }
    }

    #[must_use]
    pub fn hit(&self, x: f32, y: f32) -> Option<DatePickerHit> {
        let inside = |r: Rect| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h;
        [
            (self.close, DatePickerHit::Close),
            (self.calendar, DatePickerHit::Calendar),
            (self.start_check, DatePickerHit::Check(DateSlot::Start)),
            (self.start_field, DatePickerHit::Field(DateField::Start)),
            (self.due_check, DatePickerHit::Check(DateSlot::Due)),
            (self.due_field, DatePickerHit::Field(DateField::Due)),
            (self.time_field, DatePickerHit::Field(DateField::Time)),
            (self.reminder, DatePickerHit::Reminder),
            (self.save, DatePickerHit::Save),
            (self.remove, DatePickerHit::Remove),
        ]
        .into_iter()
        .find_map(|(r, h)| inside(r).then_some(h))
    }

    /// A caixa da opção `i` do menu do lembrete aberto.
    #[must_use]
    pub fn reminder_option(&self, i: usize) -> Rect {
        reminder_menu(None).option_rect(self.reminder, i)
    }

    #[must_use]
    pub fn reminder_option_at(&self, x: f32, y: f32) -> Option<usize> {
        (0..Reminder::ALL.len()).find(|i| {
            let r = self.reminder_option(*i);
            x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_label(ts: &mut TextSystem, scene: &mut VectorScene, text: &str, r: Rect, theme: Theme) {
    let f = TypeToken::Xs.px();
    paint_text_title(
        ts,
        scene,
        text,
        r.x,
        r.y + (r.h - f) * 0.5,
        f,
        r.w,
        resolve(ColorToken::Text2, theme),
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_field(
    p: &DatePicker,
    f: DateField,
    enabled: bool,
    r: Rect,
    k: u64,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let editing = p.editing == Some(f);
    let state = if editing {
        TextInputState::Focused
    } else if !enabled {
        TextInputState::Disabled
    } else if p.hover == Some(super::DatePickerHit::Field(f)) {
        TextInputState::Hovered
    } else {
        TextInputState::Normal
    };
    let field = p.field(f);
    let input = TextInput::new(NodeId(p.id.0.wrapping_add(k)), String::new())
        .value(field.text.clone())
        .state(state);
    let caret = editing.then_some(field.caret);
    paint_text_input_with_buffer(&input, None, caret, None, r, scene, ts, theme);
}

fn paint_check(on: bool, hovered: bool, r: Rect, scene: &mut VectorScene, theme: Theme) {
    paint_boolean_mark(
        r,
        BooleanMark {
            value: if on {
                CheckboxValue::Checked
            } else {
                CheckboxValue::Unchecked
            },
            state: if hovered {
                CheckboxState::Hovered
            } else {
                CheckboxState::Normal
            },
            hover_t: crate::motion::SETTLED,
            box_px: None,
            decorator: false,
            pinta_coluna: false,
            linha: None,
        },
        scene,
        theme,
    );
}

pub fn paint_date_picker(
    p: &DatePicker,
    rect: Rect,
    scene: &mut VectorScene,
    ts: &mut TextSystem,
    theme: Theme,
) {
    let l = DatePickerLayout::of(rect);
    paint_popover(&Popover::new(p.id), rect, scene, theme);
    let lb = &p.labels;

    // Título e fechar.
    paint_text_centered(
        ts,
        scene,
        &lb.title,
        l.title,
        TypeToken::Sm.px(),
        resolve(ColorToken::Text2, theme),
    );
    if p.hover == Some(DatePickerHit::Close) {
        let radius = crate::paint::frame_radius(theme, Radius::Sm.px());
        fill_rounded_rect(scene, l.close, radius, resolve(ColorToken::Bg2, theme));
    }
    let ip = Spacing::Xs.px();
    let g = Rect::new(
        l.close.x + ip,
        l.close.y + ip,
        l.close.w - ip * 2.0,
        l.close.h - ip * 2.0,
    );
    paint_icon(
        scene,
        IconId::Close,
        g,
        resolve(ColorToken::Text2, theme),
        StrokeToken::Default.px(),
    );

    paint_calendar(&p.calendar, l.calendar, scene, ts, theme);

    let hov = |h| p.hover == Some(h);
    paint_label(ts, scene, &lb.start, l.start_label, theme);
    paint_check(
        p.start_on,
        hov(DatePickerHit::Check(DateSlot::Start)),
        l.start_check,
        scene,
        theme,
    );
    paint_field(
        p,
        DateField::Start,
        p.start_on,
        l.start_field,
        1,
        scene,
        ts,
        theme,
    );

    paint_label(ts, scene, &lb.due, l.due_label, theme);
    paint_check(
        p.due_on,
        hov(DatePickerHit::Check(DateSlot::Due)),
        l.due_check,
        scene,
        theme,
    );
    paint_field(
        p,
        DateField::Due,
        p.due_on,
        l.due_field,
        2,
        scene,
        ts,
        theme,
    );
    paint_field(
        p,
        DateField::Time,
        p.due_on,
        l.time_field,
        3,
        scene,
        ts,
        theme,
    );

    paint_label(ts, scene, &lb.reminder, l.reminder_label, theme);
    let menu = reminder_menu(Some(p));
    paint_dropdown_chip(&menu, l.reminder, scene, ts, theme);

    let save =
        Button::new(NodeId(p.id.0.wrapping_add(4)), lb.save.clone()).kind(ButtonKind::Accent);
    paint_button(&save, l.save, scene, ts, theme);
    let remove = Button::new(NodeId(p.id.0.wrapping_add(5)), lb.remove.clone());
    paint_button(&remove, l.remove, scene, ts, theme);

    // O menu aberto vai por cima de tudo o que está por baixo dele.
    if menu.open {
        crate::widget::paint_dropdown_popover(&menu, l.reminder, scene, ts, theme);
    }
}
