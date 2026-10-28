//! As datas do verso: o instante neutro do documento (ms UTC) visto no fuso da máquina
//! (`ph2d-kanban-dates`) e os textos (pela i18n). O seletor e a leitura das caixas são a lei PÚBLICA
//! de `ph2d_kanban_views::datepick` (a mesma da Tabela) — aqui só o fuso e o «agora» da máquina.

use ph2d_editor_core::widget::{CalendarEvent, DateField};
use ph2d_i18n::{tr, tr_with};
use ph2d_kanban_dates::{DueLabel, Weekday, Zone, civil};
use ph2d_kanban_model::{CardId, Moment};
use ph2d_kanban_views::datepick::{self, Picker};

pub(crate) use datepick::{day_text, parse_date, time_text};

pub(crate) fn zone() -> Zone {
    Zone::system()
}

fn month_short(m: u8) -> &'static str {
    datepick::month_name(m, false)
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

/// O seletor montado com as datas `start` e `due` (`card` dá os ids a11y).
pub(crate) fn picker(
    card: CardId,
    start: Option<Moment>,
    due: Option<(Moment, Option<u32>)>,
    now: Moment,
) -> Picker {
    let salt = crate::A11Y_SALT ^ 0x0000_BAC0_0000_0000 ^ (card.0 << 8);
    Picker::new(salt, start, due, now, &zone(), Weekday::Monday)
}

/// Um evento de mês: o seletor mostra o mês pedido.
pub(crate) fn month_event(p: &mut Picker, ev: CalendarEvent) {
    p.month_event(ev, crate::actions::now(), &zone());
}

/// O artista escreveu numa caixa: lê-a (o que não se lê volta ao que estava).
pub(crate) fn typed(p: &mut Picker, field: DateField, text: &str) {
    let today = civil(crate::actions::now().0, &zone()).date;
    p.typed(field, text, today);
}

/// O que o seletor diz ao «Save»: o início e a entrega (com o lembrete).
pub(crate) fn read(p: &Picker) -> (Option<Moment>, Option<(Moment, Option<u32>)>) {
    p.read(&zone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_kanban_dates::CivilDate;

    #[test]
    fn dates_read_us_iso_and_short_forms() {
        let today = CivilDate::new(2026, 10, 28).expect("data");
        assert_eq!(
            parse_date("10/25/2026", today),
            CivilDate::new(2026, 10, 25)
        );
        assert_eq!(parse_date("13/40/2026", today), None);
        assert_eq!(datepick::parse_time("17:05"), Some((17, 5)));
        assert_eq!(datepick::parse_time("25:00"), None);
    }
}
