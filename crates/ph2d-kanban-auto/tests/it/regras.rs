//! Regras, botões e o executor: o exemplo do Butler, determinismo, anti-laço, condições, variáveis.

use ph2d_kanban_auto::{
    Action, Added, By, Condition, Dest, End, Kind, MAX_DEPTH, Scope, Sort, Spec, Stop, Test,
    Trigger, press_board_button, press_card_button,
};
use ph2d_kanban_model::{LabelId, Member, Subject};

use crate::{NOW, Scene, due, enio, utc};

fn rule(trigger: Trigger, conditions: Vec<Condition>, actions: Vec<Action>) -> Spec {
    Spec {
        name: String::new(),
        enabled: true,
        owner: enio(),
        kind: Kind::Rule {
            trigger,
            by: By::Anyone,
            conditions,
        },
        actions,
    }
}

fn added_to(list: ph2d_kanban_model::ListId) -> Trigger {
    Trigger::CardAdded {
        list: Some(list),
        how: Added::Any,
    }
}

fn to(list: ph2d_kanban_model::ListId) -> Dest {
    Dest { kanban: None, list }
}

#[test]
fn a_card_added_to_done_is_marked_complete_and_archived_by_the_butler() {
    let mut s = Scene::new();
    s.add(&rule(
        added_to(s.done),
        Vec::new(),
        vec![Action::MarkDue { done: true }, Action::Archive],
    ));
    let c = s.card(s.todo, "Sketch", |c| c.due = due(NOW.0 + crate::DAY));
    let o = s.move_to(c, s.done, &enio());
    assert_eq!(o.fired.len(), 1);
    assert_eq!(o.stopped, None);
    s.butler(o);
    let card = s.get(c);
    assert!(card.due.unwrap().done && card.archived);
    let k = s.set.get(s.k).unwrap();
    let by_butler = k
        .activity_of(Subject::Card(c))
        .into_iter()
        .filter(|a| a.by.is_butler())
        .count();
    assert_eq!(
        by_butler, 2,
        "concluir e arquivar aparecem na atividade em nome do Butler"
    );
}

#[test]
fn the_same_gesture_gives_the_same_plan() {
    let mut s = Scene::new();
    s.add(&rule(
        added_to(s.done),
        Vec::new(),
        vec![Action::Comment("{cardname}".into())],
    ));
    s.add(&rule(
        added_to(s.done),
        Vec::new(),
        vec![Action::AddLabel(s.urgent), Action::JoinMe],
    ));
    let c = s.card(s.todo, "A", |_| {});
    let before = s.set.clone();
    let a = s.move_to(c, s.done, &enio());
    s.set = before;
    let b = s.move_to(c, s.done, &enio());
    assert_eq!(a.plan, b.plan, "mesmo projecto, mesmo gesto, mesmo plano");
    assert_eq!(a.fired, b.fired);
    assert_eq!(a.fired.len(), 2, "as duas regras, pela ordem do painel");
    assert!(a.fired[0].automation < a.fired[1].automation);
}

#[test]
fn a_ping_pong_pair_stops_when_a_rule_meets_its_own_chain() {
    let mut s = Scene::new();
    let (doing, done) = (s.doing, s.done);
    s.add(&rule(
        added_to(doing),
        Vec::new(),
        vec![Action::MoveTo {
            to: to(done),
            at: End::Top,
        }],
    ));
    s.add(&rule(
        added_to(done),
        Vec::new(),
        vec![Action::MoveTo {
            to: to(doing),
            at: End::Top,
        }],
    ));
    let c = s.card(s.todo, "Ping", |_| {});
    let o = s.move_to(c, doing, &enio());
    let depths: Vec<usize> = o.fired.iter().map(|f| f.depth).collect();
    assert_eq!(depths, [0, 1], "A → B, e B não volta a disparar A");
    assert_eq!(o.stopped, None);
    s.butler(o);
    assert_eq!(s.get(c).list, doing);
}

