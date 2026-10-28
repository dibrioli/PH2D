//! A frase para a i18n, as sugestões e os bytes do corpo.

use std::collections::BTreeSet;

use ph2d_kanban_auto::{
    Action, Added, By, Kind, Part, SPEC_VERSION, Spec, Trigger, Word, describe, suggest,
};
use ph2d_kanban_model::{Actor, Due, KanbanOp, Member, Moment, Record, apply_logged};

use crate::{NOW, Scene, enio};

fn done_rule(s: &Scene, actions: Vec<Action>) -> Spec {
    Spec {
        name: String::new(),
        enabled: true,
        owner: enio(),
        kind: Kind::Rule {
            trigger: Trigger::CardAdded {
                list: Some(s.done),
                how: Added::Any,
            },
            by: By::Anyone,
            conditions: Vec::new(),
        },
        actions,
    }
}

#[test]
fn the_butler_sentence_comes_out_in_parts_for_the_i18n() {
    let s = Scene::new();
    let r = done_rule(&s, vec![Action::MarkDue { done: true }, Action::Archive]);
    assert_eq!(
        describe(&r),
        vec![
            Part::Word(Word::When),
            Part::Word(Word::CardAdded),
            Part::List(s.done),
            Part::Word(Word::MarkDone),
            Part::Word(Word::And),
            Part::Word(Word::Archive),
        ]
    );
    let keys: BTreeSet<&str> = Word::ALL.iter().map(|w| w.key()).collect();
    assert_eq!(keys.len(), Word::ALL.len(), "uma chave por palavra");
    assert!(keys.iter().all(|k| k.starts_with("kanban.auto.")));
}

#[test]
fn the_body_round_trips_and_a_newer_version_is_refused() {
    let s = Scene::new();
    let r = done_rule(&s, vec![Action::Comment("{cardname}".into())]);
    let bytes = r.to_bytes().unwrap();
    assert_eq!(Spec::from_bytes(&bytes).unwrap(), r);
    let newer = postcard::to_allocvec(&(SPEC_VERSION + 1, 0_u8)).unwrap();
    assert!(Spec::from_bytes(&newer).is_err());
}

/// `n` cartões movidos para Done por `who`, cada um marcado como concluído um minuto depois.
fn repeat(s: &mut Scene, n: usize, who: &Member) {
    for i in 0..n {
        let c = s.card(s.todo, &format!("C{i}"), |c| {
            c.due = Some(Due {
                at: NOW,
                done: false,
                remind_before_min: None,
            });
        });
        let t = NOW.0 + i64::try_from(i).unwrap() * 3_600_000;
        let k = s.set.doc_mut(s.k).unwrap();
        let mut card = k.cards[&c].clone();
        card.list = s.done;
        let at = |m| Actor {
            by: who.clone(),
            at: Moment(m),
        };
        apply_logged(k, vec![KanbanOp::put(Record::card(card.clone()))], &at(t));
        card.due.as_mut().unwrap().done = true;
        apply_logged(k, vec![KanbanOp::put(Record::card(card))], &at(t + 60_000));
    }
}

#[test]
fn three_repeats_suggest_the_rule_and_an_existing_rule_silences_it() {
    let mut s = Scene::new();
    repeat(&mut s, 2, &enio());
    assert!(
        suggest(s.set.get(s.k).unwrap(), &enio()).is_empty(),
        "duas vezes não chega"
    );
    repeat(&mut s, 1, &enio());
    let got = suggest(s.set.get(s.k).unwrap(), &enio());
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].times, 3);
    assert_eq!(got[0].spec.actions, vec![Action::MarkDue { done: true }]);
    assert!(matches!(&got[0].spec.kind, Kind::Rule { by: By::Only(m), .. } if *m == enio()));
    assert!(
        suggest(s.set.get(s.k).unwrap(), &Member("Ana".into())).is_empty(),
        "é o que EU repito"
    );
    let r = done_rule(&s, vec![Action::MarkDue { done: true }]);
    s.add(&r);
    assert!(suggest(s.set.get(s.k).unwrap(), &enio()).is_empty());
}
