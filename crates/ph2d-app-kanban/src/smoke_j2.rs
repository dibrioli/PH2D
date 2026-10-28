//! A cena de smoke de J2 (o roteador é o [`crate::smoke`]):
//!
//! - **8** — o quadro da cena 1 com o BUTLER montado e o verso de «Animate shot 12» aberto: na coluna
//!   «Buttons» o botão de cartão «Ship it»; na secção «Project links» uma ligação VIVA ao Quadro
//!   «Storyboard» e uma QUEBRADA («Fox rig (deleted)»); no «…» do Kanban (ou em «Add button» do
//!   verso) o painel de automação com três regras, um botão de quadro e uma SUGESTÃO («You moved 3
//!   cards to Review and did the same each time: … join the card»).

use ph2d_kanban_auto::{
    Action, Added, By, Condition, Dest, End, Every, Kind, Scope, Sort, Spec, Test, TimeOfDay,
    Trigger,
};
use ph2d_kanban_model::{
    Actor, CardId, KanbanId, KanbanOp, KanbanSet, Member, Moment, Record, Target, apply_logged,
};

const MIN: i64 = 60_000;

fn spec(name: &str, me: &Member, kind: Kind, actions: Vec<Action>) -> Spec {
    Spec {
        name: name.into(),
        enabled: true,
        owner: me.clone(),
        kind,
        actions,
    }
}

/// As automações da cena, a sugestão (três cartões movidos para a «Review» por `me`, que se junta a
/// cada um) e as duas ligações do cartão `card` (ao Quadro `board` e a um objecto que já não existe).
pub(crate) fn butler(set: &mut KanbanSet, id: KanbanId, me: &Member, card: CardId, board: u64) {
    let now = crate::actions::now();
    let Some(k) = set.doc_mut(id) else { return };
    let list = |t: &str| k.lists.values().find(|l| l.title == t).map(|l| l.id);
    let label = |t: &str| k.labels.values().find(|l| l.name == t).map(|l| l.id);
    let (Some(done), Some(review), Some(backlog)) = (list("Done"), list("Review"), list("Backlog"))
    else {
        return;
    };
    let rule = |trigger, conditions| Kind::Rule {
        trigger,
        by: By::Anyone,
        conditions,
    };
    let mut specs = vec![
        spec(
            "",
            me,
            rule(
                Trigger::CardAdded {
                    list: Some(done),
                    how: Added::Any,
                },
                Vec::new(),
            ),
            vec![Action::MarkDue { done: true }, Action::Archive],
        ),
        spec(
            "",
            me,
            Kind::Calendar {
                every: Every::Weekdays(0x1F),
                at: TimeOfDay { hour: 9, minute: 0 },
                scope: Scope::default(),
            },
            vec![Action::SortList {
                list: backlog,
                by: Sort::Due,
            }],
        ),
        spec(
            "Ship it",
            me,
            Kind::CardButton,
            vec![
                Action::MoveTo {
                    to: Dest {
                        kanban: None,
                        list: done,
                    },
                    at: End::Top,
                },
                Action::Comment("Shipped by {username} on {date}".into()),
            ],
        ),
        spec(
            "Sweep Done",
            me,
            Kind::BoardButton {
                scope: Scope {
                    lists: vec![done],
                    conditions: Vec::new(),
                },
            },
            vec![Action::Archive],
        ),
    ];
    if let Some(audio) = label("Audio") {
        specs.insert(
            1,
            spec(
                "",
                me,
                rule(
                    Trigger::Label {
                        label: Some(audio),
                        added: true,
                    },
                    vec![Condition::is(Test::NoMembers)],
                ),
                vec![Action::JoinMe],
            ),
        );
    }
    for s in specs {
        let op = s.create(k, now);
        ph2d_kanban_model::apply_batch(k, vec![op]);
    }
    // A sugestão: três cartões para a «Review», e «eu» junto-me a cada um logo a seguir.
    let moving: Vec<_> = k
        .cards
        .values()
        .filter(|c| c.list != review && c.list != done && c.id != card && !c.archived)
        .take(3)
        .cloned()
        .collect();
    for (i, c) in moving.into_iter().enumerate() {
        let at = Moment(now.0 - (90 - 20 * i64::try_from(i).unwrap_or(0)) * MIN);
        let mut moved = c.clone();
        moved.list = review;
        let actor = Actor { by: me.clone(), at };
        apply_logged(k, vec![KanbanOp::put(Record::Card(moved.clone().into()))], &actor);
        moved.members.push(me.clone());
        let actor = Actor {
            by: me.clone(),
            at: Moment(at.0 + MIN),
        };
        apply_logged(k, vec![KanbanOp::put(Record::Card(moved.into()))], &actor);
    }
    let link = |k: &ph2d_kanban_model::Kanban, t: Target, name: &str| {
        ph2d_kanban_links::link_op(k, card, t, name, now)
    };
    let a = link(
        k,
        Target::Board {
            board,
            element: None,
        },
        "Storyboard",
    );
    ph2d_kanban_model::apply_batch(k, vec![a]);
    let b = link(k, Target::Object(u64::MAX - 7), "Fox rig");
    ph2d_kanban_model::apply_batch(k, vec![b]);
}
