//! A vista Linha do tempo: pintura, toque e o que ela pede ao documento; e a foto.

use crate::photos::photo;
use crate::support::World;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_dates::{civil, day_of};
use ph2d_kanban_model::{Card, CardId};
use ph2d_kanban_views::timeline::{Button, GroupBy, Zoom};
use ph2d_kanban_views::{PointerKind, Reply, ViewKind, Views};
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, VectorScene};

const W: f32 = 1440.0;
const H: f32 = 900.0;

struct T {
    w: World,
    v: Views,
    ts: ph2d_text::TextSystem,
}

impl T {
    fn new() -> Self {
        let mut v = Views::default();
        v.kind = ViewKind::Timeline;
        let mut t = Self {
            w: World::scene1(),
            v,
            ts: ph2d_text::TextSystem::without_system_fonts(),
        };
        t.paint();
        t
    }

    fn paint(&mut self) {
        let mut scene = VectorScene::new();
        let cx = self.w.cx();
        let r = Rect::new(0.0, 0.0, W, H);
        self.v.paint(
            &mut scene,
            Affine::IDENTITY,
            r,
            &self.w.set,
            self.w.id,
            &mut self.ts,
            Theme::Light,
            &cx,
        );
    }

    fn id(&self, title: &str) -> CardId {
        self.card_by(title).id
    }

    fn card_by(&self, title: &str) -> &Card {
        let k = self.w.set.get(self.w.id).expect("kanban");
        k.cards.values().find(|c| c.title == title).expect("cartão")
    }

    fn card(&self, id: CardId) -> Card {
        self.w.set.get(self.w.id).expect("kanban").cards[&id].clone()
    }

    fn bar(&self, id: CardId) -> Rect {
        self.v
            .timeline
            .painted()
            .bars_of(id)
            .next()
            .expect("barra")
            .rect
    }

    fn ppd(&self) -> f32 {
        self.v.timeline.painted().ruler.expect("régua").ppd
    }

    fn at(&mut self, kind: PointerKind, x: f32, y: f32) -> Reply {
        let cx = self.w.cx();
        self.v.pointer(&self.w.set, self.w.id, &cx, kind, x, y)
    }

    /// Carrega em `from`, arrasta `dx` px e larga; aplica o lote.
    fn drag(&mut self, from: (f32, f32), to: (f32, f32)) -> Reply {
        self.at(PointerKind::Down, from.0, from.1);
        self.at(
            PointerKind::Move,
            (from.0 + to.0) / 2.0,
            (from.1 + to.1) / 2.0,
        );
        self.paint();
        self.at(PointerKind::Move, to.0, to.1);
        let r = self.at(PointerKind::Up, to.0, to.1);
        self.w.apply(r.plan.clone());
        self.paint();
        r
    }

    fn click(&mut self, b: Button) {
        let r = self.v.timeline.painted().button(b).expect("botão");
        self.at(PointerKind::Down, r.x + 2.0, r.y + 2.0);
        self.at(PointerKind::Up, r.x + 2.0, r.y + 2.0);
        self.paint();
    }
}

#[test]
fn timeline_bar_spans_the_start_and_due_columns() {
    let t = T::new();
    let c = t.card_by("Design the fox").clone();
    let r = t.bar(c.id);
    let ruler = t.v.timeline.painted().ruler.expect("régua");
    let s = day_of(c.start.expect("início").0, &t.w.zone);
    let d = day_of(c.due.expect("entrega").at.0, &t.w.zone);
    assert!(
        (r.x - ruler.x_of(s)).abs() <= 1.5,
        "{} vs {}",
        r.x,
        ruler.x_of(s)
    );
    let end = ruler.x_of(d.add_days(1));
    assert!((r.x + r.w - end).abs() <= 1.5, "{} vs {end}", r.x + r.w);
    // Só a entrega: uma barra de um dia.
    let one = t.bar(t.id("Fox concept sheet"));
    assert!((one.w - ruler.ppd).abs() <= 2.5);
}

#[test]
fn timeline_left_handle_moves_only_the_start() {
    let mut t = T::new();
    let id = t.id("Design the fox");
    let before = t.card(id);
    let r = t.bar(id);
    let ppd = t.ppd();
    let y = r.y + r.h / 2.0;
    let rep = t.drag((r.x + 2.0, y), (r.x + 2.0 + 2.0 * ppd, y));
    assert!(!rep.plan.is_empty());
    let after = t.card(id);
    let z = &t.w.zone;
    let (b, a) = (
        civil(before.start.expect("s").0, z),
        civil(after.start.expect("s").0, z),
    );
    assert_eq!(b.date.add_days(2), a.date);
    assert_eq!((b.hour, b.minute), (a.hour, a.minute));
    assert_eq!(after.due, before.due);
}

#[test]
fn timeline_right_handle_moves_only_the_due_and_keeps_the_hour() {
    let mut t = T::new();
    let id = t.id("Pick the music");
    let before = t.card(id);
    let r = t.bar(id);
    let ppd = t.ppd();
    let y = r.y + r.h / 2.0;
    let x = r.x + r.w - 2.0;
    t.drag((x, y), (x + 2.0 * ppd, y));
    let after = t.card(id);
    let z = &t.w.zone;
    let (b, a) = (
        civil(before.due.expect("d").at.0, z),
        civil(after.due.expect("d").at.0, z),
    );
    assert_eq!(b.date.add_days(2), a.date);
    assert_eq!((b.hour, b.minute), (a.hour, a.minute));
    assert_eq!(after.start, before.start);
}

