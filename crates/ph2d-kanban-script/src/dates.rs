//! **As datas** — ISO 8601 em texto ou `{ year, month, day, hour, min }`; sem `Z` nem desvio, a
//! hora é a local do projecto. A leitura devolve o texto local `AAAA-MM-DDTHH:MM`.

use mlua::Value;
use ph2d_kanban_dates::{CivilDate, Zone, civil, instant};
use ph2d_kanban_model::Moment;

fn num(s: &str) -> Option<i64> {
    (!s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .then(|| s.parse().ok())
        .flatten()
}

fn at(y: i64, mo: i64, d: i64, h: i64, mi: i64, zone: &Zone) -> Option<i64> {
    let date = CivilDate::new(
        i16::try_from(y).ok()?,
        u8::try_from(mo).ok()?,
        u8::try_from(d).ok()?,
    )?;
    if !(0..24).contains(&h) || !(0..60).contains(&mi) {
        return None;
    }
    instant(date, u8::try_from(h).ok()?, u8::try_from(mi).ok()?, zone)
}

/// `"2026-11-03"`, `"2026-11-03T14:30"`, `"…:05"`, com `Z` ou `±HH:MM` no fim.
#[must_use]
pub fn parse_iso(s: &str, zone: &Zone) -> Option<i64> {
    let s = s.trim();
    let (date, time) = s.split_once(['T', ' ']).unwrap_or((s, ""));
    let mut d = date.split('-');
    let (y, mo, dd) = (num(d.next()?)?, num(d.next()?)?, num(d.next()?)?);
    if d.next().is_some() {
        return None;
    }
    let (clock, offset_min) = if let Some(c) = time.strip_suffix(['Z', 'z']) {
        (c, Some(0))
    } else if let Some(i) = time.rfind(['+', '-']) {
        let (c, o) = time.split_at(i);
        let sign = if o.starts_with('-') { -1 } else { 1 };
        let (oh, om) = o[1..].split_once(':').unwrap_or((&o[1..], "0"));
        (c, Some(sign * (num(oh)? * 60 + num(om)?)))
    } else {
        (time, None)
    };
    let (h, mi) = if clock.is_empty() {
        (0, 0)
    } else {
        let mut c = clock.split(':');
        let h = num(c.next()?)?;
        let mi = num(c.next()?)?;
        if let Some(sec) = c.next() {
            num(sec.split('.').next()?)?;
        }
        (h, mi)
    };
    match offset_min {
        None => at(y, mo, dd, h, mi, zone),
        Some(o) => at(y, mo, dd, h, mi, &Zone::utc()).map(|ms| ms - o * 60_000),
    }
}

fn field(t: &mlua::Table, k: &str, default: Option<i64>) -> mlua::Result<Option<i64>> {
    match t.get::<Value>(k)? {
        Value::Nil => Ok(default),
        Value::Integer(i) => Ok(Some(i)),
        #[expect(clippy::cast_possible_truncation, reason = "um número inteiro do Luau")]
        Value::Number(n) if n.fract() == 0.0 && n.is_finite() => Ok(Some(n as i64)),
        _ => Ok(None),
    }
}

/// Uma data do script. `Err` = o texto para o erro (o que veio).
pub fn from_lua(v: &Value, zone: &Zone) -> Result<Moment, String> {
    let ms = match v {
        Value::String(s) => {
            let s = s.to_string_lossy();
            parse_iso(&s, zone).ok_or_else(|| {
                format!("\"{s}\" is not a date (use \"2026-11-03\" or \"2026-11-03T14:30\")")
            })?
        }
        Value::Table(t) => {
            let get = |k, d| field(t, k, d).ok().flatten();
            let parts = (get("year", None), get("month", None), get("day", None));
            let (Some(y), Some(mo), Some(d)) = parts else {
                return Err("a date table needs year, month and day".into());
            };
            at(
                y,
                mo,
                d,
                get("hour", Some(0)).unwrap_or(-1),
                get("min", Some(0)).unwrap_or(-1),
                zone,
            )
            .ok_or_else(|| format!("{y}-{mo}-{d} is not a date"))?
        }
        other => {
            return Err(format!(
                "got a {} — a date is text or a table",
                other.type_name()
            ));
        }
    };
    Ok(Moment(ms))
}

/// O texto local de um instante.
#[must_use]
pub fn to_iso(m: Moment, zone: &Zone) -> String {
    let c = civil(m.0, zone);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}",
        c.date.year, c.date.month, c.date.day, c.hour, c.minute
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_vai_e_volta_no_fuso_e_com_desvio() {
        let z = Zone::named("America/Sao_Paulo").expect("fuso");
        let local = parse_iso("2026-11-03T14:30", &z).expect("lê");
        assert_eq!(to_iso(Moment(local), &z), "2026-11-03T14:30");
        assert_eq!(parse_iso("2026-11-03T17:30Z", &z), Some(local));
        assert_eq!(parse_iso("2026-11-03T14:30-03:00", &z), Some(local));
        assert_eq!(parse_iso("2026-11-03T14:30:59", &z), Some(local));
        assert_eq!(
            to_iso(Moment(parse_iso("2026-11-03", &z).expect("dia")), &z),
            "2026-11-03T00:00"
        );
        for mau in ["2026-02-30", "amanhã", "2026-11-03T25:00", "2026-11"] {
            assert_eq!(parse_iso(mau, &z), None, "{mau}");
        }
    }
}
