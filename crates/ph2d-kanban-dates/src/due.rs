//! O estado e o distintivo da data de entrega — o que o Trello mostra no cartão.

use crate::civil::{CivilDate, Zone, civil, day_of};

/// Onde a entrega cai em relação a «agora», em dias civis do fuso (a fronteira é a meia-noite
/// LOCAL, não 24 h). `Overdue` = já passou a hora; `DueSoon(d)` = daqui a `d` dias (2..=7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DueState {
    Overdue,
    DueToday,
    DueTomorrow,
    DueSoon(u8),
    Later,
    Done,
    None,
}

/// A cor do distintivo no Trello: vermelho atrasado, amarelo a menos de 24 h, verde concluído.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DueTone {
    Neutral,
    Soon,
    Overdue,
    Done,
}

const DAY_MS: i64 = 24 * 60 * 60 * 1000;
const SOON_DAYS: i64 = 7;

/// Estado da entrega `due` (ms UTC) para `now` em `zone`.
pub fn due_state(due: Option<i64>, done: bool, now: i64, zone: &Zone) -> DueState {
    let Some(due) = due else {
        return DueState::None;
    };
    if done {
        return DueState::Done;
    }
    if now > due {
        return DueState::Overdue;
    }
    match day_of(now, zone).days_until(day_of(due, zone)) {
        0 => DueState::DueToday,
        1 => DueState::DueTomorrow,
        d @ 2..=SOON_DAYS => DueState::DueSoon(d as u8),
        _ => DueState::Later,
    }
}

/// A cor do distintivo (`None` = sem data). Não depende do fuso: são horas, não dias.
pub fn due_tone(due: Option<i64>, done: bool, now: i64) -> Option<DueTone> {
    let due = due?;
    Some(if done {
        DueTone::Done
    } else if now > due {
        DueTone::Overdue
    } else if due - now < DAY_MS {
        DueTone::Soon
    } else {
        DueTone::Neutral
    })
}

/// O texto curto do distintivo em PARTES — a i18n escreve-o («Today 14:00», «Oct 28»,
/// «Oct 28, 2027»). `month` = 1..=12.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DueLabel {
    Yesterday { hour: u8, minute: u8 },
    Today { hour: u8, minute: u8 },
    Tomorrow { hour: u8, minute: u8 },
    Date { month: u8, day: u8 },
    DateWithYear { year: i16, month: u8, day: u8 },
}

/// O distintivo de `due` visto a partir de `now` em `zone`: ontem/hoje/amanhã com a hora, senão a
/// data (com o ano só se não for o ano corrente).
pub fn due_label(due: i64, now: i64, zone: &Zone) -> DueLabel {
    let d = civil(due, zone);
    let today = day_of(now, zone);
    let (hour, minute) = (d.hour, d.minute);
    match today.days_until(d.date) {
        -1 => DueLabel::Yesterday { hour, minute },
        0 => DueLabel::Today { hour, minute },
        1 => DueLabel::Tomorrow { hour, minute },
        _ => date_label(d.date, today),
    }
}

/// Só a data: «Oct 28», ou com o ano se não for o de `today`.
pub fn date_label(date: CivilDate, today: CivilDate) -> DueLabel {
    if date.year == today.year {
        DueLabel::Date {
            month: date.month,
            day: date.day,
        }
    } else {
        DueLabel::DateWithYear {
            year: date.year,
            month: date.month,
            day: date.day,
        }
    }
}

/// O instante do lembrete: `remind_before_min` minutos antes da entrega.
pub fn remind_at(due: i64, remind_before_min: u32) -> i64 {
    due.saturating_sub(i64::from(remind_before_min) * 60_000)
}

/// O lembrete já devia ter tocado (e a entrega ainda não passou do lembrete por concluir)?
pub fn reminder_due(due: i64, remind_before_min: u32, now: i64) -> bool {
    now >= remind_at(due, remind_before_min)
}
