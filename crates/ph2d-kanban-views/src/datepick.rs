//! O `DatePicker` do editor-core montado e lido para UM cartão, e os textos de datas — com o fuso e
//! o «agora» DADOS (a tabela edita a entrega pelo mesmo painel que o verso). As chaves são as do
//! verso (`kanban.back.*`): a mesma palavra no mesmo sítio. ⚠️ É a MESMA lei que
//! `ph2d-app-kanban/src/back/dates.rs` (que não é pública); o app pode passar a chamar esta.

use ph2d_a11y::NodeId;
use ph2d_editor_core::widget::{
    CALENDAR_CELLS, Calendar, CalendarCell, CalendarDate, CalendarEvent, DateField, DatePicker,
    DatePickerLabels, DateSlot, Reminder,
};
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_dates::{CalendarMonth, CivilDate, Weekday, Zone, civil, instant, start_of_day};
use ph2d_kanban_model::Moment;

/// O sal das ids a11y desta crate.
const A11Y_SALT: u64 = 0x7A1E_0000_0000_0000;

/// Uma id a11y da vista (`salt` distingue as peças).
#[must_use]
pub fn node(salt: u64) -> NodeId {
    NodeId(A11Y_SALT ^ salt)
}

/// «Oct» (curto) ou «October» (`long`).
#[must_use]
pub fn month_name(m: u8, long: bool) -> &'static str {
    let k = if long { "month" } else { "mon" };
    tr(&format!("kanban.back.{k}.{m}"))
}

/// «Mo».. pela ordem de segunda (`monday_index`).
#[must_use]
pub fn weekday_name(w: Weekday) -> &'static str {
    tr(&format!("kanban.back.wd.{}", w.monday_index()))
}

