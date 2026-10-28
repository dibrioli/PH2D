//! Fuso, dia civil e as conversões instante ↔ hora local. Nenhum tipo do `jiff` sai daqui.

use jiff::civil::Date;
use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan, Zoned};

/// Um fuso horário (opaco). As datas do documento são instantes UTC em ms (`Moment`).
#[derive(Clone, Debug)]
pub struct Zone(pub(crate) TimeZone);

impl Zone {
    /// O fuso do sistema (UTC se não se conseguir saber).
    pub fn system() -> Zone {
        Zone(TimeZone::system())
    }

    pub fn utc() -> Zone {
        Zone(TimeZone::UTC)
    }

    /// Fuso IANA pelo nome (`"America/Sao_Paulo"`).
    pub fn named(name: &str) -> Option<Zone> {
        TimeZone::get(name).ok().map(Zone)
    }

    /// Fuso por regra POSIX (`"EST5EDT,M3.2.0,M11.1.0"`) — não precisa da base IANA.
    pub fn posix(rule: &str) -> Option<Zone> {
        TimeZone::posix(rule).ok().map(Zone)
    }

    pub(crate) fn zoned(&self, ms: i64) -> Zoned {
        let ts = Timestamp::from_millisecond(ms).unwrap_or(if ms < 0 {
            Timestamp::MIN
        } else {
            Timestamp::MAX
        });
        ts.to_zoned(self.0.clone())
    }
}

/// Dia da semana.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Weekday {
    #[default]
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    pub const ALL: [Weekday; 7] = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ];

    /// 0 = segunda … 6 = domingo.
    pub fn monday_index(self) -> u8 {
        self as u8
    }

    pub(crate) fn from_jiff(w: jiff::civil::Weekday) -> Weekday {
        Weekday::ALL[usize::try_from(w.to_monday_zero_offset()).unwrap_or(0)]
    }
}

/// Primeiro dia da semana pela região (CLDR `weekData/firstDay`); as outras são segunda.
pub fn week_start_for_region(region: &str) -> Weekday {
    const SUNDAY: &[&str] = &[
        "AG", "AS", "BD", "BR", "BS", "BT", "BW", "BZ", "CA", "CN", "CO", "DM", "DO", "ET", "GT",
        "GU", "HK", "HN", "ID", "IL", "IN", "JM", "JP", "KE", "KH", "KR", "LA", "MH", "MM", "MO",
        "MT", "MX", "MZ", "NI", "NP", "PA", "PE", "PH", "PK", "PR", "PT", "PY", "SA", "SG", "SV",
        "TH", "TT", "TW", "UM", "US", "VE", "VI", "WS", "YE", "ZA", "ZW",
    ];
    const SATURDAY: &[&str] = &[
        "AE", "AF", "BH", "DJ", "DZ", "EG", "IQ", "IR", "JO", "KW", "LY", "OM", "QA", "SD", "SY",
    ];
    let r = region.to_ascii_uppercase();
    if SUNDAY.contains(&r.as_str()) {
        Weekday::Sunday
    } else if SATURDAY.contains(&r.as_str()) {
        Weekday::Saturday
    } else if r == "MV" {
        Weekday::Friday
    } else {
        Weekday::Monday
    }
}

/// Um dia do calendário (sem fuso).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CivilDate {
    pub year: i16,
    pub month: u8,
    pub day: u8,
}

impl CivilDate {
    /// `None` se a data não existir (31 de abril…).
    pub fn new(year: i16, month: u8, day: u8) -> Option<CivilDate> {
        let d = Date::new(year, i8::try_from(month).ok()?, i8::try_from(day).ok()?).ok()?;
        Some(CivilDate::from_jiff(d))
    }

    pub fn weekday(self) -> Weekday {
        Weekday::from_jiff(self.to_jiff().weekday())
    }

    /// Dias civis de `self` até `other` (negativo se `other` vem antes).
    pub fn days_until(self, other: CivilDate) -> i64 {
        self.to_jiff()
            .until(other.to_jiff())
            .map_or(0, |s| i64::from(s.get_days()))
    }

    /// `self` mais `n` dias civis (preso aos limites do calendário).
    pub fn add_days(self, n: i64) -> CivilDate {
        let d = self.to_jiff();
        CivilDate::from_jiff(d.checked_add(n.days()).unwrap_or(if n < 0 {
            Date::MIN
        } else {
            Date::MAX
        }))
    }

    pub(crate) fn to_jiff(self) -> Date {
        Date::new(self.year, self.month as i8, self.day as i8).unwrap_or(Date::MIN)
    }

    pub(crate) fn from_jiff(d: Date) -> CivilDate {
        CivilDate {
            year: d.year(),
            month: d.month() as u8,
            day: d.day() as u8,
        }
    }
}

/// Um instante visto num fuso.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CivilDateTime {
    pub date: CivilDate,
    pub hour: u8,
    pub minute: u8,
    pub weekday: Weekday,
}

/// O instante `ms` (UTC) visto em `zone`.
pub fn civil(ms: i64, zone: &Zone) -> CivilDateTime {
    let z = zone.zoned(ms);
    CivilDateTime {
        date: CivilDate::from_jiff(z.date()),
        hour: z.hour() as u8,
        minute: z.minute() as u8,
        weekday: Weekday::from_jiff(z.date().weekday()),
    }
}

/// O dia civil de `ms` em `zone`.
pub fn day_of(ms: i64, zone: &Zone) -> CivilDate {
    CivilDate::from_jiff(zone.zoned(ms).date())
}

/// A hora local `date hh:mm` em `zone` como instante. Numa hora que não existe (o salto do
/// horário de verão) dá a hora a seguir ao salto; numa que existe duas vezes, a primeira.
pub fn instant(date: CivilDate, hour: u8, minute: u8, zone: &Zone) -> Option<i64> {
    let dt = date
        .to_jiff()
        .at(i8::try_from(hour).ok()?, i8::try_from(minute).ok()?, 0, 0);
    let z = dt.to_zoned(zone.0.clone()).ok()?;
    Some(z.timestamp().as_millisecond())
}

/// A meia-noite local que começa o dia `date` em `zone`.
pub fn start_of_day(date: CivilDate, zone: &Zone) -> Option<i64> {
    let z = date.to_jiff().to_zoned(zone.0.clone()).ok()?;
    Some(z.start_of_day().ok()?.timestamp().as_millisecond())
}

/// A mesma hora local `days` dias depois (atravessa o horário de verão sem erro).
pub fn add_days(ms: i64, days: i64, zone: &Zone) -> i64 {
    let z = zone.zoned(ms);
    z.checked_add(days.days())
        .map_or(ms, |z| z.timestamp().as_millisecond())
}
