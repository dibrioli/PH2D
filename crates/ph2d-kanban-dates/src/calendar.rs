//! A grelha do mês (6 × 7) que o widget `calendar` do editor-core pinta. Tipos simples: o widget
//! não depende desta crate, recebe as células.

use crate::civil::{CivilDate, Weekday, Zone, day_of};

/// Uma célula da grelha.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CivilDay {
    pub year: i16,
    pub month: u8,
    pub day: u8,
    /// Do mês mostrado (as outras são dos meses vizinhos, a cinzento).
    pub in_month: bool,
    pub is_today: bool,
}

/// Um mês com o dia de hoje e o primeiro dia da semana (segunda por omissão).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CalendarMonth {
    year: i16,
    month: u8,
    today: Option<CivilDate>,
    week_start: Weekday,
}

impl CalendarMonth {
    /// `None` se `month` não for 1..=12.
    pub fn new(
        year: i16,
        month: u8,
        today: Option<CivilDate>,
        week_start: Weekday,
    ) -> Option<Self> {
        CivilDate::new(year, month, 1)?;
        Some(CalendarMonth {
            year,
            month,
            today,
            week_start,
        })
    }

    /// O mês de «agora» em `zone`.
    pub fn containing(now: i64, zone: &Zone, week_start: Weekday) -> CalendarMonth {
        let today = day_of(now, zone);
        CalendarMonth {
            year: today.year,
            month: today.month,
            today: Some(today),
            week_start,
        }
    }

    pub fn year(&self) -> i16 {
        self.year
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    pub fn week_start(&self) -> Weekday {
        self.week_start
    }

    pub fn next(&self) -> CalendarMonth {
        let (year, month) = if self.month == 12 {
            (self.year.saturating_add(1), 1)
        } else {
            (self.year, self.month + 1)
        };
        CalendarMonth {
            year,
            month,
            ..*self
        }
    }

    pub fn prev(&self) -> CalendarMonth {
        let (year, month) = if self.month == 1 {
            (self.year.saturating_sub(1), 12)
        } else {
            (self.year, self.month - 1)
        };
        CalendarMonth {
            year,
            month,
            ..*self
        }
    }

    /// Os cabeçalhos das colunas, a começar em `week_start`.
    pub fn weekdays(&self) -> [Weekday; 7] {
        let s = usize::from(self.week_start.monday_index());
        std::array::from_fn(|i| Weekday::ALL[(s + i) % 7])
    }

    /// As 42 células, por linhas, a começar no dia da coluna 0 da semana do dia 1.
    pub fn cells(&self) -> [CivilDay; 42] {
        let first = CivilDate {
            year: self.year,
            month: self.month,
            day: 1,
        };
        let back = (7 + i64::from(first.weekday().monday_index())
            - i64::from(self.week_start.monday_index()))
            % 7;
        let start = first.add_days(-back);
        std::array::from_fn(|i| {
            let d = start.add_days(i as i64);
            CivilDay {
                year: d.year,
                month: d.month,
                day: d.day,
                in_month: d.year == self.year && d.month == self.month,
                is_today: self.today == Some(d),
            }
        })
    }
}
