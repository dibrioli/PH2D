mod calendar;
mod due;

use ph2d_kanban_dates::{CivilDate, Zone, instant};

/// Nova Iorque com horário de verão: salta 02:00→03:00 a 8/3/2026 e recua 02:00→01:00 a 1/11/2026.
/// Regra POSIX: não depende da base IANA da máquina.
pub fn ny() -> Zone {
    Zone::posix("EST5EDT,M3.2.0,M11.1.0").expect("regra POSIX")
}

pub fn at(zone: &Zone, y: i16, m: u8, d: u8, hh: u8, mm: u8) -> i64 {
    instant(CivilDate::new(y, m, d).expect("data"), hh, mm, zone).expect("instante")
}
