use crate::{at, ny};
use ph2d_kanban_dates::{CalendarMonth, CivilDate, CivilDay, Weekday};

fn day(c: &CivilDay) -> (i16, u8, u8, bool) {
    (c.year, c.month, c.day, c.in_month)
}

#[test]
fn calendar_october_2026_starts_on_monday_by_default_with_neighbour_days() {
    // 1/10/2026 é quinta-feira.
    let today = CivilDate::new(2026, 10, 28);
    let m = CalendarMonth::new(2026, 10, today, Weekday::default()).unwrap();
    let c = m.cells();
    assert_eq!(day(&c[0]), (2026, 9, 28, false));
    assert_eq!(day(&c[3]), (2026, 10, 1, true));
    assert_eq!(day(&c[33]), (2026, 10, 31, true));
    assert_eq!(day(&c[34]), (2026, 11, 1, false));
    assert_eq!(day(&c[41]), (2026, 11, 8, false));
    let todays: Vec<usize> = (0..42).filter(|&i| c[i].is_today).collect();
    assert_eq!(todays, vec![30]);
    assert_eq!(m.weekdays()[0], Weekday::Monday);
}

#[test]
fn calendar_first_column_is_configurable() {
    let m = CalendarMonth::new(2026, 10, None, Weekday::Sunday).unwrap();
    assert_eq!(day(&m.cells()[0]), (2026, 9, 27, false));
    assert_eq!(
        m.weekdays(),
        [
            Weekday::Sunday,
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
        ]
    );
    // Fevereiro de 2027 começa a uma segunda: a 1.ª célula já é do mês.
    let feb = CalendarMonth::new(2027, 2, None, Weekday::Monday).unwrap();
    assert_eq!(day(&feb.cells()[0]), (2027, 2, 1, true));
    assert_eq!(feb.cells().iter().filter(|c| c.in_month).count(), 28);
}

#[test]
fn calendar_navigation_wraps_years_and_rejects_bad_months() {
    let m = CalendarMonth::new(2026, 12, None, Weekday::Monday).unwrap();
    assert_eq!((m.next().year(), m.next().month()), (2027, 1));
    assert_eq!(
        (m.next().prev().year(), m.next().prev().month()),
        (2026, 12)
    );
    assert!(CalendarMonth::new(2026, 13, None, Weekday::Monday).is_none());
    assert!(CalendarMonth::new(2026, 0, None, Weekday::Monday).is_none());
}

#[test]
fn calendar_containing_now_uses_the_local_day() {
    let z = ny();
    // 23:30 de 31/10 em Nova Iorque já é 1/11 em UTC.
    let now = at(&z, 2026, 10, 31, 23, 30);
    let m = CalendarMonth::containing(now, &z, Weekday::Monday);
    assert_eq!((m.year(), m.month()), (2026, 10));
    assert!(
        m.cells()
            .iter()
            .any(|c| c.is_today && c.day == 31 && c.in_month)
    );
    let leap = CalendarMonth::new(2028, 2, None, Weekday::Monday).unwrap();
    assert_eq!(leap.cells().iter().filter(|c| c.in_month).count(), 29);
}
