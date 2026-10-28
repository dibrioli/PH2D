//! As datas do verso: o instante neutro do documento (ms UTC) visto no fuso da máquina
//! (`ph2d-kanban-dates`), os textos (pela i18n), e o `DatePicker` montado e lido.

use ph2d_editor_core::widget::{
    CALENDAR_CELLS, Calendar, CalendarCell, CalendarDate, CalendarEvent, DatePicker,
    DatePickerLabels, DateSlot, Reminder,
};
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_dates::{CalendarMonth, CivilDate, DueLabel, Weekday, Zone, civil, instant};
use ph2d_kanban_model::{CardId, Moment};

pub(crate) fn zone() -> Zone {
    Zone::system()
}

fn month_short(m: u8) -> String {
    tr(&format!("kanban.back.mon.{m}")).to_owned()
}

/// «5:00 PM».
pub(crate) fn time_text(hour: u8, minute: u8) -> String {
    let h12 = match hour % 12 {
        0 => 12,
        h => h,
    };
    let key = if hour < 12 {
        "kanban.back.time_am"
    } else {
        "kanban.back.time_pm"
    };
    tr_with(key, &[("h", &h12), ("m", &format!("{minute:02}"))])
}

/// «Oct 25» (com o ano se não for o de hoje).
pub(crate) fn day_text(d: CivilDate, today: CivilDate) -> String {
    if d.year == today.year {
        tr_with(
            "kanban.back.day",
            &[("mon", &month_short(d.month)), ("d", &d.day)],
        )
    } else {
        tr_with(
            "kanban.back.day_year",
            &[
                ("mon", &month_short(d.month)),
                ("d", &d.day),
                ("y", &d.year),
            ],
        )
    }
}

/// O texto do prazo no verso: «Oct 22 – Oct 25, 5:00 PM» (o início, se há, e a entrega com hora).
pub(crate) fn range_text(start: Option<Moment>, due: Option<Moment>, now: Moment) -> String {
    let z = zone();
    let today = civil(now.0, &z).date;
    let s = start.map(|s| day_text(civil(s.0, &z).date, today));
    let d = due.map(|d| {
        let c = civil(d.0, &z);
        tr_with(
            "kanban.back.day_time",
            &[
                ("day", &day_text(c.date, today)),
                ("time", &time_text(c.hour, c.minute)),
            ],
        )
    });
    match (s, d) {
        (Some(s), Some(d)) => tr_with("kanban.back.range", &[("a", &s), ("b", &d)]),
        (Some(s), None) => s,
        (None, Some(d)) => d,
        (None, None) => String::new(),
    }
}

/// O texto curto de uma data de item («Oct 29», «Today 14:00»).
pub(crate) fn short(at: Moment, now: Moment) -> String {
    let z = zone();
    let time = |hour, minute| time_text(hour, minute);
    match ph2d_kanban_dates::due_label(at.0, now.0, &z) {
        DueLabel::Today { hour, minute } => {
            tr_with("kanban.back.today_at", &[("time", &time(hour, minute))])
        }
        DueLabel::Tomorrow { hour, minute } => {
            tr_with("kanban.back.tomorrow_at", &[("time", &time(hour, minute))])
        }
        DueLabel::Yesterday { hour, minute } => {
            tr_with("kanban.back.yesterday_at", &[("time", &time(hour, minute))])
        }
        DueLabel::Date { month, day } => tr_with(
            "kanban.back.day",
            &[("mon", &month_short(month)), ("d", &day)],
        ),
        DueLabel::DateWithYear { year, month, day } => tr_with(
            "kanban.back.day_year",
            &[("mon", &month_short(month)), ("d", &day), ("y", &year)],
        ),
    }
}

/// «2 hours ago», «Yesterday at 4:12 PM», «Oct 22 at 9:00 AM».
pub(crate) fn ago(at: Moment, now: Moment) -> String {
    let s = (now.0 - at.0).max(0) / 1000;
    match s {
        0..60 => tr("kanban.back.ago.now").to_owned(),
        60..3600 => tr_with("kanban.back.ago.min", &[("n", &(s / 60))]),
        3600..86_400 => tr_with("kanban.back.ago.hour", &[("n", &(s / 3600))]),
        _ => {
            let z = zone();
            let c = civil(at.0, &z);
            tr_with(
                "kanban.back.at",
                &[
                    ("day", &day_text(c.date, civil(now.0, &z).date)),
                    ("time", &time_text(c.hour, c.minute)),
                ],
            )
        }
    }
}

