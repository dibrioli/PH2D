//! A vista Calendário: pintura, toque e o que ela pede ao documento; e a foto.

use std::cell::RefCell;

use crate::support::{World, now};
use ph2d_editor_core::doc_surface::{Key, KeyInput};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_dates::{CivilDate, civil, day_of};
use ph2d_kanban_model::{CardId, CardRef, Moment, Plan};
use ph2d_kanban_views::calendar::{self, CalendarState, Mode, Target};
use ph2d_kanban_views::{Item, ItemKey, ItemTone, PointerKind, Source, ViewKind, Views};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Affine, VectorScene};

const BIG: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1440.0,
    h: 900.0,
};

fn date(m: u8, d: u8) -> CivilDate {
    CivilDate::new(2026, m, d).expect("data")
}

fn card(w: &World, title: &str) -> CardId {
    w.set
        .get(w.id)
        .and_then(|k| k.cards.values().find(|c| c.title == title))
        .map(|c| c.id)
        .expect("cartão")
}

fn key_of(w: &World, title: &str) -> ItemKey {
    ItemKey::Card(CardRef {
        kanban: w.id,
        card: card(w, title),
    })
}

fn paint(v: &mut Views, w: &World, ts: &mut TextSystem, rect: Rect) {
    let mut scene = VectorScene::new();
    v.paint(&mut scene, Affine::IDENTITY, rect, &w.set, w.id, ts, Theme::Light, &w.cx());
}

fn calendar_views() -> Views {
    let mut v = Views::default();
    v.kind = ViewKind::Calendar;
    v
}

