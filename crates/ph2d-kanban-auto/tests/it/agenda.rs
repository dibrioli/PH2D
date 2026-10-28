//! A agenda: sem repetir depois de reabrir, recuperar o que caiu com o app fechado, os dias.

use ph2d_kanban_auto::{
    Action, DueWhen, Every, Kind, Scope, Spec, TimeOfDay, Unit, due_commands, is_day, run_scheduled,
};
use ph2d_kanban_dates::CivilDate;
use ph2d_kanban_model::{AutomationId, KanbanOp, KanbanSet, Moment, Record};

use crate::{DAY, HOUR, NOW, Scene, due, enio, utc};

fn command(kind: Kind, actions: Vec<Action>) -> Spec {
    Spec {
        name: String::new(),
        enabled: true,
        owner: enio(),
        kind,
        actions,
    }
}

/// Recua o último disparo de `id` para `at` (o app esteve fechado desde então).
fn last_run(s: &mut Scene, id: AutomationId, at: Option<Moment>) {
    let mut a = s.set.get(s.k).unwrap().automations[&id].clone();
    a.last_run = at;
    s.set.apply(s.k, vec![KanbanOp::put(Record::Automation(a))]);
}

#[test]
fn a_daily_command_missed_for_a_week_runs_once_and_never_again_after_reopening() {
    let mut s = Scene::new();
    let nine = TimeOfDay { hour: 9, minute: 0 };
    let id = s.add(&command(
        Kind::Calendar {
            every: Every::Day,
            at: nine,
            scope: Scope {
                lists: vec![s.todo],
                conditions: Vec::new(),
            },
        },
        vec![Action::Comment("bom dia".into())],
    ));
    let c = s.card(s.todo, "Diário", |_| {});
    last_run(&mut s, id, Some(Moment(NOW.0 - 7 * DAY)));
    let f = due_commands(&s.set, NOW, &utc());
    assert_eq!(f.len(), 1, "sete dias fechado = UM disparo");
    assert_eq!(f[0].at, Moment(NOW.0 - 3 * HOUR), "o de hoje às 09:00");
    let o = run_scheduled(&s.set, NOW, &utc());
    assert_eq!(o.fired.len(), 1);
    s.butler(o);
    let k = s.set.get(s.k).unwrap();
    assert_eq!(k.comments.values().filter(|m| m.card == c).count(), 1);
    assert_eq!(k.automations[&id].last_run, Some(NOW));
    let reopened = KanbanSet::from_bytes(&s.set.to_bytes().unwrap()).unwrap();
    assert!(
        due_commands(&reopened, NOW, &utc()).is_empty(),
        "reabrir não repete"
    );
    let tomorrow = Moment(NOW.0 + DAY);
    assert_eq!(due_commands(&reopened, tomorrow, &utc()).len(), 1);
}

#[test]
fn an_unarmed_command_is_armed_without_running() {
    let mut s = Scene::new();
    let id = s.add(&command(
        Kind::Calendar {
            every: Every::Day,
            at: TimeOfDay { hour: 0, minute: 0 },
            scope: Scope::default(),
        },
        vec![Action::Archive],
    ));
    last_run(&mut s, id, None);
    assert!(due_commands(&s.set, NOW, &utc()).is_empty());
    let o = run_scheduled(&s.set, NOW, &utc());
    assert!(o.fired.is_empty() && !o.plan.is_empty(), "só o carimbo");
    s.butler(o);
    assert_eq!(s.set.get(s.k).unwrap().automations[&id].last_run, Some(NOW));
}

