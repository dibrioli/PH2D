//! O calendário e o seletor de datas do verso do cartão (Kanban W3), conduzidos como o chamador os
//! conduz: a grelha real de cada mês (calculada AQUI, como a crate de datas a calcula), cliques
//! nas caixas que a MESMA disposição pintou, teclas, e a resposta do chamador a cada evento.

use ph2d_editor_core::doc_surface::{Key, KeyInput};
use ph2d_editor_core::widget::{
    CALENDAR_CELLS, Calendar, CalendarCell, CalendarDate, CalendarEvent, CalendarHit,
    CalendarLayout, DateField, DatePicker, DatePickerEvent, DatePickerLabels, DatePickerLayout,
    DateSlot, Reminder, paint_calendar, paint_date_picker,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// Dias desde 1970-01-01 (Howard Hinnant, `days_from_civil`).
fn days(y: i32, m: u8, d: u8) -> i64 {
    let y = i64::from(if m <= 2 { y - 1 } else { y });
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = i64::from(m);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil(z: i64) -> (i32, u8, u8) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
    let y = (yoe + era * 400 + i64::from(m <= 2)) as i32;
    (y, m, d)
}

const TODAY: (i32, u8, u8) = (2026, 10, 28);

/// A grelha de `y/m`, semana a começar à segunda.
fn month(y: i32, m: u8) -> [CalendarCell; CALENDAR_CELLS] {
    let first = days(y, m, 1);
    let weekday_mon0 = (first + 3).rem_euclid(7); // 1970-01-01 foi quinta
    let start = first - weekday_mon0;
    let today = days(TODAY.0, TODAY.1, TODAY.2);
    std::array::from_fn(|i| {
        let z = start + i as i64;
        let (yy, mm, dd) = civil(z);
        CalendarCell {
            year: yy,
            month: mm,
            day: dd,
            in_month: mm == m,
            is_today: z == today,
        }
    })
}

fn calendar() -> Calendar {
    let mut c = Calendar::new(ph2d_a11y::NodeId(7));
    c.set_month("October 2026", month(2026, 10));
    c
}

fn rect() -> Rect {
    Rect::new(40.0, 30.0, 300.0, CalendarLayout::preferred_height())
}

fn centre(r: Rect) -> (f32, f32) {
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn d(y: i32, m: u8, dd: u8) -> CalendarDate {
    CalendarDate::new(y, m, dd)
}

#[test]
fn clicking_a_painted_day_picks_it_and_the_header_asks_for_months() {
    let mut c = calendar();
    let l = CalendarLayout::of(rect());
    let i = c.index_of(d(2026, 10, 14)).unwrap();
    let (x, y) = centre(l.cell(i));
    assert_eq!(l.hit(x, y), Some(CalendarHit::Cell(i)));
    assert_eq!(
        c.pointer_down(rect(), x, y),
        Some(CalendarEvent::Pick(d(2026, 10, 14)))
    );
    assert_eq!(c.selected, Some(d(2026, 10, 14)));
    for (r, ev) in [
        (l.prev, CalendarEvent::PrevMonth),
        (l.next, CalendarEvent::NextMonth),
        (l.today, CalendarEvent::Today),
    ] {
        let (x, y) = centre(r);
        assert_eq!(c.pointer_down(rect(), x, y), Some(ev));
    }
}

#[test]
fn arrows_walk_by_day_and_week_and_cross_into_the_month_the_caller_hands_back() {
    let mut c = calendar();
    c.focus = Some(d(2026, 10, 1));
    let none = Modifiers::default();
    assert_eq!(c.key(Key::Left, none), (true, None));
    assert_eq!(c.focus, Some(d(2026, 9, 30)));
    assert_eq!(c.key(Key::Down, none), (true, None));
    assert_eq!(c.focus, Some(d(2026, 10, 7)));
    // Do 1.º dia da grelha para cima: sai da grelha e pede o mês anterior.
    c.focus = Some(c.cells[0].date());
    assert_eq!(c.key(Key::Up, none), (true, Some(CalendarEvent::PrevMonth)));
    c.set_month("September 2026", month(2026, 9));
    assert_eq!(c.focus, Some(d(2026, 9, 21)), "uma semana antes de 28/9");
    // PageDown leva o mesmo dia ao mês seguinte, preso ao último dia dele.
    c.focus = Some(d(2026, 10, 31));
    c.set_month("October 2026", month(2026, 10));
    assert_eq!(c.key(Key::PageDown, none), (true, Some(CalendarEvent::NextMonth)));
    c.set_month("November 2026", month(2026, 11));
    assert_eq!(c.focus, Some(d(2026, 11, 30)));
    assert_eq!(
        c.key(Key::Enter, none),
        (true, Some(CalendarEvent::Pick(d(2026, 11, 30))))
    );
    assert_eq!(c.key(Key::Char('q'), none), (false, None));
}

#[test]
fn the_range_is_inclusive_whichever_end_comes_first() {
    let mut c = calendar();
    c.range = Some((d(2026, 10, 30), d(2026, 10, 20)));
    assert!(c.in_range(d(2026, 10, 20)));
    assert!(c.in_range(d(2026, 10, 25)));
    assert!(c.in_range(d(2026, 10, 30)));
    assert!(!c.in_range(d(2026, 10, 31)));
}

fn labels() -> DatePickerLabels {
    DatePickerLabels {
        reminder_options: std::array::from_fn(|i| format!("r{i}")),
        ..DatePickerLabels::default()
    }
}

fn picker() -> DatePicker {
    DatePicker::new(ph2d_a11y::NodeId(90), calendar(), labels())
}

fn prect() -> Rect {
    Rect::new(10.0, 10.0, 304.0, DatePickerLayout::preferred_height())
}

fn click(p: &mut DatePicker, r: Rect) -> Vec<DatePickerEvent> {
    let (x, y) = centre(r);
    p.pointer_down(prect(), x, y)
}

fn day_rect(p: &DatePicker, date: CalendarDate) -> Rect {
    let l = DatePickerLayout::of(prect());
    CalendarLayout::of(l.calendar).cell(p.calendar.index_of(date).unwrap())
}

fn key(k: Key, text: Option<&str>) -> KeyInput<'_> {
    KeyInput {
        key: k,
        mods: Modifiers::default(),
        pressed: true,
        text,
        paste: None,
    }
}

#[test]
fn picking_start_then_due_paints_the_range_between_them() {
    let mut p = picker();
    let l = DatePickerLayout::of(prect());
    // Por omissão o clique escolhe a ENTREGA.
    let r = day_rect(&p, d(2026, 10, 30));
    assert_eq!(
        click(&mut p, r),
        vec![DatePickerEvent::Pick {
            slot: DateSlot::Due,
            date: d(2026, 10, 30)
        }]
    );
    p.set_date(DateSlot::Due, Some(d(2026, 10, 30)), "30/10/2026");
    assert_eq!(p.calendar.selected, Some(d(2026, 10, 30)));
    // Activar o início torna-o a data que o clique escolhe.
    assert_eq!(
        click(&mut p, l.start_check),
        vec![DatePickerEvent::Toggled {
            slot: DateSlot::Start,
            on: true
        }]
    );
    let r = day_rect(&p, d(2026, 10, 20));
    assert_eq!(
        click(&mut p, r),
        vec![DatePickerEvent::Pick {
            slot: DateSlot::Start,
            date: d(2026, 10, 20)
        }]
    );
    p.set_date(DateSlot::Start, Some(d(2026, 10, 20)), "20/10/2026");
    assert_eq!(p.calendar.range, Some((d(2026, 10, 20), d(2026, 10, 30))));
    assert!(p.calendar.in_range(d(2026, 10, 25)));
    // Desligar o início apaga o intervalo e devolve o clique à entrega.
    click(&mut p, l.start_check);
    assert_eq!(p.calendar.range, None);
    assert_eq!(p.active, DateSlot::Due);
}

#[test]
fn typing_in_a_box_commits_on_enter_and_on_leaving_it() {
    let mut p = picker();
    let l = DatePickerLayout::of(prect());
    assert!(click(&mut p, l.time_field).is_empty());
    assert_eq!(p.editing, Some(DateField::Time));
    for ch in ["9", ":", "3", "0"] {
        assert_eq!(p.key(&key(Key::Char(ch.chars().next().unwrap()), Some(ch))), (true, None));
    }
    p.key(&key(Key::Backspace, None));
    assert_eq!(
        p.key(&key(Key::Enter, None)),
        (
            true,
            Some(DatePickerEvent::TextCommitted {
                field: DateField::Time,
                text: String::from("9:3")
            })
        )
    );
    assert_eq!(p.editing, None);
    // Sair da caixa por um clique noutro sítio também confirma.
    click(&mut p, l.due_field);
    p.key(&key(Key::Char('x'), Some("x")));
    let out = click(&mut p, l.save);
    assert_eq!(
        out,
        vec![
            DatePickerEvent::TextCommitted {
                field: DateField::Due,
                text: String::from("x")
            },
            DatePickerEvent::Save
        ]
    );
}

#[test]
fn the_reminder_menu_opens_and_its_painted_row_chooses() {
    let mut p = picker();
    let l = DatePickerLayout::of(prect());
    assert!(click(&mut p, l.reminder).is_empty());
    assert!(p.reminder_open);
    let out = click(&mut p, l.reminder_option(5));
    assert_eq!(out, vec![DatePickerEvent::Reminder(Reminder::Hour1)]);
    assert_eq!(p.reminder.minutes_before(), Some(60));
    assert!(!p.reminder_open);
    assert_eq!(click(&mut p, l.remove), vec![DatePickerEvent::Remove]);
    assert_eq!(click(&mut p, l.close), vec![DatePickerEvent::Close]);
    assert_eq!(p.key(&key(Key::Escape, None)), (true, Some(DatePickerEvent::Close)));
}

#[test]
fn the_keyboard_reaches_the_calendar_inside_the_picker() {
    let mut p = picker();
    p.calendar.focus = Some(d(2026, 10, 1));
    p.key(&key(Key::Right, None));
    assert_eq!(
        p.key(&key(Key::Enter, None)),
        (
            true,
            Some(DatePickerEvent::Pick {
                slot: DateSlot::Due,
                date: d(2026, 10, 2)
            })
        )
    );
    assert_eq!(
        p.key(&key(Key::PageUp, None)),
        (true, Some(DatePickerEvent::Month(CalendarEvent::PrevMonth)))
    );
}

#[test]
fn both_widgets_paint_every_theme_and_close_what_they_open() {
    let mut ts = TextSystem::without_system_fonts();
    for theme in [Theme::Forge, Theme::Sunstone, Theme::Blueprint] {
        let mut scene = VectorScene::new();
        let mut c = calendar();
        c.range = Some((d(2026, 10, 20), d(2026, 10, 30)));
        c.focused = true;
        paint_calendar(&c, rect(), &mut scene, &mut ts, theme);
        let mut p = picker();
        p.reminder_open = true;
        p.editing = Some(DateField::Due);
        paint_date_picker(&p, prect(), &mut scene, &mut ts, theme);
        let enc = scene.inner().encoding();
        assert_eq!(enc.n_open_clips, 0, "{theme:?}");
        assert!(enc.n_paths > 0, "{theme:?}: nada pintado");
    }
}