/// «10/25/2026» (o que a caixa do seletor mostra e lê).
fn date_field(d: CivilDate) -> String {
    format!("{}/{}/{}", d.month, d.day, d.year)
}

/// Lê «10/25/2026», «10/25» (este ano) ou «2026-10-25».
pub(crate) fn parse_date(s: &str, today: CivilDate) -> Option<CivilDate> {
    let s = s.trim();
    let n: Vec<&str> = s.split(['/', '-', '.']).collect();
    let num = |x: &str| x.trim().parse::<i32>().ok();
    match n.as_slice() {
        [y, m, d] if y.len() == 4 => CivilDate::new(
            i16::try_from(num(y)?).ok()?,
            u8::try_from(num(m)?).ok()?,
            u8::try_from(num(d)?).ok()?,
        ),
        [m, d, y] => CivilDate::new(
            i16::try_from(num(y)?).ok()?,
            u8::try_from(num(m)?).ok()?,
            u8::try_from(num(d)?).ok()?,
        ),
        [m, d] => CivilDate::new(
            today.year,
            u8::try_from(num(m)?).ok()?,
            u8::try_from(num(d)?).ok()?,
        ),
        _ => None,
    }
}

/// Lê «5:00 PM», «5 pm», «17:00».
pub(crate) fn parse_time(s: &str) -> Option<(u8, u8)> {
    let t = s.trim().to_ascii_lowercase();
    let pm = t.ends_with("pm") || t.ends_with('p');
    let am = t.ends_with("am") || t.ends_with('a');
    let core = t.trim_end_matches(['a', 'p', 'm', ' ']);
    let mut it = core.split(':');
    let h: u8 = it.next()?.trim().parse().ok()?;
    let m: u8 = it.next().map_or(Some(0), |m| m.trim().parse().ok())?;
    let h = match (am, pm, h) {
        (_, true, 1..=11) => h + 12,
        (true, _, 12) => 0,
        _ => h,
    };
    (h < 24 && m < 60).then_some((h, m))
}

fn cal_date(d: CivilDate) -> CalendarDate {
    CalendarDate::new(i32::from(d.year), d.month, d.day)
}

fn civil_date(d: CalendarDate) -> Option<CivilDate> {
    CivilDate::new(i16::try_from(d.year).ok()?, d.month, d.day)
}

/// Mostra o mês `m` no calendário do seletor.
pub(crate) fn show_month(p: &mut DatePicker, m: &CalendarMonth) {
    let cells: [CalendarCell; CALENDAR_CELLS] = {
        let src = m.cells();
        std::array::from_fn(|i| CalendarCell {
            year: i32::from(src[i].year),
            month: src[i].month,
            day: src[i].day,
            in_month: src[i].in_month,
            is_today: src[i].is_today,
        })
    };
    let title = tr_with(
        "kanban.back.month_title",
        &[
            ("month", &tr(&format!("kanban.back.month.{}", m.month()))),
            ("y", &m.year()),
        ],
    );
    p.calendar.set_month(title, cells);
}

fn weekday_key(w: Weekday) -> String {
    format!("kanban.back.wd.{}", w.monday_index())
}

/// O seletor montado com as datas `start` e `due` (`card` dá os ids a11y).
pub(crate) fn picker(
    card: CardId,
    start: Option<Moment>,
    due: Option<(Moment, Option<u32>)>,
    now: Moment,
) -> (DatePicker, CalendarMonth) {
    let z = zone();
    let week = Weekday::Monday;
    let shown = due.map(|d| d.0).or(start).unwrap_or(now);
    let month = CalendarMonth::containing(shown.0, &z, week);
    let days: [String; 7] = month.weekdays().map(|w| tr(&weekday_key(w)).to_owned());
    let cal = Calendar::new(super::node(card, 10)).labels(
        days,
        tr("kanban.back.prev_month"),
        tr("kanban.back.next_month"),
        tr("kanban.back.today"),
    );
    let labels = DatePickerLabels {
        title: tr("kanban.back.pop.dates").to_owned(),
        close: tr("kanban.back.close").to_owned(),
        start: tr("kanban.back.start_date").to_owned(),
        due: tr("kanban.back.due_date").to_owned(),
        time: tr("kanban.back.time").to_owned(),
        reminder: tr("kanban.back.reminder").to_owned(),
        reminder_options: std::array::from_fn(|i| {
            tr(&format!("kanban.back.remind.{i}")).to_owned()
        }),
        save: tr("kanban.back.save").to_owned(),
        remove: tr("kanban.back.remove").to_owned(),
    };
    let mut p = DatePicker::new(super::node(card, 11), cal, labels);
    show_month(&mut p, &month);
    p.start_on = start.is_some();
    p.due_on = due.is_some() || start.is_none();
    if let Some(s) = start {
        let d = civil(s.0, &z).date;
        p.set_date(DateSlot::Start, Some(cal_date(d)), date_field(d));
    }
    let (d, hour, minute) = match due {
        Some((at, _)) => {
            let c = civil(at.0, &z);
            (Some(c.date), c.hour, c.minute)
        }
        None => (None, 12, 0),
    };
    if let Some(d) = d {
        p.set_date(DateSlot::Due, Some(cal_date(d)), date_field(d));
    }
    p.set_time(time_text(hour, minute));
    p.reminder = due
        .and_then(|d| d.1)
        .and_then(|m| {
            Reminder::ALL
                .into_iter()
                .find(|r| r.minutes_before() == Some(m))
        })
        .unwrap_or(Reminder::None);
    (p, month)
}

