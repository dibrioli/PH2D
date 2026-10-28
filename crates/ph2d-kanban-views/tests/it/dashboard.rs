//! A vista Painel: as contagens são as do filtro, a dica dá o número exacto e a forma muda; e a foto.

use crate::support::{World, now};
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_filter::{DueWindow, Filter, count};
use ph2d_kanban_model::Member;
use ph2d_kanban_views::dashboard::{Bucket, Chart, Form, Who, stats};
use ph2d_kanban_views::{PointerKind, ViewKind, Views};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, VectorScene};

const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1440.0,
    h: 900.0,
};

fn paint(v: &mut Views, w: &World, ts: &mut TextSystem) {
    let mut s = VectorScene::new();
    v.paint(
        &mut s,
        Affine::IDENTITY,
        AREA,
        &w.set,
        w.id,
        ts,
        Theme::Light,
        &w.cx(),
    );
}

fn setup() -> (Views, World, TextSystem) {
    let w = World::scene1();
    let mut v = Views::default();
    v.kind = ViewKind::Dashboard;
    let mut ts = TextSystem::without_system_fonts();
    paint(&mut v, &w, &mut ts);
    (v, w, ts)
}

/// Quantos cartões o FILTRO do quadro deixa ver com `f`.
fn filtered(w: &World, f: &Filter) -> usize {
    let k = w.set.get(w.id).expect("kanban");
    count(f, k, now(), &w.zone, &w.me)
}

#[test]
fn dashboard_counts_are_the_filters_counts() {
    let w = World::scene1();
    let k = w.set.get(w.id).expect("kanban");
    let s = stats(k, &w.cx());
    assert_eq!(s.total, filtered(&w, &Filter::default()));
    for b in &s.per_label {
        let Who::Label(id, _) = b.who else { continue };
        let f = Filter {
            labels: [id].into(),
            ..Filter::default()
        };
        assert_eq!(b.count, filtered(&w, &f), "etiqueta {}", b.name);
    }
    for b in &s.per_member {
        let Who::Member(m) = &b.who else { continue };
        let f = Filter {
            members: [m.clone()].into(),
            ..Filter::default()
        };
        assert_eq!(b.count, filtered(&w, &f), "membro {}", b.name);
    }
    let window = |d: DueWindow| {
        filtered(
            &w,
            &Filter {
                due: [d].into(),
                ..Filter::default()
            },
        )
    };
    assert_eq!(s.due(Bucket::Overdue), window(DueWindow::Overdue));
    assert_eq!(s.due(Bucket::NextDay), window(DueWindow::NextDay));
    assert_eq!(
        s.due(Bucket::NextWeek),
        window(DueWindow::NextWeek) - window(DueWindow::NextDay)
    );
    assert_eq!(s.due(Bucket::NoDates), window(DueWindow::NoDates));
    let complete = Filter {
        complete: true,
        ..Filter::default()
    };
    assert_eq!(s.due(Bucket::Complete), filtered(&w, &complete));
    let per_list: usize = s.per_list.iter().map(|b| b.count).sum();
    let per_due: usize = s.per_due.iter().map(|b| b.count).sum();
    assert_eq!(
        (per_list, per_due),
        (s.total, s.total),
        "lista e entrega PARTEM os cartões"
    );
    assert_eq!(
        s.due(Bucket::Overdue),
        2,
        "a cena 1: «Fox concept sheet» e «Animate shot 12»"
    );
}

#[test]
fn dashboard_follows_the_board_filter() {
    let mut w = World::scene1();
    w.filter = Filter {
        members: [Member("Mia".into())].into(),
        ..Filter::default()
    };
    let k = w.set.get(w.id).expect("kanban");
    let s = stats(k, &w.cx());
    assert_eq!(s.total, filtered(&w, &w.filter.clone()));
    let mia = s.per_member.iter().find(|b| b.name == "Mia").expect("Mia");
    assert_eq!(mia.count, s.total, "só os cartões da Mia passam");
}

#[test]
fn dashboard_hover_names_the_bar_and_its_exact_count() {
    let (mut v, w, mut ts) = setup();
    let bars = v
        .dashboard
        .painted_bars(Chart::PerList)
        .expect("barras")
        .to_vec();
    let doing = bars.iter().position(|b| b.name == "Doing").expect("Doing");
    let at = v.dashboard.bar_at(Chart::PerList, doing).expect("alvo");
    let r = v.pointer(&w.set, w.id, &w.cx(), PointerKind::Move, at.0, at.1);
    assert!(
        r.redraw && r.plan.is_empty(),
        "pairar pede outro quadro, nunca muda o documento"
    );
    assert_eq!(v.dashboard.hover, Some((Chart::PerList, doing)));
    assert_eq!(bars[doing].count, 3);
    paint(&mut v, &w, &mut ts);
    let r = v.pointer(&w.set, w.id, &w.cx(), PointerKind::Move, 2.0, 2.0);
    assert!(r.redraw);
    assert_eq!(v.dashboard.hover, None, "fora das marcas, a dica some");
}

