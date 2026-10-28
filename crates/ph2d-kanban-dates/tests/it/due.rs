use crate::{at, ny};
use ph2d_kanban_dates::{
    CivilDate, DueLabel, DueState as S, DueTone as T, Weekday, Zone, add_days, civil, due_label,
    due_state, due_tone, remind_at, reminder_due, start_of_day, week_start_for_region,
};

#[test]
fn due_state_midnight_boundary_is_local_23_59_vs_00_00() {
    let z = ny();
    let now = at(&z, 2026, 10, 28, 23, 59);
    assert_eq!(
        due_state(Some(at(&z, 2026, 10, 29, 0, 0)), false, now, &z),
        S::DueTomorrow
    );
    assert_eq!(due_state(Some(now + 30_000), false, now, &z), S::DueToday);
    let midnight = at(&z, 2026, 10, 29, 0, 0);
    assert_eq!(
        due_state(Some(at(&z, 2026, 10, 29, 23, 59)), false, midnight, &z),
        S::DueToday
    );
    assert_eq!(
        due_state(Some(at(&z, 2026, 10, 30, 0, 0)), false, midnight, &z),
        S::DueTomorrow
    );
    // O mesmo instante noutro fuso é outro dia.
    let utc = Zone::utc();
    assert_eq!(
        due_state(Some(at(&z, 2026, 10, 29, 0, 0)), false, now, &utc),
        S::DueToday
    );
}

#[test]
fn due_state_overdue_only_after_the_due_instant_and_done_wins() {
    let z = ny();
    let due = at(&z, 2026, 10, 28, 14, 0);
    assert_eq!(
        due_state(Some(due), false, due, &z),
        S::DueToday,
        "na hora exacta ainda não"
    );
    assert_eq!(due_state(Some(due), false, due + 1, &z), S::Overdue);
    assert_eq!(due_state(Some(due), true, due + 1, &z), S::Done);
    assert_eq!(due_state(None, true, due, &z), S::None);
}

#[test]
fn due_state_week_buckets() {
    let z = ny();
    let now = at(&z, 2026, 10, 28, 9, 0);
    let on = |d: u8| due_state(Some(at(&z, 2026, 11, d, 8, 0)), false, now, &z);
    assert_eq!(on(1), S::DueSoon(4));
    assert_eq!(on(4), S::DueSoon(7));
    assert_eq!(on(5), S::Later);
    assert_eq!(
        due_state(Some(at(&z, 2026, 10, 30, 0, 0)), false, now, &z),
        S::DueSoon(2)
    );
}

#[test]
fn due_state_counts_civil_days_across_daylight_saving() {
    let z = ny();
    // 8/3: dia de 23 h. 00:30 + 23 h 30 = 01:00 do dia 9 → amanhã (24 h ingénuas diriam hoje).
    let now = at(&z, 2026, 3, 8, 0, 30);
    assert_eq!(
        due_state(Some(now + 47 * 30 * 60_000), false, now, &z),
        S::DueTomorrow
    );
    assert_eq!(
        due_state(Some(at(&z, 2026, 3, 8, 23, 30)), false, now, &z),
        S::DueToday
    );
    // 1/11: dia de 25 h. 00:30 + 24 h = 23:30 do mesmo dia → hoje (24 h ingénuas diriam amanhã).
    let now = at(&z, 2026, 11, 1, 0, 30);
    let due = now + 24 * 3_600_000;
    assert_eq!(civil(due, &z).date, CivilDate::new(2026, 11, 1).unwrap());
    assert_eq!(due_state(Some(due), false, now, &z), S::DueToday);
    assert_eq!(
        start_of_day(CivilDate::new(2026, 11, 2).unwrap(), &z),
        Some(at(&z, 2026, 11, 2, 0, 0))
    );
}

#[test]
fn due_civil_arithmetic_keeps_the_local_hour_across_dst_and_gaps_move_forward() {
    let z = ny();
    let sat = at(&z, 2026, 3, 7, 9, 0);
    let sun = add_days(sat, 1, &z);
    assert_eq!(sun - sat, 23 * 3_600_000, "o dia do salto tem 23 h");
    assert_eq!(
        (civil(sun, &z).hour, civil(sun, &z).weekday),
        (9, Weekday::Sunday)
    );
    // 02:30 não existe a 8/3: dá a hora a seguir ao salto.
    let gap = at(&z, 2026, 3, 8, 2, 30);
    assert_eq!((civil(gap, &z).hour, civil(gap, &z).minute), (3, 30));
}

#[test]
fn due_tone_follows_trello_red_yellow_green() {
    let h = 3_600_000;
    assert_eq!(due_tone(None, false, 0), None);
    assert_eq!(
        due_tone(Some(24 * h), false, 0),
        Some(T::Neutral),
        "24 h exactas ainda não"
    );
    assert_eq!(due_tone(Some(24 * h - 1), false, 0), Some(T::Soon));
    assert_eq!(due_tone(Some(0), false, 0), Some(T::Soon));
    assert_eq!(due_tone(Some(0), false, 1), Some(T::Overdue));
    assert_eq!(due_tone(Some(0), true, 1), Some(T::Done));
}

#[test]
fn due_label_parts_for_the_badge() {
    let z = ny();
    let now = at(&z, 2026, 10, 28, 9, 0);
    let l = |y, m, d, hh, mm| due_label(at(&z, y, m, d, hh, mm), now, &z);
    assert_eq!(
        l(2026, 10, 28, 14, 0),
        DueLabel::Today {
            hour: 14,
            minute: 0
        }
    );
    assert_eq!(
        l(2026, 10, 29, 8, 5),
        DueLabel::Tomorrow { hour: 8, minute: 5 }
    );
    assert_eq!(
        l(2026, 10, 27, 23, 59),
        DueLabel::Yesterday {
            hour: 23,
            minute: 59
        }
    );
    assert_eq!(l(2026, 12, 31, 0, 0), DueLabel::Date { month: 12, day: 31 });
    assert_eq!(
        l(2027, 10, 28, 0, 0),
        DueLabel::DateWithYear {
            year: 2027,
            month: 10,
            day: 28
        }
    );
}

#[test]
fn due_reminder_instant() {
    assert_eq!(remind_at(10 * 60_000, 3), 7 * 60_000);
    assert!(!reminder_due(10 * 60_000, 3, 7 * 60_000 - 1));
    assert!(reminder_due(10 * 60_000, 3, 7 * 60_000));
    assert_eq!(
        remind_at(i64::MIN + 1, u32::MAX),
        i64::MIN,
        "satura, não transborda"
    );
}

#[test]
fn due_week_start_by_region_follows_cldr() {
    assert_eq!(week_start_for_region("PT"), Weekday::Sunday);
    assert_eq!(week_start_for_region("br"), Weekday::Sunday);
    assert_eq!(week_start_for_region("US"), Weekday::Sunday);
    assert_eq!(week_start_for_region("DE"), Weekday::Monday);
    assert_eq!(week_start_for_region("EG"), Weekday::Saturday);
    assert_eq!(week_start_for_region(""), Weekday::Monday);
}

#[test]
fn due_named_iana_zone_when_the_machine_has_the_database() {
    // A base IANA é do sistema; onde existir, Lisboa salta a 29/3/2026.
    if let Some(z) = Zone::named("Europe/Lisbon") {
        let before = at(&z, 2026, 3, 28, 12, 0);
        assert_eq!(add_days(before, 1, &z) - before, 23 * 3_600_000);
    }
    assert!(Zone::named("Not/AZone").is_none());
}