#[test]
fn a_chain_longer_than_max_depth_stops_there() {
    let mut s = Scene::new();
    let mut ids = vec![s.urgent];
    for _ in 0..=MAX_DEPTH {
        let id = LabelId(s.set.get(s.k).unwrap().mint());
        let l = ph2d_kanban_model::Label {
            id,
            name: format!("L{}", id.0),
            color: None,
            meta: ph2d_kanban_model::Meta::default(),
        };
        s.set.apply(
            s.k,
            vec![ph2d_kanban_model::KanbanOp::put(
                ph2d_kanban_model::Record::Label(l),
            )],
        );
        ids.push(id);
    }
    for w in ids.windows(2) {
        let t = Trigger::Label {
            label: Some(w[0]),
            added: true,
        };
        s.add(&rule(t, Vec::new(), vec![Action::AddLabel(w[1])]));
    }
    let c = s.card(s.todo, "Chain", |_| {});
    let u = s.urgent;
    let o = s.edit(c, &enio(), |c| c.labels.push(u));
    assert_eq!(o.fired.len(), MAX_DEPTH);
    assert_eq!(o.fired.last().unwrap().depth, MAX_DEPTH - 1);
    assert_eq!(o.stopped, Some(Stop::Depth));
}

#[test]
fn conditions_use_the_filter_law_and_not_negates() {
    let mut s = Scene::new();
    let urgent = Condition::is(Test::Labels {
        labels: vec![s.urgent],
        all: false,
    });
    let calm = Condition {
        not: true,
        ..urgent.clone()
    };
    s.add(&rule(
        added_to(s.done),
        vec![urgent],
        vec![Action::Comment("hot".into())],
    ));
    s.add(&rule(
        added_to(s.done),
        vec![calm],
        vec![Action::Comment("calm".into())],
    ));
    let u = s.urgent;
    let hot = s.card(s.todo, "Hot", |c| c.labels.push(u));
    let cold = s.card(s.todo, "Cold", |_| {});
    let (oh, oc) = (
        s.move_to(hot, s.done, &enio()),
        s.move_to(cold, s.done, &enio()),
    );
    assert_eq!((oh.fired.len(), oc.fired.len()), (1, 1));
    assert_ne!(oh.fired[0].automation, oc.fired[0].automation);
    let list = Condition::is(Test::InList(vec![s.doing]));
    s.add(&rule(Trigger::DueSet, vec![list], vec![Action::JoinMe]));
    let o = s.edit(cold, &enio(), |c| c.due = due(NOW.0));
    assert!(o.fired.is_empty(), "o cartão está em Done, não em Doing");
}

#[test]
fn by_filters_who_made_the_gesture_and_the_butler_is_not_me() {
    let mut s = Scene::new();
    let mut r = rule(added_to(s.doing), Vec::new(), vec![Action::JoinMe]);
    if let Kind::Rule { by, .. } = &mut r.kind {
        *by = By::Only(enio());
    }
    s.add(&r);
    let c = s.card(s.todo, "Mine", |_| {});
    let ana = Member("Ana".into());
    assert!(s.move_to(c, s.doing, &ana).fired.is_empty());
    let o = s.move_to(c, s.todo, &enio());
    assert!(o.fired.is_empty());
    let o = s.move_to(c, s.doing, &enio());
    assert_eq!(o.fired.len(), 1);
    s.butler(o);
    assert_eq!(
        s.get(c).members,
        vec![enio()],
        "«juntar-me» junta quem fez o gesto"
    );
}

#[test]
fn a_comment_expands_the_butler_variables() {
    let mut s = Scene::new();
    let text = "{cardname} em {listname} por {username} a {date} {time} {nada} {";
    s.add(&rule(
        added_to(s.done),
        Vec::new(),
        vec![Action::Comment(text.into())],
    ));
    let c = s.card(s.todo, "Logo", |_| {});
    let o = s.move_to(c, s.done, &enio());
    s.butler(o);
    let k = s.set.get(s.k).unwrap();
    let cm = k.comments.values().find(|m| m.card == c).unwrap();
    assert_eq!(cm.text, "Logo em Done por Enio a 2026-10-28 12:00 {nada} {");
    assert!(cm.author.is_butler());
}