#[test]
fn dashboard_ring_slice_is_hit_by_angle_and_the_form_toggles() {
    let (mut v, w, mut ts) = setup();
    assert_eq!(v.dashboard.form(Chart::PerDue), Form::Ring);
    assert_eq!(v.dashboard.form(Chart::PerLabel), Form::Bars);
    assert!(
        v.dashboard.toggle_at(Chart::PerLabel, Form::Ring).is_none(),
        "etiquetas não partem os cartões: sem anel"
    );
    let bars = v
        .dashboard
        .painted_bars(Chart::PerDue)
        .expect("fatias")
        .to_vec();
    let over = bars
        .iter()
        .position(|b| b.who == Who::Due(Bucket::Overdue))
        .expect("atrasados");
    let at = v.dashboard.bar_at(Chart::PerDue, over).expect("fatia");
    v.pointer(&w.set, w.id, &w.cx(), PointerKind::Move, at.0, at.1);
    assert_eq!(v.dashboard.hover, Some((Chart::PerDue, over)));
    let t = v
        .dashboard
        .toggle_at(Chart::PerList, Form::Ring)
        .expect("botão Pie");
    let r = v.pointer(&w.set, w.id, &w.cx(), PointerKind::Down, t.0, t.1);
    assert!(r.taken && r.plan.is_empty());
    assert_eq!(v.dashboard.form(Chart::PerList), Form::Ring);
    paint(&mut v, &w, &mut ts);
    assert!(v.dashboard.bar_at(Chart::PerList, 0).is_some());
}

#[test]
fn dashboard_folds_what_does_not_fit_into_other() {
    use ph2d_kanban_views::dashboard::data::fold;
    let mut set = ph2d_kanban_model::KanbanSet::default();
    let id = ph2d_kanban_views::fixture::write(&mut set, "Many labels", now(), |s| {
        let list = s.list("L", None, None);
        let labels: Vec<_> = (0..40)
            .map(|i| {
                s.label(
                    &format!("Tag {i}"),
                    ph2d_kanban_model::Hue::ALL[i % 10],
                    ph2d_kanban_model::Shade::Normal,
                )
            })
            .collect();
        for (i, l) in labels.iter().enumerate() {
            s.card(list, &format!("C{i}"), |c, _| c.labels = vec![*l]);
        }
    });
    let w = World::over(set, id);
    let k = w.set.get(w.id).expect("kanban");
    let s = stats(k, &w.cx());
    let folded = fold(&s.per_label, 8);
    assert_eq!(folded.len(), 8);
    assert_eq!(folded[7].who, Who::Other);
    let sum: usize = folded.iter().map(|b| b.count).sum();
    assert_eq!(sum, 40, "o que se dobra soma-se, não se perde");
    let mut v = Views::default();
    v.kind = ViewKind::Dashboard;
    let mut ts = TextSystem::without_system_fonts();
    paint(&mut v, &w, &mut ts);
    let painted = v.dashboard.painted_bars(Chart::PerLabel).expect("barras");
    assert!(painted.len() < 40 && painted.last().is_some_and(|b| b.who == Who::Other));
}

#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/W7/"]
fn photo_dashboard() {
    crate::photos::photo("painel", ViewKind::Dashboard, |_, _| {});
    crate::photos::photo("painel_dica", ViewKind::Dashboard, |v, w| {
        let mut ts = TextSystem::without_system_fonts();
        paint(v, w, &mut ts);
        if let Some(t) = v.dashboard.toggle_at(Chart::PerList, Form::Ring) {
            v.pointer(&w.set, w.id, &w.cx(), PointerKind::Down, t.0, t.1);
        }
        paint(v, w, &mut ts);
        let i = v
            .dashboard
            .painted_bars(Chart::PerLabel)
            .and_then(|b| b.iter().position(|x| x.name == "Animation"))
            .unwrap_or(0);
        if let Some(at) = v.dashboard.bar_at(Chart::PerLabel, i) {
            v.pointer(&w.set, w.id, &w.cx(), PointerKind::Move, at.0, at.1);
        }
    });
}