/// Um evento de mês: o seletor mostra o mês pedido.
pub(crate) fn month_event(p: &mut DatePicker, month: &mut CalendarMonth, ev: CalendarEvent) {
    let z = zone();
    *month = match ev {
        CalendarEvent::PrevMonth => month.prev(),
        CalendarEvent::NextMonth => month.next(),
        CalendarEvent::Today => {
            CalendarMonth::containing(super::super::actions::now().0, &z, month.week_start())
        }
        CalendarEvent::Pick(_) => return,
    };
    show_month(p, month);
}

/// Um dia escolhido: a caixa da data mostra-o.
pub(crate) fn picked(p: &mut DatePicker, slot: DateSlot, date: CalendarDate) {
    if let Some(d) = civil_date(date) {
        p.set_date(slot, Some(date), date_field(d));
    }
}

/// O artista escreveu numa caixa: lê-a (o que não se lê volta ao que estava).
pub(crate) fn typed(p: &mut DatePicker, field: ph2d_editor_core::widget::DateField, text: &str) {
    use ph2d_editor_core::widget::DateField as F;
    let z = zone();
    let today = civil(super::super::actions::now().0, &z).date;
    match field {
        F::Time => {
            let t =
                parse_time(text).map_or_else(|| "12:00 PM".to_owned(), |(h, m)| time_text(h, m));
            p.set_time(t);
        }
        F::Start | F::Due => {
            let slot = if field == F::Start {
                DateSlot::Start
            } else {
                DateSlot::Due
            };
            if let Some(d) = parse_date(text, today) {
                p.set_date(slot, Some(cal_date(d)), date_field(d));
            } else {
                let keep = if slot == DateSlot::Start {
                    p.start
                } else {
                    p.due
                };
                let txt = keep
                    .and_then(civil_date)
                    .map(date_field)
                    .unwrap_or_default();
                p.set_date(slot, keep, txt);
            }
        }
    }
}

/// O que o seletor diz ao «Save»: o início e a entrega (com o lembrete).
pub(crate) fn read(p: &DatePicker) -> (Option<Moment>, Option<(Moment, Option<u32>)>) {
    let z = zone();
    let start = p
        .start
        .filter(|_| p.start_on)
        .and_then(civil_date)
        .and_then(|d| ph2d_kanban_dates::start_of_day(d, &z))
        .map(Moment);
    let (h, m) = parse_time(&p.time_text.text).unwrap_or((12, 0));
    let due = p
        .due
        .filter(|_| p.due_on)
        .and_then(civil_date)
        .and_then(|d| instant(d, h, m, &z))
        .map(|at| (Moment(at), p.reminder.minutes_before()));
    (start, due)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_read_twelve_and_twenty_four_hour_forms() {
        assert_eq!(parse_time("5:00 PM"), Some((17, 0)));
        assert_eq!(parse_time("12:30 am"), Some((0, 30)));
        assert_eq!(parse_time("17:05"), Some((17, 5)));
        assert_eq!(parse_time("25:00"), None);
    }

    #[test]
    fn dates_read_us_iso_and_short_forms() {
        let today = CivilDate::new(2026, 10, 28).expect("data");
        assert_eq!(
            parse_date("10/25/2026", today),
            CivilDate::new(2026, 10, 25)
        );
        assert_eq!(parse_date("2026-11-02", today), CivilDate::new(2026, 11, 2));
        assert_eq!(parse_date("1/3", today), CivilDate::new(2026, 1, 3));
        assert_eq!(parse_date("13/40/2026", today), None);
    }
}
