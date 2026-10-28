//! **As ocorrências** de um evento com recorrência simples (diária, semanal, mensal, anual), na hora
//! LOCAL: «toda segunda às 10:00» atravessa o horário de verão sem andar. O dia inteiro vive em UTC
//! (o `DATE` do RFC 5545). Um mês sem o dia (31 de abril) salta, como no Google Calendar. As
//! excepções e o fim da série (`until`, v7) tiram ocorrências; o evento avulso que substitui uma
//! aparece por si.

use ph2d_kanban_dates::{CivilDate, Zone, add_days, civil, instant};
use ph2d_kanban_model::{Event, Moment, Repeat};

const DAY_MS: i64 = 86_400_000;

/// O fuso em que o evento se repete.
fn zone_of(e: &Event, zone: &Zone) -> Zone {
    if e.all_day { Zone::utc() } else { zone.clone() }
}

/// A `n`-ésima ocorrência (0 = a própria), `None` = esse mês não tem o dia.
fn nth(e: &Event, r: Repeat, n: i64, z: &Zone) -> Option<i64> {
    let s = e.start.0;
    match r {
        Repeat::Daily => Some(add_days(s, n, z)),
        Repeat::Weekly => Some(add_days(s, n * 7, z)),
        Repeat::Monthly | Repeat::Yearly => {
            let c = civil(s, z);
            let months = if r == Repeat::Monthly { n } else { n * 12 };
            let m0 = i64::from(c.date.year) * 12 + i64::from(c.date.month) - 1 + months;
            let year = i16::try_from(m0.div_euclid(12)).ok()?;
            let month = u8::try_from(m0.rem_euclid(12) + 1).ok()?;
            let d = CivilDate::new(year, month, c.date.day)?;
            instant(d, c.hour, c.minute, z)
        }
    }
}

/// O período aproximado, para saltar as ocorrências antigas sem as percorrer uma a uma.
fn period_ms(r: Repeat) -> i64 {
    match r {
        Repeat::Daily => DAY_MS,
        Repeat::Weekly => 7 * DAY_MS,
        Repeat::Monthly => 28 * DAY_MS,
        Repeat::Yearly => 365 * DAY_MS,
    }
}

/// As ocorrências `(início, fim)` que tocam `[from, to)`, pela ordem.
#[must_use]
pub fn occurrences(e: &Event, from: Moment, to: Moment, zone: &Zone) -> Vec<(Moment, Moment)> {
    let to = e.until.map_or(to, |u| to.min(u));
    let mut v = by_rule(e, from, to, zone);
    if e.repeat.is_some() {
        v.retain(|(s, _)| !e.skips(*s));
    }
    v
}

/// O que a regra dá, sem as excepções.
fn by_rule(e: &Event, from: Moment, to: Moment, zone: &Zone) -> Vec<(Moment, Moment)> {
    let len = (e.end.0 - e.start.0).max(0);
    let touches = |s: i64| s < to.0 && s + len.max(1) > from.0;
    let Some(r) = e.repeat else {
        return if touches(e.start.0) {
            vec![(e.start, e.end)]
        } else {
            Vec::new()
        };
    };
    let z = zone_of(e, zone);
    // Começa um pouco antes da primeira que pode tocar (o salto é por defeito: nunca passa dela).
    let mut n = ((from.0 - len - e.start.0) / period_ms(r) - 2).max(0);
    let mut out = Vec::new();
    loop {
        match nth(e, r, n, &z) {
            Some(s) if s >= to.0 => break,
            Some(s) if touches(s) => out.push((Moment(s), Moment(s + len))),
            _ => {}
        }
        n += 1;
    }
    out
}

/// O início da ocorrência mais próxima de `at` (o arrasto de uma ocorrência move a série inteira
/// pela diferença entre ela e onde caiu).
#[must_use]
pub fn nearest(e: &Event, at: Moment, zone: &Zone) -> Moment {
    let Some(r) = e.repeat else {
        return e.start;
    };
    let p = period_ms(r) * 2;
    occurrences(e, Moment(at.0 - p), Moment(at.0 + p), zone)
        .into_iter()
        .map(|(s, _)| s)
        .min_by_key(|s| (s.0 - at.0).abs())
        .unwrap_or(e.start)
}

/// O próximo início em `now` ou depois (o lembrete de um evento que se repete).
#[must_use]
pub fn next_start(e: &Event, now: Moment, zone: &Zone) -> Option<Moment> {
    let horizon = Moment(now.0 + period_ms(e.repeat.unwrap_or(Repeat::Yearly)) * 2 + DAY_MS * 31);
    occurrences(e, now, horizon, zone)
        .into_iter()
        .map(|(s, _)| s)
        .find(|s| s.0 >= now.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_kanban_model::EventId;

    fn ev(start: i64, repeat: Option<Repeat>) -> Event {
        let mut e = Event::new(EventId(1), "e", Moment(start), Moment(start + 3_600_000), false);
        e.repeat = repeat;
        e
    }

    #[test]
    fn weekly_occurrences_land_once_a_week_and_only_in_the_window() {
        let e = ev(0, Some(Repeat::Weekly));
        let z = Zone::utc();
        let got = occurrences(&e, Moment(10 * DAY_MS), Moment(31 * DAY_MS), &z);
        let days: Vec<i64> = got.iter().map(|(s, _)| s.0 / DAY_MS).collect();
        assert_eq!(days, [14, 21, 28]);
        assert!(occurrences(&ev(0, None), Moment(DAY_MS), Moment(2 * DAY_MS), &z).is_empty());
    }

    #[test]
    fn the_31st_skips_the_months_without_it() {
        // 31 de jan de 2026, 10:00 UTC.
        let s = instant(CivilDate::new(2026, 1, 31).unwrap(), 10, 0, &Zone::utc()).unwrap();
        let e = ev(s, Some(Repeat::Monthly));
        let z = Zone::utc();
        let got = occurrences(&e, Moment(s), Moment(s + 121 * DAY_MS), &z);
        let months: Vec<u8> = got.iter().map(|(s, _)| civil(s.0, &z).date.month).collect();
        assert_eq!(months, [1, 3, 5]);
    }

    #[test]
    fn a_weekly_meeting_keeps_its_local_hour_across_daylight_saving() {
        let z = Zone::posix("EST5EDT,M3.2.0,M11.1.0").unwrap();
        let s = instant(CivilDate::new(2026, 3, 2).unwrap(), 10, 0, &z).unwrap();
        let e = ev(s, Some(Repeat::Weekly));
        let got = occurrences(&e, Moment(s), Moment(s + 15 * DAY_MS), &z);
        assert!(got.iter().all(|(s, _)| civil(s.0, &z).hour == 10));
        assert_eq!(nearest(&e, Moment(got[1].0.0 + 3_600_000), &z), got[1].0);
    }
}