#[test]
fn a_comment_with_text_and_a_complete_checklist_fire_their_rules() {
    let mut s = Scene::new();
    let says = Trigger::Commented {
        contains: Some("PRONTO".into()),
    };
    s.add(&rule(
        says,
        Vec::new(),
        vec![Action::MarkDue { done: true }],
    ));
    s.add(&rule(
        Trigger::ChecklistComplete,
        Vec::new(),
        vec![Action::MoveTo {
            to: to(s.done),
            at: End::Bottom,
        }],
    ));
    let button = Spec {
        name: "Preparar".into(),
        enabled: true,
        owner: enio(),
        kind: Kind::CardButton,
        actions: vec![
            Action::AddChecklist {
                title: "Passos".into(),
                items: vec!["a".into(), "b".into()],
            },
            Action::CheckAll { done: true },
            Action::Comment("está pronto".into()),
        ],
    };
    let b = s.add(&button);
    let c = s.card(s.todo, "Card", |c| c.due = due(NOW.0));
    let o = press_card_button(&s.set, s.k, b, c, &enio(), NOW, &utc());
    let depths: Vec<usize> = o.fired.iter().map(|f| f.depth).collect();
    assert_eq!(
        depths,
        [0, 1, 1],
        "o botão, e as duas regras que ele disparou"
    );
    s.butler(o);
    let card = s.get(c);
    assert!(
        card.due.unwrap().done,
        "«pronto» sem caixa nem acentos casa «PRONTO»"
    );
    assert_eq!(card.list, s.done);
}

#[test]
fn a_board_button_acts_on_its_scope_and_sorts_once() {
    let mut s = Scene::new();
    let a = s.card(s.done, "Zeta", |_| {});
    let b = s.card(s.done, "Alfa", |c| c.due = due(NOW.0));
    let c = s.card(s.todo, "Fora", |_| {});
    let button = Spec {
        name: "Limpar".into(),
        enabled: true,
        owner: enio(),
        kind: Kind::BoardButton {
            scope: Scope {
                lists: vec![s.done],
                conditions: vec![Condition::is(Test::Due(ph2d_kanban_auto::DueIn::NoDates))],
            },
        },
        actions: vec![
            Action::SortList {
                list: s.done,
                by: Sort::Name,
            },
            Action::Archive,
        ],
    };
    let id = s.add(&button);
    let o = press_board_button(&s.set, s.k, id, &enio(), NOW, &utc());
    s.butler(o);
    assert!(s.get(a).archived && !s.get(b).archived && !s.get(c).archived);
    let k = s.set.get(s.k).unwrap();
    assert!(k.cards[&b].pos < k.cards[&a].pos, "Alfa antes de Zeta");
}

#[test]
fn moving_to_another_kanban_keeps_acting_on_the_moved_card() {
    let mut s = Scene::new();
    let other = s.set.create("Arquivo");
    let mut o2 = Scene {
        set: s.set.clone(),
        k: other,
        ..Scene::new()
    };
    let shelf = o2.list("Prateleira");
    s.set = o2.set;
    let dest = Dest {
        kanban: Some(other),
        list: shelf,
    };
    s.add(&rule(
        added_to(s.done),
        Vec::new(),
        vec![
            Action::MoveTo {
                to: dest,
                at: End::Top,
            },
            Action::Comment("chegou".into()),
        ],
    ));
    let c = s.card(s.todo, "Viajante", |_| {});
    let o = s.move_to(c, s.done, &enio());
    s.butler(o);
    let k2 = s.set.get(other).unwrap();
    let moved = k2.cards.values().find(|c| c.title == "Viajante").unwrap();
    assert!(
        k2.comments
            .values()
            .any(|m| m.card == moved.id && m.text == "chegou")
    );
}