fn center(r: Rect) -> (f32, f32) {
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn inside(inner: Rect, outer: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.w <= outer.x + outer.w + 0.5
        && inner.y + inner.h <= outer.y + outer.h + 0.5
}

#[test]
fn calendar_item_lands_in_the_cell_of_its_due_day() {
    let w = World::scene1();
    let mut ts = TextSystem::new();
    let mut v = calendar_views();
    paint(&mut v, &w, &mut ts, BIG);
    let rects = v.calendar.item_rects(key_of(&w, "Fox concept sheet"));
    assert_eq!(rects.len(), 1, "um dia, uma pastilha");
    let cell = v.calendar.cell_rect(date(10, 26)).expect("célula de 26/10");
    assert!(inside(rects[0], cell), "{:?} fora de {cell:?}", rects[0]);
}

#[test]
fn calendar_drag_moves_start_and_due_by_whole_days_keeping_the_hour() {
    let mut w = World::scene1();
    let mut ts = TextSystem::new();
    let mut v = calendar_views();
    paint(&mut v, &w, &mut ts, BIG);
    let id = card(&w, "Design the fox");
    let before = w.set.get(w.id).unwrap().cards[&id].clone();
    let bar = v.calendar.item_rects(key_of(&w, "Design the fox"))[0];
    let from = v.calendar.cell_rect(date(10, 28)).expect("28/10");
    let to = v.calendar.cell_rect(date(10, 30)).expect("30/10");
    let y = bar.y + bar.h / 2.0;
    let cx = w.cx();
    let _ = v.pointer(&w.set, w.id, &cx, PointerKind::Down, center(from).0, y);
    let _ = v.pointer(&w.set, w.id, &cx, PointerKind::Move, center(to).0, y);
    paint(&mut v, &w, &mut ts, BIG);
    let r = v.pointer(&w.set, w.id, &cx, PointerKind::Up, center(to).0, y);
    assert!(!r.plan.is_empty(), "o arrasto pede uma mudança");
    w.apply(r.plan);
    let after = &w.set.get(w.id).unwrap().cards[&id];
    let two = |m: Moment| {
        let c = civil(m.0, &w.zone);
        (c.date, c.hour, c.minute)
    };
    let (bd, ad) = (before.due.unwrap().at, after.due.unwrap().at);
    assert_eq!(two(ad).0, two(bd).0.add_days(2));
    assert_eq!((two(ad).1, two(ad).2), (two(bd).1, two(bd).2), "a hora fica");
    assert_eq!(
        day_of(after.start.unwrap().0, &w.zone),
        day_of(before.start.unwrap().0, &w.zone).add_days(2)
    );
}

#[test]
fn calendar_shows_plus_n_more_when_the_cell_does_not_fit() {
    let w = World::scene1();
    let mut ts = TextSystem::new();
    let mut v = calendar_views();
    let small = Rect::new(0.0, 0.0, 700.0, 420.0);
    paint(&mut v, &w, &mut ts, small);
    let (r, n) = v.calendar.more_at(date(10, 28)).expect("«+N more» em 28/10");
    let cell = v.calendar.cell_rect(date(10, 28)).unwrap();
    let shown = v
        .calendar
        .item_rects(key_of(&w, "Clean up shot 08"))
        .into_iter()
        .filter(|s| inside(*s, cell))
        .count();
    assert!(n >= 1 && n + shown <= 4, "n = {n}");
    let cx = w.cx();
    let _ = v.pointer(&w.set, w.id, &cx, PointerKind::Down, center(r).0, center(r).1);
    assert_eq!(v.calendar.day_popover(), Some(date(10, 28)));
    paint(&mut v, &w, &mut ts, BIG);
    assert!(
        v.calendar.more_at(date(10, 28)).is_none(),
        "na célula grande cabe tudo"
    );
}

#[test]
fn calendar_plus_and_enter_create_a_card_due_that_day_in_the_first_list() {
    let mut w = World::scene1();
    let mut ts = TextSystem::new();
    let mut v = calendar_views();
    paint(&mut v, &w, &mut ts, BIG);
    let cell = v.calendar.cell_rect(date(10, 30)).unwrap();
    let cx = w.cx();
    let _ = v.pointer(&w.set, w.id, &cx, PointerKind::Move, center(cell).0, center(cell).1);
    paint(&mut v, &w, &mut ts, BIG);
    let (plus, day) = v.calendar.plus_rect().expect("o «+» do dia sob o rato");
    assert_eq!(day, date(10, 30));
    let _ = v.pointer(&w.set, w.id, &cx, PointerKind::Down, center(plus).0, center(plus).1);
    let _ = v.pointer(&w.set, w.id, &cx, PointerKind::Up, center(plus).0, center(plus).1);
    assert_eq!(v.calendar.composing(), Some(date(10, 30)));
    paint(&mut v, &w, &mut ts, BIG);
    let mods = ph2d_host::Modifiers::default();
    for ch in "New shot".chars() {
        let s = ch.to_string();
        let k = KeyInput {
            key: Key::Char(ch),
            mods,
            pressed: true,
            text: Some(&s),
            paste: None,
        };
        let _ = v.key(&w.set, w.id, &cx, &k);
    }
    let enter = KeyInput {
        key: Key::Enter,
        mods,
        pressed: true,
        text: None,
        paste: None,
    };
    let r = v.key(&w.set, w.id, &cx, &enter);
    w.apply(r.plan);
    let k = w.set.get(w.id).unwrap();
    let c = k.cards.values().find(|c| c.title == "New shot").expect("criado");
    let first = k.lists_in_order()[0].id;
    assert_eq!(c.list, first, "a primeira lista por omissão");
    let due = civil(c.due.unwrap().at.0, &w.zone);
    assert_eq!(due.date, date(10, 30));
    assert!(v.calendar.composing().is_none());
}

/// Uma fonte que não é um Kanban: um evento do projecto (a W9).
struct Fake {
    moved: RefCell<Vec<(ItemKey, Moment)>>,
}

impl Source for Fake {
    fn items(&self, _from: Moment, _to: Moment) -> Vec<Item> {
        vec![Item {
            key: ItemKey::Event(7),
            title: "Client review".into(),
            start: None,
            end: now(),
            done: false,
            tone: ItemTone::Token(ColorToken::Accent),
        }]
    }
    fn reschedule(&self, key: ItemKey, _start: Option<Moment>, end: Moment) -> Plan {
        self.moved.borrow_mut().push((key, end));
        Plan::default()
    }
    fn targets(&self) -> Vec<Target> {
        Vec::new()
    }
    fn create(&self, _at: &Target, _title: &str, _due: Moment) -> Plan {
        Plan::default()
    }
}

#[test]
fn calendar_paints_and_drags_any_source_not_only_a_kanban() {
    let w = World::scene1();
    let cx = w.cx();
    let mut ts = TextSystem::new();
    let fake = Fake {
        moved: RefCell::new(Vec::new()),
    };
    let mut st = CalendarState::default();
    st.mode = Mode::Week;
    let mut scene = VectorScene::new();
    calendar::paint(&mut scene, &mut ts, Theme::Dark, BIG, &mut st, &fake, &cx);
    let rects = st.item_rects(ItemKey::Event(7));
    let today = st.cell_rect(date(10, 28)).expect("hoje está na semana");
    assert!(rects.len() == 1 && inside(rects[0], today));
    let to = st.cell_rect(date(10, 29)).unwrap();
    let p = |kind, (x, y)| ph2d_kanban_views::PointerAt { kind, x, y };
    let _ = calendar::pointer(&mut st, &fake, &cx, p(PointerKind::Down, center(rects[0])));
    let _ = calendar::pointer(&mut st, &fake, &cx, p(PointerKind::Move, center(to)));
    let _ = calendar::pointer(&mut st, &fake, &cx, p(PointerKind::Up, center(to)));
    let moved = fake.moved.borrow();
    assert_eq!(moved.len(), 1);
    assert_eq!(day_of(moved[0].1.0, &w.zone), date(10, 29));
}

#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/W7/"]
fn photo_calendar() {
    crate::photos::photo("calendario", ViewKind::Calendar, |_, _| {});
    crate::photos::photo("calendario_semana", ViewKind::Calendar, |v, _| {
        v.calendar.mode = Mode::Week;
    });
}