#[test]
fn timeline_body_drag_moves_both_dates_at_the_same_hour() {
    let mut t = T::new();
    let id = t.id("Pick the music");
    let before = t.card(id);
    let r = t.bar(id);
    let ppd = t.ppd();
    let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    t.drag((x, y), (x - 3.0 * ppd, y));
    let after = t.card(id);
    let z = &t.w.zone;
    for (b, a) in [
        (before.start.expect("s").0, after.start.expect("s").0),
        (before.due.expect("d").at.0, after.due.expect("d").at.0),
    ] {
        let (b, a) = (civil(b, z), civil(a, z));
        assert_eq!(b.date.add_days(-3), a.date);
        assert_eq!((b.hour, b.minute), (a.hour, a.minute));
    }
    assert_eq!(after.list, before.list);
}

#[test]
fn timeline_handles_never_cross() {
    let mut t = T::new();
    let id = t.id("Design the fox");
    let r = t.bar(id);
    let ppd = t.ppd();
    let y = r.y + r.h / 2.0;
    t.drag((r.x + 2.0, y), (r.x + 2.0 + 12.0 * ppd, y));
    let c = t.card(id);
    let z = &t.w.zone;
    assert_eq!(
        day_of(c.start.expect("s").0, z),
        day_of(c.due.expect("d").at.0, z)
    );
}

#[test]
fn timeline_click_without_drag_opens_the_card() {
    let mut t = T::new();
    let id = t.id("Pick the music");
    let r = t.bar(id);
    let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    t.at(PointerKind::Down, x, y);
    let rep = t.at(PointerKind::Up, x, y);
    assert_eq!(rep.open.map(|c| c.card), Some(id));
    assert!(rep.plan.is_empty());
}

#[test]
fn timeline_group_by_member_puts_a_card_in_each_members_lane() {
    let mut t = T::new();
    t.click(Button::Group(GroupBy::Member));
    assert_eq!(t.v.timeline.group, GroupBy::Member);
    let id = t.id("Light pass \u{2014} forest");
    assert_eq!(t.v.timeline.painted().bars_of(id).count(), 3);
    t.click(Button::Group(GroupBy::List));
    assert_eq!(t.v.timeline.painted().bars_of(id).count(), 1);
}

#[test]
fn timeline_drawer_lists_undated_cards_and_a_drop_schedules_them() {
    let mut t = T::new();
    let id = t.id("Voice casting");
    let p = t.v.timeline.painted();
    assert!(p.drawer_cards.iter().any(|(c, _)| *c == id));
    assert!(
        !p.drawer_cards
            .iter()
            .any(|(c, _)| *c == t.id("Pick the music"))
    );
    let from = p
        .drawer_cards
        .iter()
        .find(|(c, _)| *c == id)
        .expect("na gaveta")
        .1;
    let ruler = p.ruler.expect("régua");
    let day = ruler.anchor.add_days(3);
    let x = ruler.x_of(day) + ruler.ppd / 2.0;
    let y = ruler.grid.y + ruler.grid.h - 10.0;
    t.drag((from.x + 10.0, from.y + 10.0), (x, y));
    let c = t.card(id);
    let z = &t.w.zone;
    assert_eq!(day_of(c.start.expect("início").0, z), day);
    assert_eq!(day_of(c.due.expect("entrega").at.0, z), day);
    assert!(
        !t.v.timeline
            .painted()
            .drawer_cards
            .iter()
            .any(|(c, _)| *c == id)
    );
    assert_eq!(t.v.timeline.painted().bars_of(id).count(), 1);
}

#[test]
fn timeline_zoom_changes_px_per_day() {
    let mut t = T::new();
    let week = t.ppd();
    t.click(Button::Zoom(Zoom::Month));
    let month = t.ppd();
    t.click(Button::Zoom(Zoom::Day));
    let day = t.ppd();
    assert!(day > week && week > month, "{day} {week} {month}");
    let r = t.bar(t.id("Design the fox"));
    assert!((r.w - 6.0 * day).abs() <= 2.5);
}

#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/W7/"]
fn photo_timeline() {
    photo("linha_do_tempo", ViewKind::Timeline, |v, _| {
        v.timeline.zoom = Zoom::Week;
    });
}

#[test]
fn timeline_ctrl_wheel_zooms_and_plain_wheel_scrolls() {
    let w = World::scene1();
    let mut v = ph2d_kanban_views::Views::default();
    v.kind = ph2d_kanban_views::ViewKind::Timeline;
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let area = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1440.0, 900.0);
    let mut s = ph2d_vector::VectorScene::new();
    v.paint(
        &mut s,
        ph2d_vector::Affine::IDENTITY,
        area,
        &w.set,
        w.id,
        &mut ts,
        ph2d_tokens::Theme::Light,
        &w.cx(),
    );
    let before = v.timeline.zoom;
    let r = v.wheel(&w.set, w.id, &w.cx(), (700.0, 500.0), -16.0, (false, true));
    assert!(r.taken && r.plan.is_empty());
    assert_ne!(v.timeline.zoom, before, "Ctrl+roda muda o zoom");
    let z = v.timeline.zoom;
    v.wheel(&w.set, w.id, &w.cx(), (700.0, 500.0), -16.0, (false, false));
    assert_eq!(v.timeline.zoom, z, "a roda sozinha não muda o zoom");
}
