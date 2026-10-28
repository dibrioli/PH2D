//! `ph2d-kanban-dates` — o que «hoje», «atrasado», «amanhã» e «esta semana» significam num fuso
//! (ADR-0195 §8), sobre `jiff` 0.2. O documento guarda instantes neutros (ms UTC); aqui eles
//! ganham dia civil, distintivo, lembrete e a grelha do mês. A API pública não expõe tipos do
//! `jiff`, e os textos saem em partes (enums) para a i18n.

mod calendar;
mod civil;
mod due;

pub use calendar::{CalendarMonth, CivilDay};
pub use civil::{
    CivilDate, CivilDateTime, Weekday, Zone, add_days, civil, day_of, instant, start_of_day,
    week_start_for_region,
};
pub use due::{
    DueLabel, DueState, DueTone, date_label, due_label, due_state, due_tone, remind_at,
    reminder_due,
};