/// «5:00 PM».
#[must_use]
pub fn time_text(hour: u8, minute: u8) -> String {
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
#[must_use]
pub fn day_text(d: CivilDate, today: CivilDate) -> String {
    if d.year == today.year {
        tr_with(
            "kanban.back.day",
            &[("mon", &month_name(d.month, false)), ("d", &d.day)],
        )
    } else {
        tr_with(
            "kanban.back.day_year",
            &[
                ("mon", &month_name(d.month, false)),
                ("d", &d.day),
                ("y", &d.year),
            ],
        )
    }
}

/// «10/25/2026» (o que a caixa do seletor mostra e lê).
fn date_field(d: CivilDate) -> String {
    format!("{}/{}/{}", d.month, d.day, d.year)
}

/// Lê «10/25/2026», «10/25» (este ano) ou «2026-10-25».
#[must_use]
pub fn parse_date(s: &str, today: CivilDate) -> Option<CivilDate> {
    let n: Vec<&str> = s.trim().split(['/', '-', '.']).collect();
    let num = |x: &str| x.trim().parse::<i32>().ok();
    let ymd = |y: &str, m: &str, d: &str| {
        CivilDate::new(
            i16::try_from(num(y)?).ok()?,
            u8::try_from(num(m)?).ok()?,
            u8::try_from(num(d)?).ok()?,
        )
    };
    match n.as_slice() {
        [y, m, d] if y.len() == 4 => ymd(y, m, d),
        [m, d, y] => ymd(y, m, d),
        [m, d] => CivilDate::new(
            today.year,
            u8::try_from(num(m)?).ok()?,
            u8::try_from(num(d)?).ok()?,
        ),
        _ => None,
    }
}

/// Lê «5:00 PM», «5 pm», «17:00».
#[must_use]
pub fn parse_time(s: &str) -> Option<(u8, u8)> {
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

/// O seletor aberto e o mês que ele mostra.
#[derive(Clone, Debug)]
pub struct Picker {
    pub widget: DatePicker,
    pub month: CalendarMonth,
}

/// Mostra o mês `m` no calendário do seletor.
fn show_month(p: &mut DatePicker, m: &CalendarMonth) {
    let src = m.cells();
    let cells: [CalendarCell; CALENDAR_CELLS] = std::array::from_fn(|i| CalendarCell {
        year: i32::from(src[i].year),
        month: src[i].month,
        day: src[i].day,
        in_month: src[i].in_month,
        is_today: src[i].is_today,
    });
    let title = tr_with(
        "kanban.back.month_title",
        &[("month", &month_name(m.month(), true)), ("y", &m.year())],
    );
    p.calendar.set_month(title, cells);
}

impl Picker {
    /// O seletor com as datas `start` e `due` (e o lembrete), no fuso `zone`.
    #[must_use]
    pub fn new(
        salt: u64,
        start: Option<Moment>,
        due: Option<(Moment, Option<u32>)>,
        now: Moment,
        zone: &Zone,
        week: Weekday,
    ) -> Self {
        let shown = due.map(|d| d.0).or(start).unwrap_or(now);
        let month = CalendarMonth::containing(shown.0, zone, week);
        let days: [String; 7] = month.weekdays().map(|w| weekday_name(w).to_owned());
        let cal = Calendar::new(node(salt ^ 0x10)).labels(
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
        let mut p = DatePicker::new(node(salt ^ 0x11), cal, labels);
        show_month(&mut p, &month);
        p.start_on = start.is_some();
        p.due_on = due.is_some() || start.is_none();
        if let Some(s) = start {
            let d = civil(s.0, zone).date;
            p.set_date(DateSlot::Start, Some(cal_date(d)), date_field(d));
        }
        let (d, hour, minute) = match due {
            Some((at, _)) => {
                let c = civil(at.0, zone);
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
        Self { widget: p, month }
    }

    /// Um evento de mês: o seletor mostra o mês pedido.
    pub fn month_event(&mut self, ev: CalendarEvent, now: Moment, zone: &Zone) {
        self.month = match ev {
            CalendarEvent::PrevMonth => self.month.prev(),
            CalendarEvent::NextMonth => self.month.next(),
            CalendarEvent::Today => CalendarMonth::containing(now.0, zone, self.month.week_start()),
            CalendarEvent::Pick(_) => return,
        };
        show_month(&mut self.widget, &self.month);
    }

    /// Um dia escolhido: a caixa da data mostra-o.
    pub fn picked(&mut self, slot: DateSlot, date: CalendarDate) {
        if let Some(d) = civil_date(date) {
            self.widget.set_date(slot, Some(date), date_field(d));
        }
    }

    /// O artista escreveu numa caixa: lê-a (o que não se lê volta ao que estava).
    pub fn typed(&mut self, field: DateField, text: &str, today: CivilDate) {
        let p = &mut self.widget;
        match field {
            DateField::Time => {
                let (h, m) = parse_time(text).unwrap_or((12, 0));
                p.set_time(time_text(h, m));
            }
            DateField::Start | DateField::Due => {
                let slot = if field == DateField::Start {
                    DateSlot::Start
                } else {
                    DateSlot::Due
                };
                let keep = if slot == DateSlot::Start {
                    p.start
                } else {
                    p.due
                };
                let d = parse_date(text, today).map(cal_date).or(keep);
                let txt = d.and_then(civil_date).map(date_field).unwrap_or_default();
                p.set_date(slot, d, txt);
            }
        }
    }

    /// O que o seletor diz ao «Save»: o início e a entrega (com o lembrete).
    #[must_use]
    pub fn read(&self, zone: &Zone) -> (Option<Moment>, Option<(Moment, Option<u32>)>) {
        let p = &self.widget;
        let start = p
            .start
            .filter(|_| p.start_on)
            .and_then(civil_date)
            .and_then(|d| start_of_day(d, zone))
            .map(Moment);
        let (h, m) = parse_time(&p.time_text.text).unwrap_or((12, 0));
        let due = p
            .due
            .filter(|_| p.due_on)
            .and_then(civil_date)
            .and_then(|d| instant(d, h, m, zone))
            .map(|at| (Moment(at), p.reminder.minutes_before()));
        (start, due)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_reads_back_what_it_was_given() {
        let z = Zone::utc();
        let d = CivilDate::new(2026, 10, 25).expect("data");
        let due = Moment(instant(d, 17, 30, &z).expect("instante"));
        let start = Moment(start_of_day(d.add_days(-3), &z).expect("início"));
        let p = Picker::new(
            1,
            Some(start),
            Some((due, Some(60))),
            due,
            &z,
            Weekday::Monday,
        );
        assert_eq!(p.read(&z), (Some(start), Some((due, Some(60)))));
    }

    #[test]
    fn times_and_dates_parse_the_us_and_iso_forms() {
        assert_eq!(parse_time("5:00 PM"), Some((17, 0)));
        assert_eq!(parse_time("12:30 am"), Some((0, 30)));
        let today = CivilDate::new(2026, 10, 28).expect("data");
        assert_eq!(parse_date("2026-11-02", today), CivilDate::new(2026, 11, 2));
        assert_eq!(parse_date("1/3", today), CivilDate::new(2026, 1, 3));
    }
}