#[test]
fn a_due_command_fires_each_card_once_when_its_moment_passes() {
    let mut s = Scene::new();
    let when = DueWhen::Before {
        n: 1,
        unit: Unit::Days,
    };
    let id = s.add(&command(
        Kind::DueDate {
            when,
            scope: Scope::default(),
        },
        vec![Action::JoinMe],
    ));
    let soon = s.card(s.todo, "Amanhã cedo", |c| c.due = due(NOW.0 + 12 * HOUR));
    let late = s.card(s.todo, "Daqui a 3 dias", |c| c.due = due(NOW.0 + 3 * DAY));
    last_run(&mut s, id, Some(Moment(NOW.0 - DAY)));
    let f = due_commands(&s.set, NOW, &utc());
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].cards, vec![soon]);
    let o = run_scheduled(&s.set, NOW, &utc());
    s.butler(o);
    assert_eq!(
        s.get(soon).members,
        vec![enio()],
        "o «eu» de um comando é o dono dele"
    );
    assert!(s.get(late).members.is_empty());
    assert!(due_commands(&s.set, NOW, &utc()).is_empty());
    let f = due_commands(&s.set, Moment(NOW.0 + 2 * DAY + HOUR), &utc());
    assert_eq!(f[0].cards, vec![late]);
}

#[test]
fn days_past_the_end_of_the_month_fall_on_its_last_day() {
    let d = |y, m, dd| CivilDate::new(y, m, dd).unwrap();
    assert!(is_day(Every::MonthDay(31), d(2026, 4, 30)));
    assert!(!is_day(Every::MonthDay(31), d(2026, 4, 29)));
    let leap = Every::YearDay { month: 2, day: 29 };
    assert!(is_day(leap, d(2027, 2, 28)) && is_day(leap, d(2028, 2, 29)));
    assert!(!is_day(leap, d(2028, 2, 28)));
    let mon_fri = 0b001_1111;
    assert!(is_day(Every::Weekdays(mon_fri), d(2026, 10, 28)), "quarta");
    assert!(!is_day(Every::Weekdays(mon_fri), d(2026, 10, 31)), "sábado");
}

/// Liga ou desliga a automação `id` (o interruptor do painel: troca só o corpo).
fn set_enabled(s: &mut Scene, id: AutomationId, on: bool) {
    let a = s.set.get(s.k).unwrap().automations[&id].clone();
    let mut spec = Spec::from_bytes(&a.body).unwrap();
    spec.enabled = on;
    let op = spec.update(&a);
    s.set.apply(s.k, vec![op]);
}

/// R1 — desligar uma automação uma semana e religá-la NÃO dispara o passado: o que caiu enquanto
/// estava desligada consome-se sem correr (calendário e prazos, estes em massa).
#[test]
fn a_command_switched_off_for_a_week_does_not_fire_the_past_when_switched_back_on() {
    let mut s = Scene::new();
    let daily = s.add(&command(
        Kind::Calendar {
            every: Every::Day,
            at: TimeOfDay { hour: 9, minute: 0 },
            scope: Scope::default(),
        },
        vec![Action::Comment("bom dia".into())],
    ));
    let on_due = s.add(&command(
        Kind::DueDate {
            when: DueWhen::After {
                n: 1,
                unit: Unit::Hours,
            },
            scope: Scope::default(),
        },
        vec![Action::JoinMe],
    ));
    for d in 1..=5 {
        s.card(s.todo, "Atrasado", |c| c.due = due(NOW.0 - d * DAY));
    }
    let week_ago = Moment(NOW.0 - 7 * DAY);
    for id in [daily, on_due] {
        last_run(&mut s, id, Some(week_ago));
        set_enabled(&mut s, id, false);
    }
    let o = run_scheduled(&s.set, NOW, &utc());
    assert!(o.fired.is_empty(), "desligada não corre");
    s.butler(o);
    for id in [daily, on_due] {
        set_enabled(&mut s, id, true);
    }
    assert!(
        due_commands(&s.set, NOW, &utc()).is_empty(),
        "religar não dispara o que caiu enquanto desligada"
    );
    let tomorrow = Moment(NOW.0 + DAY);
    assert_eq!(due_commands(&s.set, tomorrow, &utc()).len(), 1, "a próxima ocorrência corre");
}
